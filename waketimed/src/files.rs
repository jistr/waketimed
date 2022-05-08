use crate::get_config;
use anyhow::{anyhow, Context, Error as AnyError};
use log::debug;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::path::{Path, PathBuf};
use wtd_core::rules::{RuleDef, RuleName};
use wtd_core::vars::{VarDef, VarName};

pub fn load_rule_defs() -> Result<HashMap<RuleName, RuleDef>, AnyError> {
    let rule_def_dirs = get_config().borrow().rule_def_dirs();
    let rule_def_dirs_existing = into_existing_dirs(rule_def_dirs)?;
    debug!("Using rule_def directories: {:?}.", &rule_def_dirs_existing);
    let rule_def_paths = list_yaml_files_in_dirs(&rule_def_dirs_existing)?;
    let mut rule_defs = HashMap::new();
    for def_path in rule_def_paths.iter() {
        debug!("Loading rule def '{}'.", def_path.display());
        let rule_def = parse_rule_def(&def_path)?;
        rule_defs.insert(rule_def.name().clone(), rule_def);
    }
    Ok(rule_defs)
}

pub fn load_var_defs() -> Result<HashMap<VarName, VarDef>, AnyError> {
    let var_def_dirs = get_config().borrow().var_def_dirs();
    let var_def_dirs_existing = into_existing_dirs(var_def_dirs)?;
    debug!("Using var_def directories: {:?}.", &var_def_dirs_existing);
    let var_def_paths = list_yaml_files_in_dirs(&var_def_dirs_existing)?;
    let mut var_defs = HashMap::new();
    for def_path in var_def_paths.iter() {
        debug!("Loading var def '{}'.", def_path.display());
        let var_def = parse_var_def(&def_path)?;
        var_defs.insert(var_def.name().clone(), var_def);
    }
    Ok(var_defs)
}

fn parse_rule_def<P: AsRef<Path>>(def_path: P) -> Result<RuleDef, AnyError> {
    let raw_name = def_path
        .as_ref()
        .file_stem()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| {
            anyhow!(
                "Could not get rule name from path: '{}'",
                def_path.as_ref().display()
            )
        })?;
    let mut rule_def: RuleDef = parse_yaml_file(&def_path)?;
    rule_def.name = Some(RuleName::try_from(raw_name.to_string())?);
    Ok(rule_def)
}

fn parse_var_def<P: AsRef<Path>>(def_path: P) -> Result<VarDef, AnyError> {
    let raw_name = def_path
        .as_ref()
        .file_stem()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| {
            anyhow!(
                "Could not get var name from path: '{}'",
                def_path.as_ref().display()
            )
        })?;
    let mut var_def: VarDef = parse_yaml_file(&def_path)?;
    var_def.name = Some(VarName::try_from(raw_name.to_string())?);
    Ok(var_def)
}

fn parse_yaml_file<T: DeserializeOwned, P: AsRef<Path>>(file_path: P) -> Result<T, AnyError> {
    let reader = File::open(&file_path)
        .with_context(|| format!("Cannot read file '{}'", file_path.as_ref().display(),))?;
    let parsed: T = serde_yaml::from_reader(reader)
        .with_context(|| format!("Cannot parse file '{}'", file_path.as_ref().display(),))?;
    Ok(parsed)
}

fn into_existing_dirs(dirs: Vec<PathBuf>) -> Result<Vec<PathBuf>, AnyError> {
    let mut existing_dirs = Vec::new();
    for dir in dirs.into_iter() {
        // NOTE: When Path.try_exists is stabilized, it can be used instead.
        let read_dir = dir.as_path().read_dir();
        let exists = match read_dir {
            Ok(_) => true,
            Err(e) => {
                if matches!(e.kind(), std::io::ErrorKind::NotFound) {
                    false
                } else {
                    return Err(e).with_context(|| {
                        format!(
                            "Error trying to read directory '{}'.",
                            dir.as_path().display()
                        )
                    });
                }
            }
        };
        if exists {
            existing_dirs.push(dir);
        }
    }
    Ok(existing_dirs)
}

fn list_yaml_files_in_dirs<P: AsRef<Path>>(dirs: &[P]) -> Result<Vec<PathBuf>, AnyError> {
    let read_dirs: Result<Vec<ReadDir>, AnyError> = dirs
        .iter()
        .map(|d| {
            d.as_ref()
                .read_dir()
                .with_context(|| format!("Failed to read directory '{}'", d.as_ref().display()))
        })
        .collect();
    let all_entries = read_dirs?.into_iter().flatten();
    let yaml_paths: Result<Vec<PathBuf>, AnyError> = all_entries
        .filter_map(|res_entry| match res_entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension() == Some("yaml".as_ref()) {
                    Some(Ok(path))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(AnyError::from(e))),
        })
        .collect();
    yaml_paths
}
