use anyhow::{anyhow, Context, Error, Result};
use log::{debug, trace};
use std::{convert::TryFrom, process::Command};

use crate::config::{Config, ConfigAccountEntry};

/// Represents a user account.
#[derive(Debug)]
pub enum Account {
    Local(LocalAccount),
    Remote(RemoteAccount),
}

/// Represents a local user account.
#[derive(Debug, Default)]
pub struct LocalAccount {
    pub name: String,
    pub path: String,
}

/// Represents a remote user account.
#[derive(Debug, Default)]
pub struct RemoteAccount {
    pub name: String,
    pub url: String,
    pub login: String,
    pub passwd_cmd: String,
}

impl RemoteAccount {
    pub fn passwd(&self) -> Result<String> {
        let passwd = run_cmd(&self.passwd_cmd).context("cannot run passwd cmd")?;
        let passwd = passwd
            .trim_end_matches(|c| c == '\r' || c == '\n')
            .to_owned();
        Ok(passwd)
    }
}

impl<'a> TryFrom<(&'a Config, Option<&str>)> for Account {
    type Error = Error;

    fn try_from((config, account_name): (&'a Config, Option<&str>)) -> Result<Self, Self::Error> {
        debug!(r#"init account "{}""#, account_name.unwrap_or("default"));

        let (name, entry) = match account_name.map(|name| name.trim()) {
            Some("default") | Some("") | None => config
                .accounts
                .iter()
                .find(|(_, entry)| match entry {
                    ConfigAccountEntry::Local(entry) => entry.default.unwrap_or_default(),
                    ConfigAccountEntry::Remote(entry) => entry.default.unwrap_or_default(),
                })
                .map(|(name, account)| (name.to_owned(), account))
                .ok_or_else(|| anyhow!("cannot find default account")),
            Some(name) => config
                .accounts
                .get(name)
                .map(|account| (name.to_owned(), account))
                .ok_or_else(|| anyhow!(r#"cannot find account "{}""#, name)),
        }?;

        let account = match entry {
            ConfigAccountEntry::Local(entry) => Account::Local(LocalAccount {
                name,
                path: entry.path.clone(),
            }),
            ConfigAccountEntry::Remote(entry) => Account::Remote(RemoteAccount {
                name,
                url: entry.url.clone(),
                login: entry.login.clone(),
                passwd_cmd: entry.passwd_cmd.clone(),
            }),
        };
        trace!("account: {:#?}", account);
        Ok(account)
    }
}

// TODO: move me
pub fn run_cmd(cmd: &str) -> Result<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", cmd]).output()
    } else {
        Command::new("sh").arg("-c").arg(cmd).output()
    }?;

    Ok(String::from_utf8(output.stdout)?)
}
