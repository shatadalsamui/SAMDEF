
# SAMDEF Apps

## Overview

This repository contains the core applications for the SAMDEF system, divided into deployment and training components. The deployment apps handle real-time processing of ISR (Intelligence, Surveillance, Reconnaissance) data, while the training apps manage data ingestion and model training.

## Apps Deploy

### Detector
A Rust application that performs object detection on GeoTIFF images using virtual tiling. It processes large geospatial imagery by generating overlapping tiles on-the-fly, runs inference with a YOLOv26s ONNX model on CUDA GPUs, and outputs detection results in JSON format with global coordinates.

### Post Processor
A Rust application for visualizing detection results. It reads JSON outputs from the detector, loads original GeoTIFF images, draws bounding boxes with class-specific colors, and saves annotated images.

## Apps Training

### Ingestor
A Rust application for data ingestion and preparation. It includes tools for balancing datasets, parsing labels, labeling images, and image utilities. This app processes raw data to create labeled datasets suitable for training machine learning models.

### Model Trainer
A Python application for training machine learning models. It uses YOLO framework (with weights like yolo26n.pt) and includes a trainer module. The app handles model training runs, constraint files, and utility scripts for extracting metrics from trained models.

## Tech Stack

- **Deployment Apps:** Rust for high-performance processing
- **Training Apps:** Rust for data processing, Python for AI/ML training
- **Models:** ONNX for inference, PyTorch/YOLO for training

## License

[Specify your license here]