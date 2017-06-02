extern crate px8;
extern crate sdl2;
extern crate time;
extern crate rand;
#[macro_use]
extern crate log;
extern crate fern;
extern crate nalgebra;

use std::sync::{Arc, Mutex};

use px8::px8::math;
use px8::frontend;
use px8::gfx;
use px8::cartridge;
use px8::px8::RustPlugin;
use px8::config::Players;
use nalgebra::Vector2;


pub struct Fighter {
    pub pos: Vector2<u32>,
}

impl Fighter {
    fn new(x: u32, y: u32) -> Fighter {
        Fighter {
            pos: Vector2::new(x, y)
        }
    }

    fn update(&mut self) {

    }

    fn draw(&mut self, screen: Arc<Mutex<gfx::Screen>>) {
        screen.lock().unwrap().pset(self.pos.x as i32, self.pos.y as i32, 8);
    }
}

pub struct FourmisWar {
    pub sprite_filename: String,
    pub fighters: Vec<Fighter>,
}

impl FourmisWar {
    pub fn new(sprite_filename: String) -> FourmisWar {
        FourmisWar {
            sprite_filename: sprite_filename,
            fighters: Vec::new(),
        }
    }
}

impl RustPlugin for FourmisWar {
    fn init(&mut self, screen: Arc<Mutex<gfx::Screen>>) -> f64 {
        self.fighters.push(Fighter::new(40, 50));

        match cartridge::Cartridge::parse(self.sprite_filename.clone(), false) {
            Ok(c) => screen.lock().unwrap().set_sprites(c.gfx.sprites),
            Err(e) => panic!("Impossible to load the assets {:?}", e),
        }

        return 0.;
    }

    fn update(&mut self, players: Arc<Mutex<Players>>) -> f64 {

        let mouse_x = players.lock().unwrap().mouse_coordinate(0);
        let mouse_y = players.lock().unwrap().mouse_coordinate(1);

        if players.lock().unwrap().mouse_state() == 1 {
            info!("CLICK {:?} {:?}", mouse_x, mouse_y);
        }

        for fighter in self.fighters.iter_mut() {
            fighter.update();
        }
        return 0.;
    }

    fn draw(&mut self, screen: Arc<Mutex<gfx::Screen>>) -> f64 {
        screen.lock().unwrap().cls();

        for fighter in self.fighters.iter_mut() {
            fighter.draw(screen.clone());
        }

        return 0.;
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
        output: vec![fern::OutputConfig::stdout(),
                     fern::OutputConfig::file("output.log")],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Info) {
        panic!("Failed to initialize global logger: {}", e);
    }


    let war = FourmisWar::new("./fourmiswar.dpx8".to_string());

    let mut frontend = match frontend::Frontend::init(px8::gfx::Scale::Scale4x, false, true, true) {
        Err(error) => panic!("{:?}", error),
        Ok(frontend) => frontend,
    };

    frontend.px8.register(war);
    frontend.start("./gamecontrollerdb.txt".to_string());
    frontend.run_native_cartridge();
}
