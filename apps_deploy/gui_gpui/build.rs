use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

fn main() {
    #[cfg(target_os = "linux")]
    {
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let fallback_lib_dir = out_dir.join("lib_symlinks");
        let _ = fs::create_dir_all(&fallback_lib_dir);

        // Standard Linux multiarch library paths
        let search_dirs = [
            "/usr/lib/x86_64-linux-gnu",
            "/usr/lib64",
            "/usr/lib",
            "/lib/x86_64-linux-gnu",
        ];

        let targets = [
            ("libxcb.so", "libxcb.so.1"),
            ("libxkbcommon.so", "libxkbcommon.so.0"),
            ("libxkbcommon-x11.so", "libxkbcommon-x11.so.0"),
        ];

        for (unversioned, versioned) in targets {
            let mut found = false;
            for dir in &search_dirs {
                let dev_file = Path::new(dir).join(unversioned);
                if dev_file.exists() {
                    found = true;
                    break;
                }
            }

            if !found {
                for dir in &search_dirs {
                    let runtime_file = Path::new(dir).join(versioned);
                    if runtime_file.exists() {
                        let symlink_path = fallback_lib_dir.join(unversioned);
                        let _ = fs::remove_file(&symlink_path);
                        let _ = symlink(&runtime_file, &symlink_path);
                        break;
                    }
                }
            }
        }

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let local_lib = manifest_dir.join(".lib");
        if local_lib.exists() {
            println!("cargo:rustc-link-search=native={}", local_lib.display());
        }
        println!("cargo:rustc-link-search=native={}", fallback_lib_dir.display());
    }
}

