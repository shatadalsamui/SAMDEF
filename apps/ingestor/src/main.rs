mod modules;
use modules::image_utils::{find_tif_images, tile_image};
use rayon::prelude::*;
use modules::label_parser::{load_labels, Label};
use std::collections::HashMap;

fn main() {
    let labels_file = "/home/shatadal/SAMDEF_DATA/train_labels/xView_train.geojson";
    let base_output_dir = "/home/shatadal/SAMDEF/raw_data/processed_tiles";
    let images_dir = format!("{}/images/train", base_output_dir);
    let labels_dir = format!("{}/labels/train", base_output_dir);
    std::fs::create_dir_all(&images_dir).expect("Failed to create images dir");
    std::fs::create_dir_all(&labels_dir).expect("Failed to create labels dir");
    println!("Output will be saved to: {}", base_output_dir);
    match load_labels(labels_file) {
        Ok(features) => {
            println!("Loaded {} label features.", features.len());
            let label_map = build_label_map(features);
            println!("Label map built for {} images.", label_map.len());
            let image_dir = "/home/shatadal/SAMDEF_DATA/train_images/";
            let tile_size = 1024;
            let stride = 1024; // or 824 for overlap

            let tif_files = find_tif_images(image_dir);
            println!("Found {} .tif images.", tif_files.len());

            let start = std::time::Instant::now();
            tif_files.par_iter().for_each(|path| {
                let original_name = path.file_stem().unwrap().to_str().unwrap();
                let empty: Vec<Label> = Vec::new();
                let image_labels = label_map.get(original_name).unwrap_or(&empty);
                let _ = tile_image(path, base_output_dir, tile_size, stride, original_name, image_labels);
            });
            let duration = start.elapsed();
            println!("Tiling completed in {:.2?}", duration);

            // Write data.yaml
            let yaml_content = format!(
                "path: {}\ntrain: images/train\nval: images/train\nnames:\n  0: Building\n  1: Pickup\n  2: Car\n  3: Motorbike\n  4: Truck\n",
                base_output_dir
            );
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
