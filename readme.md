# SAMNDEF_CORE

## Overview

**SAMNDEF_CORE** is a hybrid defense ISR (Intelligence, Surveillance, Reconnaissance) system, leveraging a "System of Experts" architecture. It integrates a high-performance Rust engine for data ingestion and tiling, a Python-based AI orchestration layer, a modern Iced dashboard UI, and shared Protobuf protocols for seamless communication.

---


## High-Level Architecture

- **engine_rust/**: Ingestion & Tiling (Rust, Kafka Producer)
- **brain_python/**: AI Orchestration & Cortex (Python, Kafka Consumer)
- **command_ui/**: Iced Dashboard (Rust)
- **protocols/**: Shared Protobuf Schemas
- **plan/**: Project planning and documentation
- **packages/**: Shared libraries and utilities
- **db_schemas/**: Database schema files (SQL, migrations, etc.)
- **kafka_configs/**: Kafka configuration files
- **redis_configs/**: Redis configuration files
- **arrow_configs/**: Apache Arrow schema/configuration files

**Data Bus:** Kafka (Dockerized, single-node for development)

**Spatial Memory:** PostGIS (Dockerized)

**Other Infrastructure:** Redis (caching, coordination), Apache Arrow (in-memory data format)

---


## Tech Stack

- **Rust** (engine, UI)
- **Python** (AI, inference)
- **Kafka** (event bus)
- **PostGIS** (spatial database)
- **Protobuf** (wire protocol)
- **Docker** (infrastructure)
- **Redis** (caching, coordination)
- **Apache Arrow** (in-memory data format)

---

## System of Experts Philosophy

Each subsystem is an "expert" in its domain, communicating via well-defined protocols and a robust event bus. This modular approach enables rapid iteration, clear team boundaries, and scalable intelligence fusion.

---

## Team Roles

- **Shatadal:** Systems & UI Lead
- **Sarnab:** AI & Inference Lead

---


## Getting Started

1. Clone the repository.
2. See `plan/` for project planning and milestones.
3. See `protocols/` for Protobuf schemas.
4. See subsystem folders for implementation details.
5. See `db_schemas/`, `kafka_configs/`, `redis_configs/`, and `arrow_configs/` for infrastructure and data format configurations.

---


## Architecture Diagram (Description)

```
[engine_rust] <--> [Kafka Bus] <--> [synapse_python]
      |                                   |
[command_ui]                        [PostGIS]
      |                                   |
 [Redis]                            [Arrow]
```
- Rust engine ingests and tiles data, sends via Kafka.
- Python cortex consumes, analyzes, and stores results.
- UI dashboard visualizes system state and intelligence.
- Redis and Arrow provide fast data access and in-memory analytics.

---

## License

[Specify your license here]