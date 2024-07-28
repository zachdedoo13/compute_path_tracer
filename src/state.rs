use egui::Label;
use std::iter;
use bytemuck::Zeroable;
use cgmath::{Vector3, Zero};
use egui::{Align2, Context, Frame, Pos2, Rect, Sense, Style, Ui, Vec2};
use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandEncoder, TextureView, TextureViewDescriptor};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;
use crate::inbuilt::gui::EguiRenderer;
use crate::inbuilt::setup::Setup;
use crate::packages::input_manager_package::InputManager;
use crate::packages::time_package::TimePackage;
use crate::path_tracer::path_tracer::PathTracer;
use crate::pipelines::render_texture_pipeline::RenderTexturePipeline;
use crate::utility::structs::StorageTexturePackage;

pub struct State<'a> {
   pub setup: Setup<'a>,
   pub egui: EguiRenderer,

   pub resized: bool,

   // packages
   time_package: TimePackage,
   input_manager: InputManager,

   render_texture: StorageTexturePackage,
   render_texture_pipeline: RenderTexturePipeline,

   path_tracer: PathTracer,
   node_editor: NodeEditor,
}

impl<'a> State<'a> {
   pub async fn new(window: &'a Window) -> State<'a> {

      // dependents
      let setup = Setup::new(window).await;
      let egui = EguiRenderer::new(&setup.device, setup.config.format, None, 1, setup.window);


      let mut node_editor = NodeEditor::new();

      // packages
      let time_package = TimePackage::new();
      let input_manager = InputManager::new();


      let render_texture = StorageTexturePackage::new(&setup, (10.0, 10.0));
      let render_texture_pipeline = RenderTexturePipeline::new(&setup, &render_texture);

      let path_tracer = PathTracer::new(&setup, &render_texture, node_editor.generate_map());



      Self {
         setup,
         egui,
         resized: false,

         time_package,
         input_manager,

         render_texture,
         render_texture_pipeline,

         path_tracer,

         node_editor,
      }
   }

   pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
      if new_size.width > 0 && new_size.height > 0 {
         self.setup.size = new_size;
         self.setup.config.width = new_size.width;
         self.setup.config.height = new_size.height;
         self.setup.surface.configure(&self.setup.device, &self.setup.config);

         self.resized = true;
      }
   }

   pub fn update_input(&mut self, event: &WindowEvent) -> bool {
      self.input_manager.process_event(event);
      false
   }

   pub fn update(&mut self) {
      self.time_package.update();

      self.path_tracer.update(&self.setup, &mut self.render_texture, &self.time_package, &self.input_manager, self.resized);

      self.input_manager.reset();
      self.resized = false;
   }

   pub fn update_gui(&mut self, view: &TextureView, encoder: &mut CommandEncoder) {
      let screen_descriptor = ScreenDescriptor {
         size_in_pixels: [self.setup.config.width, self.setup.config.height],
         pixels_per_point: self.setup.window.scale_factor() as f32,
      };

      let run_ui = |ui: &Context| {
         // place ui functions hear
         let code = |ui: &mut Ui| {
            // performance ui built in
            {
               egui::CollapsingHeader::new("Performance")
                   .default_open(true)
                   .show(ui, |ui| {
                      ui.add(Label::new(format!("FPS: {}", &self.time_package.fps)));

                      ui.add(Label::new(
                         format!("Screen: {} x {} = {}",
                                 &self.setup.size.width
                                 , &self.setup.size.height,
                                 &self.setup.size.width * &self.setup.size.height
                         )
                      ));

                      ui.add(Label::new(
                         format!("Texture: {} x {} = {}",
                                 &self.render_texture.size.width
                                 , &self.render_texture.size.height,
                                 &self.render_texture.size.width * &self.render_texture.size.height
                         )
                      ));

                      ui.end_row();
                   });
            }

            // add other ui hear
            self.path_tracer.gui(ui);
         };

         // Pre draw setup
         egui::Window::new("template thinggy")
             .default_open(true)
             .max_width(1000.0)
             .max_height(800.0)
             .default_width(800.0)
             .resizable(true)
             .anchor(Align2::LEFT_TOP, [0.0, 0.0])
             .frame(Frame::window(&Style::default()))
             .show(&ui, code);

         self.node_editor.ui(ui, &mut self.path_tracer, &self.render_texture, &self.setup, &mut self.resized);
      };

      self.egui.draw(
         &self.setup.device,
         &self.setup.queue,
         encoder,
         &self.setup.window,
         &view,
         screen_descriptor,
         run_ui,
      );
   }

   pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let output = self.setup.surface.get_current_texture()?;
      let view = output.texture.create_view(&TextureViewDescriptor::default());
      let mut encoder = self.setup.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         self.path_tracer.compute_pass(&mut encoder, &self.render_texture);
         self.render_texture_pipeline.render_pass(&mut encoder, &view, &self.render_texture);
      }

      self.update_gui(&view, &mut encoder);

      self.setup.queue.submit(iter::once(encoder.finish()));

      output.present();

      Ok(())
   }
}





// todo make a editor package

struct NodeEditor {
   max_size: Vec2,
   nodes: Vec<Node>,
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

   fn generate_map(&mut self) -> String {
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

   fn save_scene(&mut self) {} // todo

   fn load_scenes(&mut self) {} // todo

}


#[derive(Debug, PartialEq, Clone)]
enum Shapes {
   Sphere,
   Cube,
}
impl Shapes {
   fn show_enum_dropdown(ui: &mut Ui, selected: &mut Shapes) {
      egui::ComboBox::from_label("")
          .selected_text(format!("{:?}", selected))
          .show_ui(ui, |ui| {
             ui.selectable_value(selected, Shapes::Sphere, "Sphere");
             ui.selectable_value(selected, Shapes::Cube, "Cube");
          });
   }

   fn size_options(&self, ui: &mut Ui, size: &mut Vector3<f32>) {
      match self {
         Shapes::Sphere => {
            ui.add(egui::DragValue::new(&mut size.x).speed(0.01).clamp_range(0.01..=100.0).prefix("Size: "));
         }
         Shapes::Cube => {
            ui.add(egui::DragValue::new(&mut size.x).speed(0.01).clamp_range(0.01..=100.0).prefix("L: "));
            ui.add(egui::DragValue::new(&mut size.y).speed(0.01).clamp_range(0.01..=100.0).prefix("W: "));
            ui.add(egui::DragValue::new(&mut size.z).speed(0.01).clamp_range(0.01..=100.0).prefix("H: "));
         }
      }
   }
}


#[derive(Clone)]
#[derive(PartialEq)]
struct Node {
   screen_position: Pos2,
   screen_size: Vec2,
   title: String,

   shape: Shapes,
   scale: f32,
   size: Vector3<f32>,
   position: Vector3<f32>,
   rotation: Vector3<f32>,
}
impl Node {
   fn new(screen_position: Pos2, size: Vec2, title: String) -> Self {
      Self {
         screen_position,
         screen_size: size,
         title,

         ..Default::default()
      }
   }

   fn contents<'a>(&'a mut self, back: &'a mut bool) -> impl FnMut(&mut Ui) + '_ {
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

                       ui.add(egui::DragValue::new(&mut self.position.x).speed(0.01).clamp_range(-100.0..=100.0).prefix("X: "));
                       ui.add(egui::DragValue::new(&mut self.position.y).speed(0.01).clamp_range(-100.0..=100.0).prefix("Y: "));
                       ui.add(egui::DragValue::new(&mut self.position.z).speed(0.01).clamp_range(-100.0..=100.0).prefix("Z: "));
                    });

                egui::CollapsingHeader::new("rotation")
                    .default_open(false)
                    .show(ui, |ui| {
                       if ui.button("reset").clicked() { self.rotation = Self::default().rotation; }

                       ui.add(egui::DragValue::new(&mut self.rotation.x).speed(0.01).clamp_range(-1000.0..=1000.0).prefix("X: "));
                       ui.add(egui::DragValue::new(&mut self.rotation.y).speed(0.01).clamp_range(-1000.0..=1000.0).prefix("Y: "));
                       ui.add(egui::DragValue::new(&mut self.rotation.z).speed(0.01).clamp_range(-1000.0..=1000.0).prefix("Z: "));
                    });

             });
      };

      contents
   }

   fn draw(&mut self, ui: &mut Ui, window_pos: Pos2, max_size: Vec2) -> bool {
      let rect = Rect::from_min_size(window_pos + self.screen_position.to_vec2(), self.screen_size);
      let response = ui.allocate_rect(rect, Sense::click_and_drag());

      if response.dragged() {
         self.screen_position += response.drag_delta();

         if self.screen_position.x < 0.0 {self.screen_position.x = 0.0}
         if self.screen_position.y < 0.0 {self.screen_position.y = 0.0}

         if self.screen_position.x > max_size.x {self.screen_position.x = max_size.x}
         if self.screen_position.y > max_size.y {self.screen_position.y = max_size.y}
      }

      let mut back = false;

      ui.allocate_ui_at_rect(rect, |ui| {
         ui.group(self.contents(&mut back));
      });

      back
   }

   fn map(&self, index: usize) -> String {
      let mut back = String::new();

      let shape_option = match self.shape {
         Shapes::Sphere => {"sdSphere"}
         Shapes::Cube => {"sdCube"}
      };

      let size_option = match self.shape {
         Shapes::Sphere => {format!("{}", self.size.x)}
         Shapes::Cube => {format!("vec3({}, {}, {})", self.size.x, self.size.y, self.size.z)}
      };

      let p = self.position;
      let r = self.rotation;

      let pos = if self.position != Vector3::zero() { format!("tr = move(tr, vec3({}, {}, {}));", p.x, p.y, p.z) } else {"//pos".to_string()};
      let rot = if self.rotation != Vector3::zero() { format!("tr = rot3D(tr, vec3({}, {}, {}));",  r.x, r.y, r.z) } else {"//rot".to_string()};
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
         screen_position: Pos2::zeroed(),
         screen_size: Vec2::zeroed(),
         title: "default".to_string(),

         shape: Shapes::Sphere,
         scale: 1.0,
         size: Vector3::new(1.0, 1.0, 1.0),
         position: Vector3::new(0.0, 0.0, 0.0),
         rotation: Vector3::new(0.0, 0.0, 0.0),
      }
   }
}