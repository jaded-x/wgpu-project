use std::{path::{PathBuf, Path}, sync::mpsc::Receiver, collections::HashMap};

use notify::{RecursiveMode, Watcher, Result};

pub struct FileWatcher {
    watcher: notify::ReadDirectoryChangesWatcher,
    recently_removed: HashMap<String, PathBuf>,
    rx: Receiver<Result<notify::Event>>
}

impl FileWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let watcher = notify::recommended_watcher(move |res| tx.send(res).unwrap())?;

        Ok(Self {
            watcher,
            recently_removed: HashMap::new(),
            rx,
        })
    }

    pub fn watch(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)
    }

    pub fn handle_events(&mut self, textures: &mut TextureManager) {
        if let Ok(event) = self.rx.try_recv() {
            match event {
                Ok(event) => {
                    match event.kind {
                        notify::EventKind::Remove(_) => {
                            self.recently_removed.insert(event.paths[0].file_name().unwrap().to_string_lossy().into_owned(), event.paths[0].clone());
                        }
                        notify::EventKind::Create(_) => {
                            let file_name = event.paths[0].file_name().unwrap().to_string_lossy().into_owned();
                            if let Some(old_path) = self.recently_removed.remove(&file_name) {
                                textures.update_location(old_path, event.paths[0].clone())
                            }
                        }
                        _ => {}
                    }
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }
}