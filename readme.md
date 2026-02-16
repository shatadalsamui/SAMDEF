# SAMDEF: High-Performance Edge Platform for ISR

---

## Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Workflow & Data Flow](#workflow--data-flow)
- [Modular Apps](#modular-apps)
  - [Deployment Modules](#deployment-modules)
  - [Training Modules](#training-modules)
- [Tech Stack](#tech-stack)
- [Usage](#usage)
- [License](#license)

---

## Overview

**SAMDEF** is a high-performance, modular monolith platform for Intelligence, Surveillance, and Reconnaissance (ISR) data processing at the edge. It is designed for real-time geospatial imagery analysis, object detection, and continuous model improvement, all running on-premise without reliance on cloud or SaaS infrastructure.

- **Deployment modules** handle live GeoTIFF image processing, object detection, and visualization.
- **Training modules** manage data preparation and model training to enhance detection accuracy.

## Folder Structure
### apps_deploy
```
apps_deploy/
├── db_processor/
│   ├── docs/
│   ├── src/
│   │   └── modules/
├── detector/
│   ├── docs/
│   ├── model/
│   ├── src/
│   │   └── modules/
├── post_processor/
│   ├── src/
```
### apps_training
```
apps_training/
├── ingestor/
│   ├── src/
│   │   └── modules/
├── yolo_model_trainer/
│   ├── build/
│   ├── src/
│   │   ├── cortex/
│   │   ├── models/
│   │   ├── samdef_brain.egg-info/
│   │   ├── utils/
│   │   └── SAMDEF_ISR/
│   ├── venv/
````

## Architecture

SAMDEF is structured as a modular monolith: all core logic resides in a unified codebase, with each module responsible for a distinct function. Modules are decoupled for maintainability and communicate via Zenoh in peer-to-peer mode, ensuring efficient and scalable data exchange.

**Key architectural highlights:**
- **Modular Monolith:** Unified codebase with clear module boundaries.
- **Decoupled Modules:** Each module can be developed and maintained independently.
- **Zenoh Peer Communication:** All inter-module communication is handled via Zenoh in peer mode.
- **High Performance:** Rust for deployment/data processing, Python for AI/ML training.
- **Database Integration:** PostgreSQL for persistent storage of detection results and metadata.

---

## Workflow & Data Flow

The SAMDEF workflow consists of two main phases: **training** and **deployment**.

### Training Phase
1. Raw ISR data is ingested and labeled using the Ingestor module.
2. Labeled datasets are used by the Model Trainer for YOLO model training.
3. Trained models are exported in ONNX format for deployment.

### Deployment Phase
1. GeoTIFF images are processed by the Detector module using virtual tiling.
2. Detections are performed on GPU using ONNX models.
3. Results are stored in the database via DB Processor.
4. Post Processor visualizes detections by annotating images with bounding boxes.

---

## Modular Apps

### Deployment Modules

#### detector
- **Purpose:** High-performance object detection on large-scale geospatial imagery (GeoTIFF).
- **How it works:** 
  - Uses a producer-consumer pattern with virtual tiling to break large images into manageable tiles.
  - Tiles are preprocessed and batched for GPU inference using a YOLOv26s ONNX model.
  - Results are post-processed to global coordinates and output as JSON.
- **Key features:** Virtual tiling, parallel CPU preprocessing, GPU batching, global coordinate mapping, efficient memory usage.
- **Folder structure:**
  - `src/`: Rust source code for tiling, inference, and postprocessing.
  - `model/`: Contains the ONNX model used for inference.
  - `docs/`: Architecture and planning documentation.

#### post_processor
- **Purpose:** Visualization of detection results.
- **How it works:** 
  - Reads JSON outputs from the detector.
  - Loads original GeoTIFF images and draws bounding boxes with class-specific colors.
  - Saves annotated images for review and analysis.
- **Key features:** High-fidelity image annotation, class-specific color coding, efficient handling of large images.
- **Folder structure:**
  - `src/`: Rust source code for image annotation and visualization.

#### db_processor
- **Purpose:** Database operations for detection results.
- **How it works:** 
  - Stores detection metadata in PostgreSQL.
  - Publishes and receives data via Zenoh peer-to-peer communication.
  - Enables querying and persistence of historical detections.
- **Key features:** Asynchronous database interactions, real-time data publishing, robust error handling.
- **Folder structure:**
  - `src/`: Rust source code for database and Zenoh integration.
  - `docs/`: Documentation for database schema and integration.

---

### Training Modules

#### ingestor
- **Purpose:** Data ingestion and preparation for model training.
- **How it works:** 
  - Processes raw ISR data, balances datasets, parses and converts labels, and prepares images.
  - Outputs labeled datasets suitable for machine learning.
- **Key features:** Automated dataset balancing, label format conversion, image preprocessing utilities.
- **Folder structure:**
  - `src/`: Rust source code for data ingestion, balancing, and preprocessing.

#### yolo_model_trainer
- **Purpose:** Training of object detection models using YOLO.
- **How it works:** 
  - Trains YOLO models on labeled datasets.
  - Supports exporting trained models to ONNX format for deployment.
  - Includes scripts for diagnostics, metric extraction, and model export.
- **Key features:** Configurable training pipelines, performance monitoring, ONNX export, integration with ingestor outputs.
- **Folder structure:**
  - `src/`: Python source code for training, evaluation, and export.
  - `build/`: Build artifacts and logs.
  - `venv/`: Python virtual environment for dependencies.

---
## Inference Examples

<!-- Example 1 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1I3NC_L90FkkwJYp0a8qdfq5HWpT9_jcy" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1I3NC_L90FkkwJYp0a8qdfq5HWpT9_jcy" alt="Original 1" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1_PrYFdD512fMXMlqjCkGtAvN07-Tz8eE" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1_PrYFdD512fMXMlqjCkGtAvN07-Tz8eE" alt="Annotated 1" width="800"/>
  </a>
</p>

<!-- Example 2 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1mrrexN9Zjj5fbdNYXL48M9Y4RH8s4ftG" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1mrrexN9Zjj5fbdNYXL48M9Y4RH8s4ftG" alt="Original 2" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1x25Gdt1bePWUUlhfBGahnnFpkOikMhWE" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1x25Gdt1bePWUUlhfBGahnnFpkOikMhWE" alt="Annotated 2" width="800"/>
  </a>
</p>

<!-- Example 3 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1ar86yUXqa7jKuEcH1LItjXCyoJcDjLlh" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1ar86yUXqa7jKuEcH1LItjXCyoJcDjLlh" alt="Original 3" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1PQdU61qQCKkvPk9G7NOuWi72yhoGA8Oc" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1PQdU61qQCKkvPk9G7NOuWi72yhoGA8Oc" alt="Annotated 3" width="800"/>
  </a>
</p>

<!-- Example 4 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=18TGDYTibh1gepL8GygAqfnrXL7Nv5vc9" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=18TGDYTibh1gepL8GygAqfnrXL7Nv5vc9" alt="Original 4" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1Bx7tU3TSXu5Y9udNxfJNr33n3Huv29YW" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1Bx7tU3TSXu5Y9udNxfJNr33n3Huv29YW" alt="Annotated 4" width="800"/>
  </a>
</p>

<!-- Example 5 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1myGQEGM0jNnGfcoTgnF5TSet21nQjlGE" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1myGQEGM0jNnGfcoTgnF5TSet21nQjlGE" alt="Original 5" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1dXyBWzfvqb9GrHcWm4kidX7XpMlcmTqv" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1dXyBWzfvqb9GrHcWm4kidX7XpMlcmTqv" alt="Annotated 5" width="800"/>
  </a>
</p>

<!-- Example 6 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1hfTugCYAa9qRot_1VfkvQDG7i41JLWLk" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1hfTugCYAa9qRot_1VfkvQDG7i41JLWLk" alt="Original 6" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1LIzM8ueYkO5-ojtxuHwc5kOYhCaetc3C" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1LIzM8ueYkO5-ojtxuHwc5kOYhCaetc3C" alt="Annotated 6" width="800"/>
  </a>
</p>

<!-- Example 7 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1X8xk4-8mnudJQAU4OFS2TohW-sopWK3G" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1X8xk4-8mnudJQAU4OFS2TohW-sopWK3G" alt="Original 7" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1BQByY_SxBeVCd3BvZC32WPifUFU5xq-F" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1BQByY_SxBeVCd3BvZC32WPifUFU5xq-F" alt="Annotated 7" width="800"/>
  </a>
</p>

<!-- Example 8 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1MlMEvAb20kcFavhl5Rp0LT2T0Ex4P9JB" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1MlMEvAb20kcFavhl5Rp0LT2T0Ex4P9JB" alt="Original 8" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1tXpjPnjyycQ5CzxUiwsBeOdE6rl1He6Z" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1tXpjPnjyycQ5CzxUiwsBeOdE6rl1He6Z" alt="Annotated 8" width="800"/>
  </a>
</p>

<!-- Example 9 -->
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=1XUFDQVnXvOu_Ee7MzoNhda43AL6acr_m" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=1XUFDQVnXvOu_Ee7MzoNhda43AL6acr_m" alt="Original 9" width="800"/>
  </a>
</p>
<p align="center">
  <a href="https://drive.google.com/uc?export=view&id=19Op1TTKu2C6xb8CTNV0wmTlhjizHGabQ" target="_blank">
    <img src="https://drive.google.com/uc?export=view&id=19Op1TTKu2C6xb8CTNV0wmTlhjizHGabQ" alt="Annotated 9" width="800"/>
  </a>
</p>

## Tech Stack

- **Languages:** Rust (deployment, data processing), Python (AI/ML training)
- **Frameworks:** YOLO (training), ONNX (inference)
- **Databases:** PostgreSQL
- **Messaging:** Zenoh (peer mode)
- **Hardware Acceleration:** CUDA GPUs
- **Image Formats:** GeoTIFF

---

## Usage

- **Start training:** Use the Model Trainer with datasets prepared by the Ingestor.
- **Deploy modules:** Launch Detector, DB Processor, and Post Processor as needed.
- **Monitor:** Use logs and Zenoh topics for real-time monitoring.

---

## License

This project is **proprietary software