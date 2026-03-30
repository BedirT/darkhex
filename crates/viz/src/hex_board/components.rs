use bevy::prelude::*;
use hexx::Hex;

/// Marker for the hex board root entity.
#[derive(Component)]
pub struct HexBoardMarker;

/// A hex cell entity. `pos` is the linear index (row * cols + col).
#[derive(Component)]
pub struct HexCell {
    pub hex: Hex,
    pub pos: usize,
}

/// Visual state of a cell.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CellState {
    #[default]
    Empty,
    Black,
    White,
}

/// Marker for cell label text.
#[derive(Component)]
pub struct CellLabel;

/// Event emitted when a hex cell is clicked.
#[derive(Message)]
pub struct HexClickEvent {
    pub pos: usize,
    pub hex: Hex,
}
