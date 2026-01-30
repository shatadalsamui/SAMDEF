# SAMDEF Detector Plan

## 1. The High-Level Architecture
Think of the system as a **factory line** with three distinct stations. The goal is to keep the GPU ("Station 3") running at **100% capacity** by preparing materials perfectly in "Station 2".

```mermaid
graph LR
    A[Disk: Tiler] -->|Writes .pb Receipt| B(Storage)
    A -->|Pushes Raw Bytes + Metadata| C{Crossbeam Channel}
    C -->|Pops Batch of 12| D[Station 2: CPU Pre-Processor]
    D -->|Parallel Norm (i9)| E[Station 3: GPU Inference]
    E -->|Raw Tensors| F[Station 4: Coordinate Translator]
    F -->|Global Detections| G[Output: Proto/Verification]
```

- [ ] Section 1: High-Level Architecture reviewed.

## 2. Key Components & Responsibilities

### A. Station 1: The Input Bridge (The "Envelope")
Since we aren't using Kafka, we need a data structure that carries everything the detector needs to be "stateless."

**Component:** InferenceTask Struct.

**Payload:**
- `image_data`: The raw `Vec<u8>` RGB pixels. We do not convert to float here to save 4x RAM bandwidth.
- `global_offset_x/y`: The "Global Address" of this tile. This allows the detector to map the result back to the original TIFF without reading the .pb file.
- `tile_filename`: For debugging (e.g., "tile_0_1.jpg").

### B. Station 2: The Parallel Pre-Processor (The i9 Engine)
This is where your **24-core CPU** shines. The GPU is fast, but it hates waiting for memory.

**Goal:** Convert 12 raw images into one massive Float Tensor **[12, 3, 896, 896]** before the GPU asks for it.

**Mechanism:**
- Uses Rayon (Parallel Iterator) to split the batch of 12 images across 12 CPU cores.
- **Operation:** u8 (0-255) → f32 (0.0-1.0).
- **Math:** Strict division by 255.0. No Mean/Std subtraction (as per your YOLO training).
- **Layout Change:** Converts HWC (Standard Image) → CHW (YOLO Format) in the same pass.

### C. Station 3: The Inference Core (RTX 4060)
The dedicated neural engine.

- **Model:** YOLOv26s ("YOLO26s").
- **Input Shape:** Fixed at 896x896.
- **Batch Size:** 8 to 12. (896px is large; 12 images ≈ 1GB of VRAM for tensors, leaving 7GB for the model computation).
- **Execution Provider:** CUDAExecutionProvider (Device 0). This locks the model to the dGPU and keeps it off the iGPU.

### D. Station 4: The Translator (Coordinate Reconstruction)
This is the math step that makes the "Physical Verification" possible.

**Raw Output:** YOLO gives (center_x, center_y, width, height) relative to the 896x896 tile.

**The Transformation:**
- Global_X = center_x + task.global_offset_x
- Global_Y = center_y + task.global_offset_y

**Output:** A DetectionBox Proto that contains Global Coordinates.

- [ ] Section 2: Key Components & Responsibilities reviewed.

## 3. The "Missing Link": Exporting best.pt
Your Rust code will fail if you don't export the model correctly. The standard yolo export often defaults to 640x640. You must export it with the settings matching your training.

Run this command in your Python environment before coding Rust:

```bash
yolo export model=SAMDEF_ISR/Run2/weights/best.pt format=onnx imgsz=896 dynamic=True simplify=True opset=12
```

Why these specific constraints?

| Constraint | Value | The "Startup Demo" Reason |
|------------|-------|---------------------------|
| imgsz     | 896  | CRITICAL. Your trainer.py used 896. If you use the default (640), your model will be blind to small vehicles because the input pixels will be "smushed." |
| dynamic   | True | MANDATORY for Batching. Your Rust code sends batches of 1 to 12 images. If you don't set this, the model is hard-coded to Batch=1. Sending 12 images will cause a Shape Mismatch Error. |
| simplify  | True | Cleans up "Python-only" layers. This makes the model load 2x faster in Rust and prevents weird ONNX Runtime errors. |
| opset     | 12   | The "Universal Language." Opset 12 is the most stable version for the Rust ort crate and NVIDIA GPUs. |

**The "Batch Size" Trap (Warning)**  
You might be tempted to set batch=12 to force performance. Do not do this.  
If you export with batch=12 (static), and your Tiler only has 5 tiles left at the end of the image, the model will crash because it demands exactly 12.  
By using dynamic=True, the model becomes flexible: it will happily accept [1, 3, 896, 896] OR [12, 3, 896, 896].

**Verification Step**  
After exporting, you will get a file named best.onnx. To confirm it's correct before writing Rust code, you can use this quick Python check:

```python
import onnx
model = onnx.load("best.onnx")
# The input text should say: "float32[?, 3, 896, 896]"
# The '?' means it accepts your variable batch size.
print(model.graph.input[0].type.tensor_type.shape)
```

- [ ] Exported best.onnx with correct constraints.
- [ ] Verified input shape shows "float32[?, 3, 896, 896]".

Once you have the .onnx file generated, we can finalize the detector.rs path.

- [ ] Section 3: Model Export completed.

## 4. The Data Contract (Classes)
Based on your final12.csv, your Rust code needs to map Class IDs to these names for the UI/Database:

| Component          | Target Class   | Rule                  | Reason |
|--------------------|----------------|-----------------------|--------|
| High-Recall Filter | Small_Vehicle | Threshold: 0.25      | Most important class (mAP 0.73). We want to see every car. |
| Contextual Filter  | Building      | Threshold: 0.50      | Strong performance (mAP 0.65), used to provide "map context." |
| Moderate          | Long_Haul_Truck| Default threshold     | Often confused with buildings. |
| Moderate          | Work_Truck    | Default threshold     | - |
| Noise Gate         | Temp_Structure| Threshold: 0.85      | Weak performance (mAP 0.06). Only show if the model is extremely certain. |
| Hard Suppressor    | Construction  | Drop All             | mAP 0.01 is too low for a reliable demo. |

**Note:** Classes 0-3 are the primary detections, with a focus on vehicles (0-2) and structures (3). Metrics are tuned to ensure all 4 are detected with very low false negatives, accepting higher false positives as acceptable.

**Data Flow: From Pixel to Global Coordinate**
To ensure the math is perfect, the flow of coordinates must follow this strict sequence:
- **YOLO Local Space:** YOLO26s returns [cx, cy, w, h] relative to the 896x896 tile.
- **Anchor-to-Pixel Conversion:** The detector converts these normalized anchors back into local pixel integers.
- **Top-Left Shift:** 
  - local_x = cx - (w / 2)
  - local_y = cy - (h / 2)
- **Global Transformation:**
  - final_x = local_x + manifest.x_offset
  - final_y = local_y + manifest.y_offset

- [x] Section 4: Data Contract & Flow reviewed.

## 5. The Verification Pipeline (The "Truth" Script)
This component sits outside the main production loop. Its only job is to physically verify that a detection on a small tile matches the correct pixel on the original large TIFF.

**Architecture:**
- **Input:** DetectionBatch Protobuf/JSON (from the Detector).
- **Process:** 
  1. **TIFF Pointer:** Loads the original source TIFF into memory (or a low-res proxy for speed).
  2. **Global Draw:** Iterates through all detected global_x and global_y coordinates.
  3. **Physical Burn:** Draws a hollow 2px-wide red rectangle for every object found.

- [x] Section 5: Verification Pipeline understood.

## 6. Failure Handling & Recovery (Non-Kafka)
Since we are using Crossbeam (In-Memory), we handle "The Detector Blink" through Backpressure:

- **Buffer Limit:** The Crossbeam channel is capped at 50 tiles.
- **The "Full" State:** If the GPU thread lags, the Tiler automatically sleeps. No RAM is wasted, and no tiles are dropped.
- **The "Crash" State:** If the Detector panics, the Tiler receives a SendError.
- **Action:** Tiler logs the last successfully processed tile_id and stops.

- [x] Section 6: Failure Handling reviewed.

## 7. Final Verification Milestone
Before filling the KIIT Google Form, the following "Green Light" must be met:

- [ ] **Coordinate Integrity:** Red boxes in the verification script align within +/- 5 pixels of the vehicles.
- [ ] **VRAM Stability:** RTX 4060 usage stays under 6GB (to leave room for Ubuntu/UI).
- [ ] **Throughput:** Inference time for an 896px batch is < 15ms.

- [ ] Section 7: Final Milestones achieved.