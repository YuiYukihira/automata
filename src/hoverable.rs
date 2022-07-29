use bevy::prelude::*;

use crate::basic_setup::CursorState;

#[derive(Component)]
pub struct Hoverable;

#[derive(Component)]
pub struct Hovering;

/// Hovering
pub fn hoverable(
    mut commands: Commands,
    cursor_state: Query<&CursorState>,
    hoverable: Query<(Entity, &Transform, &Sprite), With<Hoverable>>,
) {
    let cursor_state = cursor_state.iter().next().unwrap();

    if cursor_state.cursor_moved {
        for (entity, transform, sprite) in hoverable.iter() {
            if cursor_state.in_range_xy(transform, sprite) {
                commands.entity(entity).insert(Hovering);
            } else {
                commands.entity(entity).remove::<Hovering>();
            }
        }
    }
}
