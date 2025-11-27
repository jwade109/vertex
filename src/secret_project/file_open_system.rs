use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use rfd::FileDialog;
use std::path::PathBuf;

pub struct FilePlugin;

impl Plugin for FilePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FileMessage>();
        app.add_message::<OpenPuzzle>();
        app.add_systems(Update, (open_dialogue, poll_tasks));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Any,
    Puzzle,
    ReferenceImage,
}

impl FileType {
    fn filter(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Any => ("Anything", &["txt", "png", "jpg", "jpeg"]),
            Self::Puzzle => ("Puzzle", &["txt"]),
            Self::ReferenceImage => ("Image", &["png", "jpg", "jpeg"]),
        }
    }
}

#[derive(Message, Debug)]
pub enum FileMessage {
    OpenFile(FileType),
    Opened(FileType, PathBuf),
}

#[derive(Component)]
struct SelectedFile {
    id: FileType,
    task: Task<Option<PathBuf>>,
}

#[derive(Message, Debug)]
pub struct OpenPuzzle(pub PathBuf);

fn open_dialogue(mut commands: Commands, mut msg: MessageReader<FileMessage>) {
    for msg in msg.read() {
        let id = if let FileMessage::OpenFile(id) = msg {
            id
        } else {
            continue;
        };

        let thread_pool = AsyncComputeTaskPool::get();
        let (name, ext) = id.filter();
        let dg = FileDialog::new().set_directory("/").add_filter(name, ext);
        let task = thread_pool.spawn(async move { dg.pick_file() });
        commands.spawn(SelectedFile { id: *id, task });
    }
}

use futures_lite::future;

fn poll_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut SelectedFile)>) {
    for (entity, mut sel) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut sel.task)) {
            info!("{:?}", result);
            commands.entity(entity).despawn();
            if let Some(path) = result {
                commands.write_message(FileMessage::Opened(sel.id, path));
            }
        }
    }
}
