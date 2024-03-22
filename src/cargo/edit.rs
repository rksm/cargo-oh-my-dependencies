use cargo_metadata::{DependencyKind, Package};
use eyre::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct EditDependency<'a> {
    pub package: &'a Package,
    pub dep_name: &'a str,
    pub features_to_add: HashSet<String>,
    pub features_to_remove: HashSet<String>,
    pub kind: DependencyKind,
}

impl<'a> EditDependency<'a> {
    pub fn new(package: &'a Package, dep_name: &'a str) -> Self {
        let kind = package
            .dependencies
            .iter()
            .find(|dep| dep.name == dep_name)
            .map(|dep| dep.kind)
            .unwrap_or(DependencyKind::Normal);
        Self {
            package,
            dep_name,
            features_to_add: Default::default(),
            features_to_remove: Default::default(),
            kind,
        }
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
    pub fn add_feature(mut self, feature: impl Into<String>) -> Self {
        self.features_to_add.insert(feature.into());
        self
    }

    #[must_use]
    pub fn remove_feature(mut self, feature: impl Into<String>) -> Self {
        self.features_to_remove.insert(feature.into());
        self
    }

    #[must_use]
    pub fn set_kind(mut self, kind: DependencyKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub fn build_dep(self) -> Self {
        self.set_kind(DependencyKind::Build)
    }

    #[must_use]
    pub fn dev_dep(self) -> Self {
        self.set_kind(DependencyKind::Development)
    }

    #[must_use]
    pub fn normal_dep(self) -> Self {
        self.set_kind(DependencyKind::Normal)
    }

    fn cmd(&self) -> String {
        let mut cmds = Vec::new();
        let mut features = HashSet::new();

        let kind_param = match self.kind {
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

    pub fn apply(self) -> Result<()> {
        let dir = self
            .package
            .manifest_path
            .clone()
            .into_std_path_buf()
            .parent()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| PathBuf::from("."));

        let cmd = self.cmd();

        debug!(?self, ?dir, ?cmd);

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

        let cmd = EditDependency::new(p, dep_name)
            .add_feature("derive")
            .remove_feature("rc")
            .cmd();

        let expected = "(cargo rm serde --package cargo-oh-my-dependencies || true) && cargo add serde --features derive --no-default-features  --package cargo-oh-my-dependencies";
        assert_eq!(expected, cmd);
    }
}
