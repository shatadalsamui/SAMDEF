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
  <img src="https://www.dropbox.com/scl/fi/wmnhp9aykaw8bmn1njn9z/1434.png?raw=1" alt="Original 1" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/wcfz9i0sp99r9a6u9amf4/1434_annotated.png?raw=1" alt="Annotated 1" width="800"/>
</p>

<!-- Example 2 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/wm6sticm3c15djgh79zci/1464.png?raw=1" alt="Original 2" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/kftke9o2z6sz0rocj7bum/1464_annotated.png?raw=1" alt="Annotated 2" width="800"/>
</p>

<!-- Example 3 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/31a64vv4fu1rv9crt86d5/1471.png?raw=1" alt="Original 3" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/wmvshqa7yjfq0w3rf0cmt/1471_annotated.png?raw=1" alt="Annotated 3" width="800"/>
</p>

<!-- Example 4 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/uc5e330ytnv32dk4x451i/2016.png?raw=1" alt="Original 4" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/q5cu90uqkiz0v4rju05f3/2016_annotated.png?raw=1" alt="Annotated 4" width="800"/>
</p>

<!-- Example 5 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/zqqmnl36npy8adht2k5q1/2342.png?raw=1" alt="Original 5" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/8bunzg4ly0snt78jui46p/2342_annotated.png?raw=1" alt="Annotated 5" width="800"/>
</p>

<!-- Example 6 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/oj52biuuny3yyzuewpwzo/2363.png?raw=1" alt="Original 6" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/5x17nvosbnb7ohaacebtq/2363_annotated.png?raw=1" alt="Annotated 6" width="800"/>
</p>

<!-- Example 7 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/tsb0p6fy8gclzs5n64ua9/2411.png?raw=1" alt="Original 7" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/txyzx8m93dh18kfcdmklp/2411_annotated.png?raw=1" alt="Annotated 7" width="800"/>
</p>

<!-- Example 8 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/g3vl1kb6pgfvvgrtqs3ga/2473.png?raw=1" alt="Original 8" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/guit90tvjnqd3dbwnddth/2473_annotated.png?raw=1" alt="Annotated 8" width="800"/>
</p>

<!-- Example 9 -->
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/fulx97d1rwub787nfxku4/2613.png?raw=1" alt="Original 9" width="800"/>
</p>
<p align="center">
  <img src="https://www.dropbox.com/scl/fi/06pa59ekpreipu4d4ylbf/2613_annotated.png?raw=1" alt="Annotated 9" width="800"/>
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
