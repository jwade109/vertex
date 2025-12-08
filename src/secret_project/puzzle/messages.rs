use crate::*;

pub struct PuzzleMessagePlugin;

impl Plugin for PuzzleMessagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DeleteVertex>()
            .add_message::<AddVertex>()
            .add_message::<AddEdge>()
            .add_message::<DeleteEdge>()
            .add_message::<ToggleEdge>()
            .add_message::<Quantize>()
            .add_systems(
                Update,
                (
                    on_delete_vertex,
                    on_delete_edge,
                    on_add_edge,
                    on_add_vertex,
                    on_quantize,
                    on_toggle_edge,
                ),
            );
    }
}

#[derive(Message, Debug)]
pub struct AddVertex(pub Vec2);

#[derive(Message, Debug)]
pub struct DeleteVertex(pub usize);

#[derive(Message, Debug)]
pub struct AddEdge(pub usize, pub usize);

#[derive(Message, Debug)]
pub struct DeleteEdge(pub usize, pub usize);

#[derive(Message, Debug)]
pub struct ToggleEdge(pub usize, pub usize);

#[derive(Message, Debug)]
pub struct Quantize(pub u16);

fn on_add_vertex(mut puzzle: Single<&mut Puzzle>, mut messages: MessageReader<AddVertex>) {
    for msg in messages.read() {
        puzzle.add_point(msg.0);
    }
}

fn on_delete_vertex(
    mut commands: Commands,
    mut puzzle: Single<&mut Puzzle>,
    mut save: ResMut<SaveData>,
    mut messages: MessageReader<DeleteVertex>,
) {
    for msg in messages.read() {
        if let Some(v) = puzzle.vertex_n(msg.0) {
            commands.spawn(Ripple::new(v.pos));
        }
        puzzle.remove_vertex(msg.0, &mut save);
    }
}

fn on_add_edge(
    mut puzzle: Single<&mut Puzzle>,
    mut save: ResMut<SaveData>,
    mut messages: MessageReader<AddEdge>,
    state: Res<State<AppState>>,
) {
    for msg in messages.read() {
        if state.is_editor() {
            puzzle.add_solution_edge(msg.0, msg.1);
        } else {
            puzzle.add_game_edge(msg.0, msg.1, &mut save);
        }
    }
}

fn on_delete_edge(
    mut puzzle: Single<&mut Puzzle>,
    mut messages: MessageReader<DeleteEdge>,
    mut save: ResMut<SaveData>,
    state: Res<State<AppState>>,
) {
    for msg in messages.read() {
        if state.is_editor() {
            puzzle.remove_solution_edge(msg.0, msg.1);
        } else {
            save.edges.remove_edge(msg.0, msg.1);
        }
    }
}

fn on_toggle_edge(
    mut puzzle: Single<&mut Puzzle>,
    mut save: ResMut<SaveData>,
    mut messages: MessageReader<ToggleEdge>,
    state: Res<State<AppState>>,
) {
    let is_play = !state.is_editor();
    for msg in messages.read() {
        puzzle.toggle_edge(&mut save, msg.0, msg.1, is_play);
    }
}

fn on_quantize(mut puzzle: Single<&mut Puzzle>, mut messages: MessageReader<Quantize>) {
    for msg in messages.read() {
        puzzle.quantize_colors(msg.0);
    }
}
