use std::cmp::PartialEq;
use std::fmt::format;
use std::fs;
use std::ops::{Add, RangeInclusive};
use std::time::Instant;
use egui::{Color32, ComboBox, Context, DragValue, Frame, Label, menu, ScrollArea, Style, TextEdit, Ui, Window};
use rand::{random, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};

const S1: f32 = 0.001;
const S2: f32 = 0.01;
const S3: f32 = 0.1;

const MAP_PATH: &str = "data/maps/";


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SDFEditor {
   header_unions: Vec<Union>,
   header_shapes: Vec<Shape>,
   rec_update: RecUpdate,

   save_name: String,
}
impl SDFEditor {
   pub fn new() -> Self {

      // init folders this struct uses
      fs::create_dir_all(MAP_PATH).expect("Failed to create dir");


      let header_unions = vec![Union::new()];
      let header_shapes = vec![Shape::new()];

      Self {
         header_unions,
         header_shapes,
         rec_update: RecUpdate::set_true(),

         save_name: String::new(),
      }
   }

   pub fn update(&mut self) {
      if self.rec_update.queue_compile {
         let map = self.compile();
      }

      if self.rec_update.queue_update {
         let data = self.data_update();
      }


      self.rec_update.reset();
   }

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
            ui.menu_button("Open", |ui| {
               self.open_ui(ui);
            });

            ui.menu_button("Save", |ui| {
               ui.add(TextEdit::singleline(&mut self.save_name)
                   .hint_text("Save name")
                   .desired_width(75.0)
               );
               if ui.button("Save as").clicked() {
                  self.save(&self.save_name);
               }
            });

            if ui.button("Force compile").clicked() {
               self.rec_update.compile();
            }
            if ui.button("Force update").clicked() {
               self.rec_update.update();
            }
         });

         ui.menu_button("Add", |ui| {
            if ui.button("Union").clicked() {
               self.header_unions.push(Union::new());
               self.rec_update.both();
            }
            if ui.button("Shape").clicked() {
               self.header_shapes.push(Shape::new());
               self.rec_update.both();
            }
         });

      });
   }

   fn editor_contents(&mut self, ui: &mut Ui) {
      let mut i = 0;
      // unions
      let mut exucute = None;
      for (temp_i, union) in self.header_unions.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
               union.ui(ui, &mut self.rec_update);

               if ui.button("Delete").clicked() {
                  exucute = Some(temp_i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.header_unions.remove(index);
         self.rec_update.both();
      }

      // shapes
      let mut exucute = None;
      for (temp_i, shape) in self.header_shapes.iter_mut().enumerate()  {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               shape.ui(ui, &mut self.rec_update);

               if ui.button("Delete").clicked() {
                  exucute = Some(temp_i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.header_shapes.remove(index);
         self.rec_update.both();
      }
   }

   fn save(&self, name: &String) {
      println!("Saving as {name}");
      let file_name = format!("{name}.json");
      let path = format!("{MAP_PATH}{file_name}");

      let serialized = to_string_pretty(self).unwrap();

      fs::write(&path, serialized).expect(format!("Failed to write to {path}").as_str());
   }

   fn open_ui(&mut self, ui: &mut Ui) {
      let files = fs::read_dir(MAP_PATH).expect("Failed to read map path");

      files.for_each(|entry| {
         let file = entry.expect("Invalid entry");
         let name = file.file_name().into_string().unwrap();

         if ui.button(&name).clicked() {
            self.open(&name);
         }
      });
   }

   fn open(&mut self, name: &String) {
      let contents = fs::read_to_string(format!("{MAP_PATH}{name}")).expect("Failed to read file");
      let deserialize: SDFEditor = from_str(&contents).expect("Failed to deserialize");

      *self = deserialize;
   }



   // compiler functions
   pub fn compile(&mut self) -> String {
      let st = Instant::now();
      println!("Compiling {}", char::from(random::<u8>().to_ascii_uppercase()) );

      let mut comp_data = CompData { union_index: 0, union_depth: 0 };

      let mut out = String::new();

      out.push_str(r#"
      #define MAXHIT Hit(10000.0, MDEF)

      Hit map(vec3 pu0) {

      "#);


      for union in self.header_unions.iter() {
         out.push_str(union.compile(&mut comp_data).as_str());
      }










      println!("Compiled in {:?}", st.elapsed());

      println!("{out}");
      String::new()
   }

   pub fn data_update(&mut self) -> Vec<f32> {
      let st = Instant::now();

      println!("Updating {}", char::from(random::<u8>().to_ascii_uppercase()) );


      println!("Updated in {:?}", st.elapsed());
      vec![]
   }
}

struct CompData {
   union_index: u32,
   union_depth: u32,
}
impl CompData {
   pub fn new() -> Self {
      Self {
         union_index: 0,
         union_depth: 0,
      }
   }

   pub fn add_union(&mut self) {
      self.union_index += 1;
   }

   pub fn init_union(&mut self) -> String {
      format!("Hit u{} = MAXHIT", self.union_index)
   }

   pub fn get_union(&self) -> String {
      format!("u{}", self.union_index)
   }

   pub fn get_transform(&self) -> String {
      format!("pu{}", self.union_index)
   }
}


///////////
// Nodes //
///////////
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Union {
   name: String,
   transform: Transform,

   children_unions: Vec<Union>,
   children_shapes: Vec<Shape>,
}
impl Union {
   pub fn new() -> Self {
      Self {
         name: "Union".to_string(),
         transform: Transform::new(),

         children_unions: vec![],
         children_shapes: vec![],
      }
   }

   pub fn ui(&mut self, ui: &mut Ui, rec_update: &mut RecUpdate) {
      let frame = Frame::group(&Style::default())
          .fill(Color32::from_rgb(50, 0, 0));

      frame.show(ui, |ui| {
         egui::CollapsingHeader::new(format!("{}", self.name))
             .id_source(1)
             .show(ui, |ui| {
                self.contents(ui, rec_update);
             });
      });
   }

   fn contents(&mut self, ui: &mut Ui, rec_update: &mut RecUpdate) {
      egui::CollapsingHeader::new("Settings")
          .show(ui, |ui| {

             ui.add(TextEdit::singleline(&mut self.name)
                 .hint_text("Enter name")
                 .desired_width(75.0)
             );

             egui::CollapsingHeader::new("Bounding Area")
                 .show(ui, |ui| {
                    ui.add(Label::new("Not implemented"))
                 });

             let check = self.transform.clone();
             self.transform.ui(ui);
             if self.transform != check { rec_update.update(); }

          });

      egui::CollapsingHeader::new("Child nodes")
          .show(ui, |ui| {
             self.display_children(ui, rec_update);
          });

   }

   fn display_children(&mut self, ui: &mut Ui, rec_update: &mut RecUpdate) {
      ui.horizontal(|ui| {
         if ui.button("Add Union").clicked() {
            self.children_unions.push(Union::new());
            rec_update.both();
         }
         if ui.button("Add Shape").clicked() {
            self.children_shapes.push(Shape::new());
            rec_update.both();
         }
      });

      let mut i = 0;
      // unions
      let mut exucute = None;
      for union in self.children_unions.iter_mut() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
               union.ui(ui, rec_update);

               if ui.button("Delete").clicked() {
                  exucute = Some(i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.children_unions.remove(index);
         rec_update.both();
      }

      // shapes
      let mut exucute = None;
      for shape in self.children_shapes.iter_mut() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               shape.ui(ui, rec_update);

               if ui.button("Delete").clicked() {
                  exucute = Some(i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.children_shapes.remove(index);
         rec_update.both();
      }


   }

   fn compile(&self, comp_data: &mut CompData) -> String {
      let mut out = String::new();
      comp_data.add_union();

      out.push_str(format!("{}\n", comp_data.init_union().as_str()).as_str());

      out.push_str(format!("{}", self.transform.compile(&comp_data.get_transform())).as_str());



      out
   }
}



#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Shapes {
   Sphere(Float),
   Cube(V3),
   Plane,
}
impl Shapes {
   pub fn dropdown(&mut self, ui: &mut Ui) {
      ComboBox::from_label("")
          .selected_text(self.text())
          .show_ui(ui, |ui| {
             ui.selectable_value(self, Shapes::Sphere(Float::inv("Size", S2, Some(1.0))), stringify!( Sphere ));
             ui.selectable_value(self, Shapes::Cube(V3::xyz("Size", S2, Some(1.0))), stringify!( Cube ));
             ui.selectable_value(self, Shapes::Plane, stringify!( Plane ));
          });
   }
   fn text(&self) -> &str {
      match self {
         Shapes::Sphere(_) => {"Sphere"}
         Shapes::Cube(_) => {"Cube"}
         Shapes::Plane => {"Plane"}
      }
   }
}
impl Default for Shapes {
   fn default() -> Self {
      Shapes::Sphere(Float::inv("Size", S1, Some(1.0)))
   }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shape {
   transform: Transform,
   material: Material,
   current_shape: Shapes,
   name: String,
}

impl Shape {
   pub fn new() -> Self {
      Self {
         transform: Transform::new(),
         material: Material::new(),
         current_shape: Shapes::default(),
         name: "Shape".to_string(),
      }
   }

   pub fn ui(&mut self, ui: &mut Ui, rec_update: &mut RecUpdate) {
      let frame = Frame::group(&Style::default())
          .fill(Color32::from_rgb(0, 0, 50));

      frame.show(ui, |ui| {
         egui::CollapsingHeader::new(format!("{}", self.name))
             .id_source(1)
             .show(ui, |ui| {

                ui.add(TextEdit::singleline(&mut self.name)
                    .hint_text("Enter name")
                    .desired_width(75.0)
                );

                self.contents(ui, rec_update);
             });
      });
   }

   fn contents(&mut self, ui: &mut Ui, rec_update: &mut RecUpdate) {
      let check = self.current_shape.clone();
      self.current_shape.dropdown(ui);
      if self.current_shape != check { rec_update.compile() }

      let check = self.current_shape.clone();
      self.shape_settings(ui);
      if self.current_shape != check { rec_update.update() }

      egui::CollapsingHeader::new("Bounding Area")
        .show(ui, |ui| {
           ui.add(Label::new("Not implemented"))
        });

      let check = self.transform.clone();
      self.transform.ui(ui);
      if self.transform != check { rec_update.update() }

      let check = self.material.clone();
      self.material.ui(ui);
      if self.material != check { rec_update.update() }
   }

   fn shape_settings(&mut self, ui: &mut Ui) {
      egui::CollapsingHeader::new("Shape Settings")
          .show(ui, |ui| {
             match &mut self.current_shape {

                Shapes::Sphere(size) => {
                   size.ui(ui);
                }

                Shapes::Cube(size) => {
                  size.separate_values(ui);
                }

                Shapes::Plane => {
                  ui.label("No settings");
                }

             }
          });
   }
}


/////////////////////
// Data structures //
/////////////////////
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Transform {
   position: V3,
   rotation: V3,
   scale: Float,
}
impl Transform {
   pub fn new() -> Self {
      Self {
         position: V3::xyz("Position", S2, None),
         rotation: V3::xyz("Rotation", S1, None),
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

   pub fn compile(&self, st: &String) -> String {
      return format!("vec3 {st} = move({st}, {}", self.position.compile())
   }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

         ior: Float::inv("IOR", S1, None),
         refract_chance: Float::percent("Refract chance", S1),
         refract_roughness: Float::inv("Refract roughness", S1, None),
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RecUpdate {
   pub queue_compile: bool,
   pub queue_update: bool,
}
impl RecUpdate {
   pub fn set_false() -> Self {
      Self {
         queue_update: false,
         queue_compile: false,
      }
   }

   pub fn set_true() -> Self {
      Self {
         queue_update: true,
         queue_compile: true,
      }
   }

   pub fn reset(&mut self) {
      self.queue_compile = false;
      self.queue_update = false;
   }

   pub fn update(&mut self) { self.queue_update = true; }

   pub fn compile(&mut self) { self.queue_compile = true; }

   pub fn both(&mut self) { self.queue_compile = true; self.queue_update = true; }
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Float {
   val: f32,
   range: RangeInclusive<f32>,
   speed: f32,
   name: String,
}
impl Float {
   pub fn inv(name: &str, speed: f32, def: Option<f32>) -> Self {
      Self {
         val: match def {
            None => {0.0}
            Some(def_val) => {def_val}
         },
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

   pub fn compile(&self) -> String {
      return format!("{}", self.val)
   }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct V3 {
   x: Float,
   y: Float,
   z: Float,
   name: String,
}
impl V3 {
   pub fn xyz(name: &str, speed: f32, def: Option<f32>) -> Self {
      Self {
         x: Float::inv("X", speed, def),
         y: Float::inv("Y", speed, def),
         z: Float::inv("Z", speed, def),
         name: name.to_string(),
      }
   }

   pub fn rgb(name: &str) -> Self {
      Self {
         x: Float::inv("R", 1.0, None),
         y: Float::inv("G", 1.0, None),
         z: Float::inv("B", 1.0, None),
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

   pub fn compile(&self) -> String {
      return format!("vec3({}, {}, {})", self.x.compile(), self.y.compile(), self.z.compile())
   }
}