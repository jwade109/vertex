use crate::secret_project::*;

#[derive(Resource, Debug, Clone)]
pub struct Installation(PathBuf);

impl Installation {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self(root.into())
    }

    pub fn initialize(root: impl Into<PathBuf>) -> Result<Self, VertexError> {
        let install = Self::new(root);
        initialize_install_directory(&install)?;
        Ok(install)
    }

    pub fn root(&self) -> &Path {
        &self.0
    }

    pub fn puzzles(&self) -> PathBuf {
        self.0.join("puzzles")
    }

    pub fn save_data(&self) -> PathBuf {
        self.0.join("save_data")
    }

    pub fn indicator(&self) -> PathBuf {
        self.0.join(".vertex_install")
    }

    pub fn network_manifest(&self) -> PathBuf {
        self.0.join("network_manifest.yaml")
    }

    pub fn puzzle_file(&self, short_name: &str) -> PathBuf {
        self.puzzles().join(short_name).with_extension("yaml")
    }

    pub fn save_data_file(&self, short_name: &str) -> PathBuf {
        self.save_data().join(short_name).with_extension("yaml")
    }

    pub fn settings(&self) -> PathBuf {
        self.0.join("settings.yaml")
    }
}

fn create_settings_file(install: &Installation) -> Result<(), VertexError> {
    // TODO
    Ok(())
}

pub fn initialize_install_directory(install: &Installation) -> Result<(), VertexError> {
    for dir in [install.puzzles(), install.save_data()] {
        if !std::fs::exists(&dir)? {
            match std::fs::create_dir_all(&dir) {
                Ok(_) => {
                    info!("Created install dir {}", dir.display())
                }
                Err(e) => {
                    error!("Failed to create {}: {:?}", dir.display(), e);
                }
            }
        }
    }

    create_settings_file(&install)?;

    let now = chrono::offset::Local::now();
    let s = format!("Install created on {}", now);

    if !std::fs::exists(install.indicator())? {
        match std::fs::write(install.indicator(), s) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create indicator file: {:?}", e);
            }
        }
    }

    Ok(())
}

pub fn load_puzzle_manifest(
    install: &Installation,
) -> Result<PuzzleManifest, Box<dyn std::error::Error>> {
    let paths = std::fs::read_dir(install.puzzles())?;
    let mut puzzles = PuzzleManifest::default();

    for (id, path) in paths.enumerate() {
        let path = path?;
        let path = path.path();
        let short_name = path
            .file_stem()
            .ok_or("No file stem!")?
            .to_str()
            .ok_or("File stem is not convertible to string!")?
            .to_string();
        let puzzle_file = path.join("puzzle.txt");
        let (puzzle, _) = puzzle_from_file(puzzle_file.clone())?;
        let is_complete = puzzle.is_complete();
        let info = PuzzleInstallInfo::new(
            puzzle.title().to_string(),
            short_name,
            puzzle_file,
            is_complete,
        );
        info!("Loaded puzzle info: {:?}", info);
        puzzles.insert(id, info);
    }

    Ok(puzzles)
}

pub fn install_remote_manifest(
    install: &Installation,
    overwrite: bool,
) -> Result<u64, VertexError> {
    let path = install.network_manifest();
    download_file(NETWORK_MANIFEST_URL, &path, overwrite)
}

pub fn install_puzzle_file(
    install: &Installation,
    short_name: &str,
    overwrite: bool,
) -> Result<u64, VertexError> {
    let puzzle_path = install.puzzle_file(short_name);
    let url = puzzle_file_url(short_name);
    download_file(&url, &puzzle_path, overwrite)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_install() {
        let test_dir = "/tmp/test_install";

        if std::fs::exists(test_dir).unwrap() {
            std::fs::remove_dir_all(test_dir).unwrap();
        }

        let install = Installation::initialize(test_dir).unwrap();
        let short_name = "rose";

        assert!(install_puzzle_file(&install, short_name, false).is_ok());
        assert!(std::fs::exists(install.puzzle_file(short_name)).unwrap());

        assert!(install_remote_manifest(&install, false).is_ok());
        assert!(std::fs::exists(install.network_manifest()).unwrap());
    }
}
