use crate::*;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorState::default())
            .insert_state(EditorMode::Edit)
            .add_systems(Update, update_cursor_mode);
    }
}

#[derive(Resource, Default, Debug)]
pub struct CursorState {
    pub mouse_pos: Option<Vec2>,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Sequence)]
pub enum EditorMode {
    Edit,
    Select,
    Eraser,
    Painter,
    Play,
}

fn update_cursor_mode(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<EditorMode>>,
    mut next: ResMut<NextState<EditorMode>>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        next.set(next_cycle(&state.get()));
        commands.write_message(TextMessage::new(format!("Switched to {:?}", *next)));
        commands.write_message(SoundEffect::UiTrill);
    }
}

pub fn draw_mouse_cursor(
    mut painter: ShapePainter,
    cursor: Res<CursorState>,
    camera: Single<&Transform, With<Camera>>,
) {
    let scale = camera.scale.x;

    if let Some(p) = cursor.mouse_pos {
        fill_circle(&mut painter, p, CURSOR_Z, 5.0 * scale, GRAY.with_alpha(0.3));
    }
}
