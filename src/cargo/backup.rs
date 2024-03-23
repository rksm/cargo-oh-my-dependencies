use eyre::Result;
use std::path::PathBuf;

pub struct ManifestBackup {
    manifest_path: PathBuf,
    manifest: Vec<u8>,
    manifest_lock_path: Option<PathBuf>,
    manifest_lock: Option<Vec<u8>>,
    disposed: bool,
}

impl ManifestBackup {
    pub fn create(manifest_path: impl Into<PathBuf>) -> Result<Self> {
        let manifest_path: PathBuf = manifest_path.into();

        let with_extension = &manifest_path.with_extension("lock");
        let lock_file_name = with_extension.file_name().expect("lock file name");

        let mut manifest_lock_path = Some(manifest_path.with_extension("lock"));
        let manifest_lock_path = loop {
            match manifest_lock_path.take() {
                Some(p) if p.exists() => {
                    debug!("Found lock file: {:?}", p);
                    break Some(p);
                }
                Some(p) => {
                    manifest_lock_path = p
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.join(lock_file_name))
                }
                None => {
                    debug!("No lock file found for manifest: {:?}", manifest_path);
                    break None;
                }
            }
        };

        debug!("reading manifest: {:?}", manifest_path);
        let manifest = std::fs::read(&manifest_path)?;

        let manifest_lock = match &manifest_lock_path {
            Some(path) => {
                debug!("reading manifest lock: {:?}", path);
                Some(std::fs::read(path)?)
            }
            None => None,
        };

        Ok(Self {
            manifest_path,
            manifest,
            manifest_lock_path,
            manifest_lock,
            disposed: false,
        })
    }

    pub fn restore(&mut self) {
        if self.disposed {
            return;
        };
        self.disposed = true;
        if let Err(err) = std::fs::write(&self.manifest_path, &self.manifest) {
            error!("Failed to restore manifest {:?}: {err}", self.manifest_path);
        }

        if let (Some(p), Some(lock)) = (&self.manifest_lock_path, &self.manifest_lock) {
            if let Err(err) = std::fs::write(p, lock) {
                error!(
                    "Failed to restore manifest lock {:?}: {err}",
                    self.manifest_lock_path
                );
            }
        }
    }

    pub fn dispose(mut self) {
        self.disposed = true;
    }
}

impl Drop for ManifestBackup {
    fn drop(&mut self) {
        self.restore();
    }
}
