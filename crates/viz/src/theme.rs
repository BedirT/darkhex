use bevy::color::Color;

pub const EMPTY_CELL: Color = Color::srgb(0.357, 0.486, 0.600);
pub const BLACK_STONE: Color = Color::srgb(0.102, 0.102, 0.102);
pub const WHITE_STONE: Color = Color::srgb(0.910, 0.878, 0.847);
pub const EDGE_BLACK: Color = Color::srgb(0.102, 0.102, 0.102);
pub const EDGE_WHITE: Color = Color::srgb(0.690, 0.690, 0.690);
pub const COLLISION: Color = Color::srgb(0.765, 0.290, 0.173);
pub const HIGHLIGHT: Color = Color::srgb(1.0, 0.843, 0.0);
pub const BACKGROUND: Color = Color::srgb(0.118, 0.118, 0.180);
pub const CELL_STROKE: Color = Color::srgb(0.3, 0.3, 0.4);

pub const HEX_SIZE: f32 = 40.0;
