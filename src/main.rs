#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod mandelbrot;

use crate::mandelbrot::{MandelbrotGenerator, MandelbrotRenderer};
use log::error;
use pixels::{PixelsBuilder, SurfaceTexture};
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

            wasm_bindgen_futures::spawn_local(run());
        }

    #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();

            pollster::block_on(run());
        }
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Visions of Mandelbrot")
            .with_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            // Retrieve current width and height dimensions of browser client window
            let get_window_size = || {
                let client_window = web_sys::window().unwrap();
                LogicalSize::new(
                    client_window.inner_width().unwrap().as_f64().unwrap(),
                    client_window.inner_height().unwrap().as_f64().unwrap(),
                )
            };

            let window = Rc::clone(&window);

            // Initialize winit window with current dimensions of browser client
            window.set_inner_size(get_window_size());

            let client_window = web_sys::window().unwrap();

            // Attach winit canvas to body element
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");

            // Listen for resize event on browser client. Adjust winit window dimensions
            // on event trigger
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let size = get_window_size();
                window.set_inner_size(size)
            }) as Box<dyn FnMut(_)>);
            client_window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

    let mut input = WinitInputHelper::new();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(true)
            .build_async()
            .await
            .expect("Pixels error")
    };

    let mandelbrot_set = MandelbrotGenerator::new(WIDTH as usize, HEIGHT as usize, MandelbrotGenerator::DEFAULT_MAX_ITERATIONS);
    let mut mandelbrot_renderer = MandelbrotRenderer::new(WIDTH as usize, HEIGHT as usize, mandelbrot_set);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            mandelbrot_renderer.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {:?}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Zoom events
            if input.mouse_pressed(0) {
                // Left mouse
                mandelbrot_renderer.zoom(input.mouse().unwrap(), 0.5);
            } else if input.mouse_pressed(1) {
                // Right mouse
                mandelbrot_renderer.zoom(input.mouse().unwrap(), 2.0);
            }

            // Palette events
            if input.key_pressed(VirtualKeyCode::P) {
                mandelbrot_renderer.randomize_palette();
            }

            // Reset events
            if input.key_pressed(VirtualKeyCode::R) {
                mandelbrot_renderer.resize(WIDTH as usize, HEIGHT as usize);
                mandelbrot_renderer.generator = MandelbrotGenerator::new(WIDTH as usize, HEIGHT as usize, MandelbrotGenerator::DEFAULT_MAX_ITERATIONS);
                mandelbrot_renderer.palette = MandelbrotRenderer::rainbow_palette(MandelbrotGenerator::DEFAULT_MAX_ITERATIONS as usize);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                pixels.resize_buffer(size.width, size.height);

                mandelbrot_renderer.resize(size.width as usize, size.height as usize);
            }

            window.request_redraw();
        }
    });
}
