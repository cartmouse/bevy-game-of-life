#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy::render::settings::{Backends, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::PrimaryWindow;

const CELL_SIZE: f32 = 30.0;
const HALF_SIZE: f32 = CELL_SIZE / 2.0;
const GAP_SIZE: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(RenderPlugin {
            render_creation: WgpuSettings {
                backends: Some(Backends::DX12),
                ..default()
            }
            .into(),
        }),))
        .add_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .add_systems(Update, game_loop.run_if(in_state(AppState::Running)))
        .add_systems(OnEnter(AppState::Setup), reset)
        .add_systems(Update, cursor_position.run_if(in_state(AppState::Setup)))
        .add_systems(Update, update_colors)
        .insert_resource(TickTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .run();
}

fn reset(mut q_cells: Query<(&mut Cell, &mut Sprite)>) {
    q_cells.iter_mut().for_each(|mut cell| {
        cell.0.alive = false;
        cell.1.color = Color::WHITE;
    });
}

fn setup(mut commands: Commands, q_windows: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn_batch(create_grid(q_windows.single()));
    commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(60.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Start",
                TextStyle {
                    color: Color::BLACK,
                    ..default()
                },
            ));
        });
}

fn button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &Children), With<Button>>,
    mut text_query: Query<&mut Text>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if color.0 == Color::GREEN {
                    continue;
                }
                *color = Color::GREEN.into();
                match app_state.get() {
                    AppState::Setup => next_state.set(AppState::Running),
                    AppState::Running => next_state.set(AppState::Setup),
                };
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.0, 0.5, 0.9).into();
            }
            Interaction::None => {
                *color = Color::WHITE.into();
            }
        }
        match app_state.get() {
            AppState::Setup => text.sections[0].value = "Start".to_string(),
            AppState::Running => text.sections[0].value = "Reset".to_string(),
        }
    }
}

struct Grid {
    rows: i32,
    cols: i32,
}

#[derive(Debug, Clone, Copy)]
struct Index {
    x: i32,
    y: i32,
}

impl Index {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct Cell {
    hovered: bool,
    alive: bool,
    coord: Vec2,
    index: Index,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Running,
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
                alive: false,
                coord,
                index: Index::new(row, col),
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
    mut q_cells: Query<&mut Cell>,
    buttons: Res<Input<MouseButton>>,
) {
    let size = Vec2::new(q_windows.single().width(), q_windows.single().height());

    if let Some(position) = q_windows.single().cursor_position() {
        let new_pos = Vec2::new(
            position.x - size.x / GAP_SIZE,
            size.y / GAP_SIZE - position.y,
        );

        for mut cell in q_cells.iter_mut() {
            if cell.coord.x <= new_pos.x + HALF_SIZE
                && cell.coord.x >= new_pos.x - HALF_SIZE
                && cell.coord.y <= new_pos.y + HALF_SIZE
                && cell.coord.y >= new_pos.y - HALF_SIZE
            {
                if buttons.just_pressed(MouseButton::Left) {
                    cell.alive = !cell.alive;
                }
                cell.hovered = true;
            } else {
                cell.hovered = false;
            }
        }
    }
}

#[derive(Resource)]
struct TickTimer(Timer);

fn game_loop(
    mut q_cells: Query<&mut Cell>,
    time: Res<Time>,
    mut timer: ResMut<TickTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let alive_cells: Vec<_> = q_cells
            .iter()
            .filter(|cell| cell.alive)
            .map(|c| c.index)
            .collect();
        if alive_cells.len() == 0 {
            next_state.set(AppState::Setup);
        }
        for mut cell in q_cells.iter_mut() {
            check_rules(&mut cell, &alive_cells);
        }
    }
}

fn check_rules(cell: &mut Cell, alive_cells: &Vec<Index>) {
    let x = cell.index.x;
    let y = cell.index.y;
    let live_count = [
        Index::new(x - 1, y - 1),
        Index::new(x, y - 1),
        Index::new(x + 1, y - 1),
        Index::new(x - 1, y),
        Index::new(x + 1, y),
        Index::new(x - 1, y + 1),
        Index::new(x, y + 1),
        Index::new(x + 1, y + 1),
    ]
    .iter()
    .filter(|index| {
        alive_cells
            .iter()
            .any(|cell| cell.x == index.x && cell.y == index.y)
    })
    .count();
    if cell.alive {
        cell.alive = live_count == 2 || live_count == 3;
    } else {
        cell.alive = live_count == 3;
    }
}

fn update_colors(mut q_cells: Query<(&mut Cell, &mut Sprite)>) {
    q_cells.iter_mut().for_each(|mut cell| {
        cell.1.color = match cell.0 {
            n if n.hovered => Color::BLUE,
            n if n.alive => Color::GREEN,
            _ => Color::WHITE,
        };
    });
}
