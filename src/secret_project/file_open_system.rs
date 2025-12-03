use rfd::FileDialog;
use std::path::PathBuf;

use crate::secret_project::*;

pub struct FilePlugin;

impl Plugin for FilePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FileMessage>();
        app.add_message::<OpenPuzzleById>();
        app.add_systems(Update, (open_dialogue, poll_tasks));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FileType;

impl FileType {
    fn filter() -> (&'static str, &'static [&'static str]) {
        ("Image", &["png", "jpg", "jpeg"])
    }
}

#[derive(Message, Debug)]
pub enum FileMessage {
    OpenFile(FileType),
    Opened(FileType, PathBuf),
}

#[derive(Component)]
struct SelectedFile {
    task: Task<Option<PathBuf>>,
}

#[derive(Message, Debug)]
pub struct OpenPuzzleById(pub usize);

fn open_dialogue(mut commands: Commands, mut msg: MessageReader<FileMessage>) {
    for msg in msg.read() {
        let _ = if let FileMessage::OpenFile(id) = msg {
            id
        } else {
            continue;
        };

        let thread_pool = AsyncComputeTaskPool::get();
        let (name, ext) = FileType::filter();
        let dg = FileDialog::new().set_directory("/").add_filter(name, ext);
        let task = thread_pool.spawn(async move { dg.pick_file() });
        commands.spawn(SelectedFile { task });
    }
}

fn poll_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut SelectedFile)>) {
    for (entity, mut sel) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut sel.task)) {
            info!("{:?}", result);
            commands.entity(entity).despawn();
            if let Some(path) = result {
                commands.write_message(FileMessage::Opened(FileType, path));
            }
        }
    }
}
