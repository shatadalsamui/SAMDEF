
# SAMDEF Apps

## Overview

This repository contains the core applications for the SAMDEF system, divided into deployment and training components. The deployment apps handle real-time processing of ISR (Intelligence, Surveillance, Reconnaissance) data, while the training apps manage data ingestion and model training.

## Apps Deploy

### Pre Processor
A Rust microservice responsible for pre-processing input images. It includes modules for image utilities, tiling images into manageable chunks, and watching directories for new files. This app prepares data for downstream processing by creating tiles and handling file system events.

### Detector
A Rust application that performs object detection using an ONNX model (best.onnx). It includes modules for inference, pre-processing inputs, and post-processing outputs. The detector analyzes tiled images to identify anomalies or objects of interest in real-time.

### Post Processor
A Rust microservice for post-processing detection results. It handles further analysis or visualization of the outputs from the detector, potentially including image rendering, result aggregation, or additional computations.

## Apps Training

### Ingestor
A Rust application for data ingestion and preparation. It includes tools for balancing datasets, parsing labels, labeling images, and image utilities. This app processes raw data to create labeled datasets suitable for training machine learning models.

### Model Trainer
A Python application for training machine learning models. It uses frameworks like YOLO (with weights like yolo26n.pt and yolo26s.pt) and includes a trainer module. The app handles model training runs, constraint files, and utility scripts for extracting metrics from trained models.

## Tech Stack

- **Deployment Apps:** Rust for high-performance, real-time processing
- **Training Apps:** Rust for data processing, Python for AI/ML training
- **Models:** ONNX for inference, PyTorch/YOLO for training

## License

[Specify your license here]