# SAMDEF YOLO Model Trainer

## 1. Set up a Python Virtual Environment

```bash
cd apps_training/yolo_model_trainer
python3 -m venv venv
source venv/bin/activate
```

## 2. Install Dependencies

Ensure your virtual environment is active, then install the package in editable mode:

```bash
pip install -e .
```

*Note: For ONNX export, `onnx` and `onnxslim` will automatically be installed or updated by Ultralytics if not already present.*

## 3. Base YOLO Weights

Place base YOLO weights (e.g., `yolo26s.pt` or `yolo26n.pt`) inside `src/models/` or let Ultralytics fetch them automatically when initializing training:

```bash
mkdir -p src/models
# Example: Place yolo26s.pt in src/models/
mv yolo26s.pt src/models/yolo26s.pt
```

## 4. Train the YOLO Model

Activate the virtual environment and start training via the CLI entrypoint:

```bash
source venv/bin/activate
cd src
python3 main.py train
```

This validates system resources, initializes CUDA, loads hyperparameters from configuration files, and trains the model. Checkpoints and training artifacts are stored in `src/SAMDEF_ISR/RunX/weights/`.

## 5. Export Trained Model to ONNX FP16

Ultralytics provides native end-to-end FP16 export and optimization with `onnxslim`. Run the export directly from your active virtual environment:

```bash
source venv/bin/activate

yolo export \
  model=/home/shatadal/PROJECTS/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run15/weights/best.pt \
  format=onnx \
  half=True \
  opset=17 \
  simplify=True \
  nms=True \
  max_det=5000 \
  dynamic=True \
  imgsz=1152
```

### Export Parameter Explanation:
- `format=onnx`: Exports the PyTorch `.pt` model to ONNX.
- `half=True`: Quantizes internal weights and operations to FP16 (Tensor Core acceleration).
- `opset=17`: Modern ONNX opset supporting optimized FP16 kernels and layer normalizations.
- `simplify=True`: Runs `onnxslim` to simplify redundant constant expressions, squeeze nodes, and optimize graph topology.
- `nms=True`: Instructs exporter to include NMS (for end-to-end models, YOLO26 automatically outputs fused detections `[batch, max_det, 6]`).
- `max_det=5000`: Sets the maximum candidate detections per tile output.
- `dynamic=True`: Retains dynamic batch size (`[batch, 5000, 6]`).
- `imgsz=1152`: Configures the input spatial resolution to 1152×1152 pixels.

> [!NOTE]
> The exported ONNX graph contains native FP16 internal weights while maintaining FP32 inputs and outputs (`images: float32`, `output0: float32` via a terminal `Cast` node). This is by design: host CPUs can feed and retrieve standard 32-bit floats without software half-precision conversion overhead, while GPU Tensor Cores execute all hidden layers in native FP16.
