use rayon::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

fn main() {
    let base_dir = "/home/shatadal/SAMDEF/raw_data/processed_tiles";
    let train_images_dir = format!("{}/images/train", base_dir);
    let train_labels_dir = format!("{}/labels/train", base_dir);
    let output_list = format!("{}/train_phase2_hierarchy.txt", base_dir);

    println!("📊 Building Phase 2 Playlist (6x Trucks / 15x Rare)...");

    let entries: Vec<PathBuf> = std::fs::read_dir(&train_images_dir)
        .expect("Failed to read images/train")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "jpg" || ext == "png"))
        .collect();

    let mut playlist: Vec<String> = entries.par_iter().flat_map(|img_path| {
        let stem = img_path.file_stem().unwrap().to_str().unwrap();
        let label_path = Path::new(&train_labels_dir).join(format!("{}.txt", stem));
        let mut rng = rand::thread_rng();
        let mut multiplier = 0; // Default to 0 (drop)

        if label_path.exists() {
            if let Ok(f) = File::open(&label_path) {
                let reader = BufReader::new(f);
                let mut has_rare = false;
                let mut has_car = false;
                let mut has_building = false;

                for line in reader.lines().flatten() {
                    if let Some(id_str) = line.split_whitespace().next() {
                        if let Ok(id) = id_str.parse::<u8>() {
                            match id {
                                4 | 5 => has_rare = true,
                                0 => has_car = true,
                                3 => has_building = true,
                                _ => {}
                            }
                        }
                    }
                }

                // --- RARE ONLY OVERSAMPLING LOGIC ---
                if has_rare {
                    multiplier = 15;
                } else {
                    // All other tiles: keep ~15% randomly
                    if rng.gen_bool(0.15) {
                        multiplier = 1;
                    } else {
                        multiplier = 0;
                    }
                }
            }
        }
        vec![img_path.to_string_lossy().to_string(); multiplier]
    }).collect();

    playlist.shuffle(&mut rand::thread_rng());

    let mut f = File::create(&output_list).unwrap();
    for p in playlist { writeln!(f, "{}", p).unwrap(); }
    
    let yaml = format!(
        "path: {}\ntrain: train_phase2_hierarchy.txt\nval: images/val\nnc: 6\nnames:\n  0: Small_Vehicle\n  1: Long_Haul_Truck\n  2: Work_Truck\n  3: Building\n  4: Temp_Structure\n  5: Construction\n",
        base_dir
    );
    std::fs::write(format!("{}/data_phase2.yaml", base_dir), yaml).unwrap();
    println!("✅ Phase 2 Playlist and YAML generated.");
}