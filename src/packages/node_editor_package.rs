use std::fs;
use std::path::Path;
use cgmath::{Vector3, Zero};
use egui::{Context, Frame, Label, Pos2, Rect, Sense, Style, Ui, Vec2};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use crate::inbuilt::setup::Setup;
use crate::path_tracer::path_tracer::PathTracer;
use crate::utility::structs::StorageTexturePackage;

pub struct NodeEditor {
   pub max_size: Vec2,
   pub nodes: Vec<Node>,

   save_name: String,
}

impl NodeEditor {
   pub fn new() -> Self {
      let nodes = vec![
         Node::new(Pos2::new(50.0, 50.0), Vec2::new(100.0, 50.0), "Node 1".to_string()),
         Node::new(Pos2::new(200.0, 50.0), Vec2::new(100.0, 50.0), "Node 2".to_string()),
      ];

      Self {
         max_size: Vec2::new(600.0, 600.0),
         nodes,

         save_name: "Name".to_string(),
      }
   }

   pub fn ui<'a>(&mut self, ui: &Context, path_tracer: &mut PathTracer, storage_texture_package: &StorageTexturePackage, setup: &Setup, resized: &'a mut bool) {
      let mut changed = false;

      egui::Window::new("Map editor")
          .default_open(false)
          .resizable(true)
          .default_size(Vec2::new(100.0, 100.0))
          .max_size(self.max_size)
          .frame(Frame::window(&Style::default()))
          .show(&ui, |ui| {
             if ui.button("add").clicked() {
                self.nodes.push(Node::new(Pos2::new(50.0, 50.0), Vec2::new(100.0, 50.0), format!("num={}", self.nodes.len())));
                changed = true;
             }

             ui.add(egui::TextEdit::singleline(&mut self.save_name)
                 .desired_width(75.0)
             );

             if ui.button("save").clicked() {
                let serialised = to_string(&self.nodes).unwrap();
                let name = &self.save_name;

                fs::write(&Path::new(format!("src/packages/maps/{name}.json").as_str()), serialised).unwrap();
             }

             if ui.button("load").clicked() {
                let name = &self.save_name;
                let data  = fs::read_to_string(&Path::new(format!("src/packages/maps/{name}.json").as_str()));

                if let Ok(unwrapped_data) = data {
                   self.nodes = from_str(&unwrapped_data).unwrap();
                   changed = true;
                } else {
                   self.save_name = "!!not found".to_string();
                }
             }

             if ui.button("Files").contains_pointer() {
                let entries = fs::read_dir("src/packages/maps/").unwrap();

                for entry in entries {
                   let entry = entry.unwrap();
                   let path = entry.path();
                   if path.is_file() {
                      let mut name = path.file_name().unwrap();
                      let mut string_name = name.to_str().unwrap().to_string();
                      string_name = string_name.replace(".json", "");
                      ui.add(Label::new(
                         string_name
                      ));
                   }
                }
             }

             ui.allocate_space(ui.available_size());

             let mut kill = None;
             for (i, node) in self.nodes.iter_mut().enumerate() {
                let original_node = node.clone();
                if node.draw(ui, ui.min_rect().min, self.max_size) {
                   kill = Some(i)
                }

                // todo: detects movement as a change
                if *node != original_node {
                   changed = true
                }

             }
             if let Some(index) = kill { self.nodes.remove(index); changed = true; }
          });

      if changed {
         path_tracer.remake_pipeline(setup, storage_texture_package, self.generate_map());
         *resized = true;
      }
   }

   pub fn generate_map(&mut self) -> String {
      let mut map = String::new();

      if self.nodes.len() == 0 {
         return r#"
            Hit map(vec3 pos) {{ return Hit(10000.0); }}
         "#.to_string()
      }

      map.push_str("Hit map(vec3 pos) { \n");
      map.push_str(format!("Hit[{}] shapes;\n", self.nodes.len()).as_str());
      map.push_str("vec3 tr;\n");

      for (i, node) in self.nodes.iter().enumerate() {
         map.push_str(node.map(i).as_str())
      }

      map.push_str(format!(r#"
      Hit back = Hit(10000.0);
      for (int i = 0; i < {}; i ++) {{
         back = opUnion(back, shapes[i]);
      }}

      return back;
   }}

      "#, self.nodes.len(), ).as_str());

      map
   }

   #[allow(dead_code)]
   fn save_scene(&mut self) {} // todo

   #[allow(dead_code)]
   fn load_scenes(&mut self) {} // todo

}


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Shapes {
   Sphere,
   Cube,
}
impl Shapes {
   pub fn show_enum_dropdown(ui: &mut Ui, selected: &mut Shapes) {
      egui::ComboBox::from_label("")
          .selected_text(format!("{:?}", selected))
          .show_ui(ui, |ui| {
             ui.selectable_value(selected, Shapes::Sphere, "Sphere");
             ui.selectable_value(selected, Shapes::Cube, "Cube");
          });
   }

   pub fn size_options(&self, ui: &mut Ui, size: &mut (f32, f32, f32)) {
      match self {
         Shapes::Sphere => {
            ui.add(egui::DragValue::new(&mut size.0).speed(0.01).clamp_range(0.01..=100.0).prefix("Size: "));
         }
         Shapes::Cube => {
            ui.add(egui::DragValue::new(&mut size.0).speed(0.01).clamp_range(0.01..=100.0).prefix("L: "));
            ui.add(egui::DragValue::new(&mut size.1).speed(0.01).clamp_range(0.01..=100.0).prefix("W: "));
            ui.add(egui::DragValue::new(&mut size.2).speed(0.01).clamp_range(0.01..=100.0).prefix("H: "));
         }
      }
   }
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
   screen_position: (f32, f32),
   screen_size: (f32, f32),
   title: String,

   shape: Shapes,
   scale: f32,
   size: (f32, f32, f32),
   position: (f32, f32, f32),
   rotation: (f32, f32, f32),
}
impl Node {
   pub fn new(screen_position: Pos2, size: Vec2, title: String) -> Self {
      Self {
         screen_position: (screen_position.x, screen_position.y),
         screen_size: size.into(),
         title,

         ..Default::default()
      }
   }

   pub fn contents<'a>(&'a mut self, back: &'a mut bool) -> impl FnMut(&mut Ui) + '_ {
      let contents = |ui: &mut Ui| {
         egui::CollapsingHeader::new(&self.title)
             .default_open(true)
             .show(ui, |ui| {

                if ui.button("remove").clicked() {
                   *back = true;
                }

                if ui.button("reset").clicked() {
                   let title = self.title.clone();
                   *self = Self {screen_position: self.screen_position, screen_size: self.screen_size, title, ..Self::default() };
                }

                Shapes::show_enum_dropdown(ui, &mut self.shape);

                egui::CollapsingHeader::new("size")
                    .default_open(false)
                    .show(ui, |ui| {
                       if ui.button("reset").clicked() { self.scale = Self::default().scale; self.size = Self::default().size }
                       ui.add(egui::DragValue::new(&mut self.scale).speed(0.001).clamp_range(0.001..=100.0).prefix("Scale: "));
                       self.shape.size_options(ui, &mut self.size);
                    });

                egui::CollapsingHeader::new("position")
                    .default_open(false)
                    .show(ui, |ui| {
                       if ui.button("reset").clicked() { self.position = Self::default().position; }

                       ui.add(egui::DragValue::new(&mut self.position.0).speed(0.01).clamp_range(-100.0..=100.0).prefix("X: "));
                       ui.add(egui::DragValue::new(&mut self.position.1).speed(0.01).clamp_range(-100.0..=100.0).prefix("Y: "));
                       ui.add(egui::DragValue::new(&mut self.position.2).speed(0.01).clamp_range(-100.0..=100.0).prefix("Z: "));
                    });

                egui::CollapsingHeader::new("rotation")
                    .default_open(false)
                    .show(ui, |ui| {
                       if ui.button("reset").clicked() { self.rotation = Self::default().rotation; }

                       ui.add(egui::DragValue::new(&mut self.rotation.0).speed(0.01).clamp_range(-1000.0..=1000.0).prefix("X: "));
                       ui.add(egui::DragValue::new(&mut self.rotation.1).speed(0.01).clamp_range(-1000.0..=1000.0).prefix("Y: "));
                       ui.add(egui::DragValue::new(&mut self.rotation.2).speed(0.01).clamp_range(-1000.0..=1000.0).prefix("Z: "));
                    });

             });
      };

      contents
   }

   pub fn draw(&mut self, ui: &mut Ui, window_pos: Pos2, max_size: Vec2) -> bool {
      let rect = Rect::from_min_size(window_pos + Vec2::from(self.screen_position), Vec2::from(self.screen_size));
      let response = ui.allocate_rect(rect, Sense::click_and_drag());

      if response.dragged() {
         self.screen_position.0 += response.drag_delta().x;
         self.screen_position.1 += response.drag_delta().y;

         if self.screen_position.0 < 0.0 {self.screen_position.0 = 0.0}
         if self.screen_position.1 < 0.0 {self.screen_position.1 = 0.0}

         if self.screen_position.0 > max_size.x {self.screen_position.0 = max_size.x}
         if self.screen_position.1 > max_size.y {self.screen_position.1 = max_size.y}
      }

      let mut back = false;

      ui.allocate_ui_at_rect(rect, |ui| {
         ui.group(self.contents(&mut back));
      });

      back
   }

   pub fn map(&self, index: usize) -> String {
      let mut back = String::new();

      let shape_option = match self.shape {
         Shapes::Sphere => {"sdSphere"}
         Shapes::Cube => {"sdCube"}
      };

      let size_option = match self.shape {
         Shapes::Sphere => {format!("{}", self.size.0)}
         Shapes::Cube => {format!("vec3({}, {}, {})", self.size.0, self.size.1, self.size.2)}
      };

      let p = self.position;
      let r = self.rotation;

      let pos = if self.position != Vector3::zero().into() { format!("tr = move(tr, vec3({}, {}, {}));", p.0, p.1, p.2) } else {"//pos".to_string()};
      let rot = if self.rotation != Vector3::zero().into() { format!("tr = rot3D(tr, vec3({}, {}, {}));",  r.0, r.1, r.2) } else {"//rot".to_string()};
      back.push_str(format!(r#"
      tr = pos;
      {pos}
      {rot}
      shapes[{}] = Hit(
         {}(tr * {}, {}) / {}
      );
      "#,  index, shape_option, 1.0 / self.scale, size_option, 1.0 / self.scale).as_str());

      back
   }
}

impl Default for Node {
   fn default() -> Self {
      Self {
         screen_position: (0.0, 0.0),
         screen_size: (0.0, 0.0),
         title: "default".to_string(),

         shape: Shapes::Sphere,
         scale: 1.0,
         size: Vector3::new(1.0, 1.0, 1.0).into(),
         position: Vector3::new(0.0, 0.0, 0.0).into(),
         rotation: Vector3::new(0.0, 0.0, 0.0).into(),
      }
   }
}