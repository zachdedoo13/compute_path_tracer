use egui::Ui;
use serde::{Deserialize, Serialize};
use crate::sdf_editor::primitives::{CompData, Float, S1, S2, V3};



/////////////////////
// Data structures //
/////////////////////
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Transform {
   pub position: V3,
   pub rotation: V3,
   pub scale: Float,
   pub aabb_exaggeration: Float,
   pub aabb: bool,
}
impl Transform {
   pub fn new() -> Self {
      Self {
         position: V3::xyz("Position", S2, 0.0),
         rotation: V3::xyz("Rotation", S1, 0.0),
         scale: Float::new("Scale", S1, 1.0, 0.0..=f32::MAX),
         aabb_exaggeration: Float::new("AABB_exaggeration", S2, 1.3, 0.0..=10.0),
         aabb: true,
      }
   }
   pub fn ui(&mut self, ui: &mut Ui) -> bool {
      let pre = self.aabb;
      egui::CollapsingHeader::new("Transform")
          .show(ui, |ui| {
             self.position.separate_values(ui);
             self.rotation.separate_values(ui);
             self.scale.ui(ui);
             self.aabb_exaggeration.ui(ui);
             ui.add(egui::Checkbox::new(&mut self.aabb, "Do aabb?"))
          });

      if self.aabb != pre {
         return true;
      }
      return false;
   }

   pub fn compile(&self, st: &String, comp_data: &mut CompData, reference: &String) -> String {
      let start = format!("vec3 {st} = {reference};");

      let scale = format!("{st} *= 1.0 / {};", self.scale.compile(comp_data));
      let pos =  format!("{st} = move({st}, {} * (1.0 / {}));", self.position.compile(comp_data), self.scale.compile(comp_data));
      let rot = format!("{st} = rot3D({st}, {});",  self.rotation.compile(comp_data));

      self.aabb_exaggeration.compile(comp_data);

      format!("{start}\n {scale}\n {pos}\n {rot}")
   }

   pub fn aabb_check(&self, comp_data: &mut CompData) -> String {
      if self.aabb {
         comp_data.aabb_index += 1;
         return format!("if (check[{}]) ", comp_data.aabb_index - 1)
      }
      else {
         comp_data.aabb_index += 1;
         return format!("if (true)")
      }
   }

   pub fn aabb_compile(&self, comp_data: &mut CompData, shape_options: Option<String>) -> String {
      if self.aabb {
         let pt = comp_data.aabb_pos_trail.clone();
         let st = comp_data.aabb_scale_trail.clone();

         let so = match shape_options {
            None => { "vec3(1.0)".to_string() }
            Some(data) => { data }
         };

         println!("DATA = {}", comp_data.data_array.data.len());
         let ex = self.aabb_exaggeration.compile(comp_data);
         println!("AABB EX{ex}");
         println!("DATA = {}", comp_data.data_array.data.len());

         format!("(bool_hit(intersectAABB(ray, from_pos_size({pt} {}, ({so} * ( {st} {})) * {} ))))",
                 self.position.compile(comp_data),
                 self.scale.compile(comp_data),
                 ex,
         )
      }
      else {
         format!("(false)")
      }
   }

   pub fn finalise_scale(&self, st: &String, comp_data: &mut CompData) -> String {
      format!("{st}.d /= 1.0 / {};", self.scale.compile(comp_data))
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
      self.position.refresh(comp_data);
      self.rotation.refresh(comp_data);
      self.scale.refresh(comp_data);
      self.aabb_exaggeration.refresh(comp_data);
   }

   pub fn rehash(&mut self) {
      self.position.rehash();
      self.rotation.rehash();
      self.scale.rehash();
      self.aabb_exaggeration.rehash();
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

   pub fn compile(&self, comp_data: &mut CompData) -> String {
      let col = self.color.compile(comp_data);
      let brightness = self.brightness.compile(comp_data);
      let light = self.light_col.compile(comp_data);
      let spec = self.specular_chance.compile(comp_data);
      let spec_col = self.specular_color.compile(comp_data);
      let roughness = self.roughness.compile(comp_data);
      let ior = self.ior.compile(comp_data);
      let refract_chance = self.refract_chance.compile(comp_data);
      let refract_roughness = self.refract_roughness.compile(comp_data);
      let refract_col = self.refract_color.compile(comp_data);

      format!(
         "Mat({}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
         col, brightness, light, spec, spec_col, roughness, ior, refract_chance, refract_roughness, refract_col
      )
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
      self.color.refresh(comp_data);
      self.brightness.refresh(comp_data);
      self.light_col.refresh(comp_data);
      self.specular_chance.refresh(comp_data);
      self.specular_color.refresh(comp_data);
      self.roughness.refresh(comp_data);
      self.ior.refresh(comp_data);
      self.refract_chance.refresh(comp_data);
      self.refract_roughness.refresh(comp_data);
      self.refract_color.refresh(comp_data);
   }

   pub fn rehash(&mut self) {
      self.color.rehash();
      self.brightness.rehash();
      self.light_col.rehash();
      self.specular_chance.rehash();
      self.specular_color.rehash();
      self.roughness.rehash();
      self.ior.rehash();
      self.refract_chance.rehash();
      self.refract_roughness.rehash();
      self.refract_color.rehash();
   }
}
