#![allow(dead_code)]
use gpui::{rgb, Hsla, Rgba};

pub const CLASS_COUNT: usize = 8;

/// 8 Neon colors for satellite target classes in RGB format
pub const NEON_RGB: [[u8; 3]; CLASS_COUNT] = [
    [255, 0, 128],   // 0: Neon Pink (Long Truck)
    [57, 255, 20],   // 1: Neon Green (Boxy Truck)
    [0, 255, 255],   // 2: Neon Cyan (Cars/Small Vehicles)
    [255, 255, 0],   // 3: Neon Yellow (Building)
    [255, 0, 255],   // 4: Neon Magenta (Container)
    [0, 255, 128],   // 5: Neon Aqua (Construction)
    [255, 110, 0],   // 6: Neon Orange (Tank)
    [0, 255, 200],   // 7: Neon Blue-Green (Container Lot)
];

pub const CLASS_NAMES: [&str; CLASS_COUNT] = [
    "Long Truck",
    "Boxy Truck",
    "Small Vehicle (Car)",
    "Building",
    "Container",
    "Construction",
    "Tank",
    "Container Lot",
];

pub const CLASS_HEX: [&str; CLASS_COUNT] = [
    "#FF0080",
    "#39FF14",
    "#00FFFF",
    "#FFFF00",
    "#FF00FF",
    "#00FF80",
    "#FF6E00",
    "#00FFC8",
];

pub fn class_name(class_id: usize) -> &'static str {
    CLASS_NAMES.get(class_id).copied().unwrap_or("Unknown Target")
}

pub fn class_hex(class_id: usize) -> &'static str {
    CLASS_HEX.get(class_id).copied().unwrap_or("#FFFFFF")
}

pub fn class_rgb(class_id: usize) -> [u8; 3] {
    NEON_RGB
        .get(class_id % CLASS_COUNT)
        .copied()
        .unwrap_or([255, 255, 255])
}

pub fn class_rgba(class_id: usize, alpha: f32) -> Rgba {
    let [r, g, b] = class_rgb(class_id);
    let hex = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    let mut c = rgb(hex);
    c.a = alpha;
    c
}

pub fn class_hsla(class_id: usize, alpha: f32) -> Hsla {
    class_rgba(class_id, alpha).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_names_and_colors() {
        assert_eq!(CLASS_COUNT, 8);
        assert_eq!(class_name(0), "Long Truck");
        assert_eq!(class_name(2), "Small Vehicle (Car)");
        assert_eq!(class_name(6), "Tank");
        assert_eq!(class_name(999), "Unknown Target");

        let [r, g, b] = class_rgb(2);
        assert_eq!((r, g, b), (0, 255, 255)); // Cyan
    }
}
