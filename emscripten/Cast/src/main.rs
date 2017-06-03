extern crate px8;
#[macro_use]
extern crate log;
extern crate fern;
extern crate time;

use std::sync::{Arc, Mutex};

use px8::frontend;
use px8::gfx;
use px8::px8::RustPlugin;
use px8::config::Players;
use px8::px8::PX8Mode;

fn array_to_vec(arr: &[u8]) -> Vec<u8> {
    let mut vector = Vec::new();
    for i in arr.iter() {
        vector.push(*i);
    }
    vector
}

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}",
                    time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                    level,
                    msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Info) {
        panic!("Failed to initialize global logger: {}", e);
    }

    let mut frontend =
        match frontend::Frontend::init(px8::gfx::Scale::Scale4x, false, false, true) {
            Err(error) => panic!("{:?}", error),
            Ok(frontend) => frontend,
        };

    info!("Register the cast sample");

    let data = include_bytes!("cast.p8");
    let mut data_final: Vec<u8> = array_to_vec(data);

    frontend.start("./sys/config/gamecontrollerdb.txt".to_string());
    frontend.run_cartridge_raw("Cast.p8", data_final, false, PX8Mode::PX8);
}
