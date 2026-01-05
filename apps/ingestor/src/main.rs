mod modules;
use modules::image_utils::{find_tif_images, tile_image};
use rayon::prelude::*;
use modules::label_parser::{load_labels, Label};
use std::collections::HashMap;

fn main() {
    let base_output_dir = "/home/shatadal/SAMDEF/raw_data/processed_tiles";
    let labels_file = "/home/shatadal/SAMDEF_DATA/train_labels/xView_train.geojson"; // used for both train and val splits

    // Define splits to process in one run (train and val)
    let splits = [
        (
            "train",
            "/home/shatadal/SAMDEF_DATA/train_images/",
            format!("{}/images/train", base_output_dir),
            format!("{}/labels/train", base_output_dir),
        ),
        (
            "val",
            "/home/shatadal/SAMDEF_DATA/val_images/",
            format!("{}/images/val", base_output_dir),
            format!("{}/labels/val", base_output_dir),
        ),
    ];

    println!("Output will be saved to: {}", base_output_dir);
    match load_labels(labels_file) {
        Ok(features) => {
            println!("Loaded {} label features.", features.len());
            let label_map = build_label_map(features);
            println!("Label map built for {} images.", label_map.len());
            let tile_size = 1024;
            let stride = 1024; // or 824 for overlap

            let start = std::time::Instant::now();
            for (split_name, image_dir, images_dir, labels_dir) in splits.iter() {
                std::fs::create_dir_all(images_dir).expect("Failed to create images dir");
                std::fs::create_dir_all(labels_dir).expect("Failed to create labels dir");

                let tif_files = find_tif_images(image_dir);
                println!("Processing {}: Found {} .tif images.", split_name, tif_files.len());

                tif_files.par_iter().for_each(|path| {
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

            // Write data.yaml with all 10 classes, matching labeler.rs
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
            // =========================
            // DATA.YAML CONFIGURATION
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
        Err(e) => eprintln!("Error loading labels: {}", e),
    }
}

fn build_label_map(features: Vec<Label>) -> HashMap<String, Vec<Label>> {
    let mut label_map: HashMap<String, Vec<Label>> = HashMap::new();
    for label in features {
        // Index by stem (e.g., "389" from "389.tif") to match file_stem lookups
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
