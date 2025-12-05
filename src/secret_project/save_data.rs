use crate::secret_project::*;

#[derive(Component, Debug, Default, Deserialize, Serialize)]
pub struct SaveData {
    pub is_complete: bool,
    pub was_ever_complete: bool,
    pub edges: Edges,
}

impl SaveData {
    pub fn from_file(path: &Path) -> Result<Self, VertexError> {
        if std::fs::exists(path).unwrap_or(false) {
            let repr: SaveData = load_from_file(path)?;
            info!("Loaded autosave at {}", path.display());
            Ok(repr)
        } else {
            info!("No autosave data at {}", path.display());
            Ok(SaveData::default())
        }
    }

    #[deprecated(note = "This doesn't take `was_ever_complete` into account")]
    pub fn from_puzzle(puzzle: &Puzzle) -> Self {
        let is_complete = puzzle.is_complete();
        Self {
            is_complete,
            was_ever_complete: is_complete,
            edges: puzzle.game_edges.clone(),
        }
    }
}
