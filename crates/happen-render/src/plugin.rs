use std::sync::Arc;

use happen_core::{App, Input, KeyCode, MouseButton};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::camera::Camera;
use crate::material::MaterialAssets;
use crate::mesh::MeshAssets;
use crate::renderer::{GpuContext, RenderState};

pub type InitCallback = Box<dyn FnOnce(&GpuContext, &RenderState, &mut happen_core::App)>;

pub struct RenderPlugin;

impl happen_core::Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert_resource(Input::new());
        app.add_system(
            happen_core::STAGE_PRE_UPDATE,
            "fps_controller",
            crate::controller::fps_controller_system,
        );
    }

    fn name(&self) -> &str {
        "RenderPlugin"
    }
}

pub fn default_init_callback() -> InitCallback {
    Box::new(|gpu, render_state, app| {
        let mut mesh_assets = MeshAssets::new();
        let mut material_assets = MaterialAssets::new();

        let cube_mesh = crate::mesh::Mesh::cube(1.0);
        mesh_assets.upload(&gpu.device, &cube_mesh);

        let sphere_mesh = crate::mesh::Mesh::sphere(0.5, 32, 16);
        mesh_assets.upload(&gpu.device, &sphere_mesh);

        let plane_mesh = crate::mesh::Mesh::plane(100.0, 100.0);
        mesh_assets.upload(&gpu.device, &plane_mesh);

        let default_mat = crate::material::Material::default();
        material_assets.upload(
            &gpu.device,
            &render_state.material_bind_group_layout,
            &default_mat,
        );

        app.world.insert_resource(mesh_assets);
        app.world.insert_resource(material_assets);
    })
}

struct HappenApp {
    app: Option<happen_core::App>,
    gpu: Option<GpuContext>,
    render_state: Option<RenderState>,
    init_callback: Option<InitCallback>,
    cursor_locked: bool,
}

fn map_key(key: winit::keyboard::KeyCode) -> Option<KeyCode> {
    use winit::keyboard::KeyCode as K;
    match key {
        K::KeyW => Some(KeyCode::W),
        K::KeyA => Some(KeyCode::A),
        K::KeyS => Some(KeyCode::S),
        K::KeyD => Some(KeyCode::D),
        K::KeyQ => Some(KeyCode::Q),
        K::KeyE => Some(KeyCode::E),
        K::KeyR => Some(KeyCode::R),
        K::KeyF => Some(KeyCode::F),
        K::KeyG => Some(KeyCode::G),
        K::Space => Some(KeyCode::Space),
        K::ShiftLeft => Some(KeyCode::LShift),
        K::ControlLeft => Some(KeyCode::LControl),
        K::AltLeft => Some(KeyCode::LAlt),
        K::Escape => Some(KeyCode::Escape),
        K::ArrowUp => Some(KeyCode::Up),
        K::ArrowDown => Some(KeyCode::Down),
        K::ArrowLeft => Some(KeyCode::Left),
        K::ArrowRight => Some(KeyCode::Right),
        K::Digit1 => Some(KeyCode::Num1),
        K::Digit2 => Some(KeyCode::Num2),
        K::Digit3 => Some(KeyCode::Num3),
        K::Digit4 => Some(KeyCode::Num4),
        K::Digit5 => Some(KeyCode::Num5),
        K::Tab => Some(KeyCode::Tab),
        K::Enter => Some(KeyCode::Enter),
        _ => None,
    }
}

fn map_mouse_button(btn: winit::event::MouseButton) -> Option<MouseButton> {
    match btn {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

impl HappenApp {
    fn lock_cursor(&mut self) {
        if let Some(gpu) = &self.gpu {
            let window = &gpu.window;
            let _ = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
        }
        self.cursor_locked = true;
        if let Some(app) = &mut self.app {
            if let Some(input) = app.world.get_resource_mut::<Input>() {
                input.cursor_locked = true;
            }
        }
    }

    fn unlock_cursor(&mut self) {
        if let Some(gpu) = &self.gpu {
            let window = &gpu.window;
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
        self.cursor_locked = false;
        if let Some(app) = &mut self.app {
            if let Some(input) = app.world.get_resource_mut::<Input>() {
                input.cursor_locked = false;
            }
        }
    }
}

impl ApplicationHandler for HappenApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("Happen Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        let gpu = pollster::block_on(GpuContext::new(window));
        let render_state = RenderState::new(&gpu);

        if let Some(callback) = self.init_callback.take() {
            if let Some(app) = &mut self.app {
                callback(&gpu, &render_state, app);
            }
        }

        self.gpu = Some(gpu);
        self.render_state = Some(render_state);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if !self.cursor_locked {
            return;
        }
        if let DeviceEvent::MouseMotion { delta } = event {
            if let Some(app) = &mut self.app {
                if let Some(input) = app.world.get_resource_mut::<Input>() {
                    input.accumulate_mouse_delta(delta.0 as f32, delta.1 as f32);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(physical_size) => {
                if let (Some(gpu), Some(rs)) = (&mut self.gpu, &mut self.render_state) {
                    gpu.resize(physical_size.width, physical_size.height);
                    rs.resize(&gpu.device, physical_size.width, physical_size.height);

                    let aspect = physical_size.width as f32 / physical_size.height.max(1) as f32;
                    if let Some(app) = &mut self.app {
                        let entities = app.world.all_entities();
                        for entity in entities {
                            if let Some(cam) = app.world.get_component_mut::<Camera>(entity) {
                                cam.projection.set_aspect(aspect);
                            }
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(key) = map_key(code) {
                        if key == KeyCode::Escape
                            && event.state == ElementState::Pressed
                            && self.cursor_locked
                        {
                            self.unlock_cursor();
                        } else if let Some(app) = &mut self.app {
                            if let Some(input) = app.world.get_resource_mut::<Input>() {
                                match event.state {
                                    ElementState::Pressed => input.press_key(key),
                                    ElementState::Released => input.release_key(key),
                                }
                            }
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(btn) = map_mouse_button(button) {
                    if state == ElementState::Pressed && !self.cursor_locked {
                        self.lock_cursor();
                    }
                    if let Some(app) = &mut self.app {
                        if let Some(input) = app.world.get_resource_mut::<Input>() {
                            match state {
                                ElementState::Pressed => input.press_mouse(btn),
                                ElementState::Released => input.release_mouse(btn),
                            }
                        }
                    }
                }
            }

            WindowEvent::Focused(false) => {
                if self.cursor_locked {
                    self.unlock_cursor();
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(app) = &mut self.app {
                    app.update();
                }

                if let (Some(gpu), Some(rs)) = (&self.gpu, &self.render_state) {
                    if let Some(app) = &self.app {
                        match rs.render(gpu, &app.world) {
                            Ok(()) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                let size = gpu.window.inner_size();
                                let gpu = self.gpu.as_mut().unwrap();
                                gpu.resize(size.width, size.height);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                event_loop.exit();
                            }
                            Err(e) => {
                                log::warn!("Render error: {:?}", e);
                            }
                        }
                    }
                }

                if let Some(app) = &mut self.app {
                    if let Some(input) = app.world.get_resource_mut::<Input>() {
                        input.end_frame();
                    }
                }

                if let Some(gpu) = &self.gpu {
                    gpu.window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

pub fn run_with_init(app: happen_core::App, init: InitCallback) {
    let _ = env_logger::try_init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut happen_app = HappenApp {
        app: Some(app),
        gpu: None,
        render_state: None,
        init_callback: Some(init),
        cursor_locked: false,
    };

    event_loop.run_app(&mut happen_app).unwrap();
}
