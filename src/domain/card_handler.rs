//! Card handling module.
//!
//! This module gathers all card actions triggered by the CLI.

use anyhow::Result;

use crate::config::Account;

/// Creates a card.
pub fn create(_raw_card: &str, _account: &Account) -> Result<()> {
    todo!();
}

/// Reads a card.
pub fn read(_id: &str, _account: &Account) -> Result<()> {
    todo!();
}

/// Updates a card.
pub fn update(_id: &str, _raw_card: &str, _account: &Account) -> Result<()> {
    todo!();
}

/// Deletes a card.
pub fn delete(_id: &str, _account: &Account) -> Result<()> {
    todo!();
}
