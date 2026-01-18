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
    println!("Loaded {} label features.", features.len());

    // Get unique image IDs
    let mut all_image_ids: HashSet<String> = features.iter().map(|l| {
        std::path::Path::new(&l.properties.image_id)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&l.properties.image_id)
            .to_string()
    }).collect();
    let mut ids_vec: Vec<String> = all_image_ids.into_iter().collect();
    let mut rng = rand::thread_rng();
    ids_vec.shuffle(&mut rng);
    let split_point = (ids_vec.len() as f64 * 0.9) as usize;
    let train_ids: HashSet<String> = ids_vec[..split_point].iter().cloned().collect();
    let val_ids: HashSet<String> = ids_vec[split_point..].iter().cloned().collect();
    println!("Train images: {}, Val images: {}", train_ids.len(), val_ids.len());

    // Define splits (both use train_images as source)
    let splits = [
        (
            "train",
            &train_ids,
            format!("{}/images/train", base_output_dir),
            format!("{}/labels/train", base_output_dir),
        ),
        (
            "val",
            &val_ids,
            format!("{}/images/val", base_output_dir),
            format!("{}/labels/val", base_output_dir),
        ),
    ];

    println!("Output will be saved to: {}", base_output_dir);
    let tile_size = 1024;
    let stride = 1024;

    // Cache tif_files once (since image_dir is the same for both splits)
    let image_dir = "/home/shatadal/SAMDEF_DATA/train_images/";
    let tif_files = find_tif_images(image_dir);
    println!("Found {} .tif images in {}.", tif_files.len(), image_dir);

    let start = std::time::Instant::now();
    for (split_name, split_ids, images_dir, labels_dir) in splits.iter() {
        std::fs::create_dir_all(images_dir).expect("Failed to create images dir");
        std::fs::create_dir_all(labels_dir).expect("Failed to create labels dir");

        // Filter features for this split
        let filtered_features: Vec<Label> = features.iter().filter(|l| {
            let key = std::path::Path::new(&l.properties.image_id)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&l.properties.image_id)
                .to_string();
            split_ids.contains(&key)
        }).cloned().collect();
        println!("Filtered {} label features for {}.", filtered_features.len(), split_name);
        let label_map = build_label_map(filtered_features);
        println!("Label map built for {} images in {}.", label_map.len(), split_name);

        // Filter tif files to those in this split
        let filtered_tifs: Vec<std::path::PathBuf> = tif_files.iter().filter(|p| {
            let stem = p.file_stem().unwrap().to_str().unwrap().to_string();
            split_ids.contains(&stem)
        }).cloned().collect();
        println!("Processing {} images for {}.", filtered_tifs.len(), split_name);

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
        println!("{} processing completed.", split_name);
    }
    let duration = start.elapsed();
    println!("Total tiling completed in {:.2?}", duration);

    // Write data.yaml
    let class_names = [
        "Container_Shed",    // 0: 94
        "Pickup_Truck",      // 1: 24
        "Small_Car",         // 2: 18
        "Motorbike",         // 3: 21
        "Bus_Truck",         // 4: 19
        "Construction_Site", // 5: 82
        "Tent",              // 6: 44
        "Shed",              // 7: 45
        "Container_Shed2",   // 8: 58
        "Huts_Small_Buildings" // 9: 73
    ];
    let mut yaml_content = format!(
        "path: {}\ntrain: images/train\nval: images/val\nnames:\n",
        base_output_dir
    );
    for (i, name) in class_names.iter().enumerate() {
        yaml_content.push_str(&format!("  {}: {}\n", i, name));
    }
    std::fs::write(format!("{}/data.yaml", base_output_dir), yaml_content)
        .expect("Failed to write data.yaml");
}

fn build_label_map(features: Vec<Label>) -> HashMap<String, Vec<Label>> {
    let mut label_map: HashMap<String, Vec<Label>> = HashMap::new();
    for label in features {
        let key = std::path::Path::new(&label.properties.image_id)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&label.properties.image_id)
            .to_string();
        label_map
            .entry(key)
            .or_insert_with(Vec::new)
            .push(label);
    }
    label_map
}
