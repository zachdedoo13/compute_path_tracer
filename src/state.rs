use std::iter;
use egui::{Align2, Context, Frame, Style, Ui};
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

   // packages
   time_package: TimePackage,
   input_manager: InputManager,

   render_texture: StorageTexturePackage,
   render_texture_pipeline: RenderTexturePipeline,

   path_tracer: PathTracer,
}

impl<'a> State<'a> {
   pub async fn new(window: &'a Window) -> State<'a> {

      // dependents
      let setup = Setup::new(window).await;
      let egui = EguiRenderer::new(&setup.device, setup.config.format, None, 1, setup.window);


      // packages
      let time_package = TimePackage::new();
      let input_manager = InputManager::new();


      let render_texture = StorageTexturePackage::new(&setup, (1800, 1800));
      let render_texture_pipeline = RenderTexturePipeline::new(&setup, &render_texture);

      let path_tracer = PathTracer::new(&setup, &render_texture);

      Self {
         setup,
         egui,

         time_package,
         input_manager,

         render_texture,
         render_texture_pipeline,

         path_tracer,
      }
   }

   pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
      if new_size.width > 0 && new_size.height > 0 {
         self.setup.size = new_size;
         self.setup.config.width = new_size.width;
         self.setup.config.height = new_size.height;
         self.setup.surface.configure(&self.setup.device, &self.setup.config);
      }
   }

   pub fn update_input(&mut self, event: &WindowEvent) -> bool {
      self.input_manager.process_event(event);
      false
   }

   pub fn update(&mut self) {
      self.time_package.update();

      self.path_tracer.update(&self.setup, &self.render_texture, &self.time_package);

      self.input_manager.reset();
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
