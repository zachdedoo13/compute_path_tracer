use std::f32::consts::PI;
use std::iter;
use cgmath::Vector3;
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


      // packages
      let time_package = TimePackage::new();
      let input_manager = InputManager::new();


      let render_texture = StorageTexturePackage::new(&setup, (10.0, 10.0));
      let render_texture_pipeline = RenderTexturePipeline::new(&setup, &render_texture);

      let path_tracer = PathTracer::new(&setup, &render_texture);

      let node_editor = NodeEditor::new();

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
                      ui.add(egui::Label::new(format!("FPS: {}", &self.time_package.fps)));

                      ui.add(egui::Label::new(
                         format!("Screen: {} x {} = {}",
                                 &self.setup.size.width
                                 , &self.setup.size.height,
                                 &self.setup.size.width * &self.setup.size.height
                         )
                      ));

                      ui.add(egui::Label::new(
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

         self.node_editor.ui(ui)
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

   pub fn ui(&mut self, ui: &Context) {
      let mut changed = false;

      egui::Window::new("Map editor")
          .default_open(false)
          .resizable(true)
          .default_size(Vec2::new(100.0, 100.0))
          .max_size(self.max_size)
          .frame(Frame::window(&Style::default()))
          .show(&ui, |ui| {
             if ui.button("add").clicked() {
                self.nodes.push(Node::new(Pos2::new(50.0, 50.0), Vec2::new(100.0, 50.0), format!("num={}", self.nodes.len())))
             }

             ui.allocate_space(ui.available_size());

             let mut kill = None;
             for (i, node) in self.nodes.iter_mut().enumerate() {
                let original_node = node.clone();
                if node.draw(ui, ui.min_rect().min, self.max_size) {
                   kill = Some(i)
                }

                if *node != original_node {
                   changed = true
                }

             }
             if let Some(index) = kill { self.nodes.remove(index); }
          });

      if changed {
         println!("changed")
      }

   }
}


#[derive(Clone)]
#[derive(PartialEq)]
struct Node {
   screen_position: Pos2,
   size: Vec2,
   title: String,

   position: Vector3<f32>,
   rotation: Vector3<f32>,
}
impl Node {
   fn new(screen_position: Pos2, size: Vec2, title: String) -> Self {
      Self {
         screen_position,
         size,
         title,

         position: Vector3::new(0.0, 0.0, 0.0),
         rotation: Vector3::new(0.0, 0.0, 0.0),
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

                egui::CollapsingHeader::new("position")
                    .default_open(false)
                    .show(ui, |ui| {
                       ui.add(egui::DragValue::new(&mut self.position.x).speed(0.01).clamp_range(-100.0..=100.0).prefix("X: "));
                       ui.add(egui::DragValue::new(&mut self.position.y).speed(0.01).clamp_range(-100.0..=100.0).prefix("Y: "));
                       ui.add(egui::DragValue::new(&mut self.position.z).speed(0.01).clamp_range(-100.0..=100.0).prefix("Z: "));
                    });

                egui::CollapsingHeader::new("rotation")
                    .default_open(false)
                    .show(ui, |ui| {
                       ui.add(egui::DragValue::new(&mut self.rotation.x).speed(0.01).clamp_range(-PI..=PI).prefix("X: "));
                       ui.add(egui::DragValue::new(&mut self.rotation.y).speed(0.01).clamp_range(-PI..=PI).prefix("Y: "));
                       ui.add(egui::DragValue::new(&mut self.rotation.z).speed(0.01).clamp_range(-PI..=PI).prefix("Z: "));
                    });

             });
      };

      contents
   }

   fn draw(&mut self, ui: &mut Ui, window_pos: Pos2, max_size: Vec2) -> bool {
      let rect = Rect::from_min_size(window_pos + self.screen_position.to_vec2(), self.size);
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
}