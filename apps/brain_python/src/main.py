import sys
import os
import argparse
import torch
from cortex import trainer  # Imports your new trainer module

def print_banner():
    print("="*50)
    print("       SAMDEF SATELLITE INTELLIGENCE SYSTEM       ")
    print("             Brain Module v1.0                    ")
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
    
    # Argument Parser to handle "train" vs "run"
    parser = argparse.ArgumentParser(description="SAMDEF Brain Controller")
    parser.add_argument('mode', choices=['train', 'run'], help="Mode to run: 'train' for learning, 'run' for inference")
    
    # If no arguments are passed, default to printing help
    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)
        
    args = parser.parse_args()

    check_environment()

    if args.mode == 'train':
        print("[*] COMMAND RECEIVED: INITIATE TRAINING PROTOCOL")
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
        # Placeholder for your future Kafka/Inference logic
        # You can import a separate module here later, e.g., cortex.inference.run()
        print("    -> Logic pending implementation. (Focus on training first!)")

if __name__ == "__main__":
    main()
