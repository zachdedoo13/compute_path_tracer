use egui::{Color32, ComboBox, Frame, Label, Style, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use crate::sdf_editor::data_structures::{Material, Transform};
use crate::sdf_editor::primitives::{CompData, Float, S2, V3};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Union {
   name: String,
   transform: Transform,

   pub children_unions: Vec<Union>,
   pub children_shapes: Vec<Shape>,
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
             if self.transform != check { comp_data.rec_update.update(); }

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
      let mut exucute = None;
      for (inner_i, union) in self.children_unions.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
               union.ui(ui, comp_data);

               if ui.button("Delete").clicked() {
                  exucute = Some(inner_i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.children_unions.remove(index);
         comp_data.rec_update.both();
      }

      // shapes
      let mut exucute = None;
      for (inner_i, shape) in self.children_shapes.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {

               shape.ui(ui, comp_data);

               if ui.button("Delete").clicked() {
                  exucute = Some(inner_i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.children_shapes.remove(index);
         comp_data.rec_update.both();
      }


   }


   pub fn compile(&self, comp_data: &mut CompData, reference: &String, in_union_depth: u32) -> String {
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
         out.push_str(union.compile(comp_data, &format!("u{union_depth}"), union_depth).as_str());
      }

      // Compile all child shapes
      let mut shape_index = 0;
      for shape in self.children_shapes.iter() {
         out.push_str(shape.compile(comp_data, union_depth, shape_index).as_str());
         shape_index += 1;
      }

      // Finalize the transform scale for the current union
      out.push_str(format!("{}\n", self.transform.finalise_scale(&format!("u{union_depth}"), comp_data)).as_str());

      // Combine the current union with the reference union
      out.push_str(format!("{reference} = opUnion({reference}, u{union_depth});\n").as_str());

      // End the union block
      out.push_str(format!("}} //{}\n", self.name).as_str());

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
      self.transform.ui(ui);
      if self.transform != check { comp_data.rec_update.update() }

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