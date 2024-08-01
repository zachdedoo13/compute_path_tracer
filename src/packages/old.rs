
use egui::{Color32, Context, Frame, Label, Pos2, Rect, Sense, Stroke, Style, TextBuffer, Ui, Vec2, Window};
use serde::{Deserialize, Serialize};
use Node::Shape;


#[derive(Debug, Default)]
struct LineState {
   dragging: bool,
   origin: usize,
   is_something_below: bool,
   was_something_below: bool,
   detected_below: usize,
   started: bool,
   released: bool,
}

pub struct NewNodeEditor {
   nodes: Vec<Node>,
   current_size: Vec2,
   line_state: LineState,
}
impl NewNodeEditor {
   pub fn new() -> Self {

      Self {
         nodes: vec![Shape(ShapeNode::new()), Shape(ShapeNode::new())],
         current_size: Vec2::new(400.0, 400.0),
         line_state: Default::default(),
      }
   }

   pub fn ui(&mut self, context: &Context) {
      let window = Window::new("NewNodeEditor")
          .default_open(false)
          .resizable(true)
          .fixed_size(self.current_size)
          .frame(Frame::window(&Style::default()));

      window.show(&context, |ui| {
         let anchor = ui.min_rect().min;

         self.line_state.was_something_below = false;
         for (i, node) in self.nodes.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
               node.show(ui, anchor.to_vec2(), &mut self.line_state);
               if self.line_state.is_something_below {
                  self.line_state.detected_below = i;
                  self.line_state.was_something_below = true;
               }
               if self.line_state.started {
                  self.line_state.started = false;
                  self.line_state.origin = i;
               }
               self.line_state.is_something_below = false;
            });
         }

         if self.line_state.released {
            println!("{:?}", self.line_state);
            self.line_state.released = false;
            let origin = self.line_state.origin;
            let target = self.line_state.detected_below;

            let node = self.nodes[target].clone();
            if self.nodes[origin].attach(node) {
               self.nodes.remove(target);
            }
         }



         ui.allocate_space(ui.available_size());
      });

   }
}


#[derive(Clone, Debug)]
struct EditorData {
   position: Pos2,
   size: Vec2,
}
impl EditorData {
   fn new(size: Vec2) -> Self {
      Self {
         position: Pos2::new(0.0, 0.0),
         size,
      }
   }
}

#[derive(Clone, Debug)]
enum Node {
   Shape(ShapeNode),
   Placeholder,
}
impl Node {
   fn show(&mut self, ui: &mut Ui, anchor: Vec2, line_state: &mut LineState) {
      match self {
         Shape(shape) => { shape.show(ui, anchor, line_state); }
         _ => {}
      }
   }

   fn attach(&mut self, node: Node) -> bool {
      match self {
         Shape(shape) => { shape.attach(node) }
         _ => {false}
      }
   }
}


const HANDLE_POS: Pos2 = Pos2::new(50.0, 15.0);
#[derive(Clone, Debug)]
struct ShapeNode {
   editor_data: EditorData,
   sub_node: Option<Box<Node>>,
}
impl ShapeNode {
   fn new() -> Self {
      Self {
         editor_data: EditorData::new(Vec2::new(75.0, 25.0)),
         sub_node: None,
      }
   }
   fn show(&mut self, ui: &mut Ui, anchor: Vec2,  line_state: &mut LineState) {
      let handle = Rect::from_min_size((anchor + self.editor_data.position.to_vec2()).to_pos2(), self.editor_data.size);

      ui.allocate_ui_at_rect(handle, |ui| {
         self.contents(ui);
      });

      ui.painter().rect_filled(handle, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20));
      let mut response = ui.allocate_rect(handle, Sense::click_and_drag());
      if response.dragged() {
         self.editor_data.position.x += response.drag_delta().x;
         self.editor_data.position.y += response.drag_delta().y;
      }


      let circle_pos = HANDLE_POS + handle.min.to_vec2();
      ui.painter().circle_filled(circle_pos, 7.0, Color32::from_rgb(255, 0, 0));
      let mut cable_response = ui.allocate_rect(Rect::from_center_size(circle_pos, Vec2::new(7.0, 7.0)), Sense::click_and_drag());

      if cable_response.hovered() {
         line_state.is_something_below = true;
         ui.painter().circle_filled(circle_pos, 3.0, Color32::from_rgb(255, 255, 255));
      }

      if cable_response.drag_stopped() {
         line_state.released = true;
      }

      if cable_response.dragged() {
         ui.painter().line_segment([circle_pos, get_mouse_position(ui.ctx())], Stroke::new(10.0, Color32::WHITE));
         line_state.started = true;
      }

      if let Some(node) = &self.sub_node {
         println!("{:?}", node)
      }
   }

   fn attach(&mut self, node: Node) -> bool {
      match node {
         Shape(_) => {}
         _ => { return false; }
      }

      self.sub_node = Some(Box::new(node));
      true
   }

   fn contents(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.add(Label::new(format!("{:?}", ui.id())).selectable(false));
         });
         let mut mat = Material::default();
         mat.ui(ui);
      });
   }
}



#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
   color: (f32, f32, f32),
   light: (f32, f32, f32),

   // only in rust
   light_strength: f32,

   spec: f32,
   spec_col: (f32, f32, f32),
   roughness: f32,

   ior: f32,
   refraction_chance: f32,
   refraction_roughness: f32,
   refraction_color: (f32, f32, f32),
}
impl Material {
   pub fn ui(&mut self, big_ui: &mut Ui) {
      egui::CollapsingHeader::new("Material")
          .default_open(false)
          .show(big_ui, |ui| {
             if ui.button("reset").clicked() { *self = Self::default() }

             let spacing = 10.0;

             // color
             display_color(&mut self.color, "Color ", ui);

             ui.add_space(spacing);
             // light
             display_color(&mut self.light, "Light ", ui);
             ui.add(egui::DragValue::new(&mut self.light_strength).speed(0.01).clamp_range(0.0..=100.0).prefix("Light strength: "));

             ui.add_space(spacing);

             // spec
             ui.add(egui::DragValue::new(&mut self.spec).speed(0.001).clamp_range(0.0..=1.0).prefix("Spec %: "));
             display_color(&mut self.spec_col, "Spec col  ", ui);

             ui.add_space(spacing);

             // rough
             ui.add(egui::DragValue::new(&mut self.roughness).speed(0.001).clamp_range(0.0..=5.0).prefix("Roughness: "));

             // refraction
             ui.add(egui::DragValue::new(&mut self.ior).speed(0.001).clamp_range(0.0..=5.0).prefix("IOR: "));
             ui.add(egui::DragValue::new(&mut self.refraction_chance).speed(0.001).clamp_range(0.0..=5.0).prefix("Refraction %: "));
             ui.add(egui::DragValue::new(&mut self.refraction_roughness).speed(0.001).clamp_range(0.0..=5.0).prefix("Refract Roughness: "));

             display_color(&mut self.refraction_color, "Refract col  ", ui);

          });
   }
}

impl Default for Material {
   fn default() -> Self {
      Self {
         color: (1.0, 1.0, 1.0),
         light: (0.0, 0.0, 0.0),

         light_strength: 0.0,

         spec: 0.0,
         spec_col: (0.0, 0.0, 0.0),
         roughness: 0.0,

         ior: 0.0,
         refraction_chance: 0.0,
         refraction_roughness: 0.0,
         refraction_color: (0.0, 0.0, 0.0),
      }
   }
}

fn display_color(the: &mut (f32, f32, f32), text: &str, ui: &mut Ui) {
   let mut in_the = Color32::from_rgb(
      (the.0 * 255.0) as u8,
      (the.1 * 255.0) as u8,
      (the.2 * 255.0) as u8,
   );

   ui.horizontal(|ui| {
      ui.label(text);
      ui.color_edit_button_srgba(&mut in_the);
   });

   *the = (
      in_the.r() as f32 / 255.0,
      in_the.g() as f32 / 255.0,
      in_the.b() as f32 / 255.0,
   )
}

fn get_mouse_position(ctx: &Context) -> Pos2 {
   ctx.input(|i| i.pointer.hover_pos().unwrap_or(Pos2::new(0.0, 0.0)))
}