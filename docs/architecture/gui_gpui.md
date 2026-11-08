# SAMDEF GIS Viewer (`gui_gpui`) Architecture

## 1. Overview

`gui_gpui` is SAMDEF's native, GPU-accelerated Geospatial Intelligence (GEOINT) and Target Viewer desktop workstation, built in Rust on top of [GPUI 0.2.2](https://github.com/zed-industries/zed) (Zed's immediate-mode Vulkan/Blade GPU rendering engine).

It replaces the legacy `gui_iced` viewer with direct instanced GPU primitive rendering, Planar Configuration 2 GeoTIFF strip decoding, GDAL 6-parameter affine georeferencing, geodesic Haversine distance/azimuth measurement, a live radar minimap, target inspection HUD, tabular detection drawer, and multi-format intelligence exports (GeoJSON, CSV, Markdown).

---

## 2. Core Engineering Principles

### 2.1 Strict Single-Responsibility Principle (SRP) & File Bounds
Every source file in `apps_deploy/gui_gpui/src/` owns a single cohesive responsibility and is strictly capped below **250–300 lines**:
- `app/`: Application state (`mod.rs`), mouse/scroll/hotkey dispatch (`events.rs`), text input handling (`input_keys.rs`), and root composition (`view.rs`).
- `viewer/`: Sub-pixel cursor-invariant affine math (`state.rs`), priority smallest-area hit-testing (`hit_test.rs`), ordered GPU raster/vector layers (`render.rs`), and shaped glyph badges + geodesic ruler (`overlays.rs`).
- `io/`: Dataset scanner (`file_scanner.rs`), zero-copy Planar & Chunky BGRA GeoTIFF strip decoder (`tiff_loader.rs`), and GeoJSON/CSV/Markdown intelligence exporters (`exporter.rs`).
- `model/`: Detector `*_results.json` schema (`detection.rs`) and 15-class neon taxonomy (`palette.rs`).
- `ui/`: Client-side window titlebar & resize handles (`titlebar.rs`), top toolbar (`toolbar.rs`), bottom QGIS-style status bar (`status_bar.rs`), radar minimap (`minimap.rs`), target inspector (`inspector.rs`), bottom detection table (`table_drawer.rs`), and modular sidebar (`sidebar/*`).

### 2.2 Ordered 5-Layer GPU Scene Graph (`window.paint_layer`)
In GPUI 0.2.2 (`scene.rs`), primitives painted within the same `paint_layer` share an identical `DrawOrder` and sort by `PrimitiveKind` (`Quad < MonochromeSprite < PolychromeSprite`). To ensure crisp vector boxes and glyph badges always render above opaque satellite rasters while preserving instanced GPU batching, `gui_gpui` separates viewport rendering into **5 sequential intersecting `paint_layer` passes**:
1. **Layer 1 (Satellite Raster)**: GeoTIFF `RenderImage` (`PolychromeSprite`) with optional panchromatic grayscale mode.
2. **Layer 2 (Dynamic Neon BBoxes)**: All visible hollow target bounding boxes and hover/selection corner brackets batched into a single instanced `Quad` pass.
3. **Layer 3 (Magnification Badges)**: Pill backgrounds (`Quad`) + GPU-shaped text runs (`MonochromeSprite` via `window.text_system().shape_line`).
4. **Layer 4 (Geodesic Ruler Vector)**: Dashed vector path (`PathBuilder`) + endpoint crosshairs.
5. **Layer 5 (Measurement Readout)**: Floating pill displaying real-world Haversine distance (`m`/`km`) and true azimuth bearing (`0°–360°`).

### 2.3 Sub-Pixel Cursor-Invariant Zoom
Given `img_x = vx + (vw - img_w) * 0.5 - offset_x`, keeping the exact satellite pixel beneath cursor `(cx, cy)` stationary across zoom ratio `r = new_scale / old_scale` satisfies:

$$\text{offset}_{x,\text{new}} = r \cdot \text{offset}_{x,\text{old}} + (r - 1.0) \cdot \left(c_x - \left(v_x + \frac{v_w}{2}\right)\right)$$

Verified by unit test `test_zoom_at_cursor_preserves_pixel_under_cursor` to `< 1e-3` pixel error across arbitrary zoom cascades.

---

## 3. Directory & Module Layout

```text
apps_deploy/gui_gpui/
├── Cargo.toml
├── build.rs                      # Self-healing Linux XCB/XKB multiarch linker resolver
├── README.md
└── src/
    ├── main.rs                   # Maximized CSD window bootstrap & keystroke observer
    ├── geo.rs                    # GDAL 6-param affine transform, DMS & Haversine math
    ├── app/
    │   ├── mod.rs                # SamdefApp state, async GeoTIFF/JSON loader & nav
    │   ├── events.rs             # Smooth scroll zoom, drag/click split & map hotkeys
    │   ├── input_keys.rs         # Focusable text input editing, paste & file search
    │   └── view.rs               # Root Render tree & live viewport bounds capture
    ├── model/
    │   ├── detection.rs          # FinalOutput JSON schema & ground footprint math
    │   └── palette.rs            # 15 neon target colors & RGBA/HSLA helpers
    ├── io/
    │   ├── file_scanner.rs       # Inference directory scanner & TIFF path resolver
    │   ├── tiff_loader.rs        # Planar Config 2 & chunky BGRA strip decoder
    │   └── exporter.rs           # GeoJSON FeatureCollection, CSV & Markdown exporters
    ├── viewer/
    │   ├── state.rs              # Viewport projection, 1:1 native zoom & clamping
    │   ├── hit_test.rs           # Smallest-area target hit-testing & unit tests
    │   ├── render.rs             # Layer 1 raster & Layer 2 neon hollow box painter
    │   └── overlays.rs           # Layer 3 shaped badges & Layers 4-5 geodesic ruler
    └── ui/
        ├── theme.rs              # Dark GIS palette & reusable button/input primitives
        ├── titlebar.rs           # Wayland/X11 CSD titlebar (Move/Min/Max/F11/Close)
        ├── toolbar.rs            # Fit, 1:1, Zoom, Ruler, B/W, Radar & Table bar
        ├── status_bar.rs         # Live WGS-84 Lat/Lon, DMS, Pixel & GSD status bar
        ├── minimap.rs            # Occluded radar overview with blips & cyan frustum
        ├── inspector.rs          # Occluded hover/pinned target inspection HUD
        ├── table_drawer.rs       # Occluded bottom detection table & live filter
        └── sidebar/
            ├── mod.rs            # Scrollable left sidebar container
            ├── file_nav.rs       # Editable dir scan, instant search & dropdown picker
            ├── class_panel.rs    # 15-class toggles, live counts & toggleable Solo
            ├── confidence_panel.rs # Interactive drag/click slider & ASCII steppers
            └── telemetry_export.rs # Scene GIS telemetry & export triggers
```

---

## 4. Build, Test & Run

```bash
# Run unit tests (geodesy, sub-pixel zoom invariant, hit-testing, GIS exports)
cargo test -p gui_gpui

# Lint with zero warnings
cargo clippy -p gui_gpui --all-targets -- -D warnings

# Launch optimized release binary
cargo run --release -p gui_gpui
```

