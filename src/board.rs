use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;
use bevy::utils::HashMap;

use crate::render::update_colors;

#[derive(Debug)]
pub struct Board {
    forward: HashMap<IVec2, Entity>,
    backward: HashMap<Entity, IVec2>,
    size: UVec2,
    start: IVec2,
}

impl Board {
    pub fn new(size: UVec2) -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new(),
            size,
            start: IVec2::splat(0),
        }
    }

    /// Insert an entity at a position, automatically expands thde region to fit.
    pub fn insert(&mut self, pos: IVec2, entity: Entity) -> Option<Entity> {
        if pos.x < self.start.x {
            let diff = self.start.x.abs_diff(pos.x);
            self.start.x = pos.x;
            self.size.x += diff;
        }
        if pos.y < self.start.y {
            let diff = self.start.y.abs_diff(pos.y);
            self.start.y = pos.y;
            self.size.y += diff;
        }

        let old_position = self.backward.insert(entity, pos);
        self.forward.insert(pos, entity);

        old_position.and_then(|p| self.forward.remove(&p))
    }

    pub fn remove(&mut self, pos: IVec2) -> Option<Entity> {
        let entity = self.forward.remove(&pos);
        if let Some(e) = entity {
            self.backward.remove(&e);
        }
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<IVec2> {
        let pos = self.backward.remove(&entity);
        if let Some(p) = pos {
            self.forward.remove(&p);
        }
        pos
    }

    pub fn get(&self, pos: IVec2) -> Option<&Entity> {
        self.forward.get(&pos)
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn dimensions(&self) -> (IVec2, IVec2) {
        (
            self.start,
            self.start + IVec2::new(self.size.x as i32, self.size.y as i32),
        )
    }
}

#[derive(Debug, Component, Deref, PartialEq, Eq)]
pub struct BoardPosition(pub IVec2);

#[derive(Debug, Component)]
pub struct Alive;

#[derive(Debug, Component)]
pub struct NextAlive;

#[derive(Debug, Component)]
pub struct NextDie;

#[derive(Debug, Component)]
pub struct Update;

#[derive(Debug)]
pub enum GameRules {
    Conway,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamePlaying {
    Playing,
    Paused,
}

impl GameRules {
    fn compute_state(&self, alive: bool, alive_neighbours: usize) -> bool {
        match self {
            GameRules::Conway if alive => (2..=3).contains(&alive_neighbours),
            GameRules::Conway => alive_neighbours == 3,
        }
    }
}

#[derive(Debug, Deref, DerefMut)]
struct GameTimer(Timer);

fn update_next_state(
    time: Res<Time>,
    pool: Res<ComputeTaskPool>,
    alive_tiles: Query<&BoardPosition, (With<Alive>, Without<Update>)>,
    board: ResMut<Board>,
    mut timer: ResMut<GameTimer>,
    game_rules: Res<GameRules>,
    commands: Commands,
) {
    if timer.tick(time.delta()).just_finished() {
        // Get all the tiles we need to compute the state of,
        // This is all the currently alive tiles and thier neighbours.
        let positions_to_check = alive_tiles
            .iter()
            .flat_map(|pos: &BoardPosition| {
                ((pos.0.x - 1)..=(pos.0.x + 1))
                    .flat_map(|x| ((pos.0.y - 1)..=(pos.0.y + 1)).map(move |y| (x, y)))
            })
            .collect::<Vec<_>>();

        let commands = Arc::new(Mutex::new(commands));
        let game_rules = Arc::new(game_rules);
        let alive_tiles = Arc::new(alive_tiles);
        let board = Arc::new(Mutex::new(board));

        let new_alive_tiles = Arc::new(AtomicUsize::new(0));
        pool.scope(|s| {
            // For each of the positions to check
            for pos in positions_to_check {
                let commands = commands.clone();
                let game_rules = game_rules.clone();
                let alive_tiles = alive_tiles.clone();
                let board = board.clone();
                let new_alive_tiles = new_alive_tiles.clone();
                s.spawn(async move {
                    // Look at how many of it's neighbours are alive
                    let alive_neighbours = ((pos.0 - 1)..=(pos.0 + 1))
                        .flat_map(|x| ((pos.1 - 1)..=(pos.1 + 1)).map(move |y| (x, y)))
                        .filter(|(x, y)| *x != pos.0 || *y != pos.1)
                        .map(|(x, y)| IVec2::new(x, y))
                        .filter_map(|pos| {
                            board
                                .lock()
                                .unwrap()
                                .get(pos)
                                .and_then(|entity| alive_tiles.get(*entity).ok())
                        })
                        .count();

                    let entity = board.lock().unwrap().get(IVec2::new(pos.0, pos.1)).copied();

                    let alive = game_rules.compute_state(
                        entity.and_then(|e| alive_tiles.get(e).ok()).is_some(),
                        alive_neighbours,
                    );

                    // Add the NextAlive component on if it will be alive next tick
                    if alive {
                        match entity {
                            Some(e) => {
                                commands
                                    .lock()
                                    .unwrap()
                                    .entity(e)
                                    .insert(NextAlive)
                                    .remove::<NextDie>()
                                    .insert(Update);
                                new_alive_tiles.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            None => {
                                let e = commands
                                    .lock()
                                    .unwrap()
                                    .spawn()
                                    .insert(BoardPosition(IVec2::new(pos.0, pos.1)))
                                    .insert(NextAlive)
                                    .remove::<NextDie>()
                                    .insert(Update)
                                    .id();
                                board.lock().unwrap().insert(IVec2::new(pos.0, pos.1), e);
                                new_alive_tiles.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        };
                    } else if let Some(e) = entity {
                        commands
                            .lock()
                            .unwrap()
                            .entity(e)
                            .remove::<NextAlive>()
                            .insert(NextDie)
                            .insert(Update);
                    }
                })
            }
        });
    }
}

type LivingTiles<'a, 'b> = Query<'a, 'b, Entity, (With<NextAlive>, With<Update>)>;
type DeadTiles<'a, 'b> = Query<'a, 'b, Entity, (With<NextDie>, With<Update>)>;

fn after_next_state(
    pool: Res<ComputeTaskPool>,
    board: ResMut<Board>,
    tiles_to_live: LivingTiles,
    tiles_to_die: DeadTiles,
    commands: Commands,
) {
    //if should_update {
    let commands = Arc::new(Mutex::new(commands));
    let board = Arc::new(Mutex::new(board));
    tiles_to_live.par_for_each(&pool, 32, |entity| {
        commands
            .lock()
            .unwrap()
            .entity(entity)
            .insert(Alive)
            .remove::<NextAlive>()
            .remove::<Update>();
    });
    tiles_to_die.par_for_each(&pool, 32, |entity| {
        commands
            .lock()
            .unwrap()
            .entity(entity)
            .remove::<Alive>()
            .remove::<Update>()
            .remove::<NextDie>()
            .despawn();
        board.lock().unwrap().remove_entity(entity);
    });
}

pub(crate) struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTimer(Timer::from_seconds(0.02, true)))
            .insert_resource(GameRules::Conway)
            .insert_resource(Board::new(UVec2::splat(0)))
            .add_state(GamePlaying::Paused)
            .add_system_set(
                SystemSet::on_update(GamePlaying::Playing)
                    .with_system(update_next_state)
                    .with_system(after_next_state),
            );
    }
}
