#[derive(Debug)]
pub struct InferenceTask {
    pub image_data: Vec<u8>,
    pub global_offset_x: f32,
    pub global_offset_y: f32,
    pub tile_filename: String,
}