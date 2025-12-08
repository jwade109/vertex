use crate::secret_project::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PuzzleManifestInfo {
    pub short_name: String,
    pub title: String,
}

impl PuzzleManifestInfo {
    pub fn new(short_name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            short_name: short_name.into(),
            title: title.into(),
        }
    }
}

#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub puzzles: Vec<PuzzleManifestInfo>,
}

impl Manifest {
    pub fn from_file(path: &Path) -> Result<Self, VertexError> {
        info!("Loading manifest at {}", path.display());
        Ok(load_from_file(path)?)
    }

    pub fn get(&self, idx: usize) -> Option<&PuzzleManifestInfo> {
        self.puzzles.get(idx)
    }
}
