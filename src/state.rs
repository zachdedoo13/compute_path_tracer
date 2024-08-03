use egui::Label;
use std::iter;
use egui::{Align2, Context, Frame, Style, Ui};
use egui_wgpu::ScreenDescriptor;
use pollster::block_on;
use wgpu::{BufferUsages, CommandEncoder, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, TextureView, TextureViewDescriptor};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;
use crate::inbuilt::gui::EguiRenderer;
use crate::inbuilt::setup::Setup;
use crate::packages::input_manager_package::InputManager;
use crate::packages::new_node_editor::NewNodeEditor;
use crate::packages::node_editor_package::NodeEditor;
use crate::packages::time_package::TimePackage;
use crate::path_tracer::path_tracer::PathTracer;
use crate::pipelines::render_texture_pipeline::RenderTexturePipeline;
use crate::sdf_editor::sdf_editor::SDFEditor;
use crate::utility::structs::StorageTexturePackage;

pub struct State<'a> {
   pub setup: Setup<'a>,
   pub egui: EguiRenderer,

   pub resized: bool,
   pub editor_open: bool,

   // packages
   time_package: TimePackage,
   input_manager: InputManager,

   render_texture: StorageTexturePackage,
   render_texture_pipeline: RenderTexturePipeline,

   path_tracer: PathTracer,
   node_editor: NodeEditor,

   new_node_editor: NewNodeEditor,

   sdf_editor: SDFEditor,
}

impl<'a> State<'a> {
   pub async fn new(window: &'a Window) -> State<'a> {

      // dependents
      let setup = Setup::new(window).await;
      let egui = EguiRenderer::new(&setup.device, setup.config.format, None, 1, setup.window);


      let mut node_editor = NodeEditor::new();
      let mut new_node_editor = NewNodeEditor::new();

      let sdf_editor = SDFEditor::new();

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
         editor_open: false,

         time_package,
         input_manager,

         render_texture,
         render_texture_pipeline,

         path_tracer,

         node_editor,
         new_node_editor,
         
         sdf_editor,
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

      if self.input_manager.is_key_just_pressed(KeyCode::Space) {self.path_tracer.remake_pipeline(&self.setup, self.node_editor.generate_map())}


      if self.input_manager.is_key_just_pressed(KeyCode::KeyS) {
         self.save_image();
      }


      self.sdf_editor.update();


      self.input_manager.reset();
      self.resized = false;
   }

   pub fn update_gui(&mut self, view: &TextureView, encoder: &mut CommandEncoder) {
      let screen_descriptor = ScreenDescriptor {
         size_in_pixels: [self.setup.config.width, self.setup.config.height],
         pixels_per_point: self.setup.window.scale_factor() as f32,
      };

      let mut save_image = false;

      let run_ui = |context: &Context| {
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

            {
               egui::CollapsingHeader::new("Other")
                   .default_open(true)
                   .show(ui, |ui| {
                      if ui.add(egui::Button::new("Save Image")).clicked() {
                         save_image = true;
                      }

                      if ui.add(egui::Button::new("Map Editor")).clicked() {
                         self.editor_open = !self.editor_open;
                      }
                      ui.end_row();
                   });
            }

         };

         // Pre draw setup
         egui::Window::new("Path Tracer")
             .default_open(true)
             .max_width(1000.0)
             .max_height(800.0)
             .default_width(800.0)
             .resizable(true)
             .anchor(Align2::LEFT_TOP, [0.0, 0.0])
             .frame(Frame::window(&Style::default()))
             .show(&context, code);

         if self.editor_open {
            self.node_editor.ui(context, &mut self.path_tracer, &self.input_manager, &self.setup, &mut self.resized);
         }

         self.new_node_editor.ui(context);
         
         self.sdf_editor.ui(context);
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

      if save_image { self.save_image(); }
   }

   pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let output = self.setup.surface.get_current_texture()?;
      let view = output.texture.create_view(&TextureViewDescriptor::default());
      let mut encoder = self.setup.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
         label: Some("Render Encoder"),
      });

      {
         // self.path_tracer.compute_pass(&mut encoder, &self.render_texture);
         self.render_texture_pipeline.render_pass(&mut encoder, &view, &self.render_texture);
      }

      self.update_gui(&view, &mut encoder);

      self.setup.queue.submit(iter::once(encoder.finish()));

      output.present();

      Ok(())
   }

   fn save_image(&mut self) {
      let staging_buffer = self.setup.device.create_buffer(&wgpu::BufferDescriptor {
         label: None,
         size: ((self.render_texture.size.width as f32 * self.render_texture.size.height as f32) * (4.0 * std::mem::size_of::<f32>() as f32)) as u64,
         usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
         mapped_at_creation: false,
      });

      let copy_staging_buffer = ImageCopyBuffer {
         buffer: &staging_buffer,
         layout: ImageDataLayout {
            offset: 0,
            bytes_per_row: Some((4 * std::mem::size_of::<f32>() as u32) * (self.render_texture.size.width)),
            rows_per_image: Some(self.render_texture.size.height),
         },
      };

      let image_copy_texture = ImageCopyTexture {
         texture: &self.render_texture.texture,
         mip_level: 0,
         origin: Default::default(),
         aspect: Default::default(),
      };

      let mut encoder = self.setup.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("texture copy encoder") });

      encoder.copy_texture_to_buffer(image_copy_texture, copy_staging_buffer, self.render_texture.size);
      self.setup.queue.submit(iter::once(encoder.finish()));

      let buffer_slice = staging_buffer.slice(..);
      let (sender, receiver) = flume::bounded(1);
      buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

      self.setup.device.poll(wgpu::Maintain::wait()).panic_on_timeout();

      let future = async {
         if let Ok(Ok(())) = receiver.recv_async().await {
            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let width = self.render_texture.size.width;
            let height = self.render_texture.size.height;

            let result: &[f32] = bytemuck::cast_slice(&data);
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
               let idx = ((height - 1 - y) * width + x) as usize * 4;
               *pixel = Rgba([
                  (result[idx].powf(1.0 / 2.2) * 255.0) as u8, // Apply gamma correction
                  (result[idx + 1].powf(1.0 / 2.2) * 255.0) as u8,
                  (result[idx + 2].powf(1.0 / 2.2) * 255.0) as u8,
                  (result[idx + 3] * 255.0) as u8,
               ]);
            }

            img.save("image.png").unwrap();

            println!("done");


            drop(data);
            staging_buffer.unmap();
         }
      };

      block_on(future);
   }
}
