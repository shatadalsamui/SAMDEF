# SAMDEF Phase 3: Satellite Training Execution Plan

## Objective
Train YOLOv11s on 1024x1024 satellite tiles without downscaling, using a "Lossless Augmentation" strategy to detect small tactical targets.

**Hardware:** NVIDIA RTX 4060 (8GB VRAM)

---

## 🛠️ Phase 1: Environment Verification

**Action:** Ensure the GPU stack is ready.

```bash
pip uninstall torch torchvision ultralytics -y
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
pip install ultralytics sahi
```

**Verify GPU Visibility:**

```bash
python -c "import torch; print(f'CUDA: {torch.cuda.is_available()} | Device: {torch.cuda.get_device_name(0)}')"
```
_Output must show "True" and "RTX 4060"_

---

## 📦 Phase 2: Data Engineering (The "Two-Run" Sequence)

We run the Ingestor twice (once for Train, once for Val) to create a clean dataset structure.

### Step 2.1: Run 1 (Training Data)
- **Context:** `main.rs` is currently set to process train_images.
- **Action:**

```bash
cd apps/ingestor
cargo run --release
```

### Step 2.2: Protect the Config
- **Action:** Rename the generated config so it isn't overwritten.

```bash
mv /home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml /home/shatadal/SAMDEF/raw_data/processed_tiles/data_train_backup.yaml
```

### Step 2.3: Modify Code for Run 2 (Validation Data)
- **Task:** Edit `apps/ingestor/src/main.rs`:
	- Change `image_dir` to `/home/shatadal/SAMDEF_DATA/val_images/`.
	- Change output subdirectories from `images/train` to `images/val` and `labels/train` to `labels/val`.

### Step 2.4: Run 2 (Validation Data)
- **Action:**

```bash
cargo run --release
```

### Step 2.5: Create the Final Config
**Action:** Create a new file `/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml` with this content:

```yaml
path: /home/shatadal/SAMDEF/raw_data/processed_tiles
train: images/train
val: images/val
names:
	0: Container_Shed
	1: Pickup_Truck
	2: Small_Car
	3: Motorbike
	4: Bus_Truck
	5: Construction_Site
	6: Tent
	7: Shed
	8: Container_Shed_Alt
	9: Hut_Small_Building
```

---

## 🧠 Phase 3: The Physics Engine (Training)

**Action:** Execute this command to train with native resolution:

```bash
yolo detect train \
	project=SAMDEF_Satellite_Ops \
	name=Run1_YOLO11S_Native_1024 \
	model=yolo11s.pt \
	data=/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml \
	epochs=100 \
	patience=15 \
	batch=4 \
	imgsz=1024 \
	device=0 \
	workers=4 \
	amp=True \
	exist_ok=True \
	degrees=180 \
	flipud=0.5 \
	fliplr=0.5 \
	scale=0.5 \
	mosaic=1.0 \
	copy_paste=0.3 \
	perspective=0.0005
```

### 🔍 Key Flag Explanations (The "Why")
- **imgsz=1024:** CRITICAL. Forces the GPU canvas to 1024x1024. Prevents small cars (15px) from shrinking to blobs (9px).
- **batch=4:** Prevents VRAM crash on the RTX 4060 while handling large 1024px images.
- **scale=0.5:** Enables "Lossless Zooming." When the model zooms in, it reveals real pixel details from the source file.
- **degrees=180:** Teaches the model that a tank facing South is the same as a tank facing North (Satellite invariance).

---

## ✅ Phase 4: Immediate Verification
- Wait for Epoch 1 to finish.
- Go to `SAMDEF_Satellite_Ops/Run1_YOLO11S_Native_1024/`.
- Open `train_batch0.jpg`.
- **Check:** Zoom in on a car. If it is a crisp square, the plan worked. If it is blurry, stop and check `imgsz`.

---

## 🏆 Extras for High Accuracy
- **Label QA:** Use overlay scripts (ImageMagick + awk) to visually verify bounding boxes and class assignments on random tiles before training.
- **Label Format:** Ensure all label files use YOLO format: `<class_id> <x_center> <y_center> <width> <height>` (all normalized to 0–1, tile size 1024).
- **Class Coverage:** Confirm all classes (0–5) are present in the dataset and correctly mapped.
- **Augmentation Monitoring:** Review augmentation images (e.g., mosaic, flip) in the training log directory to ensure no artifacts or label misalignments.
- **Reproducibility:** Save all config files, code versions, and random seeds for each run.
- **Resource Monitoring:** Use `nvidia-smi` and training logs to monitor VRAM and GPU utilization.
- **Early Stopping:** Use `patience=15` to avoid overfitting.
- **Validation:** After training, run inference on a few validation tiles and visually inspect predictions for small targets.

---

## 🚶 Phase 2: Human Detection Expansion

Detecting humans in satellite imagery requires custom annotation, as xView does not provide person labels. For IDEX, use a staged approach:

- **Custom Annotation:** Collect or annotate satellite images with individuals, small groups, and large gatherings. Use tools like CVAT, Labelbox, or Roboflow.
- **New Classes:** Add new categories to `data.yaml` (e.g., `person`, `small_group`, `large_gathering`).
- **Fine-tuning:** After initial training, fine-tune YOLOv12 on the new human-labeled data. This allows the model to learn both infrastructure and human presence.
- **Proxy Classes:** If direct annotation is difficult, use proxy indicators (e.g., tent clusters, vehicle clusters, crowd shadows) as new classes.
- **Validation:** Visually inspect predictions for human presence and adjust annotation strategy as needed.

This staged workflow is practical for border monitoring and scalable for future mission needs.

---

**This plan ensures maximum fidelity for small object detection and robust, reproducible training.**
