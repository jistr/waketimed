use crate::config::Config;
use crate::embedded_files;
use anyhow::{anyhow, Context, Error as AnyError};
use log::debug;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{File, ReadDir};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use wtd_core::rules::{RuleDef, RuleName};
use wtd_core::vars::{VarDef, VarName};

pub fn load_rule_defs(cfg: &Config) -> Result<HashMap<RuleName, RuleDef>, AnyError> {
    let rule_def_dirs = cfg.rule_def_dirs();
    let rule_def_dirs_existing = into_existing_dirs(rule_def_dirs)?;
    debug!("Using rule_def directories: {:?}.", &rule_def_dirs_existing);
    let rule_def_paths = list_yaml_files_in_dirs(&rule_def_dirs_existing)?;
    let (unique_paths, overriden_paths) = unique_and_overriden_paths_from(rule_def_paths)?;
    if !overriden_paths.is_empty() {
        debug!("Overriden rule def paths: {:?}", overriden_paths);
    }
    let mut rule_defs = HashMap::new();
    for def_path in unique_paths.iter() {
        debug!("Loading rule def '{}'.", def_path.display());
        let rule_def = parse_rule_def(&def_path)?;
        match rule_def {
            Some(def) => {
                rule_defs.insert(def.name().clone(), def);
            }
            None => {
                debug!("Rule def '{}' is void.", def_path.display());
            }
        }
    }
    Ok(rule_defs)
}

pub fn load_var_defs(cfg: &Config) -> Result<HashMap<VarName, VarDef>, AnyError> {
    let var_def_dirs = cfg.var_def_dirs();
    let var_def_dirs_existing = into_existing_dirs(var_def_dirs)?;
    debug!("Using var_def directories: {:?}.", &var_def_dirs_existing);
    let var_def_paths = list_yaml_files_in_dirs(&var_def_dirs_existing)?;
    let (unique_paths, overriden_paths) = unique_and_overriden_paths_from(var_def_paths)?;
    if !overriden_paths.is_empty() {
        debug!("Overriden var def paths: {:?}", overriden_paths);
    }
    let mut var_defs = HashMap::new();
    for def_path in unique_paths.iter() {
        debug!("Loading var def '{}'.", def_path.display());
        let var_def = parse_var_def(&def_path)?;
        match var_def {
            Some(def) => {
                var_defs.insert(def.name().clone(), def);
            }
            None => {
                debug!("Var def '{}' is void.", def_path.display());
            }
        }
    }
    Ok(var_defs)
}

fn parse_rule_def<P: AsRef<Path>>(def_path: P) -> Result<Option<RuleDef>, AnyError> {
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
    let rule_name = RuleName::try_from(raw_name.to_string())?;
    let rule_def: Option<RuleDef> = parse_yaml_file_unless_empty(&def_path)?;
    let rule_def = rule_def.map(|mut def| {
        def.name = Some(rule_name);
        def
    });
    Ok(rule_def)
}

fn parse_var_def<P: AsRef<Path>>(def_path: P) -> Result<Option<VarDef>, AnyError> {
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
    let var_name = VarName::try_from(raw_name.to_string())?;
    let var_def: Option<VarDef> = parse_yaml_file_unless_empty(&def_path)?;
    let var_def = var_def.map(|mut def| {
        def.name = Some(var_name);
        def
    });
    Ok(var_def)
}

fn parse_yaml_file_unless_empty<T: DeserializeOwned, P: AsRef<Path>>(
    file_path: P,
) -> Result<Option<T>, AnyError> {
    if file_path.as_ref().starts_with(embedded_files::PREFIX) {
        parse_embedded_yaml_file_unless_empty(file_path)
    } else {
        parse_real_yaml_file_unless_empty(file_path)
    }
}

fn parse_embedded_yaml_file_unless_empty<T: DeserializeOwned, P: AsRef<Path>>(
    file_path: P,
) -> Result<Option<T>, AnyError> {
    let data = embedded_files::embedded_file_data(file_path.as_ref())?;
    if data.len() == 0 {
        return Ok(None);
    }

    let data_string = String::from_utf8_lossy(&*data);
    let parsed: T = serde_yaml::from_str(&data_string)
        .with_context(|| format!("Cannot parse file '{}'", file_path.as_ref().display(),))?;
    Ok(Some(parsed))
}

fn parse_real_yaml_file_unless_empty<T: DeserializeOwned, P: AsRef<Path>>(
    file_path: P,
) -> Result<Option<T>, AnyError> {
    let mut reader = BufReader::new(
        File::open(&file_path)
            .with_context(|| format!("Cannot read file '{}'", file_path.as_ref().display(),))?,
    );
    if reader.fill_buf()?.is_empty() {
        // if file is empty
        return Ok(None);
    }
    let parsed: T = serde_yaml::from_reader(reader)
        .with_context(|| format!("Cannot parse file '{}'", file_path.as_ref().display(),))?;
    Ok(Some(parsed))
}

fn into_existing_dirs(dirs: Vec<PathBuf>) -> Result<Vec<PathBuf>, AnyError> {
    let mut existing_dirs = Vec::new();
    for dir in dirs.into_iter() {
        if embedded_files::EXISTING_DIRS.contains(&&*dir.to_string_lossy()) {
            existing_dirs.push(dir);
            continue;
        }

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
    let all_paths = dirs_entry_paths(dirs)?;
    let yaml_paths: Result<Vec<PathBuf>, AnyError> = all_paths
        .into_iter()
        .filter_map(|path| {
            if path.extension() == Some("yaml".as_ref()) {
                Some(Ok(path))
            } else {
                None
            }
        })
        .collect();
    yaml_paths
}

fn dirs_entry_paths<P: AsRef<Path>>(dirs: &[P]) -> Result<Vec<PathBuf>, AnyError> {
    let entries: Result<Vec<Vec<PathBuf>>, AnyError> = dirs
        .iter()
        .map(|d| {
            let dir_path = d.as_ref();
            if dir_path.starts_with(embedded_files::PREFIX) {
                embedded_files::embedded_dir_entry_paths(d)
            } else {
                real_dir_entry_paths(d)
            }
        })
        .collect();
    Ok(entries?.into_iter().flatten().collect())
}

fn real_dir_entry_paths<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>, AnyError> {
    let entries: ReadDir = dir
        .as_ref()
        .read_dir()
        .with_context(|| format!("Failed to read directory '{}'", dir.as_ref().display()))?;
    entries
        .into_iter()
        .map(|entry_res| {
            entry_res.map(|e| e.path()).with_context(|| {
                format!(
                    "Failed list files in directory '{}'",
                    dir.as_ref().display()
                )
            })
        })
        .collect()
}

// Consumes a vec of paths, returns two vecs. First vec contains all
// paths where file names are non-unique. Last occurrence of a certain
// file name stays in the path list, previous occurrences are removed.
// The second vec contains all removed paths are returned as a new vectore.
fn unique_and_overriden_paths_from(
    paths: Vec<PathBuf>,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), AnyError> {
    let mut unique_paths: Vec<PathBuf> = Vec::new();
    let mut overriden_paths: Vec<PathBuf> = Vec::new();
    let mut used_file_names: HashSet<OsString> = HashSet::new();
    for path in paths.into_iter().rev() {
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("No file name in '{}'", path.display()))?;
        if used_file_names.insert(file_name.to_os_string()) {
            unique_paths.push(path);
        } else {
            overriden_paths.push(path);
        }
    }
    let unique_paths_original_order = unique_paths.into_iter().rev().collect();
    Ok((unique_paths_original_order, overriden_paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_and_overriden_paths() -> Result<(), AnyError> {
        let paths = vec![
            PathBuf::from("/original/overriden_variable_def.yaml"),
            PathBuf::from("/original/not_overriden_variable_def.yaml"),
            PathBuf::from("/overrides/overriden_variable_def.yaml"),
            PathBuf::from("/overrides/another_variable_def.yaml"),
        ];
        let unique_expected = vec![
            PathBuf::from("/original/not_overriden_variable_def.yaml"),
            PathBuf::from("/overrides/overriden_variable_def.yaml"),
            PathBuf::from("/overrides/another_variable_def.yaml"),
        ];
        let overriden_expected = vec![PathBuf::from("/original/overriden_variable_def.yaml")];
        let (unique, overriden) = unique_and_overriden_paths_from(paths)?;
        assert_eq!(unique, unique_expected);
        assert_eq!(overriden, overriden_expected);
        Ok(())
    }
}
