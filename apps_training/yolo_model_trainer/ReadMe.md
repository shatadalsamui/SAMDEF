# SAMDEF Brain Microservice: Quick Commands

## 1. Setup Python Virtual Environment
```bash
python3 -m venv venv
source venv/bin/activate
```

## 2. Install Dependencies (from pyproject.toml)
```bash
pip install .
```

## 3. Download YOLOv26s Weights (if not present)
```bash
python3 src/utils/download_weights.py
# or manually if already downloaded:
mv yolo11s.pt src/models/yolo11s.pt
```

## 4. Start Training the YOLO Model
Activate the virtual environment, navigate to the source directory, and run the training command:
```bash
source venv/bin/activate
cd src
python3 main.py train
```
This will:
- Run system diagnostics (GPU check, data verification).
- Load the YOLO26s model.
- Train for 300 epochs on your tiled xView data.
- Save results to `SAMDEF_Satellite_Ops/Run6_HighDef_100Epochs/`.

## 5. Export PyTorch Model to ONNX
After training, export your best model to ONNX format for ONNX Runtime:
```bash
yolo export model=/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best.pt format=onnx half=True opset=17 simplify=True nms=True max_det=2000 dynamic=True
```

## 6. Full Pipeline: Convert `.pt` Model to ONNX FP16 (End-to-End)

### Step 1: Export PyTorch Model to ONNX (with FP16 weights)
```bash
yolo export model=/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best.pt format=onnx half=True opset=17 simplify=True nms=True max_det=2000 dynamic=True
```
This produces an ONNX model with FP16 weights, but the input/output nodes may still be float32.

### Step 2: Convert ONNX Model IO to FP16 (if needed)
If you need the ONNX model's input and output nodes to be FP16 (for true end-to-end FP16 inference), you can use the provided Python utility script or run the following Python code:

#### Run the utility script:
```bash
python3 /home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/utils/utils.py
```

#### Or, run this Python snippet:
```python
import onnx

model = onnx.load("/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best.onnx")
for input_tensor in model.graph.input:
    input_tensor.type.tensor_type.elem_type = 10  # FLOAT16
for output_tensor in model.graph.output:
    output_tensor.type.tensor_type.elem_type = 10  # FLOAT16
onnx.save(model, "/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best_fp16_io.onnx")
```

### Step 3: (If needed) Patch the Final Cast Node to Output FP16
If you get an ONNX Runtime error about output type mismatch, patch the final Cast node to output FP16:

```python
import onnx

model_path = "/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best_fp16_io.onnx"
patched_path = "/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best_fp16_io_patched.onnx"

model = onnx.load(model_path)

for node in model.graph.node:
    if node.op_type == "Cast" and node.name == "graph_output_cast0":
        for attr in node.attribute:
            if attr.name == "to":
                print(f"Changing Cast node {node.name} from {attr.i} to 10 (FLOAT16)")
                attr.i = 10  # 10 = FLOAT16

onnx.save(model, patched_path)
print("Patched model saved as", patched_path)
```

### Step 4: Use the Patched Model for Inference
Use the final patched ONNX model (e.g., `best_fp16_io_patched.onnx`) in your inference pipeline.

---

**Summary Table**

| Step | Command/Script | Purpose |
|------|---------------|---------|
| 1 | `yolo export ... half=True ...` | Export ONNX with FP16 weights |
| 2 | Python IO patch | Set input/output nodes to FP16 |
| 3 | Python Cast patch | Ensure output node produces FP16 |
| 4 | Use patched model | End-to-end FP16 inference |

---

## 6. (Future) Run Inference/Inference Mode
```bash
source venv/bin/activate
cd src
python3 main.py run
```
Placeholder for Kafka-based inference (to be implemented).

---

### Notes
- All commands are to be run from `apps/brain_python/src/` unless otherwise specified.
- Training requires the data from the Rust Ingestor (`/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml`).
- Monitor progress in the terminal; checkpoints save every 10 epochs.
- Stop training with `Ctrl+C` (graceful shutdown).
- For individual testing, you can run `python3 cortex/trainer.py` directly.
