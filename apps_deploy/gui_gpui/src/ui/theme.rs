#![allow(dead_code)]
use gpui::{div, prelude::*, rgb, CursorStyle, Div, ElementId, SharedString, Stateful};

// Zed / VS Code inspired high-contrast dark theme
pub const BG_ROOT: u32 = 0x09090b;
pub const BG_SIDEBAR: u32 = 0x121215;
pub const BG_CARD: u32 = 0x18181c;
pub const BG_INPUT: u32 = 0x202026;
pub const BG_HOVER: u32 = 0x27272f;
pub const BG_ACTIVE: u32 = 0x1e3a8a;

pub const BORDER_SUBTLE: u32 = 0x27272a;
pub const BORDER_STRONG: u32 = 0x3f3f46;

pub const TEXT_PRIMARY: u32 = 0xf4f4f5;
pub const TEXT_SECONDARY: u32 = 0xa1a1aa;
pub const TEXT_DIM: u32 = 0x71717a;

pub const ACCENT_BLUE: u32 = 0x3b82f6;
pub const ACCENT_GREEN: u32 = 0x10b981;
pub const ACCENT_AMBER: u32 = 0xf59e0b;
pub const ACCENT_RED: u32 = 0xef4444;
pub const ACCENT_CYAN: u32 = 0x06b6d4;

/// Reusable compact interactive card button
pub fn action_btn(id: impl Into<ElementId>, label: impl IntoElement) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_sm()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER_SUBTLE))
        .text_xs()
        .text_color(rgb(TEXT_PRIMARY))
        .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
        .child(label)
}

/// Reusable toggle pill button with active accent highlight
pub fn toggle_btn(
    id: impl Into<ElementId>,
    label: impl IntoElement,
    active: bool,
    active_border: u32,
    active_bg: u32,
    active_text: u32,
) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(rgb(if active { active_border } else { BORDER_SUBTLE }))
        .bg(rgb(if active { active_bg } else { BG_CARD }))
        .text_xs()
        .text_color(rgb(if active { active_text } else { TEXT_PRIMARY }))
        .hover(|s| s.bg(rgb(BG_HOVER)).cursor_pointer())
        .child(label)
}

/// Reusable two-column telemetry key-value row
pub fn kv_row(key: &'static str, val: impl IntoElement, val_color: u32) -> Div {
    div()
        .flex()
        .flex_row()
        .justify_between()
        .items_center()
        .child(div().text_color(rgb(TEXT_DIM)).child(key))
        .child(div().text_color(rgb(val_color)).child(val))
}

/// Reusable focusable single-line text input box with caret indicator
pub fn text_input_box(
    id: impl Into<ElementId>,
    value: &str,
    placeholder: &str,
    is_focused: bool,
    focus_border_hex: u32,
) -> Stateful<Div> {
    let display = if is_focused {
        format!("{}|", value)
    } else if value.is_empty() {
        placeholder.to_string()
    } else {
        value.to_string()
    };

    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_sm()
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(if is_focused {
            focus_border_hex
        } else {
            BORDER_SUBTLE
        }))
        .cursor(CursorStyle::IBeam)
        .text_xs()
        .text_color(rgb(if is_focused || !value.is_empty() {
            TEXT_PRIMARY
        } else {
            TEXT_DIM
        }))
        .overflow_hidden()
        .child(SharedString::from(display))
}
