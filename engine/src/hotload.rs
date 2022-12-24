use crate::Engine;
use ahash::HashSet;
use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
};

pub struct Hotloader {
    _watcher: RecommendedWatcher,
    rx: Receiver<PathBuf>,
    paths: HashSet<PathBuf>,
}

impl Hotloader {
    pub fn new(plugins: &[PathBuf]) -> Result<Self> {
        let paths: HashSet<PathBuf> = plugins
            .iter()
            .map(|p| p.canonicalize().expect("Path cannot cannonicalize"))
            .collect();

        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(Event { paths, .. }) => {
                for path in paths {
                    tx.send(path).expect("Hotloader failed to send");
                }
            }
            Err(e) => log::error!("File watch error: {:?}", e),
        })?;

        for path in &paths {
            watcher.watch(
                &path.parent().expect("File has no parent"),
                RecursiveMode::NonRecursive,
            )?;
        }

        Ok(Self {
            rx,
            paths,
            _watcher: watcher,
        })
    }

    pub fn hotload(&mut self) -> Result<HashSet<PathBuf>> {
        Ok(self
            .rx
            .try_iter()
            .filter_map(|p| p.canonicalize().ok())
            .filter(|p| self.paths.contains(p))
            .collect())
    }
}
