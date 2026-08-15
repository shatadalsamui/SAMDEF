import argparse
import os
import sys
from functools import partial
from dotenv import load_dotenv

load_dotenv()

import tqdm

# Patch tqdm to update progress bars only once every 30 seconds (must run before torch/trainer import)
tqdm.tqdm = partial(tqdm.tqdm, mininterval=30.0)

import torch

# Now we import your module. Ultralytics (inside trainer) will now see the patched tqdm.
from cortex import trainer


def print_banner():
    print("SAMDEF SYSTEMS")


def check_environment():
    """Verifies that the system is ready for training."""
    print("SYSTEM DIAGNOSTICS:")

    # Check GPU
    if torch.cuda.is_available():
        vram = torch.cuda.get_device_properties(0).total_memory / 1e9
        print(f"Gpu detected: {torch.cuda.get_device_name(0)} ({vram:.2f} gb vram)")
    else:
        print("Warning: No gpu detected. Training will be extremely slow.")

    # Check Data
    data_path = os.getenv("DATA_YAML")
    if not data_path:
        sys.exit("Error: DATA_YAML must be set in .env")

    if os.path.exists(data_path):
        print("Data configuration found.")
    else:
        print(f"Error: data.yaml not found at {data_path}")
        print("Did you run the Rust ingestor?")
        sys.exit(1)

    print("--------------------------------------------------")


def main():
    print_banner()

    parser = argparse.ArgumentParser(description="SAMDEF Brain Controller")
    parser.add_argument(
        "mode",
        choices=["train", "run"],
        help="Mode to run: 'train' for learning, 'run' for inference",
    )

    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)

    args = parser.parse_args()

    check_environment()

    if args.mode == "train":
        print("INITIATE TRAINING")
        try:
            trainer.run_training()
        except KeyboardInterrupt:
            print("\nTraining interrupted by user.")
            sys.exit(0)
        except Exception as e:
            print(f"\nCritical error: {e}")
            sys.exit(1)

    elif args.mode == "run":
        print("STARTING INFERENCE NODE")
        print("Inference logic pending implementation.")


if __name__ == "__main__":
    main()
