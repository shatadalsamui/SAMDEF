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

## 3. Download YOLOv11s Weights (if not present)
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
