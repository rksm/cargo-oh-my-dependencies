//! Following https://ratatui.rs/how-to/develop-apps/log-with-tracing/.

use color_eyre::eyre::Result;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use simple_file_rotation::FileRotation;
use std::path::PathBuf;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_PKG_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> = std::env::var(format!("{}_DATA", &*PROJECT_NAME))
        .ok()
        .map(PathBuf::from);
    pub static ref LOG_ENV: String = format!("{}_LOGLEVEL", &*PROJECT_NAME);
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_CRATE_NAME"));
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("hn", "kra", env!("CARGO_PKG_NAME"))
}

pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = &*DATA_FOLDER {
        s.clone()
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}

pub fn initialize_logging() -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(&directory)?;
    let log_path = directory.join(&*LOG_FILE);
    FileRotation::new(&log_path).max_old_files(3).rotate()?;
    let log_file = std::fs::File::create(log_path)?;
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(&*LOG_ENV))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}
