use bevy::prelude::*;

fn main() {
    App::new().add_plugins((DefaultPlugins, HelloPlugin)).run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn_batch(create_grid());
}

struct Grid {
    rows: i32,
    cols: i32,
}

fn create_grid() -> Vec<SpriteBundle> {
    let mut bundle = vec![];
    let grid = Grid { rows: 20, cols: 20 };
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            bundle.push(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2 { x: 10.0, y: 10.0 }),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(
                    row as f32 * (10.0 + 2.0),
                    col as f32 * (10.0 + 2.0),
                    0.0,
                )),
                ..Default::default()
            })
        }
    }
    return bundle;
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello, {}!", name.0);
        }
    }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Startup, setup)
            .add_systems(Update, greet_people);
    }
}
