use crate::secret_project::*;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_network_request, poll_tasks));
        app.add_message::<NetworkFetch>();
    }
}

pub fn download_file(url: &str, path: &Path, overwrite: bool) -> Result<u64, VertexError> {
    let exists = std::fs::exists(path)?;
    if exists {
        if overwrite {
            info!("Overwriting {} to {}", url, path.display());
        } else {
            info!("{} already exists", path.display());
            return Ok(0);
        }
    } else {
        info!("Downloading {} to {}", url, path.display());
    }
    let mut resp = reqwest::blocking::get(url)?.error_for_status()?;
    if exists {
        std::fs::remove_file(path)?;
    }
    let mut file = std::fs::File::create(path)?;
    Ok(std::io::copy(&mut resp, &mut file)?)
}

#[allow(unused)]
#[derive(Debug)]
pub enum VertexError {
    Reqwest(reqwest::Error),
    Serde(serde_yaml::Error),
    IO(std::io::Error),
    Str(String),
}

impl From<reqwest::Error> for VertexError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_yaml::Error> for VertexError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<std::io::Error> for VertexError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<&str> for VertexError {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

fn do_network_fetch(install: Installation) -> Result<(), VertexError> {
    // always keep this updated
    install_remote_manifest(&install, true)?;

    let manifest = Manifest::from_file(&install.network_manifest())?;

    for puzzle in manifest.puzzles {
        info!("Installing {}", puzzle.short_name);
        match install_puzzle_file(&install, &puzzle.short_name, false) {
            Ok(_) => {
                info!("Success!");
            }
            Err(e) => {
                error!("Failed to install: {:?}", e);
            }
        }
    }

    Ok(())
}

pub const NETWORK_MANIFEST_URL: &'static str =
    "https://jwade109.github.io/vertex_puzzles/manifest.yaml";

pub fn puzzle_file_url(short_name: &str) -> String {
    format!(
        "https://jwade109.github.io/vertex_puzzles/{}/puzzle.txt",
        short_name
    )
}

#[derive(Message)]
pub struct NetworkFetch;

#[derive(Component)]
struct NetworkWorker {
    task: Task<Result<(), VertexError>>,
}

fn spawn_network_request(
    mut commands: Commands,
    mut msg: MessageReader<NetworkFetch>,
    tasks: Query<&NetworkWorker>,
    install: Res<Installation>,
) {
    if msg.is_empty() {
        return;
    }

    for _ in msg.read() {}

    if !tasks.is_empty() {
        warn!("Network request already in progress");
        return;
    }

    let install = install.clone();

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { do_network_fetch(install.clone()) });
    commands.spawn(NetworkWorker { task });
}

fn poll_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut NetworkWorker)>) {
    for (entity, mut sel) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut sel.task)) {
            match result {
                Ok(_) => info!("Task successful"),
                Err(e) => error!("Task failed: {:?}", e),
            }
            commands.entity(entity).despawn();
        }
    }
}
