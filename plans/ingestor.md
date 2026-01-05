


# SAMDEF PROJECT: Phase 2 - Rust Ingestion Engine Plan

## 1. Objective

Create a high-performance, multi-threaded Rust CLI tool to ingest xView satellite imagery (BigTIFF) and process it for YOLOv12 training.
**Hardware Target:** NVIDIA RTX 4060 (8GB VRAM) -> Output Tile Size must be **1024x1024**.

**Note:** This pipeline is designed for YOLOv12 compatibility. Ensure label formatting and normalization match YOLOv12 requirements.

## 2. Input Data Specifications
* **Images:** GeoTIFF (.tif) files. Large resolution (~15,000 x 15,000 px).
* **Labels:** A single GeoJSON file (`xView_train.geojson`).
* **Key Attribute:** We MUST use the `bounds_imcoords` property from the GeoJSON features.
	* *Format:* "xmin,ymin,xmax,ymax" (String).
	* *Reason:* These are pre-calculated pixel coordinates. Do NOT use the Lat/Lon `geometry` field; it requires unnecessary projection math.

## 3. The "Tactical" Class Mapping
We are filtering the dataset to specific military-relevant classes.
**Logic:** If `type_id` matches the table, map it to the `YOLO_ID`. If not, discard.

| xView type_id | Object Name | YOLO_ID | Special Logic |
| :--- | :--- | :--- | :--- |
| **73** | Huts / Small Buildings | **0** | **FILTER:** Ignore if width or height > 100px (exclude large infrastructure). |
| **24** | Pickup Truck | **1** | Keep all. |
| **18** | Small Car | **2** | Keep all (as "civilian noise"). |
| **21** | Motorbike | **3** | Keep all. |
| **19** | Bus / Truck | **4** | Keep all. |
| **94** | Container / Shed | **0** | Map to Class 0 (Structure). |

## 4. Architecture & Logic Flow

### A. Dependencies (Cargo.toml)
* `serde`, `serde_json`: For parsing the huge GeoJSON file.
* `gdal`: For reading BigTIFF image data efficiently.
* `image`: For saving the output tiles as .jpg.
* `rayon`: For multi-threading (utilizing 24GB RAM).
* `indicatif`: For a progress bar.

### B. The "Slicing" Algorithm
1.  **Parse Labels First:**
    * Read GeoJSON once into a `HashMap<String, Vec<Label>>`.
2.  **Process Images (Parallelized):**
    * Iterate through `.tif` files using `rayon`.
    * **Grid Loop:** Slide 1024x1024 window (stride = 824 for 200px overlap).
    * **Intersection Check:** Calculate normalized YOLO coordinates for objects inside the tile.
    * **Save Condition (UPDATED):**
        * **ALWAYS SAVE THE IMAGE.** We are keeping 100% of tiles, even empty ones.
        * **IF objects exist:** Create `tile_name.txt` with class data.
        * **IF empty:** Create an empty `tile_name.txt` file (to explicitly mark as background).

### C. Output Directory Structure

Create this structure automatically:
```text
/SAMDEF_DATA/
	/processed/
		/images/train/
		/labels/train/
		data.yaml  <-- Auto-generated
```

### D. YOLOv12 Label Format
Ensure the Rust code outputs this exact format in the .txt files:
<class_id> <x_center> <y_center> <width> <height>

All values normalized 0.0 to 1.0 relative to the tile size (1024).

## 5. Specific Implementation Notes for Copilot

- **Memory Management:** Since we are processing all tiles, do not hold all images in RAM. Process and save to disk immediately.
- **Auto-Generate data.yaml:** At the end of execution, write the data.yaml file pointing to the absolute path of the processed directory.
- **Error Handling:** If a TIFF is corrupt, log "Error" and continue to the next one. Do not crash.
- **Coordinate Clamping:** Ensure bounding boxes are clipped to the edge of the 1024 tile (e.g., no negative coordinates).
- **Performance:** Use `rayon::par_iter()` on the list of images for parallel processing.

## Implementation Checklist

- [x] Parse GeoJSON labels and load features.
- [x] Build a label map: group features by image file stem (fix: ensure correct keying for tile-label association).
- [x] Iterate over all `.tif` images in the input directory.
- [x] For each image, slide a 1024x1024 window (stride 824) and extract tiles.
- [x] For each tile, check which labels fall inside and normalize coordinates (supporting YOLO classes 0-4 as per mapping table).
- [x] Save every tile as `.jpg` and write a `.txt` label file (empty if no objects).
- [x] Write all outputs to `/home/shatadal/SAMDEF/raw_data` (or your chosen output structure).
- [x] Auto-generate `data.yaml` at the end.

### Further Considerations
- [x] Add error handling for corrupt TIFFs.
- [x] Clamp bounding boxes to tile edges and ensure all YOLO coordinates are normalized to [0,1].
- [x] Use `rayon` for parallel image processing.
- [x] Monitor memory usage and process images one at a time.

## Verification & QA Workflow (2026 Update)

- All label files are now correctly populated and associated with their tiles (label map keying bug fixed).
- All YOLO classes 0–4 are supported as per tactical mapping, including special logic for class 0 and 4.
- Visual verification is performed by overlaying bounding boxes and class labels on tile images using ImageMagick and awk shell scripts.
- Overlay images (`.vis.jpg`) are generated for any tile, saved alongside or in a dedicated directory for QA.
- Label file validation: check for 5 fields per line, all coordinates in [0,1], and class in 0–4.
- For large-scale QA, batch scripts can generate overlays and help spot missing or misaligned detections.
- Empty label files are created for background tiles as required by YOLOv12.

**Note:** If any detections are missing in overlays, check label file formatting, coordinate normalization, and ensure no class is filtered out unintentionally.