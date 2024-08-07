use egui::Ui;
use serde::{Deserialize, Serialize};
use crate::sdf_editor::primitives::{CompData, Float, S1, S2, V3};



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
         position: V3::xyz("Position", S2, 0.0),
         rotation: V3::xyz("Rotation", S1, 0.0),
         scale: Float::new("Scale", S1, 1.0, 0.0..=f32::MAX),
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

         brightness: Float::new("Brightness", S2, 0.0, 0.0..=f32::MAX),
         light_col: V3::rgb("Light Color"),

         specular_chance: Float::percent("Spec chance", S1, 0.0),
         specular_color: V3::rgb("Spec color"),

         roughness: Float::new("Roughness", S1, 0.0, 0.0..=f32::MAX),

         ior: Float::inv("IOR", S1, 0.0),
         refract_chance: Float::percent("Refract chance", S1, 0.0),
         refract_roughness: Float::inv("Refract roughness", S1, 0.0),
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