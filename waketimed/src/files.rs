use crate::get_config;
use anyhow::{anyhow, Context, Error as AnyError};
use log::trace;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::path::{Path, PathBuf};
use wtd_core::model::{RawVarDef, VarDef};
use wtd_core::VarName;
use zbus::zvariant::{OwnedValue, Value};

pub fn load_var_defs() -> Result<HashMap<VarName, VarDef>, AnyError> {
    let var_def_dirs = get_config().borrow().var_def_dirs();
    let var_def_paths = list_yaml_files_in_dirs(&var_def_dirs)?;
    let mut var_defs = HashMap::new();
    for def_path in var_def_paths.iter() {
        let var_def = parse_var_def(&def_path)?;
        trace!(
            "Loaded var def '{}' from file '{}'.",
            var_def.name.as_ref(),
            def_path.display()
        );
        var_defs.insert(var_def.name.clone(), var_def);
    }
    Ok(var_defs)
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
    let raw_def: RawVarDef = parse_yaml_file(&def_path)?;
    Ok((raw_name, &raw_def).try_into()?)
}

fn parse_yaml_file<T: DeserializeOwned, P: AsRef<Path>>(file_path: P) -> Result<T, AnyError> {
    let reader = File::open(&file_path)
        .with_context(|| format!("Cannot read file '{}'", file_path.as_ref().display(),))?;
    let parsed: T = serde_yaml::from_reader(reader)
        .with_context(|| format!("Cannot parse file '{}'", file_path.as_ref().display(),))?;
    Ok(parsed)
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
