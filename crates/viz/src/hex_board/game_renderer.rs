use std::collections::HashMap;

use bevy::prelude::*;
use hexx::{Hex, HexLayout, HexOrientation};

use super::components::*;
use super::darkhex_to_hex;
use super::research_renderer::BoardSizeRequest;
use crate::theme;

/// Marker for the 3D game board root entity.
#[derive(Component)]
pub struct GameBoardRoot;

/// Marker for hex tile column entities in 3D mode.
#[derive(Component)]
pub struct GameHexTile;

/// Marker for stone sphere entities in 3D mode.
#[derive(Component)]
pub struct GameStone;

/// Marker for the 3D isometric camera.
#[derive(Component)]
pub struct GameCameraMarker;

/// Marker for the 3D directional light.
#[derive(Component)]
#[allow(dead_code)]
pub struct GameLightMarker;

/// 3D board state resource mapping hex positions to entities.
#[allow(dead_code)]
#[derive(Resource)]
pub struct GameBoardState3D {
    pub layout: HexLayout,
    pub hex_to_pos: HashMap<Hex, usize>,
    pub pos_to_entity: HashMap<usize, Entity>,
    pub rows: usize,
    pub cols: usize,
}

/// Spawn the 3D hex board with cylinder tiles.
pub fn spawn_game_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    board_req: Res<BoardSizeRequest>,
) {
    let rows = board_req.rows;
    let cols = board_req.cols;

    let layout = HexLayout {
        scale: Vec2::splat(theme::HEX_SIZE_3D),
        orientation: HexOrientation::Flat,
        ..default()
    };

    // 6-sided cylinder for hex tiles
    let tile_mesh = meshes.add(
        Cylinder::new(theme::HEX_SIZE_3D * 0.95, theme::TILE_HEIGHT)
            .mesh()
            .resolution(6),
    );

    let tile_mat = materials.add(StandardMaterial {
        base_color: theme::EMPTY_CELL,
        metallic: theme::TILE_METALLIC,
        perceptual_roughness: theme::TILE_ROUGHNESS,
        ..default()
    });

    let mut hex_to_pos = HashMap::new();
    let mut pos_to_entity = HashMap::new();

    let board_entity = commands
        .spawn((GameBoardRoot, Transform::default(), Visibility::default()))
        .id();

    for row in 0..rows {
        for col in 0..cols {
            let pos = row * cols + col;
            let hex = darkhex_to_hex(row, col);
            let world_pos = layout.hex_to_world_pos(hex);

            hex_to_pos.insert(hex, pos);

            let entity = commands
                .spawn((
                    GameHexTile,
                    HexCell { hex, pos },
                    CellState::Empty,
                    Mesh3d(tile_mesh.clone()),
                    MeshMaterial3d(tile_mat.clone()),
                    Transform::from_xyz(world_pos.x, theme::TILE_HEIGHT / 2.0, world_pos.y),
                    ChildOf(board_entity),
                ))
                .id();

            pos_to_entity.insert(pos, entity);
        }
    }

    commands.insert_resource(GameBoardState3D {
        layout,
        hex_to_pos,
        pos_to_entity,
        rows,
        cols,
    });
}

/// Despawn the 3D game board and remove its state resource.
pub fn despawn_game_board(
    mut commands: Commands,
    board_query: Query<Entity, With<GameBoardRoot>>,
) {
    for entity in board_query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<GameBoardState3D>();
}

/// Handle clicks on hex tiles via ground-plane raycast.
pub fn handle_game_hex_click(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<GameCameraMarker>>,
    board: Option<Res<GameBoardState3D>>,
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
        if let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) {
            // Intersect with Y=0 ground plane
            if let Some(dist) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) {
                let point = ray.get_point(dist);
                let hex = board.layout.world_pos_to_hex(Vec2::new(point.x, point.z));
                if let Some(&pos) = board.hex_to_pos.get(&hex) {
                    click_events.write(HexClickEvent { pos, hex });
                }
            }
        }
    }
}

/// Handle board size change requests in game mode.
pub fn handle_game_board_resize(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut board_req: ResMut<BoardSizeRequest>,
    board_query: Query<Entity, With<GameBoardRoot>>,
) {
    if !board_req.changed {
        return;
    }
    board_req.changed = false;

    for entity in board_query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<GameBoardState3D>();

    spawn_game_board(commands, meshes, materials, board_req.into());
}

/// Update cell visuals: spawn/despawn stone spheres when CellState changes.
pub fn update_game_cell_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    changed_cells: Query<(Entity, &CellState), (With<GameHexTile>, Changed<CellState>)>,
    stones: Query<Entity, With<GameStone>>,
    children_query: Query<&Children>,
) {
    for (tile_entity, state) in changed_cells.iter() {
        // Remove existing stone children
        if let Ok(children) = children_query.get(tile_entity) {
            for child in children.iter() {
                if stones.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }

        // Spawn stone if cell is occupied
        match state {
            CellState::Black => {
                let stone_mesh = meshes.add(Sphere::new(theme::STONE_RADIUS));
                let mat = materials.add(StandardMaterial {
                    base_color: theme::BLACK_STONE,
                    metallic: theme::BLACK_STONE_METALLIC,
                    perceptual_roughness: theme::BLACK_STONE_ROUGHNESS,
                    ..default()
                });
                commands.spawn((
                    GameStone,
                    Mesh3d(stone_mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(
                        0.0,
                        theme::TILE_HEIGHT / 2.0 + theme::STONE_RADIUS,
                        0.0,
                    ),
                    ChildOf(tile_entity),
                ));
            }
            CellState::White => {
                let stone_mesh = meshes.add(Sphere::new(theme::STONE_RADIUS));
                let mat = materials.add(StandardMaterial {
                    base_color: theme::WHITE_STONE,
                    metallic: theme::WHITE_STONE_METALLIC,
                    perceptual_roughness: theme::WHITE_STONE_ROUGHNESS,
                    ..default()
                });
                commands.spawn((
                    GameStone,
                    Mesh3d(stone_mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(
                        0.0,
                        theme::TILE_HEIGHT / 2.0 + theme::STONE_RADIUS,
                        0.0,
                    ),
                    ChildOf(tile_entity),
                ));
            }
            CellState::Empty => {} // stone already despawned above
        }
    }
}
