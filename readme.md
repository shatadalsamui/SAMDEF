# SAMDEF: High-Performance Edge Computer Vision System

<a href="https://github.com/user-attachments/assets/ad3f7a0d-a2a0-4783-bb32-b2e8836c4f26" target="_blank"><img src="https://github.com/user-attachments/assets/ad3f7a0d-a2a0-4783-bb32-b2e8836c4f26" alt="SAMDEF GIS Viewer (gui_gpui) - Multi-Class Satellite Object Detection" /></a>

*Native GIS Exploitation Viewer (`gui_gpui`): Real-time multi-class GPU vector detection (1,256 active targets) with live radar minimap over high-resolution GeoTIFF imagery (click to view full resolution).*

<a href="https://github.com/user-attachments/assets/9e9ac8ca-a99f-45ad-a47c-54137e0c3799" target="_blank"><img src="https://github.com/user-attachments/assets/9e9ac8ca-a99f-45ad-a47c-54137e0c3799" alt="SAMDEF GIS Viewer (gui_gpui) - Target Class Solo Isolation" /></a>

*Instant class soloing: Isolating 469 building footprints with zero GPU texture re-uploads and synchronized radar minimap blips (click to view full resolution).*

<a href="https://github.com/user-attachments/assets/a4848c7f-4fba-41ee-82fa-af0239383d62" target="_blank"><img src="https://github.com/user-attachments/assets/a4848c7f-4fba-41ee-82fa-af0239383d62" alt="SAMDEF GIS Viewer (gui_gpui) - Sub-Pixel Zoom & Target Inspector HUD" /></a>

*1600% sub-pixel magnification & pinned Target Inspector HUD: Inspecting construction assets with GPU-shaped class/confidence badges, ground dimensions (8.6m x 7.7m), and WGS-84 coordinates (click to view full resolution).*

<a href="https://github.com/user-attachments/assets/8d5b65b3-2b49-4116-92bd-b0cd3b4987a6" target="_blank"><img src="https://github.com/user-attachments/assets/8d5b65b3-2b49-4116-92bd-b0cd3b4987a6" alt="SAMDEF GIS Viewer (gui_gpui) - Target Exploitation Table & Heavy Vehicle Inspection" /></a>

*Target Exploitation Table & class filtering: Inspecting isolated long-truck detections at 873% zoom with live tabular telemetry and one-click GeoJSON/CSV exports (click to view full resolution).*

<a href="https://github.com/user-attachments/assets/feea03ec-4c42-4655-a05a-223d38f206ca" target="_blank"><img src="https://github.com/user-attachments/assets/feea03ec-4c42-4655-a05a-223d38f206ca" alt="SAMDEF GIS Viewer (gui_gpui) - Interactive Table Focus & Highway Target Tracking" /></a>

*One-click target focus: Jumping directly from the exploitation table to highway vehicle detections (#2132, 186.9 m2 footprint) with live radar frustum tracking (click to view full resolution).*

<a href="https://github.com/user-attachments/assets/efd453e0-1170-4e1d-8f95-64031d872b51" target="_blank"><img src="https://github.com/user-attachments/assets/efd453e0-1170-4e1d-8f95-64031d872b51" alt="SAMDEF GIS Viewer (gui_gpui) - Geodesic Haversine Distance & Azimuth Ruler" /></a>

*Interactive Geodesic Ruler (`Ruler [M]`): Measuring true WGS-84 Haversine ground distance and forward azimuth (121.0 m | 243.7°) directly across satellite imagery (click to view full resolution).*

> **GitHub Repository:** [https://github.com/shatadalsamui/SAMDEF](https://github.com/shatadalsamui/SAMDEF)  
> **Module Documentation:** [Detector](docs/architecture/detector.md) | [DB Processor](docs/architecture/dbprocessor.md) | [Ingestor](docs/architecture/ingestor_training.md) | [Model Trainer](docs/architecture/model_trainer.md) | [GIS Viewer](docs/architecture/gui_gpui.md)

---

## Table of Contents
- [Overview](#overview)
- [Use Cases](#use-cases)
- [Inference Examples](#inference-examples)
- [Architecture](#architecture)
- [Workflow & Data Flow](#workflow--data-flow)
- [Modular Apps](#modular-apps)
- [Tech Stack](#tech-stack)
- [Installation & Setup](#installation--setup)
- [Usage](#usage)

---

## Overview

**SAMDEF** is a high-performance, modular monolith platform designed for efficient edge computer vision and massive-scale image processing. Engineered to operate **fully air-gapped** and optimized for on-premise, mission-critical execution without reliance on cloud or external SaaS infrastructure, it enables high-throughput automated object detection on exceptionally large imagery (such as massive GeoTIFFs).

The system automates the data-intensive groundwork of transforming raw pixels into structured, geospatial knowledge. By leveraging a Zero-Wait GPU Pipeline, it ensures that while one batch of imagery is analyzed, the next is already being pre-processed in memory, maximizing hardware utilization. It provides operators with the critical "What" and "Where" by mapping every detected object (e.g., vehicles, infrastructure, specific assets) to its exact Global Geospatial Coordinates and suppressing duplicates via Spatial Grid NMS.

By delivering high-fidelity metadata and object locations directly to a local, air-gapped database in near real-time, SAMDEF acts as a high-speed programmatic scout for defense, intelligence, and remote tactical environments that are far too large for manual human review.

## Use Cases

SAMDEF is optimized for massive geospatial and satellite imagery—processing directly on the edge, not on a remote server—making it highly effective across defense, civilian, and commercial applications.

### Off-Grid Surveillance & Remote Operations
SAMDEF is engineered for remote environments where cloud connectivity is unavailable or compromised. It is highly effective for processing satellite or high-altitude drone imagery directly at the edge:
- **Air-Gapped & Offline Execution:** Operates 100% locally on isolated field hardware with zero external network dependencies, requiring no internet connectivity to process massive GeoTIFFs.
- **High-Altitude Analytics:** Rapidly scans imagery from drones or satellites, converting raw pixels into actionable intelligence in near real-time.
- **Geospatial Contextualization:** Automatically maps detected objects of interest (e.g., vehicles, equipment, structures) to exact global coordinates, providing operators with instant situational awareness in the field.

### Civilian and Commercial Applications
- **Urban & Town Planning:** Automating the detection of buildings, road networks, and infrastructure changes over time to monitor urban sprawl and inform zoning decisions.
- **Traffic & Logistics Analysis:** Identifying and counting vehicles across massive areas (e.g., highways, shipping ports, distribution centers) to analyze traffic density, parking utilization, and supply chain activity.
- **Disaster Response & Damage Assessment:** Rapidly scanning affected regions to locate structural damage, blocked roads, and temporary settlements after natural disasters.
- **Environmental & Maritime Monitoring:** Tracking changes in vegetation, monitoring coastal erosion, or detecting illicit maritime activity.
- **Agriculture:** Surveying expansive farmlands to identify crop health boundaries, water usage, and structural assets (like silos and tractors).

- **Deployment modules** handle huge GeoTIFF image processing, object detection, and visualization.
- **Training modules** manage data preparation and model training to enhance detection accuracy.

## Inference Examples

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

Below is a table of clickable links to the original and annotated images, shown side by side for easy comparison.
To view the images at full size, right-click the link and select "Open link in new tab."

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

### Inference Timing Results (YOLO26s, FP16)

- **Dataset:** 281 images (~3000×3000 pixels each, 0.3m GSD)
- **Total area:** 227.61 km²
- **Hardware Profile:** CUDA Core utilization maintained at 85–95% with decoupled 3-stage pipelining. VRAM footprint remained under 1.5GB for Batch Size 2.

| Provider | Model        | Batch Size | Total Time (sec) | Time per km² (sec) | Notes                        |
|----------|--------------|------------|------------------|--------------------|------------------------------|
| GPU      | YOLO26s FP16 | 2          | 44.59            | 0.20               | RTX 4060, full FP16 accel    |
| CPU      | YOLO26s FP16 | 4          | 999              | 4.39               | i9-13900HX, FP16 emulated    |

**Specs:** i9-13900HX, 48GB RAM, RTX 4060  
**Dataset:** 281 images, 3000×3000 px, 0.3m GSD, 227.61 km² total

- **Interpretation:**  
  - CPU is ~22x slower than GPU for this workload, which matches expectations for FP16 emulation on a high-end CPU.
  - Time per km² is a useful metric for scaling to larger areas or comparing with other systems.
  - Both CPU and GPU performance are strong for a lightweight model like YOLO26s.

### Model Metrics (Validation Set)

| Class | Images | Instances | Precision (P) | Recall (R) | mAP50 | mAP50-95 |
|---|---|---|---|---|---|---|
| all | 1289 | 127500 | 0.550 | 0.427 | 0.442 | 0.241 |
| Light-Vehicle | 697 | 50191 | 0.713 | 0.783 | 0.770 | 0.349 |
| Boxy-Truck | 542 | 4851 | 0.512 | 0.402 | 0.425 | 0.227 |
| Long-Trucks | 502 | 4820 | 0.473 | 0.350 | 0.363 | 0.180 |
| Small-Boat | 84 | 748 | 0.629 | 0.683 | 0.650 | 0.331 |
| Large-Ship | 80 | 315 | 0.479 | 0.302 | 0.360 | 0.193 |
| Fixed-Wing | 74 | 200 | 0.872 | 0.714 | 0.803 | 0.462 |
| Helicopter | 4 | 5 | 0.277 | 0.200 | 0.197 | 0.122 |
| Building | 783 | 60680 | 0.583 | 0.655 | 0.635 | 0.328 |
| Storage-Tank | 78 | 407 | 0.719 | 0.369 | 0.468 | 0.271 |
| Railway | 24 | 586 | 0.760 | 0.691 | 0.712 | 0.428 |
| Engineering-Machinery | 264 | 1433 | 0.537 | 0.426 | 0.457 | 0.260 |
| Tower-Pylon | 94 | 160 | 0.802 | 0.388 | 0.452 | 0.295 |
  
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
├── gui_gpui/
│   ├── README.md
│   ├── build.rs
│   ├── src/
│   │   ├── app/
│   │   ├── io/
│   │   ├── model/
│   │   ├── ui/
│   │   └── viewer/
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

```mermaid
flowchart TD
    A[Raw GeoTIFFs] --> B[Detector Module]
    B -->|Zenoh Pub/Sub| C[DB Processor]
    C -->|SQL Insert| D[(PostgreSQL Database)]
    B -->|Output JSON| E[Post Processor]
    A --> E
    E --> F[Annotated Images]
    B -->|Output JSON| G[GIS Viewer - gui_gpui]
    A --> G
```

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
1. Raw imagery data is ingested and labeled using the Ingestor module.
2. Labeled datasets are used by the Model Trainer for YOLO model training.
3. Trained models are exported in ONNX format for deployment.

### Deployment Phase
1. GeoTIFF images are processed by the Detector module using virtual tiling.
2. Detections are performed on GPU using ONNX models.
3. Results are stored in the database via DB Processor.
4. Post Processor visualizes detections by annotating images with bounding boxes.
5. Interactive GIS Viewer (`gui_gpui`) enables analysts to inspect raw GeoTIFFs with real-time GPU vector overlays, sub-pixel pan/zoom, radar minimap, geodesic ruler, class soloing, and multi-format GIS exports.

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
- **Database Schema:** Tables and relationships are defined in `src/modules/schema/init.sql`.
- **Folder structure:**
  - `src/`: Rust source code for database and Zenoh integration.
  - `docs/`: Documentation for database schema and integration.

#### gui_gpui (Interactive GIS Viewer & Exploitation GUI)
- **Purpose:** Native, GPU-accelerated desktop GIS exploitation workstation built on GPUI (Vulkan/Blade) for interactive inspection and tactical analysis of ultra-high-resolution satellite imagery (GeoTIFF) with vector detection overlays.
- **How it works:**
  - Fast-decodes multi-gigabyte Planar Configuration 2 and chunky GeoTIFF strips directly into zero-copy BGRA GPU textures.
  - Renders ordered immediate-mode GPU passes (`paint_layer`) for the satellite raster, dynamic hollow neon vector bounding boxes, GPU-shaped class/confidence badges, and geodesic ruler overlays with zero texture re-uploads.
  - Implements sub-pixel cursor-invariant zooming, `1:1` native pixel inspection, interactive radar minimap with frustum tracking, geodesic Haversine distance/bearing measurement, and GDAL 6-parameter WGS-84 coordinate projection.
  - Features instant file search, closed-by-default dropdown selection, floating target inspector HUD, collapsible detection table drawer, and GeoJSON/CSV/Markdown intelligence exporters.
- **Key features:** Zed / VS Code Dark styling, client-side Wayland/X11 window decorations, 8-class visibility toggles + Solo isolation, interactive confidence slider, live WGS-84 Decimal Degrees / DMS / GSD status bar, and one-click GIS exports.
- **Folder structure:**
  - `src/app/`: Application state, async GeoTIFF/JSON loader, event dispatch, and text input handling.
  - `src/io/`: Planar/chunky TIFF strip decoder, dataset scanner, and GeoJSON/CSV/Markdown exporters.
  - `src/model/`: Detection schema entities and 8-class neon color taxonomy.
  - `src/ui/`: Client-side titlebar, toolbar, status bar, radar minimap, target inspector, table drawer, and modular sidebar.
  - `src/viewer/`: Sub-pixel coordinate math, priority hit-testing, and 5-layer GPU rendering pipeline.
  - Documentation: `README.md` and [`docs/architecture/gui_gpui.md`](docs/architecture/gui_gpui.md).

---

### Training Modules

#### ingestor
- **Purpose:** Data ingestion and preparation for model training.
- **How it works:** 
  - Processes raw imagery data, balances datasets, parses and converts labels, and prepares images.
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


## Tech Stack

- **Languages:** Rust (deployment, data processing, native GUI), Python (AI/ML training)
- **Frameworks:** YOLO (training), ONNX (inference), GPUI (native Rust GPU UI via Vulkan/Blade)
- **Databases:** PostgreSQL
- **Messaging:** Zenoh (peer mode)
- **Hardware Acceleration:** CUDA GPUs
- **Image Formats:** GeoTIFF (including Planar Configuration 2)

---

## Installation & Setup

### System Dependencies

> **Operating System:** SAMDEF is designed and optimized specifically for Linux-based operating systems (Ubuntu/Debian recommended).

SAMDEF requires several OS-level dependencies for massive-scale image processing, database operations, and GPU acceleration. Ensure you have the following installed:
- **Core Languages:** Rust (via `rustup`), Python 3.9+ (with `python3-venv`), and PostgreSQL.
- **Hardware Acceleration:** CUDA Toolkit (11.x, 12.x, or 13.x) and cuDNN (v8 or v9). *(Note: The ONNX Runtime binaries are fetched automatically by the Rust build process, with native CUDA 13 support enabled in `ort` 2.0.0-rc.13).*
- **C Libraries:** `libgdal-dev` (for GeoTIFF parsing), `libturbojpeg0-dev` (for high-speed image encoding/decoding), and `pkg-config`.

### Setup Instructions
1. **Environment Variables:** 
   - Navigate to each microservice (`apps_deploy/detector`, `apps_deploy/post_processor`, `apps_deploy/db_processor`, `apps_training/ingestor`, `apps_training/yolo_model_trainer`).
   - Copy the `.env.example` file to `.env` (`cp .env.example .env`).
   - Update the paths and database credentials in the `.env` file to match your local system.
3. **Database Initialization:** Ensure your PostgreSQL instance is running. The `db_processor` will automatically initialize the schema on its first run.
4. **Python Dependencies:** For the YOLO trainer, set up the virtual environment:
   ```bash
   cd apps_training/yolo_model_trainer
   python3 -m venv venv
   source venv/bin/activate
   pip install -e .
   ```

---

## Usage

### Deployment Pipeline
To run the main detection and deployment pipeline, launch the Rust microservices:

1. **Start the DB Processor (Zenoh Listener):**
   ```bash
   cd apps_deploy/db_processor
   cargo run --release
   ```
2. **Start the Detector (GPU Inference):**
   ```bash
   cd apps_deploy/detector
   cargo run --release
   ```
3. **Run the Post Processor (Image Annotation):**
   ```bash
   cd apps_deploy/post_processor
   cargo run --release
   ```
4. **Launch the Interactive GIS Viewer (gui_gpui):**
   ```bash
   cd apps_deploy/gui_gpui
   cargo run --release
   ```

### Training Pipeline
To train new models on custom datasets:

1. **Ingest Data:**
   ```bash
   cd apps_training/ingestor
   cargo run --release
   ```
2. **Train Model:**
   ```bash
   cd apps_training/yolo_model_trainer
   source venv/bin/activate
   python src/main.py train
   ```

---

