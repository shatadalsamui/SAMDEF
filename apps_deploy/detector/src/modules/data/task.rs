#[derive(Debug)]
pub struct InferenceTask {
    pub image_data: Vec<u8>,
    pub source_path: String,
    pub global_offset_x: i32,
    pub global_offset_y: i32,
    pub geo_transform: [f64; 6],
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileMetadata {
    pub source_path: String,
    pub width: usize,
    pub height: usize,
    pub geo_transform: [f64; 6],
    pub expected_tiles: usize,
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
