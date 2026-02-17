# SAMDEF YOLO Model Trainer Microservice Architecture & Data Flow

---

## 1. Overview: What the Model Trainer Does

The SAMDEF YOLO Model Trainer microservice is responsible for orchestrating the training of deep learning models for satellite object detection. It manages environment setup, data verification, model configuration, training execution, metrics extraction, and model export. The trainer ensures reproducible, high-performance training runs and prepares models for deployment in inference pipelines.

---

## 2. Technologies, Libraries, and Key Concepts

- **Python**: The primary implementation language for flexibility and ecosystem support.
- **PyTorch**: For deep learning model training and GPU acceleration.
- **Ultralytics YOLO**: For state-of-the-art object detection model architecture and training routines.
- **Torch**: For hardware diagnostics and tensor operations.
- **tqdm**: For progress bar management during training.
- **argparse**: For command-line interface and mode selection.
- **TurboJPEG**: For fast image compression (used in data preparation).
- **NumPy**: For numerical operations.
- **Confluent-Kafka**: Placeholder for future inference pipeline integration.
- **Custom Modules**: `cortex.trainer` for training orchestration, `utils.extract_metrics_from_bestpt.py` for metrics extraction.

---

## 3. High-Level Architecture Diagram (Textual)

```
+-------------------+      +-------------------+      +-------------------+      +-------------------+
|                   |      |                   |      |                   |      |                   |
|  Data & Config    | ---> |  Environment      | ---> |  Training Engine  | ---> |  Model Export     |
| (data.yaml, imgs) |      |  Diagnostics      |      |  (YOLO, PyTorch)  |      |  (ONNX, Metrics)  |
+-------------------+      +-------------------+      +-------------------+      +-------------------+
        |                        |                           |                          |
        v                        v                           v                          v
[Input Verification]   [GPU/Config Check]      [Training Loop]           [Export, Validation]
```

**Key Data Flow:**
- Data and configuration are verified.
- Environment is checked for GPU and data readiness.
- Training is executed using YOLO and PyTorch.
- Model is exported and metrics are extracted for validation.

---

## 4. Architecture Components & Module Definitions

### 4.1. Main Entry (`main.py`)
- Prints system banner and diagnostics.
- Uses `argparse` to select mode: `train` or `run`.
- Checks for GPU availability and data configuration.
- Initiates training or inference (inference logic is a placeholder).
- Handles exceptions and graceful shutdown.

### 4.2. Training Module (`cortex/trainer.py`)
- Verifies GPU hardware and model checkpoint existence.
- Loads YOLO model with specified weights.
- Configures training parameters (epochs, batch size, learning rate, augmentation, etc.).
- Executes training loop and saves checkpoints.
- Outputs training results and best model weights.

### 4.3. Metrics Extraction Utility (`utils/extract_metrics_from_bestpt.py`)
- Loads trained model weights (`best.pt`) using Ultralytics YOLO.
- Runs validation on specified data set.
- Prints validation results and extracted metrics for performance evaluation.

### 4.4. Configuration & Dependency Management
- `pyproject.toml`: Specifies required Python packages and dependencies.
- Virtual environment setup (`venv`) for reproducibility.

### 4.5. ReadMe & Documentation
- Provides quick commands for setup, training, exporting, and inference.
- Documents environment requirements and workflow.

---

## 5. Detailed Data Flow

### 5.1. Environment Setup

- Create and activate Python virtual environment.
- Install dependencies from `pyproject.toml`.
- Verify presence of data configuration file (`data.yaml`) and model weights.

### 5.2. System Diagnostics

- Check for available GPU and VRAM.
- Print hardware and configuration status.
- Exit gracefully if requirements are not met.

### 5.3. Training Execution

- Select `train` mode via command-line.
- Load YOLO model and configuration.
- Start training loop with specified parameters:
  - Epochs, batch size, image size, optimizer, learning rate, augmentation, etc.
- Save checkpoints and best model weights at regular intervals.
- Print progress and handle interruptions.

### 5.4. Model Export

- After training, export the best model to ONNX format for deployment.
- Use Ultralytics export command with appropriate flags (dynamic batch, opset, simplification).

### 5.5. Metrics Extraction

- Run `extract_metrics_from_bestpt.py` to validate the exported model.
- Print validation results and metrics for performance assessment.

### 5.6. Inference Mode (Placeholder)

- `run` mode is reserved for future implementation of inference pipeline (e.g., Kafka integration).

---

## 6. Component Roles and Responsibilities

- **Main Entry**: Coordinates workflow, manages modes, and handles diagnostics.
- **Training Module**: Orchestrates model training, configuration, and checkpointing.
- **Metrics Utility**: Validates trained models and extracts performance metrics.
- **Configuration Manager**: Ensures reproducibility and dependency management.
- **Documentation**: Guides users through setup, training, and export.

---

## 7. Data Flow Summary (Step-by-Step)

1. **Input**: Data configuration (`data.yaml`) and image tiles are provided.
2. **Environment Setup**: Virtual environment is created and dependencies installed.
3. **Diagnostics**: GPU and data configuration are verified.
4. **Training**: YOLO model is trained with specified parameters; checkpoints are saved.
5. **Export**: Best model is exported to ONNX format for deployment.
6. **Validation**: Metrics are extracted and printed for performance evaluation.
7. **Completion**: Results and weights are available for downstream inference.

---

## 8. Extending or Debugging the System

- **Adding New Models**: Update model weights and configuration paths.
- **Changing Training Parameters**: Modify arguments in `cortex/trainer.py`.
- **Supporting New Data Formats**: Extend data loading and preprocessing logic.
- **Debugging**: Use printed diagnostics, logs, and metrics for troubleshooting.
- **Performance Tuning**: Adjust batch size, augmentation, and optimizer settings.

---

This documentation provides a complete, detailed, and accessible overview of the SAMDEF YOLO Model Trainer microservice, its architecture, and its data flow. Any engineer should be able to understand, extend, or debug the system using this guide.