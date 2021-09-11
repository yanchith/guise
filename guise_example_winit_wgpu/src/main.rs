#[path = "../../guise_example/guise_demo.rs"]
mod demo;
#[path = "../../guise_example/guise_renderer_wgpu.rs"]
mod renderer_wgpu;

use std::iter;
use std::time::{Duration, Instant};

use arrayvec::ArrayString;

fn main() {
    pretty_env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Look, a demo!")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .expect("Failed to create window");

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
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

    // TODO(yan): Find out why PresentMode::{Mailbox,Immediate} on
    // (X11,Intel,Vulkan) and PresentMode::Immediate on
    // (Windows,{Intel,Nvidia},Vulkan) doesn't have presentation latency while
    // PresentMode::Fifo does (how many frames exactly?)
    let surface_present_mode = wgpu::PresentMode::Fifo;
    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    {
        let physical_size = window.inner_size();
        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: surface_present_mode,
            },
        );
    }

    let mut ui = {
        let logical_size = window.inner_size().to_logical(window.scale_factor());

        guise::Ui::new(
            logical_size.width,
            logical_size.height,
            guise::FONT_PROGGY_CLEAN,
            guise::UnicodeRangeFlags::ALL,
            11.0 * window.scale_factor() as f32,
        )
    };

    let mut renderer = renderer_wgpu::Renderer::new(&device, surface_format);

    let font_atlas_image = ui.font_atlas_image_rgba8_unorm();
    let (font_atlas_width, font_atlas_height) = ui.font_atlas_image_extents();
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
        text_input_text_heap: String::new(),
        text_input_text_inline: ArrayString::new(),
    };

    let time_start = Instant::now();
    let mut time = time_start;

    let mut frame_count = 0;
    let mut frame_build_duration = Duration::new(0, 0);
    let mut frame_total_duration = Duration::new(0, 0);
    let mut frame_draw_list_command_count = 0;
    let mut frame_draw_list_vertex_count = 0;
    let mut frame_ctrl_count = 0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if state.poll_platform_events {
            winit::event_loop::ControlFlow::Poll
        } else {
            winit::event_loop::ControlFlow::Wait
        };

        match event {
            winit::event::Event::NewEvents(start_cause) => {
                log::trace!("Frame start cause: {:?}", start_cause);
                frame_count += 1;
                time = Instant::now();
            }
            winit::event::Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                winit::event::WindowEvent::Resized(physical_size) => {
                    let logical_size = physical_size.to_logical(window.scale_factor());

                    ui.set_render_target_extents(logical_size.width, logical_size.height);
                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format: surface_format,
                            width: physical_size.width,
                            height: physical_size.height,
                            present_mode: surface_present_mode,
                        },
                    );
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                winit::event::WindowEvent::ReceivedCharacter(character) => {
                    ui.add_window_character(character);
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let logical_position = position.to_logical(window.scale_factor());
                    ui.set_window_cursor_position(logical_position.x, logical_position.y);
                }
                winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(dx, dy) => {
                        ui.add_window_scroll_delta(dx * 10.0, dy * 10.0);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => {
                        let scale_factor = window.scale_factor();
                        let logical_position = physical_position.to_logical::<f32>(scale_factor);
                        ui.add_window_scroll_delta(logical_position.x, logical_position.y);
                    }
                },
                winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                    winit::event::ElementState::Pressed => match button {
                        winit::event::MouseButton::Left => {
                            ui.add_window_inputs_pressed(guise::Inputs::MOUSE_BUTTON_LEFT);
                        }
                        winit::event::MouseButton::Right => {
                            ui.add_window_inputs_pressed(guise::Inputs::MOUSE_BUTTON_RIGHT);
                        }
                        winit::event::MouseButton::Middle => {
                            ui.add_window_inputs_pressed(guise::Inputs::MOUSE_BUTTON_MIDDLE);
                        }
                        _ => (),
                    },
                    winit::event::ElementState::Released => match button {
                        winit::event::MouseButton::Left => {
                            ui.add_window_inputs_released(guise::Inputs::MOUSE_BUTTON_LEFT);
                        }
                        winit::event::MouseButton::Right => {
                            ui.add_window_inputs_released(guise::Inputs::MOUSE_BUTTON_RIGHT);
                        }
                        winit::event::MouseButton::Middle => {
                            ui.add_window_inputs_released(guise::Inputs::MOUSE_BUTTON_MIDDLE);
                        }
                        _ => (),
                    },
                },
                winit::event::WindowEvent::KeyboardInput { input, .. } => match input.state {
                    winit::event::ElementState::Pressed => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Tab) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_TAB);
                        }
                        Some(winit::event::VirtualKeyCode::Left) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_LEFT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Right) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_RIGHT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Up) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_UP_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Down) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_DOWN_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::PageUp) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_PAGE_UP);
                        }
                        Some(winit::event::VirtualKeyCode::PageDown) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_PAGE_DOWN);
                        }
                        Some(winit::event::VirtualKeyCode::Home) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_HOME);
                        }
                        Some(winit::event::VirtualKeyCode::End) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_END);
                        }
                        Some(winit::event::VirtualKeyCode::Insert) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_INSERT);
                        }
                        Some(winit::event::VirtualKeyCode::Delete) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_DELETE);
                        }
                        Some(winit::event::VirtualKeyCode::Back) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_BACKSPACE);
                        }
                        Some(winit::event::VirtualKeyCode::Return) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_ENTER);
                        }
                        Some(winit::event::VirtualKeyCode::Escape) => {
                            ui.add_window_inputs_pressed(guise::Inputs::KEYBOARD_ESCAPE);
                        }
                        _ => (),
                    },
                    winit::event::ElementState::Released => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Tab) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_TAB);
                        }
                        Some(winit::event::VirtualKeyCode::Left) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_LEFT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Right) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_RIGHT_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Up) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_UP_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::Down) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_DOWN_ARROW);
                        }
                        Some(winit::event::VirtualKeyCode::PageUp) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_PAGE_UP);
                        }
                        Some(winit::event::VirtualKeyCode::PageDown) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_PAGE_DOWN);
                        }
                        Some(winit::event::VirtualKeyCode::Home) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_HOME);
                        }
                        Some(winit::event::VirtualKeyCode::End) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_END);
                        }
                        Some(winit::event::VirtualKeyCode::Insert) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_INSERT);
                        }
                        Some(winit::event::VirtualKeyCode::Delete) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_DELETE);
                        }
                        Some(winit::event::VirtualKeyCode::Back) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_BACKSPACE);
                        }
                        Some(winit::event::VirtualKeyCode::Return) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_ENTER);
                        }
                        Some(winit::event::VirtualKeyCode::Escape) => {
                            ui.add_window_inputs_released(guise::Inputs::KEYBOARD_ESCAPE);
                        }
                        _ => (),
                    },
                },
                _ => (),
            },
            winit::event::Event::MainEventsCleared => {
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
                        frame_ctrl_count,
                    },
                    &mut state,
                );
                ui.end_frame();

                frame_ctrl_count = ui.ctrl_count();
                frame_build_duration = Instant::now() - time;

                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(window_id) => {
                log::trace!("Redraw requested for window {:?}", window_id);

                let physical_size = window.inner_size();
                if physical_size.width == 0 || physical_size.height == 0 {
                    return;
                }

                if let Ok(surface_frame) = surface.get_current_frame() {
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    let view = surface_frame
                        .output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    renderer
                        .draw(
                            &device,
                            &mut queue,
                            &mut encoder,
                            &view,
                            wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.2,
                                a: 1.0,
                            },
                            physical_size.width,
                            physical_size.height,
                            window.scale_factor() as f32,
                            ui.draw_list(),
                        )
                        .expect("Failed to render draw list");

                    queue.submit(iter::once(encoder.finish()));
                }
            }
            winit::event::Event::RedrawEventsCleared => {
                frame_total_duration = Instant::now() - time;
                frame_draw_list_command_count = ui.draw_list().commands().len();
                frame_draw_list_vertex_count = ui.draw_list().vertices().len();
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
