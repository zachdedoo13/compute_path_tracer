use std::collections::HashSet;
use cgmath::Vector2;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputManager {
   currently_pressed: HashSet<KeyCode>,
   just_pressed: HashSet<KeyCode>,

   mouse_currently_pressed: HashSet<MouseButton>,
   mouse_just_pressed: HashSet<MouseButton>,

   pub mouse_screen_pos: Vector2<f32>,
}
impl InputManager {
   pub fn new() -> Self {
      Self {
         currently_pressed: HashSet::new(),
         just_pressed: HashSet::new(),
         mouse_screen_pos: Vector2::new(0.0, 0.0),
         mouse_currently_pressed: HashSet::new(),
         mouse_just_pressed: HashSet::new(),
      }
   }

   pub fn process_event(&mut self, event: &WindowEvent) {

      if let WindowEvent::KeyboardInput { event, .. } = event {
         match event.state {
            ElementState::Pressed => {
               if let PhysicalKey::Code(keycode) = event.physical_key {
                  self.currently_pressed.insert(keycode);
                  self.just_pressed.insert(keycode);
               }
            }
            ElementState::Released => {
               if let PhysicalKey::Code(keycode) = event.physical_key {
                  self.currently_pressed.remove(&keycode);
               }
            }
         }
      }

      if let WindowEvent::CursorMoved { position, ..} = event {
         self.mouse_screen_pos = Vector2::new(position.x as f32, position.y as f32)
      }

      if let WindowEvent::MouseInput { button, state,  ..} = event {
         match state {
            ElementState::Pressed => {
               self.mouse_currently_pressed.insert(*button);
               self.mouse_just_pressed.insert(*button);
            }
            ElementState::Released => {
               self.mouse_just_pressed.remove(button);
            }
         }
      }

   }

   pub fn is_key_pressed(&self, key: KeyCode) -> bool {
      self.currently_pressed.contains(&key)
   }

   pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
      self.just_pressed.contains(&key)
   }

   pub fn is_mouse_key_pressed(&self, button: MouseButton) -> bool {
      self.mouse_currently_pressed.contains(&button)
   }

   pub fn is_mouse_key_just_pressed(&self, button: MouseButton) -> bool {
      self.mouse_just_pressed.contains(&button)
   }


   pub fn reset(&mut self) {
      self.just_pressed.clear();
   }
}