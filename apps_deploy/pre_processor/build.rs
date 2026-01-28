use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=protos/inference_manifest.proto");

    let proto_path = "protos/inference_manifest.proto";
    let proto_dir = "protos";

    // Check if proto file exists
    if !Path::new(proto_path).exists() {
        panic!("Proto file not found: {}", proto_path);
    }

    // Compile the protobuf
    prost_build::compile_protos(&[proto_path], &[proto_dir])
        .expect("Failed to compile protobuf");

    // prost_build generates _.rs for single proto files, but we need inference_manifest.rs
    // Copy the generated file to the expected location
    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_file = Path::new(&out_dir).join("_.rs");
    let expected_file = Path::new(&out_dir).join("inference_manifest.rs");

    if generated_file.exists() && !expected_file.exists() {
        fs::copy(&generated_file, &expected_file)
            .expect("Failed to copy generated protobuf file");
    }
}