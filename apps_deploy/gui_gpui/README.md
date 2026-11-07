# SAMDEF GPUI Exploitation Viewer (`gui_gpui`)

A high-performance, GPU-accelerated Geospatial Intelligence (GEOINT) and Target Exploitation interface for SAMDEF, engineered with the [GPUI](https://github.com/zed-industries/zed) UI framework.

---

## 🛰️ Overview & Comparison with `gui_iced`

While `gui_iced` provides a solid baseline for visualizing detection outputs, `gui_gpui` is designed for high-throughput, mission-critical satellite image exploitation with direct GPU quad-painting, zero-allocation TIFF strip decoding, and deep geospatial telemetry.

| Feature / Dimension | `gui_iced` (Legacy) | `gui_gpui` (New Engine) |
| :--- | :--- | :--- |
| **UI Framework** | Iced (CPU/wgpu retained canvas) | GPUI 0.2.2 (Zed's immediate-mode GPU engine) |
| **Rendering Pipeline** | Canvas cache redraws | Direct GPU quad & path rasterization via Vulkan/Blade |
| **TIFF Decoding** | Chunky conversion into RGB image | Direct planar & chunky zero-allocation BGRA strip unpacker |
| **Geospatial Telemetry** | Basic lon/lat display | GDAL 6-param affine projection, DMS/DD formats, GSD estimation |
| **Target Exploitation HUD** | Minimal hover tooltip | Floating Target Inspector card with ground dimensions & real area ($m^2$) |
| **Navigation & Frustum** | Mouse drag / scroll only | Viewport pan/zoom + **Interactive Radar Minimap** with frustum tracking |
| **Exploitation Drawer** | None | Collapsible tabular detection list with search, sorting & 1-click focus |
| **Measurement Tools** | None | **Interactive Geodesic Ruler**: ground distance ($m$/$km$) and azimuth bearing |
| **Panchromatic Filter** | None | Grayscale toggle for enhanced target contrast in dense urban/industrial scenes |
| **Data Export** | None | Multi-format export: **GeoJSON FeatureCollection**, **CSV**, & **Markdown Reports** |
| **Target Isolation** | Global visibility toggles | Visibility checkboxes + **"Solo" class buttons** + confidence slider |

---

## 🎯 8 NEON Target Classes

The viewer preserves the standard 8-class SAMDEF detection schema with high-visibility neon vector overlays:

| ID | Class Name | Neon Color | Hex Code |
| :---: | :--- | :--- | :--- |
| `0` | Long Truck | Neon Pink | `#FF0080` |
| `1` | Boxy Truck | Neon Green | `#39FF14` |
| `2` | Small Vehicle (Car) | Neon Cyan | `#00FFFF` |
| `3` | Building | Neon Yellow | `#FFFF00` |
| `4` | Container | Neon Magenta | `#FF00FF` |
| `5` | Construction | Neon Aqua | `#00FF80` |
| `6` | Tank | Neon Orange | `#FF6E00` |
| `7` | Container Lot | Neon Blue-Green | `#00FFC8` |

---

## 🚀 Key Features

### 1. Viewport & Vector Rendering Engine
- **Infinite Sub-Pixel Zoom & Pan**: Smooth exponential zooming (0.05x to 35x) anchored to the cursor position, with panning via mouse drag or WASD/arrow keys.
- **Hairline Vector Bounding Boxes**: Non-scaled 1.5px/2.5px hollow boxes with corner targeting brackets on hover/select.
- **Magnification Badges**: Target labels and confidence pills automatically scale down when zoomed out to reduce screen clutter, and expand into detailed chips under high magnification.

### 2. Live Geospatial HUD & Telemetry
- Supports GDAL 6-parameter affine geotransforms embedded in GeoTIFFs or JSON outputs.
- Real-time cursor coordinates in WGS-84 Decimal Degrees and Navigational DMS format (`01°19'18.9"N  103°59'43.4"E`).
- Live Ground Sample Distance (GSD) calculation in meters per pixel.
- Detection footprint estimation: calculates ground width, length, and real-world area ($m^2$).

### 3. Interactive Target Exploitation
- **Floating Inspector**: Displays selected/hovered target classification, confidence gauge, bounding coordinates, and an instant **"Focus Target"** button that centers and zooms the viewport directly onto the object.
- **Detection Table Drawer**: Collapsible bottom drawer displaying every visible detection with class filter count, confidence rating, pixel dimensions, and ground area. Clicking any row immediately jumps the camera to that detection.
- **Class Soloing**: Click "Solo" next to any class in the left panel to immediately isolate only those targets, hiding the other 7 classes.

### 4. Interactive Geodesic Ruler
- Toggle Ruler Mode via the toolbar button or hotkey `M`.
- Click two points on the satellite image to measure straight-line ground distance in meters or kilometers using the Haversine geodesic formula, along with the true azimuth bearing (0°–360°).

### 5. Multi-Format Intelligence Exporters
- **GeoJSON**: Exports all active detections as a standard GIS `FeatureCollection` with polygon bounding boxes, pixel bounds, confidence ratings, and ground dimensions for ingestion into QGIS, ArcGIS, or Cesium.
- **CSV**: Tabular dump of detection IDs, class descriptions, confidence scores, pixel coordinates, geographic centers, and ground footprint areas.
- **Intelligence Report**: Markdown summary detailing scene metadata, confidence cutoff, active target breakdown, and class proportion statistics.

---

## ⌨️ Hotkey Controls

| Key | Action |
| :--- | :--- |
| `W` / `↑` | Pan Viewport Up |
| `S` / `↓` | Pan Viewport Down |
| `A` / `←` | Pan Viewport Left |
| `D` / `→` | Pan Viewport Right |
| `+` / `=` | Zoom In |
| `-` / `_` | Zoom Out |
| `0` | Native `1:1` Pixel Zoom (1 screen px = 1 satellite px) |
| `R` / `Space` | Reset / Fit Viewport (`1.0x` centered) |
| `Escape` | Clear Ruler, Target Selection, and Table Drawer |
| `Double-Click` | Focus & Magnify Target (or Fit View on background) |
| `N` / `]` | Next Result / Image File |
| `P` / `[` | Previous Result / Image File |
| `1` – `8` | Toggle Visibility of Class 0 through 7 |
| `L` | Toggle Detection Labels & Badges |
| `M` | Toggle Geodesic Ruler Measurement Tool |
| `T` | Toggle Bottom Detections Table Drawer |
| `G` | Toggle Grayscale Panchromatic Mode |

---

## 📁 Modular SRP Architecture (`<= 260` lines per file)

```text
apps_deploy/gui_gpui/
├── Cargo.toml                  # GPUI, serde, image, tiff dependencies
├── build.rs                    # Linux multiarch XCB/XKB linker symlink resolver
└── src/
    ├── main.rs                 # Window bootstrap & global keystroke observer
    ├── geo.rs                  # GDAL affine geotransform, DMS & Haversine geodesy
    ├── app/
    │   ├── mod.rs              # SamdefApp state, async GeoTIFF/JSON loader & navigation
    │   ├── events.rs           # Smooth scroll zoom, drag/click disambiguation & hotkeys
    │   └── view.rs             # Root Render tree & live viewport bounds capture
    ├── model/
    │   ├── detection.rs        # BBoxCoords, FinalOutput & DisplayDetection
    │   └── palette.rs          # 8 neon target colors & RGBA/HSLA helpers
    ├── io/
    │   ├── file_scanner.rs     # Dataset directory scanner & path resolution
    │   ├── tiff_loader.rs      # Planar & chunky zero-copy BGRA GeoTIFF strip decoder
    │   └── exporter.rs         # GeoJSON, CSV & Markdown intelligence exporters
    ├── viewer/
    │   ├── state.rs            # Sub-pixel cursor zoom invariant & coordinate transforms
    │   ├── hit_test.rs         # Priority target hit-testing & transform unit tests
    │   ├── render.rs           # Ordered multi-layer GPU raster & neon box painter
    │   └── overlays.rs         # GPU-shaped text badges & geodesic ruler layers
    └── ui/
        ├── theme.rs            # Dark GIS palette & reusable button/row primitives
        ├── toolbar.rs          # Fit, 1:1, center-zoom, Ruler, B/W, Radar & Table bar
        ├── status_bar.rs       # Bottom QGIS-style live Lat/Lon, DMS, Pixel & GSD bar
        ├── inspector.rs        # Occluded floating target exploitation HUD card
        ├── minimap.rs          # Interactive radar overview with blips & cyan frustum
        ├── table_drawer.rs     # Occluded bottom detection table & 1-click focus
        └── sidebar/
            ├── mod.rs          # Left sidebar container & header
            ├── file_nav.rs     # Dataset scanner & full scrollable file list
            ├── class_panel.rs  # 8-class visibility toggles, counts & toggleable Solo
            ├── confidence_panel.rs # Interactive drag/click slider & ASCII steppers
            └── telemetry_export.rs # Live scene GIS telemetry & export triggers
```

---

## 🔨 Building & Running

To run unit tests:
```bash
cargo test -p gui_gpui
```

To run the application:
```bash
cargo run -p gui_gpui
```

