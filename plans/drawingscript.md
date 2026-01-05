img="images/train/5_0_0.jpg"
lbl="labels/train/5_0_0.txt"
out="/home/shatadal/SAMDEF/raw_data/processed_tiles/examples_image_labeled/5_0_0.vis.png"
W=1024; H=1024

draw_cmd=$(awk -v W=$W -v H=$H 'NF==5 {
  c=$1; cx=$2; cy=$3; w=$4; h=$5;
  x0=(cx-w/2)*W; y0=(cy-h/2)*H; x1=(cx+w/2)*W; y1=(cy+h/2)*H;
  printf "rectangle %.1f,%.1f %.1f,%.1f; text %.1f,%.1f \"%d\"; ",
         x0,y0,x1,y1,x0+3,y0+12,c
}' "$lbl")

convert "$img" -fill none -stroke red -strokewidth 0.1 \
  -draw "$draw_cmd" \
  "$out" && echo "Saved $out"