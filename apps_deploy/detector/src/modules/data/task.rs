#[derive(Debug)]
pub struct InferenceTask {
    pub image_data: Vec<u8>,
    pub source_path: String,
    pub global_offset_x: i32,
    pub global_offset_y: i32,
    pub geo_transform: [f64; 6],
}