use crate::secret_project::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
pub enum EditorMode {
    Edit,
    Images,
    Select,
    Eraser,
    Brush,
    Play,
}

impl EditorMode {
    pub fn is_play(&self) -> bool {
        *self == EditorMode::Play
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingState {
    Loading,
    Done,
}
