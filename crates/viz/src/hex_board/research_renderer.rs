use std::collections::HashMap;
use std::f32::consts::PI;

use bevy::prelude::*;
use hexx::{Hex, HexLayout, HexOrientation};

use super::components::*;
use crate::theme;

/// Resource holding hex board layout and entity mappings.
#[allow(dead_code)]
#[derive(Resource)]
pub struct HexBoardState {
    pub layout: HexLayout,
    pub hex_to_pos: HashMap<Hex, usize>,
    pub pos_to_hex: HashMap<usize, Hex>,
    pub pos_to_entity: HashMap<usize, Entity>,
    pub rows: usize,
    pub cols: usize,
}

/// Resource to request a board size change.
#[derive(Resource, Default)]
pub struct BoardSizeRequest {
    pub rows: usize,
    pub cols: usize,
    pub changed: bool,
}

/// Convert darkhex (row, col) to hexx axial Hex coordinate.
///
/// DarkHex uses an offset-row layout where each row shifts right.
/// The neighbor topology in board.rs:
///   E=(r,c+1), W=(r,c-1), SW=(r+1,c), SE=(r+1,c-1), NE=(r-1,c), NW=(r-1,c+1)
///
/// This maps to axial coordinates: q = col, r = row (with appropriate offset).
fn darkhex_to_hex(row: usize, col: usize) -> Hex {
    // Offset-row (odd-row offset) for flat-top hex:
    // axial_q = col - (row - (row & 1)) / 2
    // axial_r = row
    // But we need to match the specific darkhex topology.
    // For flat-top with even-row offset:
    let q = col as i32 - (row as i32) / 2;
    let r = row as i32;
    Hex::new(q, r)
}

/// Convert linear position to alphanumeric label (a1, b2, etc.).
fn pos_to_label(pos: usize, cols: usize) -> String {
    let row = pos / cols;
    let col = pos % cols;
    let row_char = (b'a' + row as u8) as char;
    format!("{}{}", row_char, col + 1)
}

/// Spawn the hex board for a given size.
pub fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_req: Res<BoardSizeRequest>,
) {
    let rows = board_req.rows;
    let cols = board_req.cols;

    let layout = HexLayout {
        scale: Vec2::splat(theme::HEX_SIZE),
        orientation: HexOrientation::Flat,
        ..default()
    };

    // Create shared hex mesh (flat-top regular polygon)
    // RegularPolygon creates pointy-top by default; we rotate 30° for flat-top
    let hex_shape = RegularPolygon::new(theme::HEX_SIZE, 6);
    let hex_mesh = meshes.add(Mesh::from(hex_shape));

    let empty_mat = materials.add(ColorMaterial::from_color(theme::EMPTY_CELL));
    let stroke_mat = materials.add(ColorMaterial::from_color(theme::CELL_STROKE));

    // Slightly larger hex for the stroke/outline effect
    let stroke_shape = RegularPolygon::new(theme::HEX_SIZE + 2.0, 6);
    let stroke_mesh = meshes.add(Mesh::from(stroke_shape));

    let mut hex_to_pos = HashMap::new();
    let mut pos_to_hex = HashMap::new();
    let mut pos_to_entity = HashMap::new();

    // Spawn board root
    let board_entity = commands
        .spawn((HexBoardMarker, Transform::default(), Visibility::default()))
        .id();

    for row in 0..rows {
        for col in 0..cols {
            let pos = row * cols + col;
            let hex = darkhex_to_hex(row, col);
            let world_pos = layout.hex_to_world_pos(hex);

            hex_to_pos.insert(hex, pos);
            pos_to_hex.insert(pos, hex);

            // Spawn stroke (outline) behind the cell
            commands
                .spawn((
                    Mesh2d(stroke_mesh.clone()),
                    MeshMaterial2d(stroke_mat.clone()),
                    Transform::from_xyz(world_pos.x, world_pos.y, 0.0)
                        .with_rotation(Quat::from_rotation_z(PI / 6.0)),
                    ChildOf(board_entity),
                ));

            // Spawn hex cell
            let cell_entity = commands
                .spawn((
                    HexCell { hex, pos },
                    CellState::Empty,
                    Mesh2d(hex_mesh.clone()),
                    MeshMaterial2d(empty_mat.clone()),
                    Transform::from_xyz(world_pos.x, world_pos.y, 1.0)
                        .with_rotation(Quat::from_rotation_z(PI / 6.0)),
                    ChildOf(board_entity),
                ))
                .id();

            pos_to_entity.insert(pos, cell_entity);

            // Spawn cell label as child
            let label = pos_to_label(pos, cols);
            commands
                .spawn((
                    CellLabel,
                    Text2d::new(label),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(world_pos.x, world_pos.y, 2.0),
                    ChildOf(board_entity),
                ));
        }
    }

    commands.insert_resource(HexBoardState {
        layout,
        hex_to_pos,
        pos_to_hex,
        pos_to_entity,
        rows,
        cols,
    });
}

/// Handle hex cell clicks.
pub fn handle_hex_click(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    board: Option<Res<HexBoardState>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut click_events: MessageWriter<HexClickEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(board) = board else { return };
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };

    if let Some(cursor) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor) {
            let hex = board.layout.world_pos_to_hex(world_pos);
            if let Some(&pos) = board.hex_to_pos.get(&hex) {
                click_events.write(HexClickEvent { pos, hex });
            }
        }
    }
}

/// Handle board size change requests.
pub fn handle_board_resize(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut board_req: ResMut<BoardSizeRequest>,
    board_query: Query<Entity, With<HexBoardMarker>>,
    label_query: Query<Entity, With<CellLabel>>,
) {
    if !board_req.changed {
        return;
    }
    board_req.changed = false;

    // Despawn old board
    for entity in board_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in label_query.iter() {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<HexBoardState>();

    // Spawn new board
    spawn_board(commands, meshes, materials, board_req.into());
}
