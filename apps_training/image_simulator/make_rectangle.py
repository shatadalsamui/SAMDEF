import os
import numpy as np
import tifffile
from PIL import Image

# Configuration
SOURCE_DIR = "/home/shatadal/SAMDEF_DATA/simulations/long_rectangle/raw"
OUTPUT_DIR = "/home/shatadal/SAMDEF_DATA/simulations/long_rectangle/ready"
ROWS = 3

os.makedirs(OUTPUT_DIR, exist_ok=True)

def create_tight_rectangle():
    img_files = sorted([f for f in os.listdir(SOURCE_DIR) if f.endswith(".tif")])
    if not img_files:
        print("No files found!")
        return

    # 1. Pre-scan to calculate dimensions and offsets
    row_widths = [0, 0, 0]
    row_heights = [0, 0, 0] 
    img_data = []

    print("Scanning images for tight-fit dimensions...")
    for index, img_name in enumerate(img_files):
        row = index % ROWS
        with Image.open(os.path.join(SOURCE_DIR, img_name)) as img:
            w, h = img.size
            img_data.append((img_name, w, h, row))
            row_widths[row] += w
            if h > row_heights[row]:
                row_heights[row] = h

    # Calculate final canvas size
    max_total_width = max(row_widths)
    total_height = sum(row_heights)
    y_offsets = [0, row_heights[0], row_heights[0] + row_heights[1]]

    print(f"Canvas size: {max_total_width}x{total_height}")
    canvas = Image.new("RGB", (max_total_width, total_height), (0, 0, 0))

    # 2. Build the canvas
    current_x = [0, 0, 0]
    for img_name, w, h, row in img_data:
        x_pos = current_x[row]
        y_pos = y_offsets[row]
        
        with Image.open(os.path.join(SOURCE_DIR, img_name)) as img:
            canvas.paste(img.convert("RGB"), (x_pos, y_pos))
        
        current_x[row] += w
        if index % 10 == 0:
            print(f"Stitching {index}/{len(img_files)} images...")

    # 3. Save as Tiled TIFF using tifffile (The Fix for Rust)
    output_file = os.path.join(OUTPUT_DIR, "stress_rectangle_tight.tif")
    print(f"Converting to array and saving tiled format to {output_file}...")
    
    # Convert PIL to Numpy array for tifffile
    img_array = np.array(canvas)
    
    # tile=(512, 512) makes the file readable by your 896x896 Rust Virtual Tiler
    # compression='deflate' is 100% LOSSLESS (keeps all Run 15 quality)
    tifffile.imwrite(
        output_file,
        img_array,
        tile=(512, 512),
        compression='deflate',
        photometric='rgb',
        planarconfig='contig'
    )
    
    print("Done! The file is now optimized for the Rust pipeline.")

if __name__ == "__main__":
    # Ensure you have tifffile installed: pip install tifffile
    create_tight_rectangle()