#[derive(Debug)]
pub struct InferenceTask {
    pub image_data: Vec<u8>,
    pub source_path: String,
    pub global_offset_x: i32,
    pub global_offset_y: i32,
}

#[derive(Debug)]
pub enum PipelineMessage {
    Process(InferenceTask),
    EndOfFile {
        source_path: String,
        geo_transform: [f64; 6],
        width: u32,
        height: u32,
        expected_tiles: usize,
    },
    Terminate,
}
