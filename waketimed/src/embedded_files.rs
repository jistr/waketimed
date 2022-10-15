use anyhow::{anyhow, Error as AnyError};
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub const PREFIX: &str = "/__WAKETIMED_EMBEDDED__";
pub const PREFIX_RULE_DEF: &str = "/__WAKETIMED_EMBEDDED__/rule_def";
pub const PREFIX_VAR_DEF: &str = "/__WAKETIMED_EMBEDDED__/var_def";

pub const EXISTING_DIRS: &[&str] = &[PREFIX_RULE_DEF, PREFIX_VAR_DEF];

#[derive(RustEmbed)]
#[folder = "embed/rule_def/"]
#[prefix = "/__WAKETIMED_EMBEDDED__/rule_def/"]
struct EmbeddedRuleDefs;

#[derive(RustEmbed)]
#[folder = "embed/var_def/"]
#[prefix = "/__WAKETIMED_EMBEDDED__/var_def/"]
struct EmbeddedVarDefs;

pub fn embedded_dir_entry_paths<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>, AnyError> {
    let dir_ref: &Path = dir.as_ref();
    let filenames: Vec<Cow<'static, str>> = match &dir_ref.to_string_lossy() as &str {
        PREFIX_RULE_DEF => EmbeddedRuleDefs::iter().collect(),
        PREFIX_VAR_DEF => EmbeddedVarDefs::iter().collect(),
        _ => {
            return Err(anyhow!(
                "Unknown embedded directory '{}'.",
                dir_ref.display()
            ))
        }
    };
    Ok(filenames
        .into_iter()
        .map(|filename| dir_ref.join(&*filename))
        .collect())
}

pub fn embedded_file_data<P: AsRef<Path>>(path: P) -> Result<Cow<'static, [u8]>, AnyError> {
    let path_ref: &Path = path.as_ref();
    let path_str: &str = &*path.as_ref().to_string_lossy();
    let data: Option<Cow<'static, [u8]>> = if path_ref.starts_with(PREFIX_RULE_DEF) {
        EmbeddedRuleDefs::get(path_str).map(|file| file.data)
    } else if path_ref.starts_with(PREFIX_VAR_DEF) {
        EmbeddedVarDefs::get(path_str).map(|file| file.data)
    } else {
        return Err(anyhow!(
            "Unknown embedded file path '{}'.",
            path_ref.display()
        ));
    };
    data.ok_or_else(|| anyhow!("Embedded file '{}' not found.", path_ref.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_def_entries() -> Result<(), AnyError> {
        let entries: Vec<String> = embedded_dir_entry_paths(PREFIX_RULE_DEF)?
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        let mut ref_entries: Vec<&str> = entries.iter().map(|path| path.as_str()).collect();
        ref_entries.sort_unstable();
        assert_eq!(
            &ref_entries,
            &[
                "/__WAKETIMED_EMBEDDED__/rule_def/wtd_call_present.yaml",
                "/__WAKETIMED_EMBEDDED__/rule_def/wtd_sleep_block_inhibited.yaml",
                "/__WAKETIMED_EMBEDDED__/rule_def/wtd_user_busy.yaml"
            ],
        );
        Ok(())
    }

    #[test]
    fn test_var_def_entries() -> Result<(), AnyError> {
        let entries: Vec<String> = embedded_dir_entry_paths(PREFIX_VAR_DEF)?
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        let mut ref_entries: Vec<&str> = entries.iter().map(|path| path.as_str()).collect();
        ref_entries.sort_unstable();
        assert_eq!(
            &ref_entries,
            &[
                "/__WAKETIMED_EMBEDDED__/var_def/wtd_call_present.yaml",
                "/__WAKETIMED_EMBEDDED__/var_def/wtd_login_seat_busy.yaml",
                "/__WAKETIMED_EMBEDDED__/var_def/wtd_modem_voice_call_present.yaml",
                "/__WAKETIMED_EMBEDDED__/var_def/wtd_sleep_block_inhibited.yaml",
                "/__WAKETIMED_EMBEDDED__/var_def/wtd_user_busy.yaml",
            ],
        );
        Ok(())
    }

    #[test]
    fn test_embedded_file_data() -> Result<(), AnyError> {
        let data = embedded_file_data("/__WAKETIMED_EMBEDDED__/rule_def/wtd_user_busy.yaml")?;
        let expected_data = include_bytes!("../embed/rule_def/wtd_user_busy.yaml");
        assert_eq!(&*data, expected_data);
        Ok(())
    }

    #[test]
    fn test_embedded_file_data_error() -> Result<(), AnyError> {
        let data = embedded_file_data("/__WAKETIMED_EMBEDDED__/rule_def/nonexistent.yaml");
        assert!(data.is_err());
        Ok(())
    }
}
