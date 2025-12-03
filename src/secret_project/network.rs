use crate::secret_project::*;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_network_request, spawn_network_req_on_key, poll_tasks),
        );
        app.add_message::<NetworkFetch>();
    }
}

#[allow(unused)]
#[derive(Debug)]
enum PuzzleIndexError {
    Reqwest(reqwest::Error),
    Serde(serde_yaml::Error),
}

impl From<reqwest::Error> for PuzzleIndexError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_yaml::Error> for PuzzleIndexError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Serde(value)
    }
}

fn do_network_fetch() -> Result<PuzzleIndex, PuzzleIndexError> {
    // let url = "";
    let url = "https://jwade109.github.io/vertex_puzzles/manifest.txt";
    let resp = reqwest::blocking::get(url)?;

    let mut index = PuzzleIndex::default();

    if let Ok(text) = resp.text() {
        let lines: Vec<&str> = text.lines().collect();
        println!("Got {} puzzles", lines.len());
        for (id, name) in lines.iter().enumerate() {
            let url = format!(
                "https://jwade109.github.io/vertex_puzzles/{}/puzzle.txt",
                name
            );
            let resp = reqwest::blocking::get(url.clone())?.error_for_status()?;
            let text = resp.text()?;

            let r: PuzzleFileStorage = serde_yaml::from_str(&text)?;

            let info = PuzzleInfo::new(r.title, PathBuf::new());

            info!(?info);

            index.insert(id, info);
        }
    }

    Ok(index)
}

#[derive(Message)]
pub struct NetworkFetch;

#[derive(Component)]
struct NetworkWorker {
    task: Task<Result<PuzzleIndex, PuzzleIndexError>>,
}

fn spawn_network_req_on_key(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyB) {
        commands.write_message(NetworkFetch);
    }
}

fn spawn_network_request(mut commands: Commands, mut msg: MessageReader<NetworkFetch>) {
    if msg.is_empty() {
        return;
    }

    for _ in msg.read() {}

    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { do_network_fetch() });
    commands.spawn(NetworkWorker { task });
}

fn poll_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut NetworkWorker)>) {
    for (entity, mut sel) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut sel.task)) {
            info!("{:?}", result);
            commands.entity(entity).despawn();
        }
    }
}
