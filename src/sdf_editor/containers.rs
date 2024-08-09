use egui::{Color32, ComboBox, Frame, Label, Style, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use crate::sdf_editor::data_structures::{Material, Transform};
use crate::sdf_editor::primitives::{CompData, Float, S2, V3};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Union {
   name: String,
   transform: Transform,
   union_type: UnionType,

   pub children_unions: Vec<Union>,
   pub children_shapes: Vec<Shape>,
}
impl Union {
   pub fn new() -> Self {
      Self {
         name: "Union".to_string(),
         transform: Transform::new(),

         union_type: UnionType::Union,
         children_unions: vec![],
         children_shapes: vec![],
      }
   }

   pub fn ui(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      let frame = Frame::group(&Style::default())
          .fill(Color32::from_rgb((50.0 * comp_data.check()) as u8, 0, 0));

      frame.show(ui, |ui| {
         egui::CollapsingHeader::new(format!("{}", self.name))
             .id_source(1)
             .show(ui, |ui| {
                self.contents(ui, comp_data);
             });
      });
   }

   fn contents(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      egui::CollapsingHeader::new("Settings")
          .show(ui, |ui| {

             let check = self.union_type.clone();
             self.union_type.dropdown(ui);
             if self.union_type != check { comp_data.rec_update.both(); }

             let check = self.union_type.clone();
             self.union_type.settings_ui(ui);
             if self.union_type != check { comp_data.rec_update.update(); }


             ui.add(TextEdit::singleline(&mut self.name)
                 .hint_text("Enter name")
                 .desired_width(75.0)
             );

             egui::CollapsingHeader::new("Bounding Area")
                 .show(ui, |ui| {
                    ui.add(Label::new("Not implemented"))
                 });

             let check = self.transform.clone();
             let bloop = self.transform.ui(ui);
             if self.transform != check { comp_data.rec_update.update(); }
             if bloop { comp_data.rec_update.both(); }

          });

      egui::CollapsingHeader::new("Child nodes")
          .show(ui, |ui| {
             self.display_children(ui, comp_data);
          });

   }

   fn display_children(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      ui.horizontal(|ui| {
         if ui.button("Add Union").clicked() {
            self.children_unions.push(Union::new());
            comp_data.rec_update.both();
         }
         if ui.button("Add Shape").clicked() {
            self.children_shapes.push(Shape::new());
            comp_data.rec_update.both();
         }
      });

      let mut i = 0;
      // unions
      let mut execute = None;
      for (inner_i, union) in self.children_unions.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
               union.ui(ui, comp_data);

               if ui.button("Delete").clicked() {
                  execute = Some(inner_i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = execute {
         self.children_unions.remove(index);
         comp_data.rec_update.both();
      }

      // shapes
      let mut exucute = None;
      let mut dupe = None;
      for (inner_i, shape) in self.children_shapes.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               shape.ui(ui, comp_data);

               if ui.button("Delete").clicked() {
                  exucute = Some(inner_i);
               }

               if ui.button("Duplicate").clicked() {
                  dupe = Some(shape.clone());
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.children_shapes.remove(index);
         comp_data.rec_update.both();
      }
      if let Some(mut shape) = dupe {
         shape.rehash();
         self.children_shapes.push(shape);
         comp_data.rec_update.both();
      }
   }


   pub fn compile(&self, comp_data: &mut CompData, reference: &String, in_union_depth: u32, union_type: &UnionType) -> String {
      let mut out = String::new();
      let union_depth = in_union_depth + 1;

      // Start the union block
      out.push_str("{\n");

      // Initialize the union hit variable
      out.push_str(format!("Hit u{union_depth} = MAXHIT; \n").as_str());

      // Compile the transform for the current union
      out.push_str(format!("{}\n", self.transform.compile(&format!("pu{union_depth}"), comp_data, &format!("pu{}", union_depth - 1))).as_str());

      // Compile all child unions
      for union in self.children_unions.iter() {
         out.push_str(union.compile(comp_data, &format!("u{union_depth}"), union_depth, &self.union_type).as_str());
      }

      // Compile all child shapes
      let mut shape_index = 0;
      for shape in self.children_shapes.iter() {
         out.push_str(shape.compile(comp_data, union_depth, shape_index, &self.union_type).as_str());
         shape_index += 1;
      }

      // Finalize the transform scale for the current union
      out.push_str(format!("{}\n", self.transform.finalise_scale(&format!("u{union_depth}"), comp_data)).as_str());

      // Combine the current union with the reference union
      // out.push_str(format!("{reference} = opUnion({reference}, u{union_depth});\n").as_str());
      out.push_str(union_type.compile(&reference, &format!("u{union_depth}"), 1).as_str());

      // End the union block
      out.push_str(format!("}} //{}\n", self.name).as_str());

      out
   }

   pub fn aabb_compile(&self, comp_data: &mut CompData) -> String {
      let mut out = String::new();


      let pos_trail = comp_data.aabb_pos_trail.clone();
      let pos_trans = self.transform.position.compile(comp_data);
      comp_data.aabb_pos_trail.push_str(format!("{} +", pos_trans).as_str());

      let scale_trail = comp_data.aabb_scale_trail.clone();
      let scale_trans = self.transform.scale.compile(comp_data);
      comp_data.aabb_scale_trail.push_str(format!("{} *", scale_trans).as_str());

      for shape in self.children_shapes.iter() {
         out.push_str(shape.aabb_compile(comp_data).as_str())
      }

      comp_data.aabb_pos_trail = pos_trail;
      comp_data.aabb_scale_trail = scale_trail;


      out
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
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
pub enum UnionType {
   Union,
   Subtraction,
}
impl UnionType {
   pub fn dropdown(&mut self, ui: &mut Ui) {
      ComboBox::from_label("")
          .selected_text(self.text())
          .show_ui(ui, |ui| {
             ui.selectable_value(self, UnionType::Union, "Union");
             ui.selectable_value(self, UnionType::Subtraction, "Subtraction");
          });
   }

   pub fn settings_ui(&mut self, ui: &mut Ui) {
      match self {
         UnionType::Union => { ui.label("No settings"); }
         UnionType::Subtraction => { ui.label("No settings"); }
      }
   }

   fn text(&self) -> &str {
      match self {
         UnionType::Union => {"Union"}
         UnionType::Subtraction => {"Subtraction"}
      }
   }

   pub fn compile(&self, reference: &String, with: &String, index: u32) -> String {
      if index == 0 {
         return format!("{reference} = {with};")
      }
      match self {
         UnionType::Union => { format!("{reference} = opUnion({reference}, {with});") }
         UnionType::Subtraction => { format!("{reference} = opSubtraction({reference}, {with});") }
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
             ui.selectable_value(self, Shapes::Sphere(Float::inv("Size", S2, 1.0)), "Sphere");
             ui.selectable_value(self, Shapes::Cube(V3::xyz("Size", S2, 1.0)), "Cube");
             ui.selectable_value(self, Shapes::Plane, "Plane");
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

   fn rehash(&mut self) {
      match self {
         Shapes::Sphere(data) => { data.rehash(); }
         Shapes::Cube(data) => { data.rehash(); }
         Shapes::Plane => {}
      }
   }
}
impl Default for Shapes {
   fn default() -> Self {
      Shapes::Sphere(Float::inv("Size", S2, 1.0))
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

   pub fn ui(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
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

                self.contents(ui, comp_data);
             });
      });
   }

   fn contents(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      let check = self.current_shape.clone();
      self.current_shape.dropdown(ui);
      if self.current_shape != check { comp_data.rec_update.compile() }

      let check = self.current_shape.clone();
      self.shape_settings(ui);
      if self.current_shape != check { comp_data.rec_update.update() }

      egui::CollapsingHeader::new("Bounding Area")
          .show(ui, |ui| {
             ui.add(Label::new("Not implemented"))
          });

      let check = self.transform.clone();
      let bloop = self.transform.ui(ui);
      if self.transform != check { comp_data.rec_update.update() }
      if bloop { comp_data.rec_update.both(); }

      let check = self.material.clone();
      self.material.ui(ui);
      if self.material != check { comp_data.rec_update.update() }
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

   pub fn compile(&self, comp_data: &mut CompData, union_depth: u32, current_shape: u32, union_type: &UnionType) -> String {
      let mut out = String::new();

      let ui = union_depth; // union index
      let si = current_shape; // shape index

      let shape_name = self.current_shape.compile_name();

      let transform_name = format!("u{ui}s{si}p");
      let transform_code = self.transform.compile(&transform_name, comp_data, &format!("pu{}", union_depth));

      let shape_settings = self.current_shape.compile_settings(comp_data);

      let material = self.material.compile(comp_data);

      out.push_str(format!("{} {{\n", self.transform.aabb_check(comp_data)).as_str()); // todo bounds check

      let union = union_type.compile(&format!("u{ui}"), &format!("u{ui}s{si}"), si);

      out.push_str(format!(r#"
      {transform_code}

      Hit u{ui}s{si} = Hit(
         {shape_name}({transform_name}, {shape_settings}),
         {material}
      );
      {}

      {union}


      "#, self.transform.finalise_scale(&format!("u{ui}s{si}"), comp_data)).as_str());

      out.push_str("}\n");

      out
   }

   pub fn aabb_compile(&self, comp_data: &mut CompData) -> String {
      let mut out = String::new();

      let so = match &self.current_shape {
         Shapes::Sphere(data) => {Some(format!("vec3({})", data.compile(comp_data)))}
         Shapes::Cube(data) => {Some(data.compile(comp_data))}
         Shapes::Plane => {None}
      };

      out.push_str(format!(r#"
      if {} {{

      back[{}] = true;
      debug += 0.1;

      }}
      "#, self.transform.aabb_compile(comp_data, so), comp_data.aabb_index).as_str());


      comp_data.aabb_index += 1;
      out
   }


   fn refresh(&self, comp_data: &mut CompData) {
      self.transform.refresh(comp_data);
      self.material.refresh(comp_data);
      self.current_shape.refresh(comp_data);
   }

   fn rehash(&mut self) {
      self.transform.rehash();
      self.material.rehash();
      self.current_shape.rehash();
   }
}