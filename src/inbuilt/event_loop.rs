use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use crate::state::State;

pub async fn run() {
   env_logger::init();
   let event_loop = EventLoop::new().unwrap();
   let window = WindowBuilder::new().build(&event_loop).unwrap();

   let mut state = State::new(&window).await;
   let mut surface_configured = false;

   event_loop.run(move |event, control_flow| {
      match event {
         Event::WindowEvent {
            ref event,
            window_id,
         } if window_id == state.setup.window.id() => {
            if !state.update_input(event) {
               // UPDATED!
               match event {
                  WindowEvent::CloseRequested
                  | WindowEvent::KeyboardInput {
                     event:
                     KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                     },
                     ..
                  } => control_flow.exit(),

                  WindowEvent::Resized(physical_size) => {
                     log::info!("physical_size: {physical_size:?}");
                     surface_configured = true;
                     state.resize(*physical_size);
                  }

                  WindowEvent::RedrawRequested => {
                     // This tells winit that we want another frame after this one
                     state.setup.window.request_redraw();

                     if !surface_configured {
                        return;
                     }

                     state.update();
                     match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(
                           wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                        ) => state.resize(state.setup.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                           log::error!("OutOfMemory");
                           control_flow.exit();
                        }

                        // This happens when the frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                           log::warn!("Surface timeout")
                        }
                     }
                  }
                  _ => {}
               }
               state.egui.handle_input(&mut state.setup.window, &event);
            }
         }
         _ => {}
      }
   }).unwrap();
}