use anyhow::{Context, Error, Result};
use log::{debug, trace};
use serde::Deserialize;
use std::{collections::HashMap, convert::TryFrom, env, fs, path::PathBuf};
use toml;

/// Represents the config file of the user.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(flatten)]
    pub accounts: ConfigAccountsMap,
}

/// Represents the accounts section of the config.
pub type ConfigAccountsMap = HashMap<String, ConfigAccountEntry>;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigAccountEntry {
    Local(LocalConfigAccountEntry),
    Remote(RemoteConfigAccountEntry),
}

/// Represents an account in the accounts section.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfigAccountEntry {
    pub default: Option<bool>,
    pub path: String,
}

/// Represents an account in the accounts section.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteConfigAccountEntry {
    pub default: Option<bool>,

    pub url: String,
    pub login: String,
    pub passwd_cmd: String,
}

impl Config {
    fn path_from_xdg() -> Result<PathBuf> {
        let path = env::var("XDG_CONFIG_HOME")
            .with_context(|| r#"cannot find "XDG_CONFIG_HOME" env var"#)?;
        let mut path = PathBuf::from(path);
        path.push("cardamom");
        path.push("config.toml");
        Ok(path)
    }

    fn path_from_xdg_alt() -> Result<PathBuf> {
        let home_var = if cfg!(target_family = "windows") {
            "USERPROFILE"
        } else {
            "HOME"
        };
        let mut path: PathBuf = env::var(home_var)
            .with_context(|| format!(r#"cannot find "{}" env var"#, home_var))?
            .into();
        path.push(".config");
        path.push("cardamom");
        path.push("config.toml");
        Ok(path)
    }

    fn path_from_home() -> Result<PathBuf> {
        let home_var = if cfg!(target_family = "windows") {
            "USERPROFILE"
        } else {
            "HOME"
        };
        let mut path: PathBuf = env::var(home_var)
            .with_context(|| format!(r#"cannot find "{}" env var"#, home_var))?
            .into();
        path.push(".cardamomrc");
        Ok(path)
    }

    pub fn path() -> Result<PathBuf> {
        let path = Self::path_from_xdg()
            .or_else(|_| Self::path_from_xdg_alt())
            .or_else(|_| Self::path_from_home())
            .with_context(|| "cannot find config path")?;
        Ok(path)
    }
}

impl TryFrom<Option<&str>> for Config {
    type Error = Error;

    fn try_from(path: Option<&str>) -> Result<Self, Self::Error> {
        debug!("init config from `{:?}`", path);
        let path = path.map(|s| s.into()).unwrap_or(Config::path()?);
        let content = fs::read_to_string(path).context("cannot read config file")?;
        let config = toml::from_str(&content).context("cannot parse config file")?;
        trace!("{:#?}", config);
        Ok(config)
    }
}
