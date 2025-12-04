use crate::secret_project::*;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_network_request, poll_tasks));
        app.add_message::<NetworkFetch>();
        app.insert_resource(NetworkManifest::default());
    }
}

pub fn download_file(url: &str, path: &Path, overwrite: bool) -> Result<u64, PuzzleManifestError> {
    if std::fs::exists(path)? {
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
    let mut file = std::fs::File::create(path)?;
    Ok(std::io::copy(&mut resp, &mut file)?)
}

#[derive(Debug, Clone)]
pub struct PuzzleNetworkInfo {
    pub short_name: String,
    pub url: String,
}

impl PuzzleNetworkInfo {
    pub fn new(short_name: String, url: String) -> Self {
        Self { short_name, url }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum PuzzleManifestError {
    Reqwest(reqwest::Error),
    Serde(serde_yaml::Error),
    IO(std::io::Error),
}

impl From<reqwest::Error> for PuzzleManifestError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_yaml::Error> for PuzzleManifestError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<std::io::Error> for PuzzleManifestError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

pub type NetworkPuzzleManifest = HashMap<usize, PuzzleNetworkInfo>;

pub const NETWORK_MANIFEST_URL: &'static str =
    "https://jwade109.github.io/vertex_puzzles/manifest.txt";

fn do_network_fetch(install: Installation) -> Result<(), PuzzleManifestError> {

    // always keep this updated
    install_remote_manifest(&install, true)?;

    install_puzzle_file(&install, "rose", false)?;
    install_puzzle_file(&install, "doggo", false)?;
    install_puzzle_file(&install, "rubik", false)?;
    install_puzzle_file(&install, "potato", false)?;

    Ok(())
}

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
    task: Task<Result<(), PuzzleManifestError>>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct NetworkManifest(pub Option<NetworkPuzzleManifest>);

fn spawn_network_request(
    mut commands: Commands,
    mut msg: MessageReader<NetworkFetch>,
    install: Res<Installation>,
) {
    if msg.is_empty() {
        return;
    }

    for _ in msg.read() {}

    let install = install.clone();

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { do_network_fetch(install.clone()) });
    commands.spawn(NetworkWorker { task });
}

fn poll_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut NetworkWorker)>,
    mut manifest: ResMut<NetworkManifest>,
) {
    for (entity, mut sel) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut sel.task)) {
            info!("{:?}", result);
            commands.entity(entity).despawn();
            // if let Ok(man) = result {
            //     manifest.0 = Some(man);
            // }
        }
    }
}
