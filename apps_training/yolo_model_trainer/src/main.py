import sys
import os
import argparse
import tqdm
from functools import partial

# --- GLOBAL SCROLLING FIX ---
# This must run before 'import torch' or 'from cortex import trainer'
# It forces all progress bars to update only once every 30 seconds.
tqdm.tqdm = partial(tqdm.tqdm, mininterval=30.0)

import torch
# Now we import your module. Ultralytics (inside trainer) will now see the patched tqdm.
from cortex import trainer  

def print_banner():
    print("="*50)
    print("       SAMDEF SYSTEMS       ")
    print("="*50)

def check_environment():
    """Verifies that the system is ready for heavy lifting."""
    print("[*] System Diagnostics:")
    
    # 1. Check GPU
    if torch.cuda.is_available():
        vram = torch.cuda.get_device_properties(0).total_memory / 1e9
        print(f"    ✓ GPU Detected: {torch.cuda.get_device_name(0)} ({vram:.2f} GB VRAM)")
    else:
        print("    ! WARNING: No GPU detected. Training will be extremely slow.")

    # 2. Check Data
    data_path = '/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml'
    if os.path.exists(data_path):
        print("    ✓ Data Configuration Found")
    else:
        print(f"    X ERROR: data.yaml not found at {data_path}")
        print("      Did you run the Rust Ingestor?")
        sys.exit(1)
    
    print("-" * 50)

def main():
    print_banner()
    
    parser = argparse.ArgumentParser(description="SAMDEF Brain Controller")
    parser.add_argument('mode', choices=['train', 'run'], help="Mode to run: 'train' for learning, 'run' for inference")
    
    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)
        
    args = parser.parse_args()

    check_environment()

    if args.mode == 'train':
        print("[*] COMMAND RECEIVED: INITIATE TRAINING ")
        try:
            trainer.run_training()
        except KeyboardInterrupt:
            print("\n[!] Training interrupted by user.")
            sys.exit(0)
        except Exception as e:
            print(f"\n[X] CRITICAL ERROR: {e}")
            sys.exit(1)

    elif args.mode == 'run':
        print("[*] COMMAND RECEIVED: STARTING INFERENCE NODE")
        print("    -> Logic pending implementation. (Focus on training first!)")

if __name__ == "__main__":
    main()
