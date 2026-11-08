#![allow(dead_code)]
use gpui::{rgb, Hsla, Rgba};

pub const CLASS_COUNT: usize = 15;

/// 15 Neon colors for satellite target classes in RGB format
pub const NEON_RGB: [[u8; 3]; CLASS_COUNT] = [
    [0, 229, 255],   // 0: Light Vehicle (Neon Cyan)
    [57, 255, 20],   // 1: Boxy Truck (Neon Lime)
    [0, 255, 136],   // 2: Long Truck (Neon Emerald)
    [41, 121, 255],  // 3: Small Boat (Electric Blue)
    [101, 31, 255],  // 4: Large Ship (Deep Indigo)
    [255, 255, 0],   // 5: Fixed Wing (Bright Yellow)
    [255, 109, 0],   // 6: Helicopter (Neon Orange)
    [255, 23, 68],   // 7: Building (Neon Red)
    [255, 82, 82],   // 8: Other Structure (Coral)
    [213, 0, 249],   // 9: Storage Tank (Neon Purple)
    [255, 214, 0],   // 10: Container Lot (Amber/Gold)
    [255, 145, 0],   // 11: Construction Site (Warm Ochre)
    [0, 191, 165],   // 12: Railway (Mint/Teal)
    [245, 0, 87],    // 13: Engineering Machinery (Hot Pink)
    [224, 64, 251],  // 14: Tower Pylon (Lavender)
];

pub const CLASS_NAMES: [&str; CLASS_COUNT] = [
    "Light Vehicle",
    "Boxy Truck",
    "Long Truck",
    "Small Boat",
    "Large Ship",
    "Fixed Wing",
    "Helicopter",
    "Building",
    "Other Structure",
    "Storage Tank",
    "Container Lot",
    "Construction Site",
    "Railway",
    "Engineering Machinery",
    "Tower Pylon",
];

pub const CLASS_HEX: [&str; CLASS_COUNT] = [
    "#00E5FF",
    "#39FF14",
    "#00FF88",
    "#2979FF",
    "#651FFF",
    "#FFFF00",
    "#FF6D00",
    "#FF1744",
    "#FF5252",
    "#D500F9",
    "#FFD600",
    "#FF9100",
    "#00BFA5",
    "#F50057",
    "#E040FB",
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
        assert_eq!(CLASS_COUNT, 15);
        assert_eq!(class_name(0), "Light Vehicle");
        assert_eq!(class_name(1), "Boxy Truck");
        assert_eq!(class_name(7), "Building");
        assert_eq!(class_name(14), "Tower Pylon");
        assert_eq!(class_name(999), "Unknown Target");

        let [r, g, b] = class_rgb(0);
        assert_eq!((r, g, b), (0, 229, 255)); // Cyan
    }
}
