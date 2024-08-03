use egui::{Context, Frame, Label, menu, ScrollArea, Style, Ui, Window};



pub struct SDFEditor {
   header_unions: Vec<Union>,
   header_shapes: Vec<bool>,
}
impl SDFEditor {
   pub fn new() -> Self {
      let header_unions = vec![Union::new(), Union::new()];
      let header_shapes = vec![false];

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
                   ui.set_min_width(ui.available_width());
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

      for shape in self.header_shapes.iter_mut() {
         // add shapes
      }
   }
}

///////////
// Nodes //
///////////

pub struct Union {
   transform: Transform,

   children_unions: Vec<Union>,
   children_shapes: Vec<bool>
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


   }
}



/////////////////////
// Data structures //
/////////////////////

#[derive(Default, Debug)]
pub struct Transform {
   position: V3,
   rotation: V3
}
impl Transform {
   pub fn new() -> Self {
      Self {
         position: V3::new("Position"),
         rotation: V3::new("Rotation"),
      }
   }
   pub fn ui(&mut self, ui: &mut Ui) {
      egui::CollapsingHeader::new("Transform")
          .show(ui, |ui| {
             self.position.ui(ui);
             self.rotation.ui(ui);
          });
   }
}

////////////////
// primitives //
////////////////

#[derive(Default, Debug)]
pub struct Float {
   val: f32,
   name: String,
}
impl Float {
   pub fn new(name: &str) -> Self {
      Self {
         val: 0.0,
         name: name.to_string(),
      }
   }
   pub fn ui(&mut self, ui: &mut Ui) {
      ui.add(egui::DragValue::new(&mut self.val).speed(0.001).clamp_range(0.000..=100.0).prefix(format!("{}: ", self.name)));
   }
}

#[derive(Default, Debug)]
pub struct V3 {
   x: Float,
   y: Float,
   z: Float,
   name: String,
}
impl V3 {
   pub fn new(name: &str) -> Self {
      Self {
         x: Float::new("X"),
         y: Float::new("Y"),
         z: Float::new("Z"),
         name: name.to_string(),
      }
   }
   pub fn ui(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         ui.add(Label::new(format!("{}", self.name)));
         ui.horizontal(|ui| {
            self.x.ui(ui);
            self.y.ui(ui);
            self.z.ui(ui);
         });
      });
   }
}


