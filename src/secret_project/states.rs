use crate::secret_project::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Playing {
        victory: bool,
    },
    Editing {
        mode: EditorMode,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum EditorMode {
    #[default]
    Edit,
    Images,
    Select,
    Eraser,
    Brush,
}

impl AppState {
    pub fn is_menu(&self) -> bool {
        match self {
            Self::Menu => true,
            _ => false,
        }
    }

    pub fn is_editor(&self) -> bool {
        match self {
            Self::Editing { .. } => true,
            _ => false,
        }
    }

    pub fn is_playing(&self) -> bool {
        match self {
            Self::Playing { .. } => true,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InEditorOrPlaying;

impl ComputedStates for InEditorOrPlaying {
    type SourceStates = AppState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            AppState::Loading => None,
            AppState::Menu => None,
            AppState::Playing { .. } => Some(Self),
            AppState::Editing { .. } => Some(Self),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct VictoryScreen;

impl ComputedStates for VictoryScreen {
    type SourceStates = AppState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            AppState::Playing { victory } => victory.then(|| Self),
            _ => None,
        }
    }
}

pub fn log_app_state_transitions(mut tr: MessageReader<StateTransitionEvent<AppState>>) {
    for msg in tr.read() {
        info!(
            "App state transition: {:?} -> {:?}",
            msg.exited, msg.entered
        );
    }
}

pub fn log_in_editor_state_transitions(
    mut tr: MessageReader<StateTransitionEvent<InEditorOrPlaying>>,
) {
    for msg in tr.read() {
        info!(
            "InEditorOrPlaying state transition: {:?} -> {:?}",
            msg.exited, msg.entered
        );
    }
}

pub fn log_victory_state_transitions(mut tr: MessageReader<StateTransitionEvent<VictoryScreen>>) {
    for msg in tr.read() {
        info!(
            "VictoryScreen state transition: {:?} -> {:?}",
            msg.exited, msg.entered
        );
    }
}

// run conditions

pub fn is_editor(state: Res<State<AppState>>) -> bool {
    state.is_editor()
}

pub fn is_playing(state: Res<State<AppState>>) -> bool {
    state.is_playing()
}

pub fn is_editor_or_playing(state: Res<State<AppState>>) -> bool {
    state.is_editor() || state.is_playing()
}

pub fn is_menu_or_playing(state: Res<State<AppState>>) -> bool {
    state.is_menu() || state.is_playing()
}

pub fn camera_is_moveable(state: Res<State<AppState>>) -> bool {
    match **state {
        AppState::Playing { victory } => !victory,
        AppState::Editing { .. } => true,
        _ => false,
    }
}
