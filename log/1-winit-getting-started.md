---
date: 24/02/2026
crate: crates/winit-example
---

# Getting started with winit

---
**Goals**:
 - get started with winit
 - spawn a window
 - capture keypresses and mouse pointer
 - render an image to the window
---


winit is a window management library. It allows you to: spawn a window, capture events (mouse, keyboard, resize, window moved, etc)

## spawning a window and capturing events

Sample code to spawn a window, and capture keyboard/mouse events:

```rust
use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::{self, ControlFlow, EventLoop},
    window::Window,
};

struct App {
    window: Option<Window>,
}

impl App {
    fn new() -> Self {
        Self { window: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let logical = event.logical_key;
                let physical = event.physical_key;
                println!("keypress:\n\tphysical: {physical:?}\n\tlogical: {logical:?}");
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                println!("mouse input:\n\tstate: {state:?}\n\tbutton: {button:?}");
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                println!("mouse moved:\n\tdelta: {delta:?}\n\tphase: {phase:?}");
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}

```

## drawing

winit by itself does not provide any APIs to draw inside the window you create.

you can instead get a handle to the window and pass it to a rendering library to render graphics inside the window.

**Using wgpu with winit**

```rust
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, OwnedDisplayHandle},
    window::Window,
};

// ------ BEGIN WGPU GRAPHICS STUFF --------------------------------------------
struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        dbg!(surface_format);

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn render(&mut self) {
        let surface_texture = self.surface.get_current_texture().unwrap();

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

// ------ END WGPU GRAPHICS STUFF ---------------------------------------------

// ------ BEGIN WINIT WINDOW STUFF --------------------------------------------
struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => {}
        }
    }
}

// ------ END WINIT WINDOW STUFF --------------------------------------------

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}
```

This code looks (and is!) complicated as wgpu is very low level. even after all this we are not really drawing anything to the screen.
Instead, we're simply clearing the screen (and setting a green background). we do not know how to draw anything yet.

but, we can see how the window handle can be passed to WGPU for it to use it as a surface to draw on.

