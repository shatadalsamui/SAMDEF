from ultralytics import YOLO
import torch
import os
import sys

def run_training():
    """
    The main training pipeline logic.
    Called by main.py
    """
    # --- 1. CONFIGURATION ---
    data_yaml = '/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml'
    project_name = 'SAMDEF_Satellite_Ops'
    run_name = 'Run6_HighDef_100Epochs'

    # --- 2. HARDWARE CHECK ---
    device = 0 if torch.cuda.is_available() else 'cpu'
    
    # --- 3. LOAD MODEL ---
    # We resolve the path relative to THIS file to avoid "File Not Found" errors
    # This finds 'src/models/yolo11s.pt' automatically
    current_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(current_dir) # Go up to 'src'
    model_path = os.path.join(project_root, 'models', 'yolo11s.pt')

    print(f"Loading YOLO11s Model: {model_path}")
    model = YOLO(model_path)

    # --- 4. START TRAINING ---
    print(f"Starting Training Run: {run_name}")
    results = model.train(
        data=data_yaml,
        
        # Training Dynamics
        epochs=100,          
        imgsz=640,           
        batch=16,            
        
        # Hardware
        device=device,       
        workers=12,          
        
        # Tuning
        patience=15,         
        cos_lr=True,         
        optimizer='auto',    
        
        # Logging
        project=project_name,
        name=run_name,
        exist_ok=True,
        save=True,
        save_period=10,      
        plots=True
    )

    print("-" * 50)
    print("TRAINING COMPLETE")
    # Note: Ultralytics saves relative to the CWD (Current Working Directory)
    print(f"   Best Model: {project_name}/{run_name}/weights/best.pt")

if __name__ == '__main__':
    # Allows testing this script individually if needed
    run_training()