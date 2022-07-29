use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::WindowResizeConstraints};

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct MainCamera;

#[derive(Default, Component)]
pub struct CursorState {
    pub cursor_world: Vec2,
    pub cursor_moved: bool,
}

impl CursorState {
    pub fn in_range_xy(&self, transform: &Transform, sprite: &Sprite) -> bool {
        if sprite.custom_size.is_none() {
            return false;
        }
        let sprite_size = sprite.custom_size.unwrap();
        let half_size: Vec2 = sprite_size / 2.0;

        transform.translation.x - half_size.x < self.cursor_world.x
            && transform.translation.x + half_size.x > self.cursor_world.x
            && transform.translation.y - half_size.y < self.cursor_world.y
            && transform.translation.y + half_size.y > self.cursor_world.y
    }
}

fn cursor_state(
    mut events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut cursor_state: Query<&mut CursorState>,
    camera: Query<&Transform, With<MainCamera>>,
) {
    let window = windows.get_primary().unwrap();
    let camera_transform = camera.iter().last().unwrap();

    for mut cursor_state in cursor_state.iter_mut() {
        for event_cursor_screen in events.iter() {
            cursor_state.cursor_world =
                cursor_to_world(window, camera_transform, event_cursor_screen.position);

            cursor_state.cursor_moved = true;
        }
    }
}

fn cursor_transform(
    mut commands: Commands,
    cursor_state: Query<&CursorState>,
    mut cursor: Query<(Entity, &mut Transform), With<Cursor>>,
) {
    let cursor_state = cursor_state.iter().next().unwrap();

    for (cursor_entity, mut transform) in cursor.iter_mut() {
        transform.translation.x = cursor_state.cursor_world.x;
        transform.translation.y = cursor_state.cursor_world.y;
        commands.entity(cursor_entity).remove::<Parent>();
    }
}

fn cursor_to_world(window: &Window, cam_transform: &Transform, cursor_position: Vec2) -> Vec2 {
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

    // (0, 0) is in the middle of the screen
    let screen_position = cursor_position - window_size / 2.0;

    // apply the camera transform
    let cam_to_screen = cam_transform.compute_matrix() * screen_position.extend(0.0).extend(1.0);
    Vec2::new(cam_to_screen.x, cam_to_screen.y)
}

fn basic_setup(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);

    commands.spawn().insert(CursorState::default());
    commands
        .spawn()
        .insert_bundle((Transform::default(), GlobalTransform::default(), Cursor));
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct BasicSetup;

pub(crate) struct BasicSetupPlugin;
impl Plugin for BasicSetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(WindowDescriptor {
                width: 800.0,
                height: 800.0,
                resize_constraints: WindowResizeConstraints {
                    min_width: 600.0,
                    min_height: 800.0,
                    max_width: f32::INFINITY,
                    max_height: f32::INFINITY,
                },
                resizable: true,
                title: "Conway's Game of Life".to_string(),
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#gol_canvas".to_string()),
                ..Default::default()
            })
            .add_plugins(DefaultPlugins)
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(basic_setup)
            .add_system(cursor_state)
            .add_system(cursor_transform);
    }
}
