use std::fs;
use std::time::Instant;
use egui::{Context, Frame, menu, ScrollArea, Style, TextEdit, Ui, Window};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use winit::keyboard::KeyCode;
use crate::inbuilt::setup::Setup;
use crate::packages::input_manager_package::InputManager;
use crate::path_tracer::path_tracer::PathTracer;
use crate::sdf_editor::containers::{Shape, Union};
use crate::sdf_editor::primitives::{CompData, RecUpdate};

const MAP_PATH: &str = "data/maps/";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SDFEditor {
   header_unions: Vec<Union>,

   save_name: String,
}
impl SDFEditor {
   pub fn new() -> Self {

      // init folders this struct uses
      fs::create_dir_all(MAP_PATH).expect("Failed to create dir");


      let mut header_unions = vec![Union::new()];
      header_unions[0].children_shapes.push(Shape::new());

      Self {
         header_unions,
         save_name: String::new(),
      }
   }

   pub fn update(&mut self, path_tracer: &mut PathTracer, setup: &Setup, comp_data: &mut CompData) {
      if comp_data.rec_update.queue_compile {
         comp_data.reset_data_array();
         let map = self.compile(comp_data);
         path_tracer.remake_pipeline(setup, map);
      }

      if comp_data.rec_update.queue_update {
         self.data_update(comp_data);
      }

      comp_data.rec_update.reset();
   }

   pub fn ui(&mut self, context: &Context, comp_data: &mut CompData) {
      let window = Window::new("SDF Editor")
          .resizable(true)
          .frame(Frame::window(&Style::default()));

      window.show(context, |ui| {

         self.menubar(ui, comp_data);

         ui.group(|ui| {
            ScrollArea::both()
                .show(ui, |ui| {
                   ui.set_min_size(ui.available_size());
                   self.editor_contents(ui, comp_data);
                });
         });

      });
   }


   // related private functions //
   fn menubar(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      menu::bar(ui, |ui| {
         ui.menu_button("File", |ui| {
            ui.menu_button("Open", |ui| {
               self.open_ui(ui, comp_data);
            });

            ui.menu_button("Save", |ui| {
               ui.add(TextEdit::singleline(&mut self.save_name)
                   .hint_text("Save name")
                   .desired_width(75.0)
               );

               self.overwrite_check(ui);

               if ui.button("Save as").clicked() {
                  self.save(&self.save_name);
               }
            });

            if ui.button("Force compile").clicked() {
               comp_data.rec_update.compile();
            }
            if ui.button("Force update").clicked() {
               comp_data.rec_update.update();
            }
         });

         ui.menu_button("Add", |ui| {
            if ui.button("Union").clicked() {
               self.header_unions.push(Union::new());
               comp_data.rec_update.both();
            }
         });

      });
   }

   fn editor_contents(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      let mut i = 0;
      // unions
      let mut exucute = None;
      for (temp_i, union) in self.header_unions.iter_mut().enumerate() {
         ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
               union.ui(ui, comp_data);

               if ui.button("Delete").clicked() {
                  exucute = Some(temp_i);
               }
            });
         });
         i += 1;
      }
      if let Some(index) = exucute {
         self.header_unions.remove(index);
         comp_data.rec_update.both();
      }
   }

   fn save(&self, name: &String) {
      println!("Saving as {name}");
      let file_name = format!("{name}.json");
      let path = format!("{MAP_PATH}{file_name}");

      let serialized = to_string_pretty(self).unwrap();

      fs::write(&path, serialized).expect(format!("Failed to write to {path}").as_str());
   }

   fn open_ui(&mut self, ui: &mut Ui, comp_data: &mut CompData) {
      let files = fs::read_dir(MAP_PATH).expect("Failed to read map path");

      let mut seen = true;
      files.for_each(|entry| {
         let file = entry.expect("Invalid entry");
         let name = file.file_name().into_string().unwrap();

         if ui.button(&name).clicked() {
            self.open(&name);
            comp_data.rec_update.both();
         }

         seen = false;
      });

      if seen {
         ui.label("No files found");
      }
   }

   fn open(&mut self, name: &String) {
      let contents = fs::read_to_string(format!("{MAP_PATH}{name}")).expect("Failed to read file");
      let deserialize: SDFEditor = from_str(&contents).expect("Failed to deserialize");

      *self = deserialize;
   }

   fn overwrite_check(&mut self, ui: &mut Ui) {
      let files = fs::read_dir(MAP_PATH).expect("Failed to read map path");

      files.for_each(|entry| {
         let file = entry.expect("Invalid entry");
         let name = file.file_name().into_string().unwrap().replace(".json", "");
         if self.save_name == name {
            ui.label(format!("!! Will overwrite {name}"));
            return;
         }
      });

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
      }


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
      self.sdfeditor.ui(context, &mut self.comp_data)
   }

   pub fn update(&mut self, path_tracer: &mut PathTracer, setup: &Setup) {
      self.comp_data.checker = true;

      if self.comp_data.rec_update.queue_compile | self.comp_data.rec_update.queue_update {
         self.sdfeditor.update(path_tracer, setup, &mut self.comp_data);
         self.comp_data.data_array.update(setup);
      }
      else {
         self.sdfeditor.update(path_tracer, setup, &mut self.comp_data);
      }
   }
}
