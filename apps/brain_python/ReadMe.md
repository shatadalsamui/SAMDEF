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

## 4. Run the Brain Microservice
```bash
python src/main.py
```

---

### Notes
- All commands are to be run from `apps/brain_python/` unless otherwise specified.
- The microservice will idle in a loop until you add Kafka or inference logic.
- Stop the service with `Ctrl+C`.
