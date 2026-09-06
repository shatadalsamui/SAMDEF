use iced::Color;

/// 8 Neon colors for the classes in RGB format
pub const NEON_RGB: [[u8; 3]; 8] = [
    [255, 0, 128],   // 0: Neon Pink (Long Truck)
    [57, 255, 20],    // 1: Neon Green (Boxy Truck)
    [0, 255, 255],    // 2: Neon Cyan (Cars/Small Vehicles)
    [255, 255, 0],    // 3: Neon Yellow (Building)
    [255, 0, 255],    // 4: Neon Magenta (Container)
    [0, 255, 128],    // 5: Neon Aqua (Construction)
    [255, 110, 0],    // 6: Neon Orange (Tank)
    [0, 255, 200],    // 7: Neon Blue-Green (Container Lot)
];

pub const CLASS_NAMES: [&str; 8] = [
    "Long Truck",
    "Boxy Truck",
    "Small Vehicle (Car)",
    "Building",
    "Container",
    "Construction",
    "Tank",
    "Container Lot",
];

pub fn class_name(class_id: usize) -> &'static str {
    CLASS_NAMES.get(class_id).copied().unwrap_or("Unknown")
}

pub fn class_color(class_id: usize) -> Color {
    let [r, g, b] = class_rgb(class_id);
    Color::from_rgb8(r, g, b)
}

pub fn class_rgb(class_id: usize) -> [u8; 3] {
    NEON_RGB
        .get(class_id % NEON_RGB.len())
        .copied()
        .unwrap_or([255, 255, 255])
}
