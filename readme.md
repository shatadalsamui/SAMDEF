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

Below is a table of clickable links to the original and annotated images, shown side by side for easy comparison.
To view the images at full size, right-click the link and select "Open link in new tab."
//here 

Below are embedded examples, with each original image shown above its annotated version for visual comparison.

**Original:**  
![Original 1](https://github.com/user-attachments/assets/54814e69-bf21-4d35-a38e-388d21318b8a)  
**Annotated:**  
![Annotated 1](https://github.com/user-attachments/assets/043f6e17-e39d-4ae6-8a7e-58a6a7a5526f)

**Original:**  
![Original 2](https://github.com/user-attachments/assets/20080ec0-0b25-4d79-b0cf-df864cb38dd9)  
**Annotated:**  
![Annotated 2](https://github.com/user-attachments/assets/05149a35-b828-4406-80e6-0756eb2dd9cb)

**Original:**  
![Original 3](https://github.com/user-attachments/assets/3ef1faf1-ef82-4899-8410-2dab9540be48)  
**Annotated:**  
![Annotated 3](https://github.com/user-attachments/assets/c3b00124-880d-4dc7-9882-73075440fba4)

**Original:**  
![Original 4](https://github.com/user-attachments/assets/930f3e0b-d8a8-461f-b548-41676348023f)  
**Annotated:**  
![Annotated 4](https://github.com/user-attachments/assets/f0286cb1-0d41-426f-9686-7657690b6093)

**Original:**  
![Original 5](https://github.com/user-attachments/assets/22ed6be8-89ce-4acb-bee6-80c63678a4f6)  
**Annotated:**  
![Annotated 5](https://github.com/user-attachments/assets/6e16d846-b21c-4bfd-9f80-1a5c91acf36c)

**Original:**  
![Original 6](https://github.com/user-attachments/assets/43e28f72-6fac-41b3-9cfd-7a646be6f72b)  
**Annotated:**  
![Annotated 6](https://github.com/user-attachments/assets/f079e294-f0ec-4359-b348-f892db6da3a9)

**Original:**  
![Original 7](https://github.com/user-attachments/assets/b8107134-cdbc-42d8-a8bd-3bc38ca4f260)  
**Annotated:**  
![Annotated 7](https://github.com/user-attachments/assets/9a4aab6d-e213-40d1-a018-6eadb2785555)

**Original:**  
![Original 8](https://github.com/user-attachments/assets/c22881fe-d039-4c0a-9ae8-02d21c89dda7)  
**Annotated:**  
![Annotated 8](https://github.com/user-attachments/assets/6a3e8fac-58a9-4355-b7bf-759738dafa88)

**Original:**  
![Original 9](https://github.com/user-attachments/assets/33254737-a98b-4864-aad7-f40de5a5a4a5)  
**Annotated:**  
![Annotated 9](https://github.com/user-attachments/assets/65f445cc-5e49-4bfe-b774-223ad82283d1)

| Original Image | Annotated Image |
|---|---|
| [1434.png](https://shatadalsamui.github.io/images/1434.png) | [1434_annotated.png](https://shatadalsamui.github.io/images/1434_annotated.png) |
| [1464.png](https://shatadalsamui.github.io/images/1464.png) | [1464_annotated.png](https://shatadalsamui.github.io/images/1464_annotated.png) |
| [1471.png](https://shatadalsamui.github.io/images/1471.png) | [1471_annotated.png](https://shatadalsamui.github.io/images/1471_annotated.png) |
| [2016.png](https://shatadalsamui.github.io/images/2016.png) | [2016_annotated.png](https://shatadalsamui.github.io/images/2016_annotated.png) |
| [2342.png](https://shatadalsamui.github.io/images/2342.png) | [2342_annotated.png](https://shatadalsamui.github.io/images/2342_annotated.png) |
| [2363.png](https://shatadalsamui.github.io/images/2363.png) | [2363_annotated.png](https://shatadalsamui.github.io/images/2363_annotated.png) |
| [2411.png](https://shatadalsamui.github.io/images/2411.png) | [2411_annotated.png](https://shatadalsamui.github.io/images/2411_annotated.png) |
| [2473.png](https://shatadalsamui.github.io/images/2473.png) | [2473_annotated.png](https://shatadalsamui.github.io/images/2473_annotated.png) |
| [2613.png](https://shatadalsamui.github.io/images/2613.png) | [2613_annotated.png](https://shatadalsamui.github.io/images/2613_annotated.png) |

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
