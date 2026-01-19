import os
import sys
from ultralytics import YOLO
import torch

# Note: The scrolling fix is in main.py, so this file is clean.

def run_training():
    # --- 1. CONFIGURATION ---
    data_yaml = '/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml'
    project_name = 'SAMDEF_Satellite_Ops'
    run_name = 'Run7_YOLO26_NoPlots_Batch8' 

    # --- 2. HARDWARE CHECK ---
    device = 0 if torch.cuda.is_available() else 'cpu'
    
    # --- 3. LOAD MODEL ---
    print("Loading YOLO26s Model...")
    model = YOLO("yolo26s.pt") 

    # --- 4. START TRAINING ---
    print(f"Starting Training Run: {run_name}")
    results = model.train(
        data=data_yaml,
        
        # --- THE AGGRESSIVE CONFIG ---
        epochs=100,
        patience=25,        # Stop if no improvement in 25 epochs
        imgsz=640,           
        batch=8,            # <--- BATCH 8 (Balanced for 8GB VRAM)
        
        # --- THE SATELLITE ESSENTIALS ---
        degrees=180.0,      # Rotate images randomly 0-180 degrees
        flipud=0.5,         # 50% chance to flip upside down (standard for satellite)
        fliplr=0.5,         # 50% chance to flip left-to-right
        
        # --- THE CRASH FIX ---
        plots=False,         # <--- FALSE. Skips the "End-of-Epoch" crash.
        
        # --- MEMORY PROTECTION ---
        cache=False,         
        overlap_mask=False,  
        amp=True,            
        
        # --- HARDWARE ---
        device=device,       
        workers=12,          # 12 for high-speed data loading
        
        # --- OPTIMIZER ---
        optimizer='SGD',
        cos_lr=True,        # Use a smooth "Cosine" learning rate curve
        label_smoothing=0.1,# Help the model handle pixelated/blurry satellite edges
        
        # --- LOGGING ---
        project=project_name,
        name=run_name,
        exist_ok=True,
        save=True,
        save_period=5,       
        verbose=True
    )

    print("-" * 50)
    print("TRAINING COMPLETE")
    print(f"   Best Model: {project_name}/{run_name}/weights/best.pt")

if __name__ == '__main__':
    run_training()