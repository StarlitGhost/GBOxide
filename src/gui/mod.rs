use crate::cartridge::Cartridge;
use crate::gameboy::GameBoy;
use crate::gameboy::lcd::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::gameboy::joypad::Controls;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit_input_helper::WinitInputHelper;

pub fn run(cartridge: Cartridge) -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let (window, surface, width, height, mut hidpi_factor) = {
        let scale = 3.0;
        let width = SCREEN_WIDTH as f64 * scale;
        let height = SCREEN_HEIGHT as f64 * scale;

        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .with_title("GBOxide")
            .build(&event_loop)
            .unwrap();
        let surface = pixels::wgpu::Surface::create(&window);
        let hidpi_factor = window.hidpi_factor();
        let size = window.inner_size().to_physical(hidpi_factor);

        (
            window,
            surface,
            size.width.round() as u32,
            size.height.round() as u32,
            hidpi_factor
        )
    };

    let surface_texture = SurfaceTexture::new(width, height, surface);
    let mut pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?;
    let mut gameboy = GameBoy::new(cartridge);

    event_loop.run(move |event, _, control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            gameboy.draw_frame(pixels.get_frame());
            pixels.render();
        }

        if input.update(event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let controls = Controls {
                left: input.key_held(VirtualKeyCode::Left),
                right: input.key_held(VirtualKeyCode::Right),
                up: input.key_held(VirtualKeyCode::Up),
                down: input.key_held(VirtualKeyCode::Down),

                a: input.key_held(VirtualKeyCode::X),
                b: input.key_held(VirtualKeyCode::Z),
                start: input.key_held(VirtualKeyCode::Return),
                select: input.key_held(VirtualKeyCode::Space),
            };
            gameboy.set_controls(controls);

            if let Some(factor) = input.hidpi_changed() {
                hidpi_factor = factor;
            }

            if let Some(size) = input.window_resized() {
                let size = size.to_physical(hidpi_factor);
                let width = size.width.round() as u32;
                let height = size.height.round() as u32;

                pixels.resize(width, height);
            }

            gameboy.run_to_vblank()
                .unwrap_or_else(
                    |err| {
                        panic!("Gameboy Error: {}", err);
                    }
                );
            window.request_redraw();
        }
    });
}