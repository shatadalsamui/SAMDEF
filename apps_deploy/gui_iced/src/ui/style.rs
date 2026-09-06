use iced::widget::{button, checkbox, container, rule, slider, text_input};
use iced::{Background, Color, Theme, Vector};

// ==========================================
// VS Code / Zed Dark Palette Constants
// ==========================================
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.83, 0.83, 0.83);    // #d4d4d4 (VS Code editor text)
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.55, 0.58);  // #8c8c94 (Muted labels)
pub const TEXT_HEADER: Color = Color::from_rgb(0.78, 0.78, 0.82);     // #c7c7d1 (Clean subtle headers)
pub const ACCENT_BLUE: Color = Color::from_rgb(0.25, 0.58, 0.90);     // #4094e6 (VS Code soft blue)
pub const ACCENT_GREEN: Color = Color::from_rgb(0.31, 0.79, 0.69);    // #4ec9b0 (VS Code teal/green)
pub const ACCENT_AMBER: Color = Color::from_rgb(0.85, 0.68, 0.15);    // #d9ad26 (VS Code warning)
pub const BORDER_SUBTLE: Color = Color::from_rgb(0.20, 0.20, 0.22);   // #333338 (VS Code border)

// ==========================================
// Button Styles: Flat / Borderless (No Boxes)
// ==========================================

pub struct GhostButton;

impl button::StyleSheet for GhostButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: TEXT_PRIMARY,
            shadow_offset: Vector::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            shadow_offset: Vector::default(),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.14))),
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            shadow_offset: Vector::default(),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: TEXT_SECONDARY,
            shadow_offset: Vector::default(),
        }
    }
}

pub struct GhostButtonAccent;

impl button::StyleSheet for GhostButtonAccent {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: ACCENT_BLUE,
            shadow_offset: Vector::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.25, 0.58, 0.90, 0.12))),
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            shadow_offset: Vector::default(),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.25, 0.58, 0.90, 0.22))),
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            shadow_offset: Vector::default(),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            border_radius: [3.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: TEXT_SECONDARY,
            shadow_offset: Vector::default(),
        }
    }
}

// ==========================================
// Container Styles: VS Code / Zed Dark Surfaces
// ==========================================

pub struct SidebarContainer;

impl container::StyleSheet for SidebarContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb(0.145, 0.145, 0.155))), // #252527
            border_radius: [0.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub struct ToolbarContainer;

impl container::StyleSheet for ToolbarContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.13))), // #1f1f21
            border_radius: [0.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub struct CanvasContainer;

impl container::StyleSheet for CanvasContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb(0.10, 0.10, 0.11))), // #1a1a1c
            border_radius: [0.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub struct CardContainer;

impl container::StyleSheet for CardContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb(0.18, 0.18, 0.20))), // #2e2e33
            border_radius: [4.0; 4].into(),
            border_width: 1.0,
            border_color: BORDER_SUBTLE,
        }
    }
}

pub struct ColorSwatch(pub Color);

impl container::StyleSheet for ColorSwatch {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(self.0)),
            border_radius: [2.0; 4].into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

// ==========================================
// Input, Rule, Checkbox & Slider Styles
// ==========================================

pub struct VscodeInput;

impl text_input::StyleSheet for VscodeInput {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.22, 0.22, 0.24)), // #38383d
            border_radius: [3.0; 4].into(),
            border_width: 1.0,
            border_color: Color::from_rgb(0.28, 0.28, 0.30),
            icon_color: TEXT_SECONDARY,
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.24, 0.24, 0.26)),
            border_radius: [3.0; 4].into(),
            border_width: 1.0,
            border_color: ACCENT_BLUE,
            icon_color: ACCENT_BLUE,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        TEXT_SECONDARY
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        TEXT_PRIMARY
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.35, 0.35, 0.38)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.25, 0.58, 0.90, 0.3)
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.18, 0.18, 0.20)),
            border_radius: [3.0; 4].into(),
            border_width: 1.0,
            border_color: BORDER_SUBTLE,
            icon_color: TEXT_SECONDARY,
        }
    }
}

pub struct VscodeRule;

impl rule::StyleSheet for VscodeRule {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        rule::Appearance {
            color: BORDER_SUBTLE,
            width: 1,
            radius: [0.0; 4].into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}

pub struct VscodeCheckbox;

impl checkbox::StyleSheet for VscodeCheckbox {
    type Style = Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: if is_checked {
                Background::Color(ACCENT_BLUE)
            } else {
                Background::Color(Color::from_rgb(0.20, 0.20, 0.22))
            },
            icon_color: Color::WHITE,
            border_radius: [3.0; 4].into(),
            border_width: 1.0,
            border_color: if is_checked {
                ACCENT_BLUE
            } else {
                Color::from_rgb(0.30, 0.30, 0.34)
            },
            text_color: Some(TEXT_PRIMARY),
        }
    }

    fn hovered(&self, _style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: if is_checked {
                Background::Color(Color::from_rgb(0.30, 0.62, 0.95))
            } else {
                Background::Color(Color::from_rgb(0.25, 0.25, 0.28))
            },
            icon_color: Color::WHITE,
            border_radius: [3.0; 4].into(),
            border_width: 1.0,
            border_color: if is_checked {
                Color::from_rgb(0.35, 0.65, 0.98)
            } else {
                Color::from_rgb(0.40, 0.40, 0.45)
            },
            text_color: Some(Color::WHITE),
        }
    }
}

pub struct VscodeSlider;

impl slider::StyleSheet for VscodeSlider {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (ACCENT_BLUE, Color::from_rgb(0.25, 0.25, 0.28)),
                width: 3.0,
                border_radius: [1.5; 4].into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 6.0 },
                color: ACCENT_BLUE,
                border_width: 1.0,
                border_color: Color::WHITE,
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (ACCENT_BLUE, Color::from_rgb(0.28, 0.28, 0.32)),
                width: 3.0,
                border_radius: [1.5; 4].into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 7.0 },
                color: Color::WHITE,
                border_width: 1.5,
                border_color: ACCENT_BLUE,
            },
        }
    }

    fn dragging(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (ACCENT_BLUE, Color::from_rgb(0.28, 0.28, 0.32)),
                width: 3.0,
                border_radius: [1.5; 4].into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 7.5 },
                color: Color::WHITE,
                border_width: 2.0,
                border_color: ACCENT_BLUE,
            },
        }
    }
}

// ==========================================
// Theme Style Helper Constructors
// ==========================================

pub fn ghost_button() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(GhostButton))
}

pub fn ghost_button_accent() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(GhostButtonAccent))
}

pub fn sidebar_container() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(SidebarContainer))
}

pub fn toolbar_container() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(ToolbarContainer))
}

pub fn canvas_container() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(CanvasContainer))
}

pub fn card_container() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(CardContainer))
}

pub fn color_swatch(color: Color) -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(ColorSwatch(color)))
}

pub fn dark_input() -> iced::theme::TextInput {
    iced::theme::TextInput::Custom(Box::new(VscodeInput))
}

pub fn dark_rule() -> iced::theme::Rule {
    iced::theme::Rule::Custom(Box::new(VscodeRule))
}

pub fn vscode_checkbox() -> iced::theme::Checkbox {
    iced::theme::Checkbox::Custom(Box::new(VscodeCheckbox))
}

pub fn vscode_slider() -> iced::theme::Slider {
    iced::theme::Slider::Custom(Box::new(VscodeSlider))
}
