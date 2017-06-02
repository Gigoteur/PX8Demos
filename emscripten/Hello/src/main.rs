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

pub struct HelloWorld {
    pub t: i32,
}

impl HelloWorld {
    pub fn new() -> HelloWorld {
        HelloWorld {
            t: 0,
        }
    }
}

impl RustPlugin for HelloWorld {
    fn init(&mut self, _screen: Arc<Mutex<gfx::Screen>>) -> f64 {
        0.0
    }

    fn update(&mut self, _players: Arc<Mutex<Players>>) -> f64 {
        debug!("HelloWorld update");

        self.t += 1;

        0.0
    }

    fn draw(&mut self, screen: Arc<Mutex<gfx::Screen>>) -> f64 {
        debug!("HelloWorld draw");

        screen.lock().unwrap().cls();
        screen.lock().unwrap().print("Hello World".to_string(), 40, 64, 7);

        0.0
    }
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

    let helloworld_example = HelloWorld::new();

    let mut frontend =
        match frontend::Frontend::init(px8::gfx::Scale::Scale4x, false, false, true) {
            Err(error) => panic!("{:?}", error),
            Ok(frontend) => frontend,
        };

    info!("Register the sample");

    frontend.px8.register(helloworld_example);
    frontend.start("./sys/config/gamecontrollerdb.txt".to_string());
    frontend.run_native_cartridge();
}
