use anyhow::Result;
use clap;
use env_logger;
use std::convert::TryFrom;
use std::env;

use cardamom::{
    config::{config_arg, Account, Config},
    domain::{card_arg, card_handler},
};

fn create_app<'a>() -> clap::App<'a, 'a> {
    clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .global_setting(clap::AppSettings::GlobalVersion)
        .args(&config_arg::args())
        .subcommands(card_arg::subcmds())
}

fn main() -> Result<()> {
    // Inits env logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "off"),
    );

    let app = create_app();
    let m = app.get_matches();

    // Check completion command BEFORE entities and services initialization.
    // match compl_arg::matches(&m)? {
    //     Some(compl_arg::Command::Generate(shell)) => {
    //         return compl_handler::generate(create_app(), shell);
    //     }
    //     _ => (),
    // }

    // Inits entities and repositories.
    let config = Config::try_from(m.value_of("config"))?;
    let account = Account::try_from((&config, m.value_of("account")))?;

    // Check card commands.
    match card_arg::matches(&m)? {
        Some(card_arg::Cmd::Create(raw_card)) => {
            return card_handler::create(raw_card, &account);
        }
        Some(card_arg::Cmd::Read(id)) => {
            return card_handler::read(id, &account);
        }
        Some(card_arg::Cmd::Update(id, raw_card)) => {
            return card_handler::update(id, raw_card, &account);
        }
        Some(card_arg::Cmd::Delete(id)) => {
            return card_handler::delete(id, &account);
        }
        _ => (),
    }

    Ok(())
}
