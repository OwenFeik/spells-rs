use std::path::PathBuf;

use crate::{err, eval::Context, value::Value, Res};

const DEFAULT_SAVE_NAME: &str = "untitled";
const SAVE_EXTENSION: &str = ".tome";
const SAVE_NAME_VARIABLE: &str = "SAVE_NAME";

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

fn save_name(index: u32) -> String {
    if index == 0 {
        DEFAULT_SAVE_NAME.to_string()
    } else {
        format!("{DEFAULT_SAVE_NAME}{index}")
    }
}

fn save_file(context: &mut Context) -> Res<PathBuf> {
    let save_dir = data_directory()?;
    let save_name: String = if let Some(name) = context.get_variable(SAVE_NAME_VARIABLE) {
        name.string()?
    } else {
        let mut i = 0;
        while std::fs::try_exists(save_dir.join(format!("{}{}", save_name(i), SAVE_EXTENSION)))
            .map_err(|e| format!("Failed to check if save file exists: {e}"))?
        {
            i += 1;
        }
        save_name(i)
    };

    // Save in context.
    context.set_variable(SAVE_NAME_VARIABLE, Value::String(save_name.clone()));

    let mut save_path = save_dir;
    save_path.push(format!("{}{}", save_name, SAVE_EXTENSION));
    Ok(save_path)
}

pub fn load(context: &mut Context) -> Res<()> {
    let path = save_file(context)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Error loading from {}: {e}", path.display()))?;
    for statement in text.lines() {
        context.eval(statement)?;
    }
    Ok(())
}

pub fn save(context: &mut Context) -> Res<()> {
    let path = save_file(context)?;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }

    std::fs::write(&path, context.dump_to_string())
        .map_err(|e| format!("Error saving at {}: {e}", path.display()))
}
