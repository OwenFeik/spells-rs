use std::path::PathBuf;

use crate::{err, eval::Context, Res};

fn get_env_path(var: &str) -> Res<PathBuf> {
    let Some(val) = std::env::var_os(var) else {
        return Err(format!("Failed to get environment variable {var}."));
    };

    if val.is_empty() {
        Err(format!("{var} is an empty string."))
    } else {
        Ok(PathBuf::from(val))
    }
}

fn data_directory() -> Res<PathBuf> {
    const APP_NAME: &str = "spells";
    const HOME_ENV_VARS: &[(&str, &[&str])] = &[
        #[cfg(target_os = "windows")]
        ("LOCALAPPDATA", &[APP_NAME]),
        #[cfg(target_os = "linux")]
        ("XDG_DATA_HOME", &[APP_NAME]),
        ("HOME", &[".local", "share", APP_NAME]),
    ];

    for (var, rel) in HOME_ENV_VARS {
        let res = get_env_path(var);
        if let Ok(mut path) = res {
            for segment in *rel {
                path.push(segment);
            }
            return Ok(path);
        }
    }
    err("$HOME not defined. Unsure where to save.")
}

const SAVE_FILE: &str = "save.tome";

pub fn load(context: &mut Context) -> Res<()> {
    let mut path = data_directory()?;
    path.push(SAVE_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Error loading from {}: {e}", path.display()))?;
    for statement in text.lines() {
        context.eval(statement)?;
    }
    Ok(())
}

pub fn save(context: &Context) -> Res<()> {
    let mut path = data_directory()?;
    std::fs::create_dir_all(&path).ok();
    path.push(SAVE_FILE);

    std::fs::write(&path, context.dump_to_string())
        .map_err(|e| format!("Error saving at {}: {e}", path.display()))
}
