use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fs;
use std::ops::{Add, RangeInclusive};
use std::sync::atomic::AtomicU32;
use std::time::Instant;
use bytemuck::cast_slice;
use egui::{Color32, ComboBox, Context, DragValue, Frame, Label, menu, ScrollArea, Style, TextEdit, Ui, Window};
use rand::{random, Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ShaderStages};
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;
use crate::inbuilt::setup::Setup;
use crate::packages::input_manager_package::InputManager;
use crate::path_tracer::path_tracer::PathTracer;

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
      let header_shapes = vec![];

      Self {
         header_unions,
         header_shapes,
         rec_update: RecUpdate::set_true(),

         save_name: String::new(),
      }
   }

   pub fn update(&mut self, path_tracer: &mut PathTracer, setup: &Setup, input_manager: &InputManager, comp_data: &mut CompData) {
      if self.rec_update.queue_compile || input_manager.is_key_just_pressed(KeyCode::Space){
         let map = self.compile(comp_data);

         path_tracer.remake_pipeline(setup, map);
      }

      if self.rec_update.queue_update {
         let data = self.data_update(comp_data);
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
            self.rec_update.both();
         }
      });
   }

   fn open(&mut self, name: &String) {
      let contents = fs::read_to_string(format!("{MAP_PATH}{name}")).expect("Failed to read file");
      let deserialize: SDFEditor = from_str(&contents).expect("Failed to deserialize");

      *self = deserialize;
   }



   // compiler functions
   pub fn compile(&mut self, comp_data: &mut CompData) -> String {
      let st = Instant::now();

      let mut out = String::new();

      out.push_str(r#"
      #define MAXHIT Hit(10000.0, MDEF)

      Hit map(vec3 pu0) {
         Hit start = MAXHIT;

      "#);


      for union in self.header_unions.iter() {
         out.push_str(union.compile(comp_data, &"start".to_string(), 0).as_str());
         comp_data.union_depth = 0;
      }


      // for shape in self.header_shapes.iter_mut() {
      //    out.push_str(shape.compile(&mut comp_data).as_str());
      // }


      out.push_str(r#"
         return start;
      }

      "#);


      println!("Compiled in {:?}", st.elapsed());

      println!("{out}//////////////////////////////////////////////////////////////////////////");
      out
   }

   pub fn data_update(&mut self, comp_data: &mut CompData) -> Vec<f32> {
      let st = Instant::now();

      for union in self.header_unions.iter() {
         union.refresh(comp_data)
      }
      vec![]
   }
}


pub struct SDFEditorPackage {
   pub sdfeditor: SDFEditor,
   pub comp_data: CompData,
}
impl SDFEditorPackage {
   pub fn new(setup: &Setup) -> Self {
      Self {
         sdfeditor: SDFEditor::new(),
         comp_data: CompData::new(setup),
      }
   }

   pub fn ui(&mut self, context: &Context) {
      self.sdfeditor.ui(context)
   }

   pub fn update(&mut self, path_tracer: &mut PathTracer, setup: &Setup, input_manager: &InputManager) {

      if self.sdfeditor.rec_update.queue_update | self.sdfeditor.rec_update.queue_compile  {
         self.sdfeditor.update(path_tracer, setup, input_manager, &mut self.comp_data);
         self.comp_data.data_array.update(setup);
      } else {
         self.sdfeditor.update(path_tracer, setup, input_manager, &mut self.comp_data);
      }


   }
}



pub struct CompData {
   pub union_index: u32,
   pub union_depth: u32,

   pub current_union: u32,
   pub current_shape: u32,

   pub data_array: DataArray,

}
impl CompData {
   pub fn new(setup: &Setup) -> Self {
      Self {
         union_index: 0,
         union_depth: 0,

         current_union: 0,
         current_shape: 0,


         data_array: DataArray::new(setup),
      }
   }

   pub fn inc_union(&mut self) {
      self.union_index += 1;
   }
   pub fn inc_depth(&mut self) {
      self.union_depth += 1;
   }
}


pub fn gen_hash() -> u128 {
   let mut rng = StdRng::from_entropy();
   let random_value1: u128 = rng.gen();
   let random_value2: u128 = rng.gen();
   random_value1 ^ random_value2
}


pub struct DataArray {
   pub data: Vec<f32>,
   pub counter: u32,
   pub seen: HashMap<u128, usize>,

   pub bind_group_layout: BindGroupLayout,
   pub bind_group: BindGroup,
   pub buffer: Buffer,
}
impl DataArray {
   pub fn new(setup: &Setup) -> Self {

      let data: Vec<f32> = vec![1.0];

      let bind_group_layout = setup.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
         label: Some("Data bind group"),
         entries: &[
            BindGroupLayoutEntry {
               binding: 0,
               visibility: ShaderStages::COMPUTE,
               ty: BindingType::Buffer {
                  ty: BufferBindingType::Storage { read_only: false },
                  has_dynamic_offset: false,
                  min_binding_size: None,
               },
               count: None,
            }
         ],
      });

      let buffer = setup.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Data Buffer"),
         contents: cast_slice(&data),
         usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

      let bind_group = setup.device.create_bind_group(&wgpu::BindGroupDescriptor {
         layout: &bind_group_layout,
         entries: &[
            wgpu::BindGroupEntry {
               binding: 0,
               resource: buffer.as_entire_binding(),
            },
         ],
         label: Some("Data Bind Group"),
      });




      Self {
         data: vec![],
         counter: 0,
         seen: Default::default(),

         bind_group_layout,
         bind_group,
         buffer
      }
   }

   pub fn get_index(&mut self, in_data: f32, hash: u128) -> String {
      if self.seen.contains_key(&hash) {
         let i = self.seen.get(&hash).unwrap();
         format!("data[{i}]")
      }
      else {
         self.data.push(in_data);
         let i = self.data.len() - 1;
         self.seen.insert(hash, i);

         format!("data[{i}]")
      }
   }

   pub fn update(&mut self, setup: &Setup) {
      // Recreate the buffer with the updated data size
      self.buffer = setup.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Updated Data Buffer"),
         contents: cast_slice(&self.data),
         usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

      // Update the bind group to use the new buffer
      self.bind_group = setup.device.create_bind_group(&wgpu::BindGroupDescriptor {
         layout: &self.bind_group_layout,
         entries: &[
            wgpu::BindGroupEntry {
               binding: 0,
               resource: self.buffer.as_entire_binding(),
            },
         ],
         label: Some("Updated Data Bind Group"),
      });

   }

   pub fn refresh(&mut self, hash: u128, data: f32) {
      let entry = self.seen.get(&hash).expect("Something fucked up");
      self.data[*entry] = data;
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

   fn compile(&self, comp_data: &mut CompData, reference: &String, in_union_depth: u32) -> String {
      let mut out = String::new();

      let union_depth = in_union_depth + 1;

      out.push_str("{\n");

      out.push_str(format!("Hit u{union_depth} = MAXHIT; \n").as_str());
      out.push_str(format!("{}\n", self.transform.compile(&format!("pu{union_depth}"), comp_data, &format!("pu{}", union_depth - 1))).as_str());

      for union in self.children_unions.iter() {
         out.push_str(union.compile(comp_data, &format!("u{union_depth}"), union_depth).as_str());
      }

      let mut shape_index = 0;
      for shape in self.children_shapes.iter() {
         out.push_str(shape.compile(comp_data, union_depth, shape_index).as_str());
         shape_index += 1;
      }
      comp_data.current_shape = 0;

      out.push_str(format!("{}\n", self.transform.finalise_scale(&format!("u{union_depth}"), comp_data)).as_str());
      out.push_str(format!("{reference} = opUnion({reference}, u{union_depth});\n").as_str());

      out.push_str("}\n");

      out
   }

   fn refresh(&self, comp_data: &mut CompData) {
      self.transform.refresh(comp_data);
      for child in self.children_shapes.iter() {
         child.refresh(comp_data);
      }
      for union in self.children_unions.iter() {
         union.refresh(comp_data);
      }
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

   fn compile_name(&self) -> String {
      return match self {
         Shapes::Sphere(_) => { "sdSphere".to_string() }
         Shapes::Cube(_) => { "sdCube".to_string() }
         Shapes::Plane => { "NotImplemented".to_string() }
      }
   }

   fn compile_settings(&self, comp_data: &mut CompData) -> String {
      match &self {
         Shapes::Sphere(size) => { format!("{}", size.compile(comp_data)) }
         Shapes::Cube(xyz) => { format!("{}", xyz.compile(comp_data)) }
         Shapes::Plane => { "NotImplemented".to_string() }
      }
   }

   fn refresh(&self, comp_data: &mut CompData) {
      match self {
         Shapes::Sphere(data) => { data.refresh(comp_data); }
         Shapes::Cube(data) => { data.refresh(comp_data); }
         Shapes::Plane => {}
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

   pub fn compile(&self, comp_data: &mut CompData, union_depth: u32, current_shape: u32) -> String {
      let mut out = String::new();

      let ui = union_depth; // union index
      let si = current_shape; // shape index

      let shape_name = self.current_shape.compile_name();

      let transform_name = format!("u{ui}s{si}p");
      let transform_code = self.transform.compile(&transform_name, comp_data, &format!("pu{}", union_depth));

      let shape_settings = self.current_shape.compile_settings(comp_data);

      out.push_str("{\n"); // todo bounds check

      out.push_str(format!(r#"
      {transform_code}

      Hit u{ui}s{si} = Hit(
         {shape_name}({transform_name}, {shape_settings}),
         MDEF
      );
      {}

      u{ui} = opUnion(u{ui}, u{ui}s{si});


      "#, self.transform.finalise_scale(&format!("u{ui}s{si}"), comp_data)).as_str());

      out.push_str("}\n");

      out
   }

   fn refresh(&self, comp_data: &mut CompData) {
      self.transform.refresh(comp_data);

      self.current_shape.refresh(comp_data);
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
         scale: Float::zero_plus("Scale", S2, Some(1.0)),
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

   pub fn compile(&self, st: &String, comp_data: &mut CompData, reference: &String) -> String {
      let start = format!("vec3 {st} = {reference};");

      let scale = format!("{st} *= 1.0 / {};", self.scale.compile(comp_data));

      let pos =  format!("{st} = move({st}, {} * (1.0 / {}));", self.position.compile(comp_data), self.scale.compile(comp_data));

      let rot = format!("{st} = rot3D({st}, {});",  self.rotation.compile(comp_data));

      format!("{start}\n {scale}\n {pos}\n {rot}")
   }

   pub fn finalise_scale(&self, st: &String, comp_data: &mut CompData) -> String {
      format!("{st}.d /= 1.0 / {};", self.scale.compile(comp_data))
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
      self.position.refresh(comp_data);
      self.rotation.refresh(comp_data);
      self.scale.refresh(comp_data);
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

         brightness: Float::zero_plus("Brightness", S2, None),
         light_col: V3::rgb("Light Color    "),

         specular_chance: Float::percent("Spec chance", S1),
         specular_color: V3::rgb("Spec color"),

         roughness: Float::zero_plus("Roughness", S1, None),

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
   hash: u128,
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
         hash: gen_hash(),
      }
   }

   pub fn one_plus(name: &str, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range: 1.0..=f32::MAX,
         speed,
         hash: gen_hash(),
      }
   }

   pub fn zero_plus(name: &str, speed: f32, def: Option<f32>) -> Self {
      Self {
         val: match def {
            None => {0.0}
            Some(def_val) => {def_val}
         },
         name: name.to_string(),
         range: 0.0..=f32::MAX,
         speed,
         hash: gen_hash(),
      }
   }

   pub fn percent(name: &str, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range: 0.0..=1.0,
         speed,
         hash: gen_hash(),
      }
   }

   pub fn with_range(name: &str, range: RangeInclusive<f32>, speed: f32) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
         range,
         speed,
         hash: gen_hash(),
      }
   }

   pub fn ui(&mut self, ui: &mut Ui) {
      ui.add(
         DragValue::new(&mut self.val)
             .speed(self.speed)
             .clamp_range(self.range.clone())
             .prefix(format!("{}: ", self.name))
      );

   }

   pub fn compile(&self, comp_data: &mut CompData) -> String {

      let index = comp_data.data_array.get_index(self.val, self.hash);

      return format!("{}", index)
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
      comp_data.data_array.refresh(self.hash, self.val);
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

   pub fn compile(&self, comp_data: &mut CompData) -> String {
      return format!("vec3({}, {}, {})", self.x.compile(comp_data), self.y.compile(comp_data), self.z.compile(comp_data))
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
      self.x.refresh(comp_data);
      self.y.refresh(comp_data);
      self.z.refresh(comp_data);
   }
}