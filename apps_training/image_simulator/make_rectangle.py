import os
from PIL import Image

SOURCE_DIR = "/home/shatadal/SAMDEF_DATA/simulations/long_rectangle/raw"
OUTPUT_DIR = "/home/shatadal/SAMDEF_DATA/simulations/long_rectangle/ready"
ROWS = 3

os.makedirs(OUTPUT_DIR, exist_ok=True)

def create_tight_rectangle():
    img_files = sorted([f for f in os.listdir(SOURCE_DIR) if f.endswith(".tif")])
    if not img_files:
        print("No files found!")
        return

    # To calculate the final canvas size, we need to pre-scan the rows
    row_widths = [0, 0, 0]
    row_heights = [0, 0, 0] # We'll track max height per row for vertical offset
    
    # Store loaded dimensions to avoid opening files twice
    img_data = []

    for index, img_name in enumerate(img_files):
        row = index % ROWS
        with Image.open(os.path.join(SOURCE_DIR, img_name)) as img:
            w, h = img.size
            img_data.append((img_name, w, h, row))
            row_widths[row] += w
            if h > row_heights[row]:
                row_heights[row] = h

    # Final Canvas Dimensions
    max_total_width = max(row_widths)
    total_height = sum(row_heights)

    print(f"Creating tight canvas: {max_total_width}x{total_height}")
    canvas = Image.new("RGB", (max_total_width, total_height), (0, 0, 0))

    # Track current horizontal 'cursor' for each row
    current_x = [0, 0, 0]
    # Calculate vertical start points for each row
    y_offsets = [0, row_heights[0], row_heights[0] + row_heights[1]]

    for img_name, w, h, row in img_data:
        x_pos = current_x[row]
        y_pos = y_offsets[row]
        
        with Image.open(os.path.join(SOURCE_DIR, img_name)) as img:
            canvas.paste(img.convert("RGB"), (x_pos, y_pos))
        
        # Advance the cursor for this row only
        current_x[row] += w
        print(f"Pasted {img_name} in Row {row} at X: {x_pos}")

    output_file = os.path.join(OUTPUT_DIR, "stress_rectangle_tight.tif")
    print(f"Saving to {output_file}...")
    canvas.save(output_file, compression=None)
    print("Done! Zero wasted horizontal space within rows.")

if __name__ == "__main__":
    create_tight_rectangle()