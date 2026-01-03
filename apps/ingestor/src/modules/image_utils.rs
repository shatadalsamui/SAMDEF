use std::fs;

let image_dir = "/home/shatadal/SAMDEF_DATA/train_images/";
let tif_files = fs::read_dir(image_dir)
    .expect("Failed to read image directory")
    .filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.extension()? == "tif" {
            Some(path)
        } else {
            None
        }
    })
    .collect::<Vec<_>>();

println!("Found {} .tif images.", tif_files.len());
for path in &tif_files {
    println!("Image: {}", path.display());
}