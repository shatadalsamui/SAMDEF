use anyhow::Result;
use ndarray::{Array, ArrayD};
use ort::{inputs, session::Session, value::Value};

pub fn run_inference(session: &mut Session, input_tensor: Array<f32, ndarray::IxDyn>) -> Result<ArrayD<f32>> {
    // Convert ndarray to ort::Value
    let input_value = Value::from_array(input_tensor)?;
    
    // Run inference
    let outputs = session.run(inputs![input_value])?;
    
    // Extract the first output (assuming single output for YOLO)
    let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;
    let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
    let output_array = Array::from_shape_vec(ndarray::IxDyn(&shape_usize), data.to_vec())?;
    
    //println!("Output Tensor Shape: {:?}", output_array.shape());
    Ok(output_array)
}