use clap::{App, Arg};
use log::info;
use std::default;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub one_player: bool,
    pub swift_game_enter: bool,
    pub ignore_turns: bool,
    pub log_packet_errors: bool,
}

impl Config {
    pub fn new() -> Self {
        let version = env!("CARGO_PKG_VERSION");
        let authors = env!("CARGO_PKG_AUTHORS");

        let args =
            App::new("Stratepig Server")
                .version(version)
                .author(authors)
                .arg(
                    Arg::with_name("ONE_PLAYER").short("p").help(
                        "If specified, only one player will be required in lobby and in game and turns will be disabled",
                    ),
                )
                .arg(Arg::with_name("SWIFT_GAME_ENTER").short("s").help(
                    "If specified, upon host the player will be sent into a game immediately",
                ))
                .arg(
                    Arg::with_name("IGNORE_TURNS")
                        .short("t")
                        .help("If specified, turns will not be used in game"),
                )
                .arg(
                    Arg::with_name("LOG_PACKET_ERRORS")
                    .short("e")
                    .help("If specified, packets that return an error will be logged")
                )
                .get_matches();

        let one_player = args.is_present("ONE_PLAYER");
        let swift_game_enter = args.is_present("SWIFT_GAME_ENTER");
        let mut ignore_turns = args.is_present("IGNORE_TURNS");
        let log_packet_errors = args.is_present("LOG_PACKET_ERRORS");

        if one_player {
            ignore_turns = true;
        }

        Self {
            one_player,
            swift_game_enter,
            ignore_turns,
            log_packet_errors,
        }
    }

    pub fn log(&self) {
        info!("[Config]");

        let mut default = false;
        if self == &Config::default() {
            default = true;
        }

        info!("Default: {}", default);
        info!("| ONE_PLAYER: {}", self.one_player);
        info!("| SWIFT_GAME_ENTER: {}", self.swift_game_enter);
        info!("| IGNORE_TURNS: {}", self.ignore_turns);
        info!("| LOG_PACKET_ERRORS: {}", self.log_packet_errors);
    }
}

impl default::Default for Config {
    fn default() -> Self {
        Self {
            one_player: false,
            swift_game_enter: false,
            ignore_turns: false,
            log_packet_errors: false,
        }
    }
}
