use crate::modules::writer::payload::DetectionPayload;
use crate::modules::writer::queries::insert_detections;
use bincode;
use sqlx::PgPool;
use zenoh::Session;

pub struct ZenohSubscriber {
    session: Session,
    pool: PgPool,
}

impl ZenohSubscriber {
    pub async fn new(pool: PgPool) -> Self {
        let session = zenoh::open(zenoh::Config::default())
            .await
            .expect("Failed to open Zenoh session");
        ZenohSubscriber { session, pool }
    }

    pub async fn listen_and_print(&self) {
        let key_expr = "satellite/detections/**";
        let subscriber = self
            .session
            .declare_subscriber(key_expr)
            .await
            .expect("Failed to declare Zenoh subscriber");

        println!("Listening for detection payloads on Zenoh...");

        while let Ok(sample) = subscriber.recv_async().await {
            let payload_bytes = &*sample.payload().to_bytes();
            match bincode::deserialize::<DetectionPayload>(payload_bytes) {
                Ok(payload) => {
                    println!("Received payload: {:?}", payload);
                    if let Err(e) = insert_detections(&self.pool, &payload).await {
                        eprintln!("Failed to insert detections: {}", e);
                    }
                    if let Err(e) = crate::modules::writer::queries::insert_detections_pixels(
                        &self.pool, &payload,
                    )
                    .await
                    {
                        eprintln!("Failed to insert pixel detections: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to deserialize payload: {}", e),
            }
        }
    }
}
