use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{format_err, Result};
use cimvr_engine::{calculate_digest, interface::prelude::Digest};
use directories::ProjectDirs;

pub type Manifest = HashMap<Digest, PathBuf>;

pub struct FileCache {
    root: PathBuf,
    manifest: Manifest,
}

impl FileCache {
    /// Load the file cache (cheap)
    pub fn new(root: PathBuf) -> Result<Self> {
        let manifest = read_manifest(&root)?;

        Ok(Self { root, manifest })
    }

    /// Get the manifest describing all files in the cache
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Insert a file into the cache
    pub fn add_file(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let digest = calculate_digest(data);
        let cache_name = CacheName(digest, name.to_string());
        let fname = cache_name.to_string();
        let path = self.root.join(fname);
        Ok(std::fs::write(path, data)?)
    }
}

/// Read the file cache, parsing valid names into a manifest
fn read_manifest(path: &Path) -> Result<Manifest> {
    // Make sure directory exists
    if !path.is_dir() {
        std::fs::create_dir_all(path)?;
    }

    let mut manifest = HashMap::new();

    for file in std::fs::read_dir(path)? {
        let file = file?;
        let path = file.path();
        let Some(fname) = path.file_name() else {
            continue;
        };
        let fname = fname.to_str().ok_or(format_err!("Invalid cache name"))?;

        if let Ok(CacheName(digest, _)) = fname.parse() {
            manifest.insert(digest, path);
        }
    }

    Ok(manifest)
}

struct CacheName(Digest, String);

impl Display for CacheName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(digest, name) = self;
        write!(f, "{digest}_{name}")
    }
}

impl FromStr for CacheName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (digest, name) = s
            .split_once('_')
            .ok_or(format_err!("No _ in cached file"))?;
        Ok(Self(digest.parse()?, name.parse()?))
    }
}
