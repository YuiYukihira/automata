use bevy::{prelude::*, tasks::ComputeTaskPool};

use crate::{
    board::{Board, Update},
    hoverable::Hoverable,
    view::View,
    Alive,
};

#[derive(Component, Deref, PartialEq, Eq)]
pub struct TilePosition(UVec2);

#[derive(Debug, Deref, DerefMut)]
pub struct RenderTimer(Timer);

pub fn spawn_tiles(windows: Res<Windows>, mut commands: Commands) {
    let window = windows.primary();
    let window_dimensions = Vec4::new(
        window.height() / 2.,
        window.width() / 2.,
        window.height() / -2.,
        window.width() / -2.,
    );

    let cell_size = Vec2::splat(4.);
    let cell_count = (
        (window.width() / cell_size.x) as usize,
        (window.height() / cell_size.y) as usize,
    );

    for x in 0..cell_count.0 {
        for y in 0..cell_count.1 {
            let cell = TilePosition(UVec2::new(x as u32, y as u32));
            let pos = cell_to_pos(
                &window_dimensions,
                &(cell_count.0, cell_count.1),
                &(cell_size.x, cell_size.y),
                &cell,
            );

            commands
                .spawn_bundle(SpriteBundle {
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
                .insert(Hoverable);
        }
    }
}

fn cell_to_pos(
    window: &Vec4,
    grid_size: &(usize, usize),
    cell_size: &(f32, f32),
    cell: &TilePosition,
) -> Vec2 {
    let window_x = (window.y - window.w).abs();
    let window_y = (window.x - window.z).abs();
    let x_pixels = cell_size.0 * grid_size.0 as f32;
    let y_pixels = cell_size.1 * grid_size.1 as f32;

    Vec2::new(
        (cell_size.0 * cell.x as f32) + window.w + ((window_x - x_pixels) / 2.),
        (cell_size.1 * cell.y as f32) + window.z + ((window_y - y_pixels) / 2.),
    )
}

pub fn update_colors(
    //In(should_update): In<bool>,
    pool: Res<ComputeTaskPool>,
    board: Res<Board>,
    view: Query<&View>,
    mut sprites: Query<(&mut Sprite, &TilePosition)>,
    board_tiles: Query<&Alive>,
) {
    println!("board_tiles: {}", board_tiles.iter().count());
    if board_tiles.iter().count() == 0 {
        return;
    }
    sprites.par_for_each_mut(&pool, 32, |(mut sprite, pos)| {
        let pos = IVec2::new(pos.0.x as i32, pos.0.y as i32);
        let board_pos = pos + view.iter().next().unwrap().offset;
        sprite.color = if let Some(entity) = board.get(board_pos) {
            if board_tiles.get(*entity).is_ok() {
                Color::rgb(1., 1., 1.)
            } else {
                Color::rgb(0., 0., 0.)
            }
        } else {
            Color::rgb(0., 0., 0.)
        }
    });
}

pub(crate) struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderTimer(Timer::from_seconds(0.01, true)))
            .add_startup_system(spawn_tiles);
    }
}
