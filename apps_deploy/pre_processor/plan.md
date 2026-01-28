

# Final Master Plan for SAMDEF Preprocessor (Checklist)
## Part 2A: Output Organization (Tiles & Manifests)
- [ ] Create a dedicated folder for JPEG tiles (e.g., inference_tiles/)
- [ ] Create a dedicated folder for protobuf manifests (e.g., inference_manifests/)
- [ ] Update main.rs to accept both output folders and pass manifest folder to tiler.rs
- [ ] Update tiler.rs to write protobuf manifest to the manifest folder, not the tiles folder

## Part 1: Project Configuration
- [x] Ensure Cargo.toml matches the required dependencies and settings for parallel processing and progress bars.
	- name, version, edition
	- serde, serde_json, gdal, image, rayon, indicatif, turbojpeg, anyhow
	- rand not required

## Part 2: Rust Module Implementation
- [x] Create the directory structure: `src/modules/` if not already present.
- [x] Implement the data structures for `TileMetadata` and `InferenceManifest` in `src/modules/image_utils.rs`.
- [x] Implement the function `process_inference_image` in `src/modules/image_utils.rs` (now in tiler.rs for inference):
	- [x] Open GDAL Dataset
	- [x] Get image dimensions and GeoTransform
	- [x] Calculate stride (now configurable)
	- [x] Initialize indicatif progress bar
	- [x] Loop over y and x (nested), snap-to-edge if needed
	- [x] Read RGB bands using band.read_as
	- [x] Compress with turbojpeg (configurable quality, Subsamp::None)
	- [x] Save each tile as {stem}_{row}_{col}.jpg
	- [x] Track each tile in a TileMetadata list
	- [x] Update the progress bar
	- [x] Save manifest for each image (Protocol Buffers only)
		- [ ] If using JSON: save as `<stem>.json` using serde/serde_json
		- [x] If using protobuf: define a `.proto` schema, use prost/protobuf for serialization, and save as `<stem>.pb` or similar
		- [ ] Implement protobuf serialization and write .pb file to manifest folder
	- [x] Return the InferenceManifest struct

## Part 3: Performance and Cleanup
- [x] Use indicatif for progress bar during tiling
- [x] Use rayon to parallelize JPEG compression or loop if possible (parallelization at image level in main.rs)
- [x] Remove all legacy code related to labels, training, or xml parsing (inference-only logic)

### Note on Manifest Serialization
- JSON is simple, human-readable, and works for most ML pipelines.
- Protocol Buffers (protobuf) is a more compact, faster binary alternative, but requires:
	- Defining a `.proto` schema for your manifest and tile structures
	- Using prost/protobuf crates and code generation
	- Slightly more complex build process
- Use protobuf if you need maximum performance or have very large manifests; otherwise, JSON is sufficient.
