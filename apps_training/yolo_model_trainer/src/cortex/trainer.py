import os
import sys
import torch
from ultralytics import YOLO

def run_training():
    #HARDWARE
    if not torch.cuda.is_available():
        sys.exit("Error gpu is not available!")
    
    device = 0
    gpu_name = torch.cuda.get_device_name(0)
    print(f"HARDWARE : {gpu_name}")

    #CONFIGURATION
    model_name = '/home/shatadal/SAMDEF/apps_training/yolo_model_trainer/src/models/yolo26s.pt'
    data_yaml = '/home/shatadal/SAMDEF/raw_data/processed_tiles/data.yaml'
    project_name = 'SAMDEF_ISR'
    run_name = 'Run3'  

    # Verify weights exist before starting to avoid crashing later
    if not os.path.exists(model_name):
        sys.exit(f" ERROR: Checkpoint not found at {model_name}. verify path.")

    model = YOLO(model_name)

   
    print(f"Starting: {run_name}")

    results = model.train(
        data=data_yaml,
        
       
        epochs=200, 
        imgsz=896,
        batch=4,
        nbs=64,
        amp=True,
        cache=False,
        
        
        optimizer='MuSGD',
        lr0=0.01,            
        lrf=0.01,
        momentum=0.937,
        weight_decay=0.0005,
        cos_lr=True,
        warmup_epochs=5,
       
        
        
        box=7.5,
        cls=0.5,
        
        
        mosaic=0.5,          
        scale=0.0,
        translate=0.05,
        fliplr=0.5,
        flipud=0.5,
        degrees=0.0,
        shear=0.0,
        perspective=0.0,
        close_mosaic=40,

        
        workers=8,            
        plots=False,
        save=True,
        save_period=10,
        seed=0,
        deterministic=True,

       
        project=project_name,
        name=run_name,
        exist_ok=True,
        verbose=True,
        device=device,
        resume=False          # Must be False to apply new schedule
    )

    print("-" * 50)
    print("RUN 2 COMPLETE")
    print(f"Weights: {project_name}/{run_name}/weights/best.pt")

if __name__ == '__main__':
    run_training()