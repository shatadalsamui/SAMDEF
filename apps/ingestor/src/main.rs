mod modules;
use modules::image_utils::{find_tif_images, tile_image};
use rayon::prelude::*;
use modules::label_parser::{load_labels, Label};
use std::collections::HashMap;

fn main() {
    let labels_file = "/home/shatadal/SAMDEF_DATA/train_labels/xView_train.geojson";
    let output_labels_dir = "/home/shatadal/SAMDEF/raw_data";
    println!("Output labels will be saved to: {}", output_labels_dir);
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
                let _ = tile_image(path, output_labels_dir, tile_size, stride, original_name);
            });
            let duration = start.elapsed();
            println!("Tiling completed in {:.2?}", duration);
        }
        Err(e) => eprintln!("Error loading labels: {}", e),
    }
}

fn build_label_map(features: Vec<Label>) -> HashMap<String, Vec<Label>> {
    let mut label_map: HashMap<String, Vec<Label>> = HashMap::new();
    for label in features {
        // Use image_id from label.properties
        label_map
            .entry(label.properties.image_id.clone())
            .or_insert_with(Vec::new)
            .push(label);
    }
    label_map
}
