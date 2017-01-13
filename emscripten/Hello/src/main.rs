extern crate px8;
extern crate sdl2;
extern crate time;
extern crate rand;

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

use rand::Rng;

#[macro_use]
extern crate log;
extern crate fern;

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

    let mut demo_px8 = px8::px8::Px8New::new();
    demo_px8.init();

    emscripten_loop::set_main_loop_callback(|| {
        demo_px8.screen.lock().unwrap().cls();

        let delta = times.update();

        fps_counter.update(times.get_last_time());

        demo_px8.fps = fps_counter.get_fps();

        demo_px8.screen.lock().unwrap().rectfill(0, 0, 64, 64, px8::px8::Color::from_u8(rand::thread_rng().gen_range(0.0, 16.0) as u8));
        demo_px8.screen.lock().unwrap().rectfill(64, 0, 128, 64, px8::px8::Color::from_u8(rand::thread_rng().gen_range(0.0, 16.0) as u8));
        demo_px8.screen.lock().unwrap().rectfill(0, 64, 64, 128, px8::px8::Color::from_u8(rand::thread_rng().gen_range(0.0, 16.0) as u8));
        demo_px8.screen.lock().unwrap().rectfill(64, 64, 128, 128, px8::px8::Color::from_u8(rand::thread_rng().gen_range(0.0, 16.0) as u8));

        demo_px8.screen.lock().unwrap().print("PX8 + Emscripten".to_string(), 28, 64, px8::px8::Color::Green);

        for event in event_pump.poll_iter() {
            info!("EVENT {:?}", event);

            match event {
            //    Event::Quit { .. } => break 'main,
                _ => (),
            }
        }

        demo_px8.init_time = 0.;
        demo_px8.update_time = 0.;
        demo_px8.draw_time = 0.;

        demo_px8.update();

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