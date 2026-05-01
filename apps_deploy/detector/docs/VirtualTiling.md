
# Virtual Tiling Process (for GeoTIFF Images)

This document explains the flow and key points of the virtual tiling logic used in the detector module, specifically the `process_geotiff` function in `virtual_tiler.rs`.

## Flow Overview
1. **Input & Setup**: The function receives a GeoTIFF image file path and a message sender channel.
2. **Path Handling**: Cleans and canonicalizes the file path for consistency.
3. **Image Opening**: Opens the image using the GDAL library, reads its size and geospatial metadata.
4. **Tiling Loop**: Iterates over the image in tiles (896x896 pixels, with 20% overlap using a stride of 716).
5. **Shift-Back Strategy**: Ensures all tiles fit within the image boundaries, so no tile goes out of bounds.
6. **Tile Extraction**:
    - Reads the R, G, and B bands for each tile.
    - Combines them into a single interleaved RGB array.
    - Creates an `InferenceTask` struct with the tile data and metadata.
7. **Message Sending**: Sends each tile as a `PipelineMessage::Process` through the channel.
    - If sending fails (e.g., receiver closed), logs a warning but continues processing.
8. **End-of-File Signal**: After all tiles are processed, sends a `PipelineMessage::EndOfFile` with summary info (total tiles, image size, etc.).
9. **Return**: The function returns, having streamed all tiles for further processing.

## Additional Notes

- The function uses constants for tile size and stride, making it easy to adjust overlap and tile dimensions.
- Designed for RGB (3-band) images; assumes the image has at least three bands.
- This approach allows large images to be processed in manageable pieces, enabling parallel or sequential downstream processing.
- Crossbeam channels are used for message passing between threads; these channels internally act as a ring buffer, efficiently buffering and synchronizing data between processing stages.

---
This summary is intended for explaining the virtual tiling process to your professor.
