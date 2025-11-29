use crate::secret_project::*;

pub struct AutoSolverPlugin;

impl Plugin for AutoSolverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            FixedUpdate,
            do_autosolver.run_if(in_state(EditorMode::Play)),
        );
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Autosolver {
    timer: Timer,
    enabled: bool,
}

impl Autosolver {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            enabled: false,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Autosolver::new());
}

fn do_autosolver(
    mut commands: Commands,
    mut solver: ResMut<Autosolver>,
    puzzle: Single<&Puzzle>,
    time: Res<Time<Fixed>>,
) {
    if !solver.enabled {
        return;
    }

    let dt = time.delta();
    let mut edges = HashSet::new();

    solver.timer.tick(dt);
    if !solver.timer.just_finished() {
        return;
    }

    for (a, b) in &puzzle.solution_edges.0 {
        if !puzzle.game_edges.is_edge(*a, *b) && !edges.contains(&(*a, *b)) {
            edges.insert((*a, *b));
        }

        if edges.len() > 20 {
            break;
        }
    }

    if edges.is_empty() {
        commands.write_message(TextMessage::debug("Done!"));
        solver.enabled = false;
    }

    for (a, b) in edges {
        commands.write_message(ToggleEdge(a, b));
    }
}
