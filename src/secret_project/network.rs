use crate::secret_project::*;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(MainStartup));
        app.add_systems(
            Update,
            (
                debug_log_loading_messages,
                poll_tasks
                    .run_if(on_timer(std::time::Duration::from_millis(20)))
                    .run_if(on_loading_screen),
            ),
        );
        app.add_message::<NetworkFetch>();
    }
}

#[derive(SystemSet, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct MainStartup;

fn setup(mut commands: Commands, install: Res<Installation>) {
    commands.insert_resource(NetworkWorker::run(install.clone()));
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

fn do_network_job(install: Installation) -> Result<(), VertexError> {
    install_remote_manifest(&install, true)?;

    let manifest: Manifest = Manifest::from_file(&install.network_manifest())?;

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

type NetworkTask = Task<Result<(), VertexError>>;

#[derive(Resource, Default)]
struct NetworkWorker {
    current_job: Option<NetworkTask>,
}

impl NetworkWorker {
    fn run(install: Installation) -> Self {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move { do_network_job(install) });
        Self {
            current_job: Some(task),
        }
    }
}

fn poll_tasks(
    mut commands: Commands,
    mut worker: ResMut<NetworkWorker>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(task) = &mut worker.current_job {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            match result {
                Ok(_) => {
                    info!("Task successful");
                    commands.write_message(LoadingMessage("Success!".to_string()));
                    state.set(AppState::Menu);
                }
                Err(e) => {
                    error!("Task failed: {:?}", e);
                    let s = format!("Failed to fetch puzzles: {:?}", e);
                    commands.write_message(LoadingMessage(s));
                }
            }
            worker.current_job = None;
        }
    }
}

fn debug_log_loading_messages(mut messages: MessageReader<LoadingMessage>) {
    for msg in messages.read() {
        info!(?msg);
    }
}
