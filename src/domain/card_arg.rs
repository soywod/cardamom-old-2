//! Card CLI module.
//!
//! This module provides subcommands, arguments and a command matcher related to the card
//! domain.

use anyhow::Result;
use clap;
use log::{debug, trace};

type Id<'a> = &'a str;
type RawCard<'a> = &'a str;

/// Represents the card commands.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd<'a> {
    /// Represents the create card command.
    Create(RawCard<'a>),
    /// Represents the read card command.
    Read(Id<'a>),
    /// Represents the update card command.
    Update(Id<'a>, RawCard<'a>),
    /// Represents the delete card command.
    Delete(Id<'a>),
}

/// Defines the card command matcher.
pub fn matches<'a>(m: &'a clap::ArgMatches) -> Result<Option<Cmd<'a>>> {
    if let Some(m) = m.subcommand_matches("create") {
        debug!("create subcommand matched");
        let card = m.value_of("card").unwrap_or_default();
        trace!("card: {}", card);
        return Ok(Some(Cmd::Create(card)));
    }

    if let Some(m) = m.subcommand_matches("read") {
        debug!("read subcommand matched");
        let id = m.value_of("id").unwrap();
        trace!("id: {}", id);
        return Ok(Some(Cmd::Read(id)));
    }

    if let Some(m) = m.subcommand_matches("update") {
        debug!("update subcommand matched");
        let id = m.value_of("id").unwrap();
        trace!("id: {}", id);
        let card = m.value_of("card").unwrap_or_default();
        trace!("card: {}", card);
        return Ok(Some(Cmd::Update(id, card)));
    }

    if let Some(m) = m.subcommand_matches("delete") {
        debug!("delete subcommand matched");
        let id = m.value_of("id").unwrap();
        trace!("id: {}", id);
        return Ok(Some(Cmd::Delete(id)));
    }

    Ok(None)
}

/// Contains card subcommands.
pub fn subcmds<'a>() -> Vec<clap::App<'a, 'a>> {
    vec![
        clap::SubCommand::with_name("create")
            .aliases(&["c"])
            .about("Creates a new card")
            .arg(raw_card_arg()),
        clap::SubCommand::with_name("read")
            .aliases(&["r"])
            .about("Reads a card")
            .arg(id_arg()),
        clap::SubCommand::with_name("update")
            .aliases(&["up", "u"])
            .about("Updates a card")
            .arg(id_arg())
            .arg(raw_card_arg()),
        clap::SubCommand::with_name("delete")
            .aliases(&["del", "d"])
            .about("Deletes a card")
            .arg(id_arg()),
    ]
}

/// Defines the raw card argument.
pub fn raw_card_arg<'a>() -> clap::Arg<'a, 'a> {
    clap::Arg::with_name("card").raw(true).last(true)
}

/// Defines the card id argument.
pub fn id_arg<'a>() -> clap::Arg<'a, 'a> {
    clap::Arg::with_name("id")
        .help("Specifies the card id")
        .value_name("ID")
        .required(true)
}
