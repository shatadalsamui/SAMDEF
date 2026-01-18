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
python src/utils/download_weights.py
# or manually if already downloaded:
mv yolo11s.pt src/models/yolo11s.pt
```

## 4. Start Training the YOLO Model
Navigate to the source directory and run the training command:
```bash
cd src
python main.py train
```
This will:
- Run system diagnostics (GPU check, data verification).
- Load the YOLO11s model.
- Train for 100 epochs on your tiled xView data.
- Save results to `SAMDEF_Satellite_Ops/Run6_HighDef_100Epochs/`.

## 5. (Future) Run Inference/Inference Mode
```bash
cd src
python main.py run
```
Placeholder for Kafka-based inference (to be implemented).

---

### Notes
- All commands are to be run from `apps/brain_python/src/` unless otherwise specified.
- Training requires the data from the Rust Ingestor (`/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml`).
- Monitor progress in the terminal; checkpoints save every 10 epochs.
- Stop training with `Ctrl+C` (graceful shutdown).
- For individual testing, you can run `python cortex/trainer.py` directly.
