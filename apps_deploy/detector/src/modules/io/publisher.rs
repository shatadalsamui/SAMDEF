use crate::modules::data::payload::DetectionPayload;
use bincode;
use zenoh::Session;

pub struct ZenohPublisher {
    session: Session,
}

impl ZenohPublisher {
    /// Initializes a Zenoh session in async mode.
    pub async fn new() -> Self {
        let session = zenoh::open(zenoh::config::Config::default())
            .await
            .expect("Failed to open Zenoh session");
        ZenohPublisher { session }
    }

    /// Publishes a DetectionPayload to the appropriate Zenoh topic.
    pub async fn publish_detection(&self, payload: &DetectionPayload) {
        let payload_bytes = bincode::serialize(payload).expect("Failed to serialize payload");
        let file_name = std::path::Path::new(&payload.source_image)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let topic = format!("satellite/detections/{}", file_name);
        self.session
            .put(topic, payload_bytes)
            .await
            .expect("Failed to publish payload");
    }
}
