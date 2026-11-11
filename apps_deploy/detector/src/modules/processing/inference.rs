use anyhow::{anyhow, bail, Context, Result};
use ndarray::{Array, ArrayD, IxDyn};
use ort::{
    inputs,
    memory::{AllocationDevice, Allocator, AllocatorType, MemoryInfo, MemoryType},
    session::{IoBinding, Session},
    value::{
        DynTensor, DynTensorValueType, Tensor, TensorElementType, TensorValueTypeMarker, Value,
        ValueType,
    },
};

/// Single-thread inference runtime keeping CUDA graph shapes, device addresses,
/// and reusable H2D/D2H buffers static across batches.
/// Drop order is critical: `io_binding` MUST drop before bound tensors, `_cuda_alloc`, and `session`.
pub struct GpuInferenceEngine {
    io_binding: Option<IoBinding>,
    cuda_in: Option<Tensor<f32>>,
    host_out: Option<DynTensor>,
    _cuda_alloc: Option<Allocator>,
    session: Session,
    input_name: String,
    output_ty: TensorElementType,
    has_static_cuda_out: bool,
    device_id: i32,
}

impl GpuInferenceEngine {
    pub fn new(
        session: Session,
        cuda_device_id: Option<i32>,
        batch_size: usize,
    ) -> Result<Self> {
        let input_name = session
            .inputs()
            .first()
            .ok_or_else(|| anyhow!("Model has no inputs"))?
            .name()
            .to_string();
        let output_outlet = session
            .outputs()
            .first()
            .ok_or_else(|| anyhow!("Model has no outputs"))?;
        let output_name = output_outlet.name().to_string();

        let (output_ty, non_batch_dims) = match output_outlet.dtype() {
            ValueType::Tensor { ty, shape, .. } => {
                let tail: Vec<i64> = shape.iter().skip(1).copied().collect();
                (*ty, tail)
            }
            other => bail!("Unsupported model output ValueType: {:?}", other),
        };

        let static_shape: Option<Vec<usize>> =
            if !non_batch_dims.is_empty() && non_batch_dims.iter().all(|&d| d > 0) {
                let mut dims = Vec::with_capacity(1 + non_batch_dims.len());
                dims.push(batch_size);
                dims.extend(non_batch_dims.iter().map(|&d| d as usize));
                Some(dims)
            } else {
                None
            };

        let Some(dev) = cuda_device_id else {
            return Ok(Self {
                io_binding: None,
                cuda_in: None,
                host_out: None,
                _cuda_alloc: None,
                session,
                input_name,
                output_ty,
                has_static_cuda_out: false,
                device_id: 0,
            });
        };

        let mut io_binding = session.create_binding()?;
        let cuda_alloc = Allocator::new(
            &session,
            MemoryInfo::new(
                AllocationDevice::CUDA,
                dev,
                AllocatorType::Device,
                MemoryType::Default,
            )?,
        )?;

        let (has_static_cuda_out, host_out) = if let Some(dims) = static_shape {
            let gpu_out = DynTensor::new(&cuda_alloc, output_ty, dims.clone())?;
            io_binding.bind_output(&output_name, gpu_out)?;

            let cpu_out = DynTensor::new(&Allocator::default(), output_ty, dims)?;
            (true, Some(cpu_out))
        } else {
            let cuda_mem = MemoryInfo::new(
                AllocationDevice::CUDA,
                dev,
                AllocatorType::Device,
                MemoryType::Default,
            )?;
            io_binding.bind_output_to_device(&output_name, &cuda_mem)?;
            (false, None)
        };

        Ok(Self {
            io_binding: Some(io_binding),
            cuda_in: None,
            host_out,
            _cuda_alloc: Some(cuda_alloc),
            session,
            input_name,
            output_ty,
            has_static_cuda_out,
            device_id: dev,
        })
    }

    /// Ensures one constant CUDA device input tensor is bound to `io_binding` for the lifetime
    /// of this shape so CUDA Graph replay never reads stale pointers from rotating host rings.
    fn sync_pinned_to_cuda_in(&mut self, pinned_in: &Tensor<f32>) -> Result<()> {
        let io_binding = self
            .io_binding
            .as_mut()
            .ok_or_else(|| anyhow!("Missing IoBinding"))?;
        let cuda_alloc = self
            ._cuda_alloc
            .as_ref()
            .ok_or_else(|| anyhow!("Missing CUDA allocator"))?;
        let target_shape = pinned_in.shape();
        let needs_alloc = self
            .cuda_in
            .as_ref()
            .map_or(true, |dev| *dev.shape() != *target_shape);
        if needs_alloc {
            let dims: Vec<usize> = target_shape.iter().map(|&d| d as usize).collect();
            let dev_in = Tensor::<f32>::new(cuda_alloc, dims)?;
            io_binding
                .bind_input(&self.input_name, &dev_in)
                .context("IoBinding bind_input(cuda_in) failed")?;
            self.cuda_in = Some(dev_in);
        }
        let cuda_in = self
            .cuda_in
            .as_mut()
            .ok_or_else(|| anyhow!("Missing CUDA input tensor"))?;
        pinned_in
            .copy_into(cuda_in)
            .context("H2D pinned_in -> cuda_in copy_into failed")
    }

    pub fn run_batch(&mut self, pinned_in: &Tensor<f32>, valid_len: usize) -> Result<ArrayD<f32>> {
        if self.io_binding.is_some() {
            self.sync_pinned_to_cuda_in(pinned_in)?;
            let io_binding = self
                .io_binding
                .as_mut()
                .ok_or_else(|| anyhow!("Missing IoBinding"))?;

            let outputs = self
                .session
                .run_binding(io_binding)
                .context("session.run_binding failed")?;
            if self.has_static_cuda_out {
                let host_out = self
                    .host_out
                    .as_mut()
                    .ok_or_else(|| anyhow!("Missing pre-allocated host output"))?;
                let gpu_out = outputs[0].downcast_ref::<DynTensorValueType>()?;
                gpu_out
                    .copy_into(host_out)
                    .context("D2H gpu_out -> host_out copy_into failed")?;
                return extract_valid_output(host_out, self.output_ty, valid_len);
            }

            let cpu_out = outputs[0]
                .downcast_ref::<DynTensorValueType>()?
                .to(AllocationDevice::CPU, self.device_id)
                .context("D2H gpu_out -> CPU .to() failed")?;
            return extract_valid_output(&cpu_out, self.output_ty, valid_len);
        }

        let outputs = self.session.run(inputs![pinned_in])?;
        let cpu_out = outputs[0].downcast_ref::<DynTensorValueType>()?;
        extract_valid_output(&*cpu_out, self.output_ty, valid_len)
    }
}

fn slice_to_valid_array<T: Clone>(
    raw_shape: &[i64],
    slice: &[T],
    valid_len: usize,
    convert: impl Fn(&T) -> f32,
) -> Result<ArrayD<f32>> {
    if raw_shape.is_empty() {
        bail!("Unexpected scalar output tensor");
    }
    let tail_elems: usize = raw_shape[1..].iter().map(|&d| d as usize).product();
    let needed = valid_len * tail_elems;
    if slice.len() < needed {
        bail!("Output slice {} shorter than expected {}", slice.len(), needed);
    }

    let mut out_shape = Vec::with_capacity(raw_shape.len());
    out_shape.push(valid_len);
    out_shape.extend(raw_shape[1..].iter().map(|&d| d as usize));

    let vec_f32: Vec<f32> = slice[..needed].iter().map(convert).collect();
    Ok(Array::from_shape_vec(IxDyn(&out_shape), vec_f32)?)
}

fn extract_valid_output<M: TensorValueTypeMarker + ?Sized>(
    val: &Value<M>,
    ty: TensorElementType,
    valid_len: usize,
) -> Result<ArrayD<f32>> {
    match ty {
        TensorElementType::Float32 => {
            let (shape, data) = val.try_extract_tensor::<f32>()?;
            slice_to_valid_array(shape, data, valid_len, |&v| v)
        }
        TensorElementType::Float16 => {
            let (shape, data) = val.try_extract_tensor::<half::f16>()?;
            slice_to_valid_array(shape, data, valid_len, |v| v.to_f32())
        }
        other => bail!("Unsupported output tensor element type: {:?}", other),
    }
}
