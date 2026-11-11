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
- `docs/architecture/gui_gpui.md` — GPUI GIS Viewer architecture.
- `docs/architecture/ingestor_training.md` — Ingestor & training flow.
- `docs/architecture/model_trainer.md` — YOLO training architecture.
- `docs/drawingscript.md` — Shell script to draw boxes on imagery.
- `docs/extras.md` — Developer cargo check helper script.

### apps_deploy/detector/ (Inference Pipeline)
- `apps_deploy/detector/README.md` — Detector architecture, 3-stage pipeline, and SAHI baseline comparison.
- `apps_deploy/detector/Cargo.toml` — Crate manifest (ort, GDAL, Zenoh).

- `apps_deploy/detector/Cargo.lock` — Locked dependency versions.
- `apps_deploy/detector/.env.example` — Environment variable template.
- `apps_deploy/detector/.aiexclude` — AI editing exclusions.
- `apps_deploy/detector/docs/1_plan.md` — Detector architecture plan.
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
- `apps_deploy/detector/src/modules/io/consumer.rs` — 5-stage isolated GPU-safe consumer coordinator.
- `apps_deploy/detector/src/modules/io/aggregator.rs` — Dedicated tile parser & scene completion aggregator.
- `apps_deploy/detector/src/modules/io/nms_worker.rs` — Dedicated NMS worker thread (isolated Rayon pool).
- `apps_deploy/detector/src/modules/io/publisher.rs` — Zenoh publisher implementation.
- `apps_deploy/detector/src/modules/processing/mod.rs` — Processing submodules export.
- `apps_deploy/detector/src/modules/processing/pre_processing.rs` — FP16 CHW normalization.
- `apps_deploy/detector/src/modules/processing/inference.rs` — GPU ONNX inference.
- `apps_deploy/detector/src/modules/processing/geometry.rs` — Edge truncation flags, overlap metrics & multi-cell spatial grid.
- `apps_deploy/detector/src/modules/processing/seam_merger.rs` — Iterative X/Y cross-seam bounding box merger.
- `apps_deploy/detector/src/modules/processing/boundary_nms.rs` — Truncated fragment absorption & hybrid IoU/IoS NMS.
- `apps_deploy/detector/src/modules/processing/boundary_tests.rs` — Boundary seam-merge & hybrid NMS unit tests.
- `apps_deploy/detector/src/modules/processing/post_processing.rs` — Boundary resolution coordinator & tensor output parser.

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

### apps_deploy/gui_gpui/ (GIS Viewer - GPUI)
- `apps_deploy/gui_gpui/Cargo.toml` — Crate manifest (gpui, serde, image, tiff).
- `apps_deploy/gui_gpui/build.rs` — Self-healing Linux XCB/XKB multiarch linker symlink resolver.
- `apps_deploy/gui_gpui/README.md` — User manual, hotkeys, and SRP module layout.
- `apps_deploy/gui_gpui/src/main.rs` — Entry point (maximized CSD GPUI window & keystroke observer).
- `apps_deploy/gui_gpui/src/geo.rs` — GDAL 6-param affine transform, DMS formatter & Haversine geodesy.
- `apps_deploy/gui_gpui/src/app/mod.rs` — SamdefApp state, async GeoTIFF/JSON loader & navigation.
- `apps_deploy/gui_gpui/src/app/events.rs` — Smooth scroll zoom, drag/click disambiguation & map hotkeys.
- `apps_deploy/gui_gpui/src/app/input_keys.rs` — Focusable text input editing, clipboard paste & file search.
- `apps_deploy/gui_gpui/src/app/view.rs` — Root Render tree & live viewport bounds capture.
- `apps_deploy/gui_gpui/src/io/mod.rs` — IO submodule exports.
- `apps_deploy/gui_gpui/src/io/file_scanner.rs` — Directory scan & GeoTIFF path resolution.
- `apps_deploy/gui_gpui/src/io/tiff_loader.rs` — Planar Config 2 & chunky BGRA GeoTIFF strip decoder.
- `apps_deploy/gui_gpui/src/io/exporter.rs` — GeoJSON, CSV & Markdown intelligence report exporters.
- `apps_deploy/gui_gpui/src/model/mod.rs` — Model submodule exports.
- `apps_deploy/gui_gpui/src/model/detection.rs` — FinalOutput JSON schema & ground footprint calculations.
- `apps_deploy/gui_gpui/src/model/palette.rs` — 8 neon class colors & RGBA/HSLA helpers.
- `apps_deploy/gui_gpui/src/ui/mod.rs` — UI component module exports.
- `apps_deploy/gui_gpui/src/ui/theme.rs` — Dark GIS color tokens & reusable button/input primitives.
- `apps_deploy/gui_gpui/src/ui/titlebar.rs` — Client-side window titlebar (Move/Min/Max/F11/Close) & resize strips.
- `apps_deploy/gui_gpui/src/ui/toolbar.rs` — Top bar (Fit, 1:1, Zoom, Ruler, B/W, Radar, Table).
- `apps_deploy/gui_gpui/src/ui/status_bar.rs` — Bottom QGIS-style live Lat/Lon, DMS, Pixel & GSD bar.
- `apps_deploy/gui_gpui/src/ui/minimap.rs` — Interactive radar thumbnail, neon target blips & cyan frustum.
- `apps_deploy/gui_gpui/src/ui/inspector.rs` — Occluded hover/pinned target inspection HUD card.
- `apps_deploy/gui_gpui/src/ui/table_drawer.rs` — Occluded bottom detection table drawer & live search filter.
- `apps_deploy/gui_gpui/src/ui/sidebar/mod.rs` — Scrollable left sidebar container & header.
- `apps_deploy/gui_gpui/src/ui/sidebar/file_nav.rs` — Editable dir scanner, instant file search & dropdown picker.
- `apps_deploy/gui_gpui/src/ui/sidebar/class_panel.rs` — 8-class visibility toggles, counts & toggleable Solo.
- `apps_deploy/gui_gpui/src/ui/sidebar/confidence_panel.rs` — Interactive drag/click confidence slider & ASCII steppers.
- `apps_deploy/gui_gpui/src/ui/sidebar/telemetry_export.rs` — Scene GIS telemetry card & export triggers.
- `apps_deploy/gui_gpui/src/viewer/mod.rs` — Viewer module exports.
- `apps_deploy/gui_gpui/src/viewer/state.rs` — Sub-pixel cursor-invariant zoom math & coordinate projection.
- `apps_deploy/gui_gpui/src/viewer/hit_test.rs` — Priority smallest-area target hit-testing & unit tests.
- `apps_deploy/gui_gpui/src/viewer/render.rs` — Ordered GPU layers for GeoTIFF raster & neon hollow boxes.
- `apps_deploy/gui_gpui/src/viewer/overlays.rs` — GPU-shaped class/confidence badges & geodesic ruler layers.

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
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run11/args.yaml` — Hyperparameters for Run11.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run12/args.yaml` — Hyperparameters for Run12.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run13/args.yaml` — Hyperparameters for Run13.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/args.yaml` — Hyperparameters for Run14.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run14/weights/best_fp16.onnx` — Exported FP16 ONNX model.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run15/args.yaml` — Hyperparameters for Run15.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run15/weights/best.pt` — Best PyTorch checkpoint from latest run.
- `apps_training/yolo_model_trainer/src/SAMDEF_ISR/Run15/weights/best.onnx` — Exported FP16 ONNX model (max_det 5000, 1152px).

### baseline_comparison_SAHI/ (Baseline Benchmark Harness)
- `baseline_comparison_SAHI/README.md` — Benchmark setup, protocols, and comparative results.
- `baseline_comparison_SAHI/run_benchmark.sh` — Automated benchmark execution script.
- `baseline_comparison_SAHI/requirements.txt` — Python dependencies for SAHI benchmarking.
- `baseline_comparison_SAHI/configs/benchmark_config.yaml` — Benchmark configuration (1152px, 20% overlap, CUDA).
- `baseline_comparison_SAHI/model/run15_FP16.onnx` — Identical ONNX model used in detector.
- `baseline_comparison_SAHI/src/sahi_model.py` — Custom SAHI `DetectionModel` wrapper for YOLO ONNX on CUDA.
- `baseline_comparison_SAHI/src/benchmark_runner.py` — GDAL loader, SAHI slicing loop, and timing instrumentation.
- `baseline_comparison_SAHI/src/report_generator.py` — Markdown comparison table and metric aggregator.



---

## 2. Application Entry Points
- `apps_deploy/detector/src/main.rs` — Detector (producer/consumer inference engine).
- `apps_deploy/db_processor/src/main.rs` — DB Processor (Zenoh subscriber -> PostGIS).
- `apps_deploy/gui_gpui/src/main.rs` — GUI GPUI (GPU-accelerated Desktop GIS Viewer).
- `apps_deploy/post_processor/src/main.rs` — Post Processor (annotated PNG generator).
- `apps_training/ingestor/src/main.rs` — Ingestor (GeoTIFF tile + YOLO label generator).
- `apps_training/yolo_model_trainer/src/main.py` — Model Trainer (PyTorch/YOLO CLI).
- `baseline_comparison_SAHI/run_benchmark.sh` — SAHI Baseline Benchmark runner.


---

## 3. Quick Task Navigation
| Task | Direct Starting Files |
|---|---|
| Inference pipeline / tiling | `apps_deploy/detector/src/main.rs`, `apps_deploy/detector/src/modules/io/virtual_tiler.rs`, `apps_deploy/detector/src/modules/processing/pre_processing.rs` |
| GPU inference & NMS | `apps_deploy/detector/src/modules/processing/inference.rs`, `apps_deploy/detector/src/modules/processing/post_processing.rs` |
| Database & Zenoh storage | `apps_deploy/db_processor/src/main.rs`, `apps_deploy/db_processor/src/modules/writer/zenoh_subscriber.rs`, `apps_deploy/db_processor/src/modules/writer/queries.rs` |
| GUI rendering / controls | `apps_deploy/gui_gpui/src/viewer/render.rs`, `apps_deploy/gui_gpui/src/ui/sidebar/file_nav.rs`, `apps_deploy/gui_gpui/src/ui/sidebar/class_panel.rs`, `apps_deploy/gui_gpui/src/app/events.rs` |
| Dataset tiling / labeling | `apps_training/ingestor/src/main.rs`, `apps_training/ingestor/src/modules/image_utils.rs`, `apps_training/ingestor/src/modules/labeler.rs` |
| Model training & export | `apps_training/yolo_model_trainer/src/main.py`, `apps_training/yolo_model_trainer/src/cortex/trainer.py`, `apps_training/yolo_model_trainer/ReadMe.md` |
| Baseline comparison / SAHI | `baseline_comparison_SAHI/run_benchmark.sh`, `baseline_comparison_SAHI/src/benchmark_runner.py`, `baseline_comparison_SAHI/configs/benchmark_config.yaml` |


---

## 4. AI Agent Operational Rules
1. Never run `ls`, `dir`, `find`, or `tree`. Every active file in this repo is listed with its exact relative path above.
2. Read files directly using their full relative path (e.g. `apps_deploy/detector/src/main.rs`).
3. Only inspect files directly relevant to your current task.