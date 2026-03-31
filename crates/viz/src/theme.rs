#![allow(dead_code)]
use bevy::color::Color;
use bevy_egui::egui::Color32;

// Board colors
pub const EMPTY_CELL: Color = Color::srgb(0.357, 0.486, 0.600);
pub const BLACK_STONE: Color = Color::srgb(0.102, 0.102, 0.102);
pub const WHITE_STONE: Color = Color::srgb(0.910, 0.878, 0.847);
pub const EDGE_BLACK: Color = Color::srgb(0.102, 0.102, 0.102);
pub const EDGE_WHITE: Color = Color::srgb(0.690, 0.690, 0.690);
pub const COLLISION: Color = Color::srgb(0.765, 0.290, 0.173);
pub const HIGHLIGHT: Color = Color::srgb(1.0, 0.843, 0.0);
pub const BACKGROUND: Color = Color::srgb(0.118, 0.118, 0.180);
pub const CELL_STROKE: Color = Color::srgb(0.3, 0.3, 0.4);

// UI accent — thesis steel blue (#5B7C99)
pub const ACCENT: Color32 = Color32::from_rgb(91, 124, 153);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(110, 148, 178);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(60, 85, 105);

// Egui panel / window colors (dark blue-gray family)
pub const PANEL_BG: Color32 = Color32::from_rgb(24, 26, 36);
pub const WINDOW_BG: Color32 = Color32::from_rgb(30, 33, 44);
pub const WIDGET_BG: Color32 = Color32::from_rgb(40, 44, 58);
pub const WIDGET_BG_HOVER: Color32 = Color32::from_rgb(50, 55, 72);
pub const WIDGET_BG_ACTIVE: Color32 = Color32::from_rgb(55, 62, 82);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(210, 215, 225);
pub const TEXT_DIM: Color32 = Color32::from_rgb(140, 150, 170);

// Bevy ClearColor (dark blue-gray matching panel bg)
pub const CLEAR_COLOR: Color = Color::srgb(0.075, 0.082, 0.114);

pub const HEX_SIZE: f32 = 40.0;

// 3D isometric renderer
pub const HEX_SIZE_3D: f32 = 1.0;
pub const TILE_HEIGHT: f32 = 0.3;
pub const STONE_RADIUS: f32 = 0.35;
pub const TILE_METALLIC: f32 = 0.1;
pub const TILE_ROUGHNESS: f32 = 0.8;
pub const BLACK_STONE_METALLIC: f32 = 0.3;
pub const BLACK_STONE_ROUGHNESS: f32 = 0.4;
pub const WHITE_STONE_METALLIC: f32 = 0.0;
pub const WHITE_STONE_ROUGHNESS: f32 = 0.7;
