# SAMDEF Project Index

Machine-readable index of verified file paths for local AI models and coding agents.
**Rule for AI:** NEVER run `ls`, `find`, or `tree`. Use the exact relative paths listed below.

---

## 1. File Catalog (Full Relative Paths)

### Root
- `Cargo.toml` — Rust workspace configuration (member crates).
- `Cargo.lock` — Locked dependency versions.
- `PROJECT_INDEX.md` — Navigation index for AI models.
- `readme.md` — High-level architecture, workflow, and platform overview.
- `.gitignore` — Git ignore rules (target, venv, models).

### docs/
- `docs/architecture/detector.md` — Detector pipeline architecture & tiling.
- `docs/architecture/dbprocessor.md` — DB processor & PostGIS spatial storage.
- `docs/architecture/gui_iced.md` — Native GIS Viewer architecture.
- `docs/architecture/ingestor_training.md` — Ingestor & training flow.
- `docs/architecture/model_trainer.md` — YOLO training architecture.
- `docs/drawingscript.md` — Shell script to draw boxes on imagery.
- `docs/extras.md` — Developer cargo check helper script.

### apps_deploy/detector/ (Inference Pipeline)
- `apps_deploy/detector/Cargo.toml` — Crate manifest (ort, GDAL, Zenoh).
- `apps_deploy/detector/Cargo.lock` — Locked dependency versions.
- `apps_deploy/detector/.env.example` — Environment variable template.
- `apps_deploy/detector/.aiexclude` — AI editing exclusions.
- `apps_deploy/detector/docs/1_plan.md` — Detector architecture plan.
- `apps_deploy/detector/docs/2_plan_fp32_to_fp16.md` — FP16 migration plan.
- `apps_deploy/detector/docs/flow.md` — End-to-end detector data flow.
- `apps_deploy/detector/docs/NMS.md` — Spatial-grid NMS implementation.
- `apps_deploy/detector/docs/VirtualTiling.md` — Virtual tiling specifications.
- `apps_deploy/detector/model/best_fp16_patched.onnx` — Patched FP16 ONNX model for GPU inference.
- `apps_deploy/detector/src/main.rs` — Entry point (producer pool & 3-stage consumer).
- `apps_deploy/detector/src/modules/mod.rs` — Top-level module declarations.
- `apps_deploy/detector/src/modules/data/mod.rs` — Data submodules export.
- `apps_deploy/detector/src/modules/data/payload.rs` — Detection payload structs.
- `apps_deploy/detector/src/modules/data/results.rs` — JSON output & Zenoh publish triggers.
- `apps_deploy/detector/src/modules/data/task.rs` — InferenceTask & PipelineMessage types.
- `apps_deploy/detector/src/modules/io/mod.rs` — IO submodules export.
- `apps_deploy/detector/src/modules/io/session.rs` — CUDA ONNX Runtime session initializer.
- `apps_deploy/detector/src/modules/io/producer.rs` — GeoTIFF producer worker pool.
- `apps_deploy/detector/src/modules/io/virtual_tiler.rs` — GeoTIFF tiling logic.
- `apps_deploy/detector/src/modules/io/consumer.rs` — 3-stage pipeline (preprocess, GPU, postprocess).
- `apps_deploy/detector/src/modules/io/publisher.rs` — Zenoh publisher implementation.
- `apps_deploy/detector/src/modules/processing/mod.rs` — Processing submodules export.
- `apps_deploy/detector/src/modules/processing/pre_processing.rs` — FP16 CHW normalization.
- `apps_deploy/detector/src/modules/processing/inference.rs` — GPU ONNX inference.
- `apps_deploy/detector/src/modules/processing/post_processing.rs` — NMS & global coordinate mapping.

### apps_deploy/db_processor/ (Database Persistence)
- `apps_deploy/db_processor/Cargo.toml` — Crate manifest (SQLx, Zenoh).
- `apps_deploy/db_processor/Cargo.lock` — Locked dependency versions.
- `apps_deploy/db_processor/.env.example` — PostgreSQL connection template.
- `apps_deploy/db_processor/docs/plan.md` — Architecture & Zenoh-to-PostGIS plan.
- `apps_deploy/db_processor/src/main.rs` — Entry point (Zenoh subscriber, SQLx pool).
- `apps_deploy/db_processor/src/modules/mod.rs` — Module declarations.
- `apps_deploy/db_processor/src/modules/schema/init.sql` — PostGIS detections table schema.
- `apps_deploy/db_processor/src/modules/writer/mod.rs` — Writer submodule export.
- `apps_deploy/db_processor/src/modules/writer/payload.rs` — Detection payload structs.
- `apps_deploy/db_processor/src/modules/writer/queries.rs` — SQL insert queries for PostGIS geometry.
- `apps_deploy/db_processor/src/modules/writer/zenoh_subscriber.rs` — Zenoh subscriber handler.

### apps_deploy/gui_iced/ (GIS Desktop Viewer)
- `apps_deploy/gui_iced/Cargo.toml` — Crate manifest (iced, tiff, image).
- `apps_deploy/gui_iced/Cargo.lock` — Locked dependency versions.
- `apps_deploy/gui_iced/docs/architecture.md` — Zero-allocation vector layer & UI design.
- `apps_deploy/gui_iced/src/main.rs` — Entry point (iced Application runner).
- `apps_deploy/gui_iced/src/app/mod.rs` — App submodule export.
- `apps_deploy/gui_iced/src/app/state.rs` — Main application state struct.
- `apps_deploy/gui_iced/src/app/message.rs` — UI message enum.
- `apps_deploy/gui_iced/src/app/update.rs` — State transition & async command handlers.
- `apps_deploy/gui_iced/src/io/mod.rs` — IO submodule export.
- `apps_deploy/gui_iced/src/io/file_scanner.rs` — Directory scan & TIFF path resolution.
- `apps_deploy/gui_iced/src/io/tiff_loader.rs` — Planar TIFF decoder to GPU textures.
- `apps_deploy/gui_iced/src/model/mod.rs` — Model submodule export.
- `apps_deploy/gui_iced/src/model/detection.rs` — Bounding box & detection data structs.
- `apps_deploy/gui_iced/src/model/palette.rs` — Neon class color mappings.
- `apps_deploy/gui_iced/src/ui/mod.rs` — UI components export.
- `apps_deploy/gui_iced/src/ui/class_filters.rs` — Class toggles & confidence threshold slider.
- `apps_deploy/gui_iced/src/ui/file_nav.rs` — Search bar, dataset dropdown, prev/next buttons.
- `apps_deploy/gui_iced/src/ui/quick_actions.rs` — Select all / clear all class filters.
- `apps_deploy/gui_iced/src/ui/sidebar.rs` — Sidebar layout container.
- `apps_deploy/gui_iced/src/ui/style.rs` — VS Code / Zed Dark editor styling sheets.
- `apps_deploy/gui_iced/src/ui/telemetry.rs` — Telemetry HUD (zoom, coords, box counts).
- `apps_deploy/gui_iced/src/ui/toolbar.rs` — Top toolbar layout.
- `apps_deploy/gui_iced/src/viewer/mod.rs` — Viewer widget submodules export.
- `apps_deploy/gui_iced/src/viewer/widget.rs` — Custom low-level iced viewer widget.
- `apps_deploy/gui_iced/src/viewer/state.rs` — Coordinate projection & viewer state.
- `apps_deploy/gui_iced/src/viewer/events.rs` — Sub-pixel mouse pan and scroll-wheel zoom math.
- `apps_deploy/gui_iced/src/viewer/render.rs` — Multi-layer renderer (raster texture + hollow vector quads).

### apps_deploy/post_processor/ (Annotation Renderer)
- `apps_deploy/post_processor/Cargo.toml` — Crate manifest (image, imageproc, tiff).
- `apps_deploy/post_processor/Cargo.lock` — Locked dependency versions.
- `apps_deploy/post_processor/.env.example` — Configuration template (JSON_DIR, OUTPUT_DIR).
- `apps_deploy/post_processor/src/main.rs` — Draws hollow boxes on GeoTIFFs and saves PNGs.

### apps_training/image_simulator/
- `apps_training/image_simulator/make_rectangle.py` — Stitched rectangle stress-test GeoTIFF generator.
- `apps_training/image_simulator/make_square.py` — Stitched square stress-test GeoTIFF generator.

### apps_training/ingestor/
- `apps_training/ingestor/Cargo.toml` — Crate manifest (GDAL, rayon, turbojpeg).
- `apps_training/ingestor/Cargo.lock` — Locked dependency versions.
- `apps_training/ingestor/.env.example` — Path configuration template.
- `apps_training/ingestor/src/main.rs` — Dataset generation entry point.
- `apps_training/ingestor/src/modules/mod.rs` — Ingestor module declarations.
- `apps_training/ingestor/src/modules/image_utils.rs` — Fast parallel GeoTIFF tiling.
- `apps_training/ingestor/src/modules/label_parser.rs` — GeoJSON parsing logic.
- `apps_training/ingestor/src/modules/labeler.rs` — GeoJSON to YOLO label mapper.

### apps_training/yolo_model_trainer/
- `apps_training/yolo_model_trainer/pyproject.toml` — Python project manifest.
- `apps_training/yolo_model_trainer/ReadMe.md` — Training & export guide.
- `apps_training/yolo_model_trainer/.env.example` — Training configuration template.
- `apps_training/yolo_model_trainer/.gitignore` — Trainer git exclusions.
- `apps_training/yolo_model_trainer/src/main.py` — Trainer CLI entry point.
- `apps_training/yolo_model_trainer/src/cortex/__init__.py` — Cortex package initializer.
- `apps_training/yolo_model_trainer/src/cortex/trainer.py` — PyTorch / Ultralytics YOLO training harness.
- `apps_training/yolo_model_trainer/src/cortex/run11Constraints.txt` — Archived Run11 constraints snapshot.
- `apps_training/yolo_model_trainer/src/cortex/run12Constraints.txt` — Archived Run12 constraints snapshot.
- `apps_training/yolo_model_trainer/src/cortex/run13Constraints.txt` — Archived Run13 constraints snapshot.
- `apps_training/yolo_model_trainer/src/models/yolo26s.pt` — Small base YOLO checkpoint.
- `apps_training/yolo_model_trainer/src/yolo26n.pt` — Nano base YOLO checkpoint.
- `apps_training/yolo_model_trainer/src/utils/utils.py` — ONNX FP16 export & graph patcher.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run11/args.yaml` — Hyperparameters for Run11.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run12/args.yaml` — Hyperparameters for Run12.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run13/args.yaml` — Hyperparameters for Run13.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/args.yaml` — Hyperparameters for Run14.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best_fp16.onnx` — Exported FP16 ONNX model.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run15/args.yaml` — Hyperparameters for Run15.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run15/weights/best.pt` — Best PyTorch checkpoint from latest run.

---

## 2. Application Entry Points
- `apps_deploy/detector/src/main.rs` — Detector (producer/consumer inference engine).
- `apps_deploy/db_processor/src/main.rs` — DB Processor (Zenoh subscriber -> PostGIS).
- `apps_deploy/gui_iced/src/main.rs` — GUI Iced (Desktop GIS Viewer).
- `apps_deploy/post_processor/src/main.rs` — Post Processor (annotated PNG generator).
- `apps_training/ingestor/src/main.rs` — Ingestor (GeoTIFF tile + YOLO label generator).
- `apps_training/yolo_model_trainer/src/main.py` — Model Trainer (PyTorch/YOLO CLI).

---

## 3. Quick Task Navigation
| Task | Direct Starting Files |
|---|---|
| Inference pipeline / tiling | `apps_deploy/detector/src/main.rs`, `apps_deploy/detector/src/modules/io/virtual_tiler.rs`, `apps_deploy/detector/src/modules/processing/pre_processing.rs` |
| GPU inference & NMS | `apps_deploy/detector/src/modules/processing/inference.rs`, `apps_deploy/detector/src/modules/processing/post_processing.rs` |
| Database & Zenoh storage | `apps_deploy/db_processor/src/main.rs`, `apps_deploy/db_processor/src/modules/writer/zenoh_subscriber.rs`, `apps_deploy/db_processor/src/modules/writer/queries.rs` |
| GUI rendering / controls | `apps_deploy/gui_iced/src/viewer/render.rs`, `apps_deploy/gui_iced/src/ui/class_filters.rs`, `apps_deploy/gui_iced/src/app/update.rs` |
| Dataset tiling / labeling | `apps_training/ingestor/src/main.rs`, `apps_training/ingestor/src/modules/image_utils.rs`, `apps_training/ingestor/src/modules/labeler.rs` |
| Model training & export | `apps_training/yolo_model_trainer/src/main.py`, `apps_training/yolo_model_trainer/src/cortex/trainer.py`, `apps_training/yolo_model_trainer/src/utils/utils.py` |

---

## 4. AI Agent Operational Rules
1. Never run `ls`, `dir`, `find`, or `tree`. Every active file in this repo is listed with its exact relative path above.
2. Read files directly using their full relative path (e.g. `apps_deploy/detector/src/main.rs`).
3. Only inspect files directly relevant to your current task.