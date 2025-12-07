use crate::secret_project::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NetworkPuzzleInfo {
    pub short_name: String,
    pub title: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NetworkManifest {
    pub puzzles: Vec<NetworkPuzzleInfo>,
}

impl NetworkManifest {
    pub fn from_file(path: &Path) -> Result<Self, VertexError> {
        info!("Loading manifest at {}", path.display());
        Ok(load_from_file(path)?)
    }
}
