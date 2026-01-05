from ultralytics import YOLO
import shutil
import os

def download_yolo_weights(model_name: str, dest_path: str):
    model = YOLO(model_name)  # This triggers download if not present
    cache_dir = os.path.expanduser("~/.cache/ultralytics/")
    for root, dirs, files in os.walk(cache_dir):
        if model_name in files:
            src = os.path.join(root, model_name)
            shutil.copy2(src, dest_path)
            print(f"Copied {src} to {dest_path}")
            return
    print(f"Could not find {model_name} in Ultralytics cache.")

if __name__ == "__main__":
    download_yolo_weights("yolo11s.pt", "src/models/yolo11s.pt")
