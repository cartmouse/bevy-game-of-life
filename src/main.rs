use bevy::prelude::*;
use bevy::render::settings::{Backends, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::PrimaryWindow;

const CELL_SIZE: f32 = 20.0;
const HALF_SIZE: f32 = CELL_SIZE / 2.0;
const HOVER_SIZE: f32 = CELL_SIZE + 1.0;
const GAP_SIZE: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                }
                .into(),
            }),
            AutoKitchenPlugin,
        ))
        .run();
}

fn setup(mut commands: Commands, q_windows: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn_batch(create_grid(q_windows.single()));
}

struct Grid {
    rows: i32,
    cols: i32,
}

#[derive(Component)]
struct Cell {
    hovered: bool,
    coord: Vec2,
}

fn create_grid(window: &Window) -> Vec<(Cell, SpriteBundle)> {
    let mut bundle = vec![];
    let grid = Grid { rows: 20, cols: 20 };
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let coord = Vec2::new(
                row as f32 * (CELL_SIZE + GAP_SIZE)
                    - grid.rows as f32 * (CELL_SIZE + GAP_SIZE) * 0.5,
                col as f32 * (CELL_SIZE + GAP_SIZE) - window.height() / 4.0,
            );
            let cell = Cell {
                hovered: false,
                coord,
            };
            bundle.push((
                cell,
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: CELL_SIZE,
                            y: CELL_SIZE,
                        }),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(coord.x, coord.y, 0.0)),
                    ..Default::default()
                },
            ));
        }
    }
    return bundle;
}

fn cursor_position(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut q_cells: Query<(&mut Cell, &mut Sprite)>,
) {
    let size = Vec2::new(q_windows.single().width(), q_windows.single().height());

    if let Some(position) = q_windows.single().cursor_position() {
        let new_pos = Vec2::new(
            position.x - size.x / GAP_SIZE,
            size.y / GAP_SIZE - position.y,
        );

        for (mut cell, mut sprite) in q_cells.iter_mut() {
            if cell.coord.x <= new_pos.x + HALF_SIZE
                && cell.coord.x >= new_pos.x - HALF_SIZE
                && cell.coord.y <= new_pos.y + HALF_SIZE
                && cell.coord.y >= new_pos.y - HALF_SIZE
            {
                cell.hovered = true;
                sprite.custom_size = Some(Vec2::new(HOVER_SIZE, HOVER_SIZE));
                sprite.color = Color::GREEN;
            } else {
                cell.hovered = false;
                sprite.custom_size = Some(Vec2::new(CELL_SIZE, CELL_SIZE));
                sprite.color = Color::WHITE;
            }
        }
    }
}

pub struct AutoKitchenPlugin;

impl Plugin for AutoKitchenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cursor_position)
            .add_systems(Startup, setup);
    }
}
