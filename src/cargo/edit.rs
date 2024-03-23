use cargo_metadata::{DependencyKind, Package};
use eyre::{Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use crate::cargo::backup::ManifestBackup;

#[derive(Debug, Clone)]
pub struct EditDependency<'a> {
    manifest_path: PathBuf,
    package: &'a Package,
    dep_name: &'a str,
    dep_kind: DependencyKind,
    features_to_add: HashSet<String>,
    features_to_remove: HashSet<String>,
    is_workspace_dependency_at: Option<PathBuf>,
    dry_run: bool,
    is_test_install: bool,
}

impl<'a> EditDependency<'a> {
    pub fn new(package: &'a Package, dep_name: &'a str, dep_kind: DependencyKind) -> Self {
        Self {
            manifest_path: package.manifest_path.clone().into_std_path_buf(),
            package,
            dep_name,
            dep_kind,
            features_to_add: Default::default(),
            features_to_remove: Default::default(),
            is_workspace_dependency_at: None,
            dry_run: false,
            is_test_install: false,
        }
    }

    #[must_use]
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    #[must_use]
    pub fn toggle_feature(mut self, feature: impl Into<String>) -> Self {
        let feature = feature.into();
        let remove = self
            .package
            .dependencies
            .iter()
            .find(|dep| dep.name == self.dep_name)
            .map(|dep| {
                (feature == "default" && dep.uses_default_features)
                    || dep.features.contains(&feature)
            })
            .unwrap_or(false);

        if remove {
            self.features_to_remove.insert(feature);
        } else {
            self.features_to_add.insert(feature);
        }

        self
    }

    #[must_use]
    pub fn set_workspace_dependency_at(mut self, path: Option<impl Into<PathBuf>>) -> Self {
        self.is_workspace_dependency_at = path.map(Into::into);
        self
    }

    #[must_use]
    pub fn add_feature(mut self, feature: impl Into<String>) -> Self {
        self.features_to_add.insert(feature.into());
        self
    }

    #[must_use]
    pub fn remove_feature(mut self, feature: impl Into<String>) -> Self {
        self.features_to_remove.insert(feature.into());
        self
    }

    fn for_test_install(&self) -> Self {
        let mut clone = self.clone();
        clone.is_test_install = true;
        clone.dry_run = false;
        clone
    }

    fn cmd(&self) -> String {
        let mut cmds = Vec::new();
        let mut features = HashSet::new();

        let kind_param = match self.dep_kind {
            DependencyKind::Development => "--dev",
            DependencyKind::Build => "--build",
            _ => "",
        };

        let mut default_enabled = true;
        if let Some(existing) = self
            .package
            .dependencies
            .iter()
            .find(|dep| dep.name == self.dep_name)
        {
            features.extend(existing.features.iter().cloned());
            default_enabled = existing.uses_default_features;
            cmds.push(format!(
                "(cargo rm {} {} --package {} || true)",
                self.dep_name, kind_param, self.package.name
            ));
        };

        let features = features
            .difference(&self.features_to_remove)
            .cloned()
            .collect::<HashSet<_>>();
        let mut features = features
            .union(&self.features_to_add)
            .cloned()
            .collect::<HashSet<_>>();

        let default_enabled = !self.features_to_remove.contains("default")
            && (default_enabled || features.contains("default"));
        features.remove("default");

        let features_params = features
            .iter()
            .map(|f| format!("--features {}", f))
            .collect::<Vec<_>>()
            .join(" ");

        let default_feature_param = if default_enabled {
            "--default-features"
        } else {
            "--no-default-features"
        };

        cmds.push(format!(
            "cargo add {} --package {}",
            [
                self.dep_name,
                &features_params,
                default_feature_param,
                kind_param
            ]
            .join(" "),
            self.package.name
        ));

        cmds.join(" && ").to_string()
    }

    fn test_install(&self) -> Result<()> {
        let dir = tempfile::Builder::new()
            .prefix("cargo-oh-my-dependencies")
            .tempdir()?;
        std::fs::create_dir_all(&dir)?;

        dbg!(dir.path().display());

        let cmd = format!("cargo init --name {:?} --bin .", self.package.name);
        debug!("[test_install] {cmd} in {dir:?}");
        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(&dir)
            .status()?;
        if !status.success() {
            eyre::bail!("failed to execute command: {cmd}");
        }

        let mut cloned = self.clone();
        let manifest_path = dir.path().join("Cargo.toml");
        // std::fs::copy(&self.manifest_path, &manifest_path)?;

        // let status = Command::new("cp")
        //     .arg("-r")
        //     .arg(format!(
        //         "{}/*",
        //         &self.manifest_path.parent().unwrap().display()
        //     ))
        //     .arg(dir.path())
        //     // .current_dir(&dir)
        //     .status()?;
        // if !status.success() {
        //     eyre::bail!("failed to execute command");
        // }

        // std::thread::sleep(std::time::Duration::from_secs(30));

        cloned.manifest_path = manifest_path.clone();
        cloned.apply()?;

        println!("{}", std::fs::read_to_string(&manifest_path)?);

        let doc: toml_edit::DocumentMut = std::fs::read_to_string(&manifest_path)?.parse()?;

        if let Err(err) = dir.close() {
            error!("failed to cleanup temp dir: {err}");
        }

        Ok(())
    }

    pub fn apply(self) -> Result<()> {
        let mut backup = if !self.is_test_install {
            debug!("running test install for edit dependency");
            self.for_test_install().test_install()?;

            Some(ManifestBackup::create(&self.manifest_path).context("creating manifest backup")?)
        } else {
            None
        };

        // auto restores when dropped unless disposed
        debug!("creating manifest backup");

        let dir = self
            .manifest_path
            .parent()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| PathBuf::from("."));

        let cmd = self.cmd();

        debug!(?dir, ?cmd, "running command");

        if self.dry_run {
            if let Some(backup) = backup.take() {
                backup.dispose();
            }
            println!("DRY RUN: {cmd:?}");
            return Ok(());
        }

        let status = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(dir)
            .output()?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            let stderr = stderr.trim();
            let stdout = String::from_utf8_lossy(&status.stdout);
            let stdout = stdout.trim();
            let msg = format!("failed to execute command: {cmd}\n {stdout}\n{stderr}");
            error!("{msg}");
            eyre::bail!("{msg}");
        }

        if let Some(backup) = backup.take() {
            backup.dispose();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_gen() {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .exec()
            .expect("could not get metadata");

        let p = metadata.workspace_packages()[0];
        let dep_name = "serde";

        let cmd = EditDependency::new(p, dep_name, DependencyKind::Normal)
            .add_feature("derive")
            .remove_feature("rc")
            .cmd();

        let expected = "(cargo rm serde --package cargo-oh-my-dependencies || true) && cargo add serde --features derive --no-default-features  --package cargo-oh-my-dependencies";
        assert_eq!(expected, cmd);
    }
}
