import os
from PIL import Image

SOURCE_DIR = "/home/shatadal/SAMDEF_DATA/simulations/large_square/raw"
OUTPUT_DIR = "/home/shatadal/SAMDEF_DATA/simulations/large_square/ready"
ROWS = 8  # Increased rows to make it look like a square

os.makedirs(OUTPUT_DIR, exist_ok=True)

def create_tight_square():
    img_files = sorted([f for f in os.listdir(SOURCE_DIR) if f.endswith(".tif")])
    if not img_files:
        print(f"No files found in {SOURCE_DIR}!")
        return

    # 1. Pre-scan to calculate row dimensions
    row_widths = [0] * ROWS
    row_max_heights = [0] * ROWS
    img_metadata = []

    for index, img_name in enumerate(img_files):
        row = index % ROWS
        img_path = os.path.join(SOURCE_DIR, img_name)
        with Image.open(img_path) as img:
            w, h = img.size
            img_metadata.append((img_name, w, h, row))
            row_widths[row] += w
            if h > row_max_heights[row]:
                row_max_heights[row] = h

    # 2. Calculate Canvas Dimensions
    canvas_w = max(row_widths)
    canvas_h = sum(row_max_heights)
    
    # 3. Calculate Vertical Start Points (Cumulative height of previous rows)
    y_offsets = [0] * ROWS
    for i in range(1, ROWS):
        y_offsets[i] = y_offsets[i-1] + row_max_heights[i-1]

    print(f"Building Tight Square: {canvas_w}x{canvas_h} pixels using {ROWS} rows.")
    canvas = Image.new("RGB", (canvas_w, canvas_h), (0, 0, 0))

    # 4. Paste images with a horizontal cursor for each row
    current_x = [0] * ROWS
    for img_name, w, h, row in img_metadata:
        x_pos = current_x[row]
        y_pos = y_offsets[row]
        
        img_path = os.path.join(SOURCE_DIR, img_name)
        with Image.open(img_path) as img:
            canvas.paste(img.convert("RGB"), (x_pos, y_pos))
        
        current_x[row] += w
        print(f"Pasted {img_name} in Row {row} at X: {x_pos}, Y: {y_pos}")

    # 5. Save
    output_file = os.path.join(OUTPUT_DIR, "stress_square_tight.tif")
    print(f"Saving to {output_file}...")
    canvas.save(output_file, compression=None)
    print("Done! Square-ish simulation ready.")

if __name__ == "__main__":
    create_tight_square()