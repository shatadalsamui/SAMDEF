use crate::modules::label_parser::Label;

pub struct ParsedLabel {
    pub class_id: u32,
    pub xmin: u32,
    pub ymin: u32,
    pub xmax: u32,
    pub ymax: u32,
}

// Parse and filter labels for one image: applies class map and hut size filter.
pub fn prepare_labels(labels: &[Label]) -> Vec<ParsedLabel> {
    labels
        .iter()
        .filter_map(|lbl| {
            let (xmin, ymin, xmax, ymax) = lbl.parse_bounds()?;

            let class_id = match lbl.properties.type_id {
                // 0: Light Vehicle (Small Car, Passenger Car, Pickup)
                17 | 18 | 20 => 0,

                // 1: Boxy/Utility Truck (Utility Truck, Truck, Truck Tractor w/ Box Trailer)
                21 | 23 | 25 => 1,

                // 2: Long Trucks (Bus, Cargo Truck, Tractor, Trailer, Flatbed, Liquid Tanker, Container)
                19 | 24 | 26 | 27 | 28 | 29 | 91 => 2,

                // 3: Small Boats (Maritime Vessel, Motorboat, Sailboat, Tugboat, Fishing Vessel, Yacht)
                40 | 41 | 42 | 44 | 47 | 50 => 3,

                // 4: Large Ships (Barge, Ferry, Container Ship, Oil Tanker)
                45 | 49 | 51 | 52 => 4,

                // 5: Fixed-Wing Aircraft (Fixed-wing, Small Aircraft, Passenger/Cargo Plane)
                11 | 12 | 13 => 5,

                // 6: Rotary-Wing Aircraft (Helicopter)
                15 => 6,

                // 7: Building (ID 73 - Standalone rigid structures)
                73 => 7,

                // 8: Other Structures (Hut, Tent, Shed, Damaged Building, Facility)
                71 | 72 | 76 | 77 => 8,

                // 9: Storage Tank (Circular footprint)
                86 => 9,

                // 10: Shipping Container Lot (High-density grid of containers)
                89 => 10,

                // 11: Construction Site (Disturbed ground texture)
                79 => 11,

                // 12: Railway Assets (Locomotive and all rail car variants)
                33 | 34 | 35 | 36 | 37 | 38 => 12,

                // 13: Engineering & Construction Machinery (Cranes, Excavators, Mixers, etc.)
                32 | 53 | 54 | 55 | 56 | 57 | 59 | 60 | 61 | 62 | 63 | 64 | 65 | 66 => 13,

                // 14: Infrastructure Towers (Pylon, Telecom Tower)
                93 | 94 => 14,

                // Removed/Ignored: Aircraft Hangar (74), Helipad (84), Dam (moved/removed), Vehicle Lot (83)
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
