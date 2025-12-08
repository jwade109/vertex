use crate::secret_project::*;

#[derive(Resource, Debug, Default, Deserialize, Serialize)]
pub struct SaveData {
    pub revealed_title: String,
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
}
