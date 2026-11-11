#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -d "$SCRIPT_DIR/../apps_training/yolo_model_trainer/venv" ]; then
    PYTHON_BIN="$SCRIPT_DIR/../apps_training/yolo_model_trainer/venv/bin/python3"
elif [ -d "$SCRIPT_DIR/venv" ]; then
    PYTHON_BIN="$SCRIPT_DIR/venv/bin/python3"
else
    PYTHON_BIN="python3"
fi

echo "Starting SAHI post-processor | python=$PYTHON_BIN | dir=$SCRIPT_DIR"

"$PYTHON_BIN" src/post_processor.py --config configs/benchmark_config.yaml "$@"

