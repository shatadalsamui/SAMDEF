import sys
from pathlib import Path

# Import YOLO from ultralytics if available
try:
	from ultralytics import YOLO
except ImportError:
	print("[ERROR] Ultralytics YOLO is not installed. Please install with 'pip install ultralytics'.")
	sys.exit(1)

def main():
	# Path to best.pt (absolute path provided by user)
	weights_path = Path("/home/shatadal/SAMDEF/apps/brain_python/src/SAMDEF_ISR/Run2/weights/best.pt")
	# Path to validation data (absolute path provided by user)
	data_path = Path("/home/shatadal/SAMDEF/raw_data/processed_tiles/data_phase1.yaml")

	if not weights_path.exists():
		print(f"[ERROR] Weights file not found: {weights_path}")
		sys.exit(1)
	if not data_path.exists():
		print(f"[ERROR] Data file not found: {data_path}")
		sys.exit(1)

	print(f"[INFO] Loading model from: {weights_path}")
	model = YOLO(str(weights_path))

	print(f"[INFO] Running validation on: {data_path}")
	results = model.val(data=str(data_path))

	print("\n[INFO] Validation Results:")
	print(results)
	# Optionally, print metrics in a more readable way
	if hasattr(results, 'metrics'):
		print("\n[INFO] Metrics:")
		for k, v in results.metrics.items():
			print(f"{k}: {v}")

if __name__ == "__main__":
	main()
