#![feature(allocator_api)]

#[path = "../../guise_example/guise_demo.rs"]
mod demo;
#[path = "../../guise_example/guise_renderer_wgpu.rs"]
mod renderer_wgpu;

use std::alloc::Global;
use std::iter;
use std::ops::DerefMut;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use arrayvec::ArrayString;

static CLIPBOARD: Mutex<Option<copypasta::ClipboardContext>> = Mutex::new(None);

fn init_clipboard_or_not() {
    let mut guard = CLIPBOARD.lock().unwrap();

    if guard.is_none() {
        let clipboard = copypasta::ClipboardContext::new().unwrap();
        guard.replace(clipboard);
    }
}

fn get_clipboard() -> String {
    use copypasta::ClipboardProvider;

    let mut guard = CLIPBOARD.lock().unwrap();
    if let Some(c) = guard.deref_mut() {
        if let Ok(s) = c.get_contents() {
            s
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

fn set_clipboard(text: &str) {
    use copypasta::ClipboardProvider;

    let mut guard = CLIPBOARD.lock().unwrap();
    if let Some(c) = guard.deref_mut() {
        let s = String::from(text);
        let _ = c.set_contents(s);
    }
}

fn main() {
    pretty_env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Look, a demo!")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .expect("Failed to create window");

    init_clipboard_or_not();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("Failed to acquire gpu adapter");

    let (device, mut queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .expect("Failed to acquire gpu device and queue");

    let surface_present_mode = wgpu::PresentMode::Fifo;
    let surface_format = wgpu::TextureFormat::Bgra8Unorm;
    let initial_window_physical_size = window.inner_size();
    let initial_window_width = initial_window_physical_size.width;
    let initial_window_height = initial_window_physical_size.height;

    surface.configure(&device, &wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: initial_window_width,
        height: initial_window_height,
        present_mode: surface_present_mode,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    });

    let mut ui = {
        let scale_factor = window.scale_factor();
        let logical_size = window.inner_size().to_logical(scale_factor);

        guise::Ui::new_in(
            logical_size.width,
            logical_size.height,
            scale_factor as f32,
            guise::FONT_IBM_PLEX_MONO,
            // guise::FONT_IBM_PLEX_SANS_JP,
            // guise::FONT_PROGGY_CLEAN,
            // guise::FONT_ROBOTO,
            // guise::FONT_LIBERATION_MONO,
            guise::UnicodeRangeFlags::ALL,
            14.0,
            scale_factor as f32,
            std::alloc::Global,
        )
    };

    ui.set_clipboard_getter(get_clipboard);
    ui.set_clipboard_setter(set_clipboard);

    let mut renderer = renderer_wgpu::Renderer::new(&device, surface_format);

    let font_atlas_image = ui.font_atlas_image_rgba8_unorm();
    let (font_atlas_width, font_atlas_height) = ui.font_atlas_image_size();
    let font_atlas_texture_id = renderer.add_texture_rgba8_unorm(
        &device,
        &mut queue,
        u32::from(font_atlas_width),
        u32::from(font_atlas_height),
        font_atlas_image,
    );
    ui.set_font_atlas_texture_id(font_atlas_texture_id);

    let mut state = demo::State {
        button_click_count: 0,
        text_input_submit_count: 0,
        text_input_cancel_count: 0,
        poll_platform_events: true,
        graph: [0.0; demo::GRAPH_LEN],
        graph_max: 0.0,
        graph_frame_build: [0.0; demo::GRAPH_LEN],
        graph_frame_build_max: 0.0,
        graph_command_count: [0; demo::GRAPH_LEN],
        graph_command_count_max: 0,
        graph_vertex_count: [0; demo::GRAPH_LEN],
        graph_vertex_count_max: 0,
        graph_index_count: [0; demo::GRAPH_LEN],
        graph_index_count_max: 0,
        text_input_heap: guise::VecString::new_in(Global),
        text_input_inline: ArrayString::new(),
        float_slider_value: 1.0,
        float_slider_value_clamped: 0.0,
        float_slider2_value: [0.0; 2],
        float_slider3_value: [0.0; 3],
        float_slider4_value: [0.0; 4],
        int_slider_value: 1,
        int_slider_value_clamped: 0,
        int_slider2_value: [0; 2],
        int_slider3_value: [0; 3],
        int_slider4_value: [0; 4],
        dropdown1_selected_option: None,
        dropdown2_selected_option: None,
    };

    let time_start = Instant::now();
    let mut time = time_start;

    let mut window_width = initial_window_width;
    let mut window_height = initial_window_height;
    let mut window_size_stale = false;

    let mut frame_count = 0;
    let mut frame_build_duration = Duration::new(0, 0);
    let mut frame_total_duration = Duration::new(0, 0);
    let mut frame_draw_list_command_count = 0;
    let mut frame_draw_list_vertex_count = 0;
    let mut frame_draw_list_index_count = 0;
    let mut frame_ctrl_count = 0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if state.poll_platform_events {
            winit::event_loop::ControlFlow::Poll
        } else {
            winit::event_loop::ControlFlow::Wait
        };

        match event {
            winit::event::Event::NewEvents(_) => {
                frame_count += 1;
                time = Instant::now();
            }
            winit::event::Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                winit::event::WindowEvent::Resized(physical_size) => {
                    let logical_size = physical_size.to_logical(window.scale_factor());
                    ui.set_window_size(logical_size.width, logical_size.height);

                    if (physical_size.width, physical_size.height) != (window_width, window_height)
                    {
                        window_width = physical_size.width;
                        window_height = physical_size.height;
                        window_size_stale = true;
                    }
                }
                winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    ui.set_window_scale_factor(scale_factor as f32);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                winit::event::WindowEvent::ReceivedCharacter(character) => {
                    ui.send_character(character);
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let scale_factor = window.scale_factor();
                    let logical_position = position.to_logical(scale_factor);
                    ui.set_cursor_position(logical_position.x, logical_position.y);
                }
                winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(dx, dy) => {
                        ui.scroll(dx * 10.0, dy * 10.0);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => {
                        let scale_factor = window.scale_factor();
                        let logical_position = physical_position.to_logical::<f32>(scale_factor);
                        ui.scroll(logical_position.x, logical_position.y);
                    }
                },
                winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                    winit::event::ElementState::Pressed => match button {
                        winit::event::MouseButton::Left => {
                            ui.press_inputs(guise::Inputs::MB_LEFT);
                        }
                        winit::event::MouseButton::Right => {
                            ui.press_inputs(guise::Inputs::MB_RIGHT);
                        }
                        winit::event::MouseButton::Middle => {
                            ui.press_inputs(guise::Inputs::MB_MIDDLE);
                        }
                        _ => (),
                    },
                    winit::event::ElementState::Released => match button {
                        winit::event::MouseButton::Left => {
                            ui.release_inputs(guise::Inputs::MB_LEFT);
                        }
                        winit::event::MouseButton::Right => {
                            ui.release_inputs(guise::Inputs::MB_RIGHT);
                        }
                        winit::event::MouseButton::Middle => {
                            ui.release_inputs(guise::Inputs::MB_MIDDLE);
                        }
                        _ => (),
                    },
                },
                winit::event::WindowEvent::KeyboardInput { input, .. } => match input.state {
                    winit::event::ElementState::Pressed => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Tab) => {
                            ui.press_inputs(guise::Inputs::KB_TAB);
                        }
                        Some(winit::event::VirtualKeyCode::Left) => {
                            ui.press_inputs(guise::Inputs::KB_LEFT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Right) => {
                            ui.press_inputs(guise::Inputs::KB_RIGHT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Up) => {
                            ui.press_inputs(guise::Inputs::KB_UP_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Down) => {
                            ui.press_inputs(guise::Inputs::KB_DOWN_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::PageUp) => {
                            ui.press_inputs(guise::Inputs::KB_PAGE_UP);
                        }
                        Some(winit::event::VirtualKeyCode::PageDown) => {
                            ui.press_inputs(guise::Inputs::KB_PAGE_DOWN);
                        }
                        Some(winit::event::VirtualKeyCode::Home) => {
                            ui.press_inputs(guise::Inputs::KB_HOME);
                        }
                        Some(winit::event::VirtualKeyCode::End) => {
                            ui.press_inputs(guise::Inputs::KB_END);
                        }
                        Some(winit::event::VirtualKeyCode::Insert) => {
                            ui.press_inputs(guise::Inputs::KB_INSERT);
                        }
                        Some(winit::event::VirtualKeyCode::Delete) => {
                            ui.press_inputs(guise::Inputs::KB_DELETE);
                        }
                        Some(winit::event::VirtualKeyCode::Back) => {
                            ui.press_inputs(guise::Inputs::KB_BACKSPACE);
                        }
                        Some(winit::event::VirtualKeyCode::Return) => {
                            ui.press_inputs(guise::Inputs::KB_ENTER);
                        }
                        Some(winit::event::VirtualKeyCode::Escape) => {
                            ui.press_inputs(guise::Inputs::KB_ESCAPE);
                        }
                        Some(winit::event::VirtualKeyCode::A) => {
                            ui.press_inputs(guise::Inputs::KB_A);
                        }
                        Some(winit::event::VirtualKeyCode::F) => {
                            ui.press_inputs(guise::Inputs::KB_F);
                        }
                        Some(winit::event::VirtualKeyCode::B) => {
                            ui.press_inputs(guise::Inputs::KB_B);
                        }
                        Some(winit::event::VirtualKeyCode::X) => {
                            ui.press_inputs(guise::Inputs::KB_X);
                        }
                        Some(winit::event::VirtualKeyCode::C) => {
                            ui.press_inputs(guise::Inputs::KB_C);
                        }
                        Some(winit::event::VirtualKeyCode::V) => {
                            ui.press_inputs(guise::Inputs::KB_V);
                        }
                        _ => (),
                    },
                    winit::event::ElementState::Released => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Tab) => {
                            ui.release_inputs(guise::Inputs::KB_TAB);
                        }
                        Some(winit::event::VirtualKeyCode::Left) => {
                            ui.release_inputs(guise::Inputs::KB_LEFT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Right) => {
                            ui.release_inputs(guise::Inputs::KB_RIGHT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Up) => {
                            ui.release_inputs(guise::Inputs::KB_UP_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Down) => {
                            ui.release_inputs(guise::Inputs::KB_DOWN_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::PageUp) => {
                            ui.release_inputs(guise::Inputs::KB_PAGE_UP);
                        }
                        Some(winit::event::VirtualKeyCode::PageDown) => {
                            ui.release_inputs(guise::Inputs::KB_PAGE_DOWN);
                        }
                        Some(winit::event::VirtualKeyCode::Home) => {
                            ui.release_inputs(guise::Inputs::KB_HOME);
                        }
                        Some(winit::event::VirtualKeyCode::End) => {
                            ui.release_inputs(guise::Inputs::KB_END);
                        }
                        Some(winit::event::VirtualKeyCode::Insert) => {
                            ui.release_inputs(guise::Inputs::KB_INSERT);
                        }
                        Some(winit::event::VirtualKeyCode::Delete) => {
                            ui.release_inputs(guise::Inputs::KB_DELETE);
                        }
                        Some(winit::event::VirtualKeyCode::Back) => {
                            ui.release_inputs(guise::Inputs::KB_BACKSPACE);
                        }
                        Some(winit::event::VirtualKeyCode::Return) => {
                            ui.release_inputs(guise::Inputs::KB_ENTER);
                        }
                        Some(winit::event::VirtualKeyCode::Escape) => {
                            ui.release_inputs(guise::Inputs::KB_ESCAPE);
                        }
                        Some(winit::event::VirtualKeyCode::A) => {
                            ui.release_inputs(guise::Inputs::KB_A);
                        }
                        Some(winit::event::VirtualKeyCode::F) => {
                            ui.release_inputs(guise::Inputs::KB_F);
                        }
                        Some(winit::event::VirtualKeyCode::B) => {
                            ui.release_inputs(guise::Inputs::KB_B);
                        }
                        Some(winit::event::VirtualKeyCode::X) => {
                            ui.release_inputs(guise::Inputs::KB_X);
                        }
                        Some(winit::event::VirtualKeyCode::C) => {
                            ui.release_inputs(guise::Inputs::KB_C);
                        }
                        Some(winit::event::VirtualKeyCode::V) => {
                            ui.release_inputs(guise::Inputs::KB_V);
                        }
                        _ => (),
                    },
                },
                winit::event::WindowEvent::ModifiersChanged(state) => {
                    let mut modifiers = guise::Modifiers::empty();

                    if state.ctrl() {
                        modifiers |= guise::Modifiers::CTRL;
                    }
                    if state.alt() {
                        modifiers |= guise::Modifiers::ALT;
                    }
                    if state.shift() {
                        modifiers |= guise::Modifiers::SHIFT;
                    }

                    ui.set_modifiers(modifiers);
                }
                _ => (),
            },
            winit::event::Event::MainEventsCleared => {
                let want_capture_keyboard = ui.want_capture_keyboard();
                let want_capture_mouse = ui.want_capture_mouse();
                let mut frame = ui.begin_frame();
                demo::draw_ui(
                    &mut frame,
                    &demo::Stats {
                        running_duration: time - time_start,
                        frame_count,
                        frame_build_duration,
                        frame_total_duration,
                        frame_draw_list_command_count,
                        frame_draw_list_vertex_count,
                        frame_draw_list_index_count,
                        frame_ctrl_count,
                        want_capture_keyboard,
                        want_capture_mouse,
                    },
                    &mut state,
                );
                ui.end_frame();

                frame_ctrl_count = ui.ctrl_count();
                frame_build_duration = Instant::now() - time;

                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let physical_size = window.inner_size();
                if physical_size.width == 0 || physical_size.height == 0 {
                    return;
                }

                if window_size_stale {
                    surface.configure(&device, &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: surface_format,
                        width: window_width,
                        height: window_height,
                        present_mode: surface_present_mode,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    });
                }

                if let Ok(surface_texture) = surface.get_current_texture() {
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    let view = surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let (commands, vertices, indices) = ui.draw_list();
                    renderer.draw(
                        &device,
                        &mut queue,
                        &mut encoder,
                        &view,
                        wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                        window_width,
                        window_height,
                        window.scale_factor() as f32,
                        commands,
                        vertices,
                        indices,
                    );

                    queue.submit(iter::once(encoder.finish()));

                    surface_texture.present();
                }
            }
            winit::event::Event::RedrawEventsCleared => {
                frame_total_duration = Instant::now() - time;

                let (commands, vertices, indices) = ui.draw_list();

                frame_draw_list_command_count = commands.len();
                frame_draw_list_vertex_count = vertices.len();
                frame_draw_list_index_count = indices.len();
            }
            winit::event::Event::LoopDestroyed => {
                // TODO(yan): @Cleanup Removing the font atlas explicitly from the
                // renderer is not necessary and the renderer would clean it up
                // itself, but this shows the API exists. Remove once we have
                // more textures.
                renderer.remove_texture(font_atlas_texture_id);
            }
            _ => (),
        }
    });
}
