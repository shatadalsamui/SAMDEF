# SAMDEF YOLO Model Trainer

## 1. Set up a Python virtual environment

```bash
python3 -m venv venv
source venv/bin/activate
```

## 2. Install dependencies

```bash
pip install .
```

## 3. Download YOLO weights

If you do not have the YOLOv26s weights, run:

```bash
python3 src/utils/download_weights.py
```

If you already have the weights, move them to the correct location:

```bash
mv yolo26s.pt src/models/yolo11s.pt
```

## 4. Train the YOLO model

Activate your virtual environment, go to the source directory, and run:

```bash
source venv/bin/activate
cd src
python3 main.py train
```

This will check your system, load the model, and start training.

## 5. Export the trained model to ONNX

After training, export your best model to ONNX format:

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




