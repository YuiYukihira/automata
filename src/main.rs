use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    sync::Arc,
};

use bevy::{prelude::*, tasks::ComputeTaskPool, utils::HashMap};

static SCREEN_CELLS: (usize, usize) = (16, 16);
static SCREEN_CELLS_FLOAT: (f32, f32) = (16.0, 16.0);

#[derive(Component, Deref, PartialEq)]
struct BoardPosition(IVec2);

#[derive(Component, Deref)]
struct Alive(bool);

#[derive(Component, Deref)]
struct NextAlive(bool);

#[derive(Deref, DerefMut)]
struct CellTimer(Timer);

#[derive(Debug)]
struct Board {
    forward: Vec<Option<Entity>>,
    backward: HashMap<Entity, usize>,
    width: usize,
    height: usize,
}

impl Board {
    fn new(width: usize, height: usize) -> Self {
        Self {
            forward: vec![None; width * height],
            backward: HashMap::new(),
            width,
            height,
        }
    }
    fn get(&self, x: usize, y: usize) -> &Option<Entity> {
        &self.forward[self.pos_to_index(x, y)]
    }
    fn get_mut(&mut self, x: usize, y: usize) -> &mut Option<Entity> {
        let i = self.pos_to_index(x, y);
        &mut self.forward[i]
    }
    fn insert(&mut self, x: usize, y: usize, entity: Entity) -> Option<Entity> {
        let mut e = Some(entity);
        self.backward.insert(entity, self.pos_to_index(x, y));
        std::mem::swap(self.get_mut(x, y), &mut e);
        e
    }

    fn pos_to_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

fn cell_to_pos(window: &Vec4, grid_size: &Vec2, cell: &BoardPosition) -> Vec2 {
    let x_step_per_cell = (window.y - window.w) / grid_size.x;
    let y_step_per_cell = (window.x - window.z) / grid_size.y;

    Vec2::new(
        (x_step_per_cell * cell.x as f32) + window.w,
        (y_step_per_cell * cell.y as f32) + window.z,
    )
}

fn spawn_system(windows: Res<Windows>, mut commands: Commands) {
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

    for x in 0..SCREEN_CELLS.0 {
        for y in 0..SCREEN_CELLS.1 {
            let cell = BoardPosition(IVec2::new(x as i32, y as i32));
            let pos = cell_to_pos(
                &window_dimensions,
                &Vec2::new(SCREEN_CELLS_FLOAT.0, SCREEN_CELLS_FLOAT.1),
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
                .insert(Alive(
                    (x % 2 == 0 || y % 2 == 0) && !(x % 2 == 0 && y % 2 == 0),
                ))
                .insert(NextAlive(false));
        }
    }
}

fn after_spawn(
    tiles: Query<(Entity, &BoardPosition), Changed<BoardPosition>>,
    mut board: ResMut<Board>,
) {
    dbg!(tiles.is_empty());
    for (entity, tile) in tiles.iter() {
        board.insert(tile.x as usize, tile.y as usize, entity);
    }
}

fn update_next_state(
    time: Res<Time>,
    pool: Res<ComputeTaskPool>,
    tiles: Query<&Alive>,
    mut next_alives: Query<(&BoardPosition, &mut NextAlive)>,
    board: Res<Board>,
    mut timer: ResMut<CellTimer>,
) -> bool {
    //dbg!(&board);
    if timer.tick(time.delta()).just_finished() {
        next_alives.par_for_each_mut(&pool, 1, |(pos, mut next_alive)| {
            let alives = ((pos.x - 1)..(pos.x + 1))
                .zip((pos.y - 1)..(pos.y + 1))
                .filter_map(|(x, y)| {
                    let x = x.rem_euclid(board.width as i32);
                    let y = y.rem_euclid(board.height as i32);
                    board
                        .get(x as usize, y as usize)
                        .and_then(|e| tiles.get(e).ok())
                })
                .map(|a| {
                    println!("{}", pos.0);
                    a
                })
                .filter(|alive| ***alive)
                .map(|a| {
                    println!("{}", pos.0);
                    a
                })
                .count();
            next_alive.0 = (3..=5).contains(&alives);
        });
        true
    } else {
        false
    }
}

fn after_next_state(
    In(should_update): In<bool>,
    pool: Res<ComputeTaskPool>,
    mut tiles: Query<(&mut Alive, &NextAlive)>,
) {
    if should_update {
        tiles.par_for_each_mut(&pool, 32, |(mut alive, next_alive)| {
            alive.0 = next_alive.0;
        });
    }
}

fn update_colors(
    pool: Res<ComputeTaskPool>,
    mut tiles: Query<(&mut Sprite, &Alive), Changed<Alive>>,
) {
    tiles.par_for_each_mut(&pool, 32, |(mut sprite, alive)| {
        sprite.color = match alive.deref() {
            true => Color::rgb(1., 1., 1.),
            false => Color::rgb(0., 0., 0.),
        }
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Board::new(SCREEN_CELLS.0, SCREEN_CELLS.1))
        .insert_resource(CellTimer(Timer::from_seconds(0.5, true)))
        .add_startup_system(spawn_system)
        .add_system(after_spawn)
        .add_system(update_colors)
        .add_system(update_next_state.chain(after_next_state))
        .run();
}
