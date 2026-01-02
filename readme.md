
# SAMDEF_CORE: Tactical ISR System of Experts

## Overview

**SAMDEF_CORE** is an industrial-grade defense ISR (Intelligence, Surveillance, Reconnaissance) system. It uses a Decoupled Shared-Memory Architecture to process 100GB+ datasets and 10Gbps line rates. The system utilizes a "Scout & Sniper" recursive tiling strategy to detect, classify, and provide situational awareness in real-time.

---


## High-Level Architecture (The 5-App Pipeline)

- **apps/ingestor/**: Rust microservice for mmap logic, tiling, Arrow buffer creation
- **apps/outgestor/**: Rust microservice for intelligence fusion, Zstd/Arrow compression
- **apps/brain_python/**: Python microservice for AI/ML inference
  - **models/**: Scout (Fast) and Sniper (Deep) models
  - **cortex/**: Kafka consumer/producer & feedback logic
  - **utils/**: pyarrow and protobuf helpers
- **apps/command_ui/**: Tactical Dashboard (Iced Rust)
  - **src/assets/**: Local tile/snippet cache
  - **src/view/**: Tactical map & Detail cards
  - **src/bus/**: Kafka/Arrow consumer
- **apps/db_processor/**: Rust microservice for PostGIS persistence
  - **src/**: PostGIS/SQL transaction logic
- **protocols/**: Shared Communication (Protobuf)
  - **detection.proto**: Metadata & Arrow Pointers
  - **feedback.proto**: High-Res Detail Requests
- **arrow_configs/**: Apache Arrow Schemas
  - **scout_tile.json**: Schema for low-res grid
  - **sniper_crop.json**: Schema for high-res vehicle crops
- **db_schemas/**: PostGIS / SQL Migrations
- **kafka_configs/**: Broker & Topic configurations (Zstd enabled)
- **redis_configs/**: Caching & coordination settings
- **docker/**: Infrastructure-as-Code

---

## The "Scout & Sniper" Data Flow

1. Ingestor maps 100GB file → Sends Low-Res Arrow Tiles via Kafka.
2. Brain detects anomaly → Requests High-Res Crop from Ingestor via Feedback Topic.
3. Ingestor fetches specific pixels → Sends High-Res Sniper Crop to Brain.
4. Brain confirms ID (e.g., "T-90 Tank") → Sends Final Outcome to Outgestor.
5. Outgestor bundles Sniper image + 1km Context photo → Compresses (Zstd/Arrow) → Sends to Command UI.
6. DB Processor saves spatial metadata and image pointers to PostGIS.

---

## Tech Stack

- **Core:** Rust (System Performance), Python (AI/ML Inference)
- **Format:** Apache Arrow (In-memory zero-copy), Protobuf (Wire protocol)
- **Transport:** Kafka (Asynchronous Event Bus) with Zstd compression
- **Storage:** PostGIS (Spatial Memory), Redis (Hot-cache for UI assets)
- **UI:** Iced (Rust-native, GPU-accelerated GUI)

---

## Architecture Diagram (Enhanced)

```
[   Ingestor (Rust)   ] <---Feedback Loop---> [    Brain (Python)    ]
                               |                                          |
             (Arrow/mmap)                                (Inference)
                               |                                          |
             [  Kafka Bus (Zstd Compressed) / Redis Hot Cache ]
                               |                       |                  |
[   Outgestor (Rust)  ] --> [ DB Processor ] --> [ PostGIS ]
                               |
[   Command UI (Iced) ] <-- (Remote Tactical Link)
```

---

## Team Roles

- **Shatadal:** Systems Architect & UI Lead (Ingestor, Outgestor, UI, Kafka Optimization)
- **Sarnab:** AI & Inference Lead (Brain/Cortex Logic, Model Recursive Tiling)

---

## License

[Specify your license here]