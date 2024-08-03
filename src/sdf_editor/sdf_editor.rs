use std::ops::{RangeInclusive};
use egui::{ComboBox, Context, DragValue, Frame, Label, menu, ScrollArea, Style, Ui, Window};
use serde::{Deserialize, Serialize};
use crate::enum_egui_dropdown;


const S1: f32 = 0.001;
const S2: f32 = 0.01;
const S3: f32 = 0.1;


#[derive(Debug, Serialize, Deserialize)]
pub struct SDFEditor {
   header_unions: Vec<Union>,
   header_shapes: Vec<Shape>,
}
impl SDFEditor {
   pub fn new() -> Self {
      let header_unions = vec![Union::new()];
      let header_shapes = vec![Shape::new()];

      Self {
         header_unions,
         header_shapes
      }
   }

   pub fn update(&mut self) {}

   pub fn ui(&mut self, context: &Context) {
      let window = Window::new("SDF Editor")
          .resizable(true)
          .frame(Frame::window(&Style::default()));

      window.show(context, |ui| {

         self.menubar(ui);

         ui.group(|ui| {
            ScrollArea::both()
                .show(ui, |ui| {
                   ui.set_min_size(ui.available_size());
                   self.editor_contents(ui);
                });
         });

      });
   }


   // related private functions //
   fn menubar(&mut self, ui: &mut Ui) {
      menu::bar(ui, |ui| {
         ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
               println!("Clicked Open");
            }
         });

         ui.menu_button("Test", |ui| {
            if ui.button("Test Button").clicked() {
               println!("Test Button");
            }
         });

         ui.menu_button("Add", |ui| {
            if ui.button("Union").clicked() {
               self.header_unions.push(Union::new());
            }
            if ui.button("Shape").clicked() {
               self.header_shapes.push(Shape::new());
            }
         });

      });
   }

   fn editor_contents(&mut self, ui: &mut Ui) {
      // unions
      let mut exucute = None;
      for (i, union) in self.header_unions.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               union.ui(ui);

               if ui.button("Delete").clicked() {
                  exucute = Some(i);
               }

            });
         });
      }
      if let Some(index) = exucute {
         self.header_unions.remove(index);
      }

      // shapes
      let mut exucute = None;
      for (i, shape) in self.header_shapes.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               shape.ui(ui);

               if ui.button("Delete").clicked() {
                  exucute = Some(i);
               }

            });
         });
      }
      if let Some(index) = exucute {
         self.header_shapes.remove(index);
      }
   }
}

///////////
// Nodes //
///////////
#[derive(Debug, Serialize, Deserialize)]
pub struct Union {
   transform: Transform,

   children_unions: Vec<Union>,
   children_shapes: Vec<Shape>
}
impl Union {
   pub fn new() -> Self {
      Self {
         transform: Transform::new(),

         children_unions: vec![],
         children_shapes: vec![],
      }
   }

   pub fn ui(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         egui::CollapsingHeader::new(format!("Union : {}", "Name"))
             .show(ui, |ui| {
                self.contents(ui);
             });
      });
   }

   fn contents(&mut self, ui: &mut Ui) {
      egui::CollapsingHeader::new("Settings")
          .show(ui, |ui| {

             egui::CollapsingHeader::new("Bounding Area")
                 .show(ui, |ui| {
                    ui.add(Label::new("Not implemented"))
                 });

             self.transform.ui(ui);

          });

      egui::CollapsingHeader::new("Child nodes")
          .show(ui, |ui| {
             self.display_children(ui);
          });

   }

   fn display_children(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         if ui.button("Add Union").clicked() {
            self.children_unions.push(Union::new());
         }
         if ui.button("Add Shape").clicked() {
            self.children_shapes.push(Shape::new());
         }
      });


      // unions
      let mut exucute = None;
      for (i, union) in self.children_unions.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               union.ui(ui);

               if ui.button("Delete").clicked() {
                  exucute = Some(i);
               }

            });
         });
      }
      if let Some(index) = exucute {
         self.children_unions.remove(index);
      }

      // shapes
      let mut exucute = None;
      for (i, shape) in self.children_shapes.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               shape.ui(ui);

               if ui.button("Delete").clicked() {
                  exucute = Some(i);
               }

            });
         });
      }
      if let Some(index) = exucute {
         self.children_shapes.remove(index);
      }


   }
}




enum_egui_dropdown!(Shapes, Sphere, Cube, Plane);
#[derive(Debug, Serialize, Deserialize)]
pub struct Shape {
   transform: Transform,
   material: Material,
   current_shape: Shapes,
}
impl Shape {
   pub fn new() -> Self {
      Self {
         transform: Transform::new(),
         material: Material::new(),
         current_shape: Shapes::Sphere,
      }
   }

   pub fn ui(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         egui::CollapsingHeader::new(format!("Shape : {}", "Name"))
             .show(ui, |ui| {
                self.contents(ui);
             });
      });
   }

   fn contents(&mut self, ui: &mut Ui) {
      self.current_shape.dropdown(ui);

      egui::CollapsingHeader::new("Bounding Area")
        .show(ui, |ui| {
           ui.add(Label::new("Not implemented"))
        });

      self.transform.ui(ui);

      self.material.ui(ui);
   }
}





/////////////////////
// Data structures //
/////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct Transform {
   position: V3,
   rotation: V3,
   scale: Float,
}
impl Transform {
   pub fn new() -> Self {
      Self {
         position: V3::xyz("Position", S2),
         rotation: V3::xyz("Rotation", S1),
         scale: Float::zero_plus("Scale", S2),
      }
   }
   pub fn ui(&mut self, ui: &mut Ui) {
      egui::CollapsingHeader::new("Transform")
          .show(ui, |ui| {
             self.position.separate_values(ui);
             self.rotation.separate_values(ui);
             self.scale.ui(ui);
          });
   }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
   color: V3,

   brightness: Float,
   light_col: V3,

   specular_chance: Float,
   specular_color: V3,

   roughness: Float,

   ior: Float,
   refract_chance: Float,
   refract_roughness: Float,
   refract_color: V3,

}
impl Material {
   pub fn new() -> Self {
      Self {
         color: V3::rgb("Surface Color"),

         brightness: Float::zero_plus("Brightness", S2),
         light_col: V3::rgb("Light Color    "),

         specular_chance: Float::percent("Spec chance", S1),
         specular_color: V3::rgb("Spec color"),

         roughness: Float::zero_plus("Roughness", S1),

         ior: Float::inv("IOR", S1),
         refract_chance: Float::percent("Refract chance", S1),
         refract_roughness: Float::inv("Refract roughness", S1),
         refract_color: V3::rgb("Refract color")
      }
   }

   pub fn ui(&mut self, ui: &mut Ui) {
      const SPACING: f32 = 0.0;
      egui::CollapsingHeader::new("Material")
          .show(ui, |ui| {
             self.color.color_ui(ui);

             ui.add_space(SPACING);
             self.brightness.ui(ui);
             self.light_col.color_ui(ui);

             ui.add_space(SPACING);
             self.specular_chance.ui(ui);
             self.specular_color.color_ui(ui);

             ui.add_space(SPACING);
             self.roughness.ui(ui);

             ui.add_space(SPACING);
             self.ior.ui(ui);
             self.refract_chance.ui(ui);
             self.refract_roughness.ui(ui);
             self.refract_color.color_ui(ui);
          });
   }
}

////////////////
// primitives //
////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct Float {
   val: f32,
   range: RangeInclusive<f32>,
   speed: f32,
   name: String,
}
impl Float {
   pub fn inv(name: &str, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range: -f32::MAX..=f32::MAX,
         speed,
      }
   }

   pub fn one_plus(name: &str, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range: 1.0..=f32::MAX,
         speed,
      }
   }

   pub fn zero_plus(name: &str, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range: 0.0..=f32::MAX,
         speed,
      }
   }

   pub fn percent(name: &str, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range: 0.0..=1.0,
         speed,
      }
   }

   pub fn with_range(name: &str, range: RangeInclusive<f32>, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range,
         speed,
      }
   }

   pub fn ui(&mut self, ui: &mut Ui) {
      ui.add(
         DragValue::new(&mut self.val)
             .speed(self.speed).clamp_range(self.range.clone())
             .prefix(format!("{}: ", self.name))
      );

   }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct V3 {
   x: Float,
   y: Float,
   z: Float,
   name: String,
}
impl V3 {
   pub fn xyz(name: &str, speed: f32) -> Self {
      Self {
         x: Float::inv("X", speed),
         y: Float::inv("Y", speed),
         z: Float::inv("Z", speed),
         name: name.to_string(),
      }
   }

   pub fn rgb(name: &str) -> Self {
      Self {
         x: Float::inv("R", 1.0),
         y: Float::inv("G", 1.0),
         z: Float::inv("B", 1.0),
         name: name.to_string(),
      }
   }
   pub fn separate_values(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         ui.add(Label::new(format!("{}", self.name)));
         ui.horizontal(|ui| {
            self.x.ui(ui);
            self.y.ui(ui);
            self.z.ui(ui);
         });
      });
   }

   pub fn color_ui(&mut self, ui: &mut Ui) {
      let mut col = [self.x.val, self.y.val, self.z.val];
      ui.horizontal(|ui| {
         ui.add(Label::new(format!("{}", self.name)));
         ui.color_edit_button_rgb(&mut col);
      });

      self.x.val = col[0]; self.y.val = col[1]; self.z.val = col[2];
   }
}

