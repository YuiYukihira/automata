use std::ops::Deref;

use bevy::{prelude::*, tasks::ComputeTaskPool};
use rand::random;

static SCREEN_CELLS: (usize, usize) = (256, 256);
static SCREEN_CELLS_FLOAT: (f32, f32) = (256.0, 256.0);

#[derive(Component, Deref)]
struct Velocity(Vec2);

#[derive(Component, Deref)]
struct Cell(Vec2);

#[derive(Component, Deref)]
struct Alive(bool);

fn cell_to_pos(window: &Vec4, grid_size: &Vec2, cell: &Cell) -> Vec2 {
    let x_step_per_cell = (window.y - window.w) / grid_size.x;
    let y_step_per_cell = (window.x - window.z) / grid_size.y;

    Vec2::new(
        (x_step_per_cell * cell.x) + window.w,
        (y_step_per_cell * cell.y) + window.z,
    )
}

fn spawn_system(windows: Res<Windows>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let window = windows.primary();
    let window_dimensions = Vec4::new(
        window.height() / 2.,
        window.width() / 2.,
        window.height() / -2.,
        window.width() / -2.,
    );
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let cell_size = Vec2::new(
        (window_dimensions.y - window_dimensions.w) / (SCREEN_CELLS_FLOAT.0),
        (window_dimensions.x - window_dimensions.z) / (SCREEN_CELLS_FLOAT.1),
    );
    //let texture = asset_server.load("textures/bounce.png");
    for x in 0..SCREEN_CELLS.0 {
        for y in 0..SCREEN_CELLS.1 {
            let xint = x;
            let yint = y;
            let x = x as f32;
            let y = y as f32;
            let cell = Cell(Vec2::new(x, y));
            let pos = cell_to_pos(
                &window_dimensions,
                &Vec2::new(SCREEN_CELLS_FLOAT.0, SCREEN_CELLS_FLOAT.1),
                &cell,
            );
            commands
                .spawn_bundle(SpriteBundle {
                    //texture: texture.clone(),
                    sprite: Sprite {
                        color: Color::rgb(1., 1., 1.),
                        custom_size: Some(Vec2::new(cell_size.x, cell_size.y)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(
                        pos.x + cell_size.x / 2.,
                        pos.y + cell_size.y / 2.,
                        0.,
                    ),
                    ..Default::default()
                })
                .insert(cell)
                .insert(Alive(
                    (xint % 2 == 0 || yint % 2 == 0) && !(xint % 2 == 0 && yint % 2 == 0),
                ));
        }
    }
}

fn color_system(pool: Res<ComputeTaskPool>, mut sprites: Query<(&Cell, &mut Sprite, &Alive)>) {
    sprites.par_for_each_mut(&pool, 32, |(cell, mut sprite, alive)| {
        sprite.color = match alive.deref() {
            true => Color::rgb(1., 1., 1.),
            false => Color::rgb(0., 0., 0.),
        }
    });
}

fn move_system(pool: Res<ComputeTaskPool>, mut sprites: Query<(&mut Transform, &Velocity)>) {
    sprites.par_for_each_mut(&pool, 32, |(mut transform, velocity)| {
        transform.translation += velocity.extend(0.0);
    });
}

fn bounce_system(
    pool: Res<ComputeTaskPool>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut sprites: Query<(&mut Transform, &mut Velocity, &Handle<Image>)>,
) {
    let window = windows.primary();
    let width = window.width();
    let height = window.height();
    let left = width / -2.;
    let right = width / 2.;
    let bottom = height / -2.;
    let top = height / 2.;
    sprites.par_for_each_mut(&pool, 32, |(mut transform, mut v, handle)| {
        let image = images.get(handle);
        let (iwidth, iheight) = match image {
            Some(i) => (
                i.texture_descriptor.size.width as f32 * transform.scale.x,
                i.texture_descriptor.size.height as f32 * transform.scale.y,
            ),
            None => (0., 0.),
        };
        if left > transform.translation.x - (iwidth / 2.) {
            v.0.x = -v.0.x;
            transform.translation.x = left + (iwidth / 2.);
        } else if right < transform.translation.x + (iwidth / 2.) {
            v.0.x = -v.0.x;
            transform.translation.x = right - (iwidth / 2.);
        } else if bottom > transform.translation.y - (iheight / 2.) {
            v.0.y = -v.0.y;
            transform.translation.y = bottom + (iheight / 2.);
        } else if top < transform.translation.y + (iheight / 2.) {
            v.0.y = -v.0.y;
            transform.translation.y = top - (iheight / 2.);
        }
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_system)
        .add_system(color_system)
        //.add_system(move_system)
        //.add_system(bounce_system)
        .run();
}

fn rand_range(start: f32, end: f32) -> f32 {
    ((end - start) * random::<f32>()) + start
}
