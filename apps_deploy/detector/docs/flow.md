## Typical Data Flow for One Image

1. **Image Input**
   - `main.rs` triggers reading of an image file.
   - `virtual_tiler.rs` (via `process_geotiff`) reads the image, splits it into tiles, and sends each tile as an `InferenceTask` wrapped in a `PipelineMessage::Process`.

2. **Message Passing**
   - Tiles/messages are sent through channels to the processing pipeline.

3. **Pre-processing**
   - `pre_processing.rs` receives each tile, applies necessary transformations.

4. **Inference**
   - `inference.rs` runs the ML model on the pre-processed tile.

5. **Post-processing**
   - `post_processing.rs` processes the inference results.

6. **Result Handling**
   - Results are wrapped in data structures from `results.rs` and sent as messages.

7. **Output**
   - `consumer.rs` or `publisher.rs` receives the final results and outputs them (e.g., saves to disk, sends to a client, etc.).

8. **Session/Batch Management**
   - `session.rs` and `batch.rs` manage the state and grouping of tasks/results.