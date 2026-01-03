mod modules;
use modules::image_utils::find_tif_images;
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
            // Next step: iterate over all .tif images in the input directory
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
