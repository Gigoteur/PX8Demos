#![feature(io)]

extern crate px8;
extern crate sdl2;
extern crate time;
extern crate rand;
#[macro_use]
extern crate chan;

use std::sync::{Arc, Mutex};
use sdl2::Sdl;
use sdl2::EventPump;
use sdl2::VideoSubsystem;
use std::time::{Duration, Instant};
use std::thread;
use sdl2::event::{Event, WindowEvent};

use std::error::Error;
use std::path::Path;

use sdl2::controller::{Axis, Button};
use sdl2::keyboard::Keycode;

use chan::{Receiver, Sender};

use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io;
use std::io::BufRead;
use std::io::Read;

use std::io::prelude::*;

#[macro_use]
extern crate log;
extern crate fern;

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
            format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout(), fern::OutputConfig::file("output.log")],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Info) {
        panic!("Failed to initialize global logger: {}", e);
    }

    info!("Frontend: SDL2 init");
    let sdl = sdl2::init().unwrap();

    info!("Frontend: SDL2 Video init");
    let sdl_video = sdl.video().unwrap();

    info!("Frontend: SDL2 event pump");
    let mut event_pump = sdl.event_pump().unwrap();

    info!("Frontend: creating renderer");
    let mut renderer = px8::renderer::renderer::Renderer::new(sdl_video, false, px8::gfx::Scale::Scale4x).unwrap();

    let mut times = px8::frontend::frametimes::FrameTimes::new(Duration::from_secs(1) / 60);
    times.reset();

    let mut fps_counter = px8::frontend::fps::FpsCounter::new();

    let players_input = Arc::new(Mutex::new(px8::config::Players::new()));
    let players_clone = players_input.clone();

    let mut info = Arc::new(Mutex::new(px8::px8::info::Info::new()));
    let (tx_input, rx_input): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = chan::sync(0);
    let (tx_output, rx_output): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = chan::sync(0);

    let mut demo_px8 = px8::px8::Px8New::new();
    demo_px8.init();

    let data = include_bytes!("cast.p8");
    let mut data_final: Vec<u8> = array_to_vec(data);

    demo_px8.load_cartridge_raw("cast.p8".to_string(),
                                data_final,
                                tx_input.clone(),
                                rx_output.clone(),
                                players_input,
                                info.clone(),
                                false);


    demo_px8.init_time = demo_px8.call_init() * 1000.0;

    let mut elapsed_time: f64 = 0.;
    let start_time = time::now();

    emscripten_loop::set_main_loop_callback(|| {
        demo_px8.screen.lock().unwrap().cls();

        let delta = times.update();

        fps_counter.update(times.get_last_time());

        demo_px8.fps = fps_counter.get_fps();

        for event in event_pump.poll_iter() {
            //info!("EVENT {:?}", event);

            match event {
              //  Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(keycode), repeat, .. } => {
                    if let (Some(key), player) = px8::config::keys::map_keycode(keycode) {
                        players_clone.lock().unwrap().key_down(player, key, repeat, elapsed_time);
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    if let (Some(key), player) = px8::config::keys::map_keycode(keycode) {
                        players_clone.lock().unwrap().key_up(player, key)
                    }
                },

                _ => (),
            }
        }

        demo_px8.update_time = demo_px8.call_update() * 1000.0;
        demo_px8.draw_time = demo_px8.call_draw() * 1000.0;

        demo_px8.update();

        let new_time = time::now();
        let diff_time = new_time - start_time;
        let nanoseconds = (diff_time.num_nanoseconds().unwrap() as f64) - (diff_time.num_seconds() * 1000000000) as f64;

        elapsed_time = diff_time.num_seconds() as f64 + nanoseconds / 1000000000.0;

        info.lock().unwrap().elapsed_time = elapsed_time;

        players_clone.lock().unwrap().update(elapsed_time);

        renderer.blit(&demo_px8.screen.lock().unwrap().back_buffer);
        times.limit();
    });
}


#[cfg(target_os = "emscripten")]
pub mod emscripten_loop {
    use std::cell::RefCell;
    use std::ptr::null_mut;
    use std::os::raw::{c_int, c_void};

    #[allow(non_camel_case_types)]
    type em_callback_func = unsafe extern fn();

    extern {
        fn emscripten_set_main_loop(func: em_callback_func, fps: c_int, simulate_infinite_loop: c_int);
    }

    thread_local!(static MAIN_LOOP_CALLBACK: RefCell<*mut c_void> = RefCell::new(null_mut()));

    pub fn set_main_loop_callback<F>(callback: F) where F: FnMut() {
        MAIN_LOOP_CALLBACK.with(|log| {
            *log.borrow_mut() = &callback as *const _ as *mut c_void;
        });

        unsafe { emscripten_set_main_loop(wrapper::<F>, 0, 1); }

        unsafe extern "C" fn wrapper<F>() where F: FnMut() {
            MAIN_LOOP_CALLBACK.with(|z| {
                let closure = *z.borrow_mut() as *mut F;
                (*closure)();
            });
        }
    }
}