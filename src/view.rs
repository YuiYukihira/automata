use bevy::prelude::*;

#[derive(Component)]
pub struct View {
    pub offset: IVec2,
}

fn move_view(mut query: Query<&mut View>, keyboard: Res<Input<KeyCode>>) {
    for mut view in query.iter_mut() {
        if keyboard.any_pressed([KeyCode::W, KeyCode::Up]) {
            view.offset.y += 1;
        }
        if keyboard.any_pressed([KeyCode::S, KeyCode::Down]) {
            view.offset.y -= 1;
        }
        if keyboard.any_pressed([KeyCode::A, KeyCode::Left]) {
            view.offset.x -= 1;
        }
        if keyboard.any_pressed([KeyCode::D, KeyCode::Right]) {
            view.offset.x += 1;
        }
    }
}

pub fn startup_system(mut commands: Commands, windows: Res<Windows>) {
    let window = windows.primary();
    let cell_size = Vec2::splat(4.);
    let cell_count = IVec2::new(
        (window.width() / cell_size.x) as i32,
        (window.height() / cell_size.y) as i32,
    );

    commands.spawn().insert(View {
        offset: -cell_count / 2,
    });
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub(crate) struct ViewPlugin;
impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_view).add_startup_system(startup_system);
    }
}
