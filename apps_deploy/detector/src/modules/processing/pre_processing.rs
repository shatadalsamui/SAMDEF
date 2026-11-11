use crate::modules::data::buffer_pool::TileBytePool;
use crate::modules::data::task::InferenceTask;
use crate::modules::io::virtual_tiler::TILE_SIZE;
use anyhow::{ensure, Result};
use rayon::prelude::*;

pub const TILE_PIXELS: usize = TILE_SIZE * TILE_SIZE;
pub const TILE_PLANAR_LEN: usize = TILE_PIXELS * 3;
const CHUNK_ELEMS: usize = 8192;

const fn build_u8_to_f32_lut() -> [f32; 256] {
    let mut lut = [0.0f32; 256];
    let mut i = 0;
    while i < 256 {
        lut[i] = i as f32 / 255.0;
        i += 1;
    }
    lut
}

static U8_TO_F32_LUT: [f32; 256] = build_u8_to_f32_lut();

/// Builds a bounded Rayon pool isolated from both GDAL producers and Stage 4 NMS
/// so Stage 1 scales single-tile normalization across cores without thread oversubscription.
pub fn create_preproc_pool() -> rayon::ThreadPool {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let threads = (cores / 4).clamp(4, 8);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|idx| format!("samdef-preproc-{}", idx))
        .build()
        .expect("Failed to build isolated preprocessing thread pool")
}

/// Linearly normalizes planar `[R | G | B]` CHW `u8` pixels directly into pre-allocated
/// page-locked `f32` memory via an L1-resident LUT (zero transpose, zero heap allocations).
pub fn normalize_planar_tile_into(
    pool: &rayon::ThreadPool,
    planar_u8: &[u8],
    dst: &mut [f32],
) -> Result<()> {
    ensure!(
        planar_u8.len() == TILE_PLANAR_LEN,
        "Planar tile length {} != expected {}",
        planar_u8.len(),
        TILE_PLANAR_LEN
    );
    ensure!(
        dst.len() == TILE_PLANAR_LEN,
        "Target tensor slice length {} != expected {}",
        dst.len(),
        TILE_PLANAR_LEN
    );

    pool.install(|| {
        dst.par_chunks_mut(CHUNK_ELEMS)
            .zip(planar_u8.par_chunks(CHUNK_ELEMS))
            .for_each(|(out_chunk, in_chunk)| {
                for (o, &px) in out_chunk.iter_mut().zip(in_chunk.iter()) {
                    *o = U8_TO_F32_LUT[px as usize];
                }
            });
    });

    Ok(())
}

/// Fills a pre-allocated `[batch_size, 3, H, W]` pinned tensor slice from `tasks`,
/// immediately recycling each task's 3.98 MB planar buffer back to `TileBytePool`.
pub fn fill_batch_tensor(
    pool: &rayon::ThreadPool,
    byte_pool: &TileBytePool,
    tasks: &mut [InferenceTask],
    dst: &mut [f32],
) -> Result<()> {
    ensure!(
        dst.len() >= tasks.len() * TILE_PLANAR_LEN,
        "Batch tensor too small for {} tasks",
        tasks.len()
    );

    for (idx, task) in tasks.iter_mut().enumerate() {
        let start = idx * TILE_PLANAR_LEN;
        let end = start + TILE_PLANAR_LEN;
        normalize_planar_tile_into(pool, &task.image_data, &mut dst[start..end])?;
        let recycled = std::mem::take(&mut task.image_data);
        byte_pool.recycle(recycled);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planar_lut_matches_exact_hwc_division() {
        let pool = create_preproc_pool();
        let mut hwc = vec![0u8; TILE_PLANAR_LEN];
        let mut planar = vec![0u8; TILE_PLANAR_LEN];
        for i in 0..TILE_PIXELS {
            let r = ((i * 7) & 0xFF) as u8;
            let g = ((i * 13) & 0xFF) as u8;
            let b = ((i * 29) & 0xFF) as u8;
            hwc[i * 3] = r;
            hwc[i * 3 + 1] = g;
            hwc[i * 3 + 2] = b;
            planar[i] = r;
            planar[TILE_PIXELS + i] = g;
            planar[2 * TILE_PIXELS + i] = b;
        }

        let mut expected = vec![0.0f32; TILE_PLANAR_LEN];
        for i in 0..TILE_PIXELS {
            expected[i] = hwc[i * 3] as f32 / 255.0;
            expected[TILE_PIXELS + i] = hwc[i * 3 + 1] as f32 / 255.0;
            expected[2 * TILE_PIXELS + i] = hwc[i * 3 + 2] as f32 / 255.0;
        }

        let mut actual = vec![0.0f32; TILE_PLANAR_LEN];
        normalize_planar_tile_into(&pool, &planar, &mut actual).unwrap();
        assert_eq!(actual, expected);
    }
}
