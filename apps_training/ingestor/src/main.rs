mod modules;
use modules::image_utils::{find_tif_images, tile_image};
use rayon::prelude::*;
use modules::label_parser::{load_labels, Label};
use std::collections::{HashMap, HashSet};
use rand::seq::SliceRandom;

fn main() {
    let base_output_dir = "/home/shatadal/SAMDEF/raw_data/processed_tiles";

    // Load labels once
    let labels_file = "/home/shatadal/SAMDEF_DATA/train_labels/xView_train.geojson";
    let features = match load_labels(labels_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error loading labels: {}", e);
            return;
        }
    };
    println!("Loaded {} total label features.", features.len());

    // Get unique image IDs
    let all_image_ids: HashSet<String> = features.iter().map(|l| {
        std::path::Path::new(&l.properties.image_id)
            .file_stem().and_then(|s| s.to_str()).unwrap_or(&l.properties.image_id).to_string()
    }).collect();
    
    let mut ids_vec: Vec<String> = all_image_ids.into_iter().collect();
    let mut rng = rand::thread_rng();
    ids_vec.shuffle(&mut rng);
    
    // 90/10 Split
    let split_point = (ids_vec.len() as f64 * 0.9) as usize;
    let train_ids: HashSet<String> = ids_vec[..split_point].iter().cloned().collect();
    let val_ids: HashSet<String> = ids_vec[split_point..].iter().cloned().collect();
    
    println!("Total Images: {}", ids_vec.len());
    println!("Train: {} | Val: {}", train_ids.len(), val_ids.len());

    let splits = [
        ("train", &train_ids, format!("{}/images/train", base_output_dir), format!("{}/labels/train", base_output_dir)),
        ("val", &val_ids, format!("{}/images/val", base_output_dir), format!("{}/labels/val", base_output_dir)),
    ];

    println!("Output will be saved to: {}", base_output_dir);
    
    // FIXED CONFIG 
    let tile_size = 896;
    let stride = 716; // 20% Overlap

    let image_dir = "/home/shatadal/SAMDEF_DATA/train_images/";
    let tif_files = find_tif_images(image_dir);
    println!("Found {} .tif images.", tif_files.len());

    let start = std::time::Instant::now();
    for (split_name, split_ids, images_dir, labels_dir) in splits.iter() {
        std::fs::create_dir_all(images_dir).expect("Failed to create images dir");
        std::fs::create_dir_all(labels_dir).expect("Failed to create labels dir");

        let filtered_features: Vec<Label> = features.iter().filter(|l| {
            let key = std::path::Path::new(&l.properties.image_id)
                .file_stem().and_then(|s| s.to_str()).unwrap_or(&l.properties.image_id).to_string();
            split_ids.contains(&key)
        }).cloned().collect();
        
        let label_map = build_label_map(filtered_features);

        let filtered_tifs: Vec<std::path::PathBuf> = tif_files.iter().filter(|p| {
            let stem = p.file_stem().unwrap().to_str().unwrap().to_string();
            split_ids.contains(&stem)
        }).cloned().collect();

        println!("Processing {} images for {}...", filtered_tifs.len(), split_name);

        filtered_tifs.par_iter().for_each(|path| {
            let original_name = path.file_stem().unwrap().to_str().unwrap();
            let empty: Vec<Label> = Vec::new();
            let image_labels = label_map.get(original_name).unwrap_or(&empty);
            
            let _ = tile_image(
                path,
                images_dir,
                labels_dir,
                tile_size,
                stride,
                original_name,
                image_labels,
            );
        });
    }

    println!("Total tiling completed in {:.2?}", start.elapsed());

    // FIXED CLASS NAMES ---
    let class_names = [
        "Small_Vehicle",
        "Long_Haul_Truck",
        "Work_Truck",
        "Building",
        "Temp_Structure",
        "Construction"
    ];
    
    let mut yaml_content = format!(
        "path: {}\ntrain: images/train\nval: images/val\nnc: 6\nnames:\n",
        base_output_dir
    );
    for (i, name) in class_names.iter().enumerate() {
        yaml_content.push_str(&format!("  {}: {}\n", i, name));
    }
    std::fs::write(format!("{}/data.yaml", base_output_dir), yaml_content).expect("Failed to write data.yaml");
}

fn build_label_map(features: Vec<Label>) -> HashMap<String, Vec<Label>> {
    let mut label_map: HashMap<String, Vec<Label>> = HashMap::new();
    for label in features {
        let key = std::path::Path::new(&label.properties.image_id)
            .file_stem().and_then(|s| s.to_str()).unwrap_or(&label.properties.image_id).to_string();
        label_map.entry(key).or_insert_with(Vec::new).push(label);
    }
    label_map
}
