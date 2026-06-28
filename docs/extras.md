for sub in apps_deploy/* apps_training/ingestor; do [ -f "$sub/Cargo.toml" ] && (cd "$sub" && echo "=== Checking $sub ===" && cargo check --all-targets); done 2> errors.txt

cargo check --workspace --all-targets 2> errors.txt