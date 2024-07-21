use std::path::PathBuf;

use crate::{context::Context, err, eval_tome, Res};

pub const SAVE_PATH_VAR: &str = "SAVE_PATH";
const DEFAULT_SAVE_NAME: &str = "untitled";
const SAVE_EXTENSION: &str = ".tome";

pub enum SaveTarget {
    Generate,
    Title(String),
    Path(PathBuf),
}

impl SaveTarget {
    pub fn from<S: ToString>(target: S) -> Self {
        let string = target.to_string();
        if string.contains('.')
            || string.contains('/')
            || string.contains(std::path::MAIN_SEPARATOR)
        {
            SaveTarget::Path(PathBuf::from(string))
        } else {
            SaveTarget::Title(string)
        }
    }
}

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

fn save_file(title: Option<String>) -> Res<PathBuf> {
    let save_dir = data_directory()?;
    let save_name: String = if let Some(name) = title {
        name
    } else {
        let mut i = 0;
        while std::fs::try_exists(save_dir.join(format!("{}{}", save_name(i), SAVE_EXTENSION)))
            .map_err(|e| format!("Failed to check if save file exists: {e}"))?
        {
            i += 1;
        }
        save_name(i)
    };

    let mut save_path = save_dir;
    save_path.push(format!("{}{}", save_name, SAVE_EXTENSION));
    Ok(save_path)
}

fn normalise_to_path(target: SaveTarget) -> Res<PathBuf> {
    match target {
        SaveTarget::Generate => save_file(None),
        SaveTarget::Title(title) => save_file(Some(title)),
        SaveTarget::Path(path) => Ok(path),
    }
}

pub fn load(at: SaveTarget) -> Res<(Context, String)> {
    let path = normalise_to_path(at)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Error loading from {}: {e}", path.display()))?;
    let mut context = Context::empty();
    eval_tome(&text, &mut context)?;
    Ok((context, path.display().to_string()))
}

pub fn save(at: SaveTarget, context: &Context) -> Res<String> {
    let path = normalise_to_path(at)?;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }

    std::fs::write(&path, context.dump_to_string()?)
        .map_err(|e| format!("Error saving at {}: {e}", path.display()))
        .map_err(|e| e.to_string())?;

    Ok(path.display().to_string())
}
