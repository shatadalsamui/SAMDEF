use anyhow::{anyhow, Result};
use crossbeam::channel;
use ort::{
    memory::{AllocationDevice, Allocator, AllocatorType, MemoryInfo, MemoryType},
    session::Session,
    value::Tensor,
};

use std::sync::{Arc, Mutex};

/// Bounded lock-free ring pool for reusing planar `[R | G | B]` tile buffers
/// without repeating 3.98 MB heap allocations and page faults per GeoTIFF tile.
#[derive(Clone)]
pub struct TileBytePool {
    tx: channel::Sender<Vec<u8>>,
    rx: channel::Receiver<Vec<u8>>,
    tile_bytes: usize,
}

impl TileBytePool {
    pub fn new(capacity: usize, tile_bytes: usize) -> Self {
        let cap = capacity.max(1);
        let (tx, rx) = channel::bounded(cap);
        Self { tx, rx, tile_bytes }
    }

    pub fn acquire(&self) -> Vec<u8> {
        if let Ok(buf) = self.rx.try_recv() {
            if buf.len() == self.tile_bytes {
                return buf;
            }
        }
        vec![0u8; self.tile_bytes]
    }

    pub fn recycle(&self, buf: Vec<u8>) {
        if buf.len() == self.tile_bytes {
            let _ = self.tx.try_send(buf);
        }
    }
}

/// Sending half held exclusively by Stage 2 so dropping Stage 2 immediately closes `PinAcquirer`.
#[derive(Clone)]
pub struct PinRecycler {
    tx: channel::Sender<Tensor<f32>>,
    _allocator: Arc<Mutex<Allocator>>,
}

/// Receiving half held exclusively by Stage 1; wakes immediately if Stage 2 exits.
/// `_allocator` is retained in both halves so queued tensors always drop before their `Allocator`.
pub struct PinAcquirer {
    rx: channel::Receiver<Tensor<f32>>,
    _allocator: Arc<Mutex<Allocator>>,
}

/// Creates a matched pre-allocated page-locked (`CUDA_PINNED`) or CPU tensor ring pair
/// `[batch_size, 3, tile_size, tile_size]` shared between Stage 1 and Stage 2.
pub fn create_pinned_tensor_ring(
    session: &Session,
    cuda_device_id: Option<i32>,
    batch_size: usize,
    slots: usize,
    tile_size: usize,
) -> Result<(PinAcquirer, PinRecycler)> {
    let slot_count = slots.max(2);
    let (tx, rx) = channel::bounded(slot_count);

    let allocator = match cuda_device_id {
        Some(dev) => Allocator::new(
            session,
            MemoryInfo::new(
                AllocationDevice::CUDA_PINNED,
                dev,
                AllocatorType::Device,
                MemoryType::CPUInput,
            )?,
        )?,
        None => Allocator::default(),
    };

    let shape = [batch_size, 3, tile_size, tile_size];
    for _ in 0..slot_count {
        let mut tensor = Tensor::<f32>::new(&allocator, shape)?;
        let (_, data) = tensor.try_extract_tensor_mut::<f32>()?;
        data.fill(0.0);
        let _ = tx.try_send(tensor);
    }

    let shared_alloc = Arc::new(Mutex::new(allocator));
    Ok((
        PinAcquirer {
            rx,
            _allocator: shared_alloc.clone(),
        },
        PinRecycler {
            tx,
            _allocator: shared_alloc,
        },
    ))
}

impl PinAcquirer {
    pub fn acquire(&self) -> Result<Tensor<f32>> {
        self.rx
            .recv()
            .map_err(|_| anyhow!("GPU consumer exited; pinned tensor pool closed"))
    }
}

impl PinRecycler {
    pub fn recycle(&self, tensor: Tensor<f32>) {
        let _ = self.tx.try_send(tensor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_byte_pool_recycles_exact_sized_buffers() {
        let pool = TileBytePool::new(2, 16);
        let mut buf = pool.acquire();
        assert_eq!(buf.len(), 16);
        buf[0] = 42;
        let ptr_before = buf.as_ptr();
        pool.recycle(buf);

        let reused = pool.acquire();
        assert_eq!(reused.as_ptr(), ptr_before);
        assert_eq!(reused[0], 42);

        // Wrong-sized buffer must be ignored cleanly.
        pool.recycle(vec![0u8; 8]);
        assert!(pool.rx.try_recv().is_err());
    }
}
