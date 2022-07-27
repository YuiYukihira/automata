use std::ops::Deref;

use bevy::{prelude::*, tasks::ComputeTaskPool, utils::HashMap};

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

static BOARD: [[bool; 7]; 3] = [
    [false, true, false, false, false, false, false],
    [false, false, false, true, false, false, false],
    [true, true, false, false, true, true, true],
];

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

    fn resize(&mut self, x: usize, y: usize) {
        *self = Self::new(x, y)
    }
}

fn cell_to_pos(
    window: &Vec4,
    grid_size: &(usize, usize),
    cell_size: &(f32, f32),
    cell: &BoardPosition,
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

fn spawn_system(windows: Res<Windows>, mut board: ResMut<Board>, mut commands: Commands) {
    let window = windows.primary();
    let window_dimensions = Vec4::new(
        window.height() / 2.,
        window.width() / 2.,
        window.height() / -2.,
        window.width() / -2.,
    );

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let pixels = window.height().min(window.width());
    let cell_size = Vec2::splat(4.);
    let cell_count = (
        (window.width() / cell_size.x) as usize,
        (window.height() / cell_size.y) as usize,
    );
    dbg!(
        window.height(),
        window.width(),
        pixels,
        cell_size,
        cell_count
    );

    let board_start_stop = (
        ((cell_count.0 - 7) / 2, ((cell_count.0 - 7) / 2) + 6),
        ((cell_count.1 - 3) / 2, ((cell_count.1 - 3) / 2) + 2),
    );

    board.resize(cell_count.0, cell_count.1);

    for x in 0..cell_count.0 {
        for y in 0..cell_count.1 {
            let cell = BoardPosition(IVec2::new(x as i32, y as i32));
            let pos = cell_to_pos(
                &window_dimensions,
                &(cell_count.0, cell_count.1),
                &(cell_size.x, cell_size.y),
                &cell,
            );

            let alive = if x >= board_start_stop.0 .0
                && x <= board_start_stop.0 .1
                && y >= board_start_stop.1 .0
                && y <= board_start_stop.1 .1
            {
                BOARD[y - board_start_stop.1 .0][x - board_start_stop.0 .0]
            } else {
                false
            };

            let tile = commands
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
                .insert(Alive(alive))
                .insert(NextAlive(false))
                .id();

            board.insert(x, y, tile);
        }
    }
}

fn update_next_state(
    time: Res<Time>,
    pool: Res<ComputeTaskPool>,
    tiles: Query<&Alive>,
    mut next_alives: Query<(&BoardPosition, &mut NextAlive, &Alive)>,
    board: Res<Board>,
    mut timer: ResMut<CellTimer>,
) -> bool {
    //dbg!(&board);
    if timer.tick(time.delta()).just_finished() {
        next_alives.par_for_each_mut(&pool, 128, |(pos, mut next_alive, alive)| {
            let alives: usize = ((pos.x - 1)..=(pos.x + 1))
                .map::<usize, _>(|x| {
                    ((pos.y - 1)..=(pos.y + 1))
                        .map(move |y| (x, y))
                        .filter(|(x, y)| *x != pos.x || *y != pos.y)
                        .filter_map(|(x1, y1)| {
                            let x = x1.rem_euclid(board.width as i32);
                            let y = y1.rem_euclid(board.height as i32);
                            board
                                .get(x as usize, y as usize)
                                .and_then(|e| tiles.get(e).ok())
                        })
                        .map(|a| if **a { 1 } else { 0 })
                        .sum()
                })
                .sum();

            next_alive.0 = if **alive {
                (2..=3).contains(&alives)
            } else {
                alives == 3
            }
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
        tiles.par_for_each_mut(&pool, 128, |(mut alive, next_alive)| {
            alive.0 = next_alive.0;
        });
    }
}

fn update_colors(
    pool: Res<ComputeTaskPool>,
    mut tiles: Query<(&mut Sprite, &Alive), Changed<Alive>>,
) {
    tiles.par_for_each_mut(&pool, 128, |(mut sprite, alive)| {
        sprite.color = match alive.deref() {
            true => Color::rgb(1., 1., 1.),
            false => Color::rgb(0., 0., 0.),
        }
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Board::new(0, 0))
        .insert_resource(CellTimer(Timer::from_seconds(0.1, true)))
        .add_startup_system(spawn_system)
        //.add_startup_system(after_spawn.after(spawn_system))
        .add_system(update_colors)
        .add_system(update_next_state.chain(after_next_state))
        .run();
}
