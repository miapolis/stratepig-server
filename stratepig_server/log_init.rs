use simplelog::*;
use std::fs::File;

pub fn init() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Stdout,
            ColorChoice::Always,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("stratepig-server.log").unwrap(),
        ),
    ])
    .unwrap();
}
