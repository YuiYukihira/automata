use bevy::prelude::*;
use board::{Alive, Board, BoardPosition, GamePlaying};
use board_asset::BoardAsset;
use hoverable::Hovering;

mod basic_setup;
mod board;
mod board_asset;
mod hoverable;
mod render;
mod view;

#[derive(Debug)]
struct InitialBoard(Handle<BoardAsset>, bool);

#[derive(Debug)]
struct PauseTimer(Timer, bool);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let file = std::env::args().nth(1).unwrap();
    println!("Loading: {}", file);
    let initial_board_handle = asset_server.load(&file);
    commands.insert_resource(InitialBoard(initial_board_handle, false));
}

fn intital_board_setup(
    mut initial_board: ResMut<InitialBoard>,
    mut board: ResMut<Board>,
    assets: Res<Assets<BoardAsset>>,
    mut commands: Commands,
) {
    if initial_board.1 {
        return;
    }
    let cell_count = board.size();

    let initial_board_asset = assets.get(&initial_board.0).unwrap();

    let board_offset = (
        (cell_count.x as i32 - initial_board_asset.size.0 as i32) / 2,
        (cell_count.y as i32 - initial_board_asset.size.1 as i32) / 2,
    );

    for (y, line) in initial_board_asset.data.iter().enumerate() {
        for (x, &is_set) in line.iter().enumerate() {
            let pos = IVec2::new(
                x as i32 + board_offset.0,
                (initial_board_asset.size.1 as i32 - y as i32) + board_offset.1,
            );
            let maybe_tile = board.get(pos);
            if let Some(tile) = maybe_tile {
                if is_set {
                    commands.entity(*tile).insert(Alive);
                }
            } else {
                let entity = {
                    let mut e = commands.spawn();
                    e.insert(BoardPosition(pos));
                    if is_set {
                        e.insert(Alive);
                    }
                    e
                }
                .id();
                board.insert(pos, entity);
            }
        }
    }
    initial_board.1 = true;
}

fn board_click(
    mouse_input: Res<Input<MouseButton>>,
    mut alive: Query<Entity, (With<Hovering>, With<Alive>)>,
    mut dead: Query<Entity, (With<Hovering>, Without<Alive>)>,
    mut commands: Commands,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        for entity in alive.iter_mut() {
            commands.entity(entity).remove::<Alive>();
        }
        for entity in dead.iter_mut() {
            commands.entity(entity).insert(Alive);
        }
    }
}

fn switch_state(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_state: ResMut<State<GamePlaying>>,
    mut timer: ResMut<PauseTimer>,
    time: Res<Time>,
) {
    if !timer.1 {
        if keyboard_input.just_pressed(KeyCode::Space) {
            match game_state.current() {
                GamePlaying::Paused => game_state.set(GamePlaying::Playing),
                GamePlaying::Playing => game_state.set(GamePlaying::Paused),
            }
            .unwrap();
            timer.1 = true;
        }
    } else if timer.0.tick(time.delta()).just_finished() {
        timer.1 = false;
    }
}

fn main() {
    App::new()
        .add_plugin(basic_setup::BasicSetupPlugin)
        .add_plugin(view::ViewPlugin)
        .add_plugin(board::BoardPlugin)
        .add_plugin(render::RenderPlugin)
        .add_plugin(board_asset::BoardAssetPlugin)
        .insert_resource(PauseTimer(Timer::from_seconds(0.2, true), false))
        //.add_startup_system(after_spawn.after(spawn_system))
        .add_system_set(SystemSet::on_enter(GamePlaying::Paused).with_system(intital_board_setup))
        .add_system_set(
            SystemSet::on_update(GamePlaying::Paused)
                .with_system(board_click)
                .with_system(hoverable::hoverable),
        )
        .add_system(switch_state)
        .add_startup_system(setup)
        .run();
}
