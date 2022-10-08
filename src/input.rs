use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(get_keyboard);
    }
}

fn get_keyboard(keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::G) {}
    if keyboard.just_pressed(KeyCode::T) {}
    if keyboard.just_pressed(KeyCode::X) {}
}
