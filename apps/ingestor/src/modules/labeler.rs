use crate::modules::label_parser::Label;

pub struct ParsedLabel {
    pub class_id: u32,
    pub xmin: u32,
    pub ymin: u32,
    pub xmax: u32,
    pub ymax: u32,
}

/// Parse and filter labels for one image: applies class map and hut size filter.
pub fn prepare_labels(labels: &[Label]) -> Vec<ParsedLabel> {
    labels
        .iter()
        .filter_map(|lbl| {
            let (xmin, ymin, xmax, ymax) = lbl.parse_bounds()?;
            
            // --- THE FIXED ID MAPPING ---
            let class_id = match lbl.properties.type_id {
                89 => 0,  // Container_Shed
                24 => 1,  // Pickup Truck
                18 => 2,  // Small Car
                21 => 3,  // Utility Truck
                19 => 4,  // Bus
                83 => 5,  // Construction Site
                27 => 6,  // Tent
                25 => 7,  // Shed
                60 => 8,  // Storage Tank
                73 => 9,  // Small Building
                _ => return None, 
            };
            
            Some(ParsedLabel {
                class_id,
                xmin,
                ymin,
                xmax,
                ymax,
            })
        })
        .collect()
}

/// Given parsed labels and a tile window, produce YOLO txt content.
pub fn labels_for_tile(
    parsed: &[ParsedLabel],
    tile_x: usize,
    tile_y: usize,
    tile_size: usize,
) -> String {
    let t_min_x = tile_x;
    let t_max_x = tile_x + tile_size;
    let t_min_y = tile_y;
    let t_max_y = tile_y + tile_size;
    let mut out = String::new();

    for p in parsed {
        let pxmin = p.xmin as usize;
        let pxmax = p.xmax as usize;
        let pymin = p.ymin as usize;
        let pymax = p.ymax as usize;

        if pxmax <= t_min_x || pxmin >= t_max_x || pymax <= t_min_y || pymin >= t_max_y {
            continue;
        }

        let c_xmin = pxmin.max(t_min_x).min(t_max_x);
        let c_xmax = pxmax.max(t_min_x).min(t_max_x);
        let c_ymin = pymin.max(t_min_y).min(t_max_y);
        let c_ymax = pymax.max(t_min_y).min(t_max_y);

        if c_xmax <= c_xmin || c_ymax <= c_ymin {
            continue;
        }

        let loc_xmin = (c_xmin - t_min_x) as f64;
        let loc_xmax = (c_xmax - t_min_x) as f64;
        let loc_ymin = (c_ymin - t_min_y) as f64;
        let loc_ymax = (c_ymax - t_min_y) as f64;

        // Normalized YOLO format
        let center_x = ((loc_xmin + loc_xmax) / 2.0) / tile_size as f64;
        let center_y = ((loc_ymin + loc_ymax) / 2.0) / tile_size as f64;
        let norm_w = (loc_xmax - loc_xmin) / tile_size as f64;
        let norm_h = (loc_ymax - loc_ymin) / tile_size as f64;

        out.push_str(&format!(
            "{} {:.6} {:.6} {:.6} {:.6}\n",
            p.class_id, center_x, center_y, norm_w, norm_h
        ));
    }
    out
}
