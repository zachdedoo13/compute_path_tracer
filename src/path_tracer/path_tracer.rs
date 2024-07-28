use std::borrow::Cow;
use std::fs;
use std::path::Path;
use cgmath::Vector2;
use egui::Ui;
use wgpu::{CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};
use wgpu::naga::{FastHashMap, ShaderStage};
use winit::keyboard::KeyCode;
use crate::defaults_only_gui;
use crate::inbuilt::setup::Setup;
use crate::packages::glsl_preprocessor::GlslPreprocessor;
use crate::packages::input_manager_package::InputManager;
use crate::packages::time_package::TimePackage;
use crate::utility::structs::{StorageTexturePackage, UniformPackageSingles};

pub struct PathTracer {
   pub pipeline: ComputePipeline,
   pub constants: UniformPackageSingles<Constants>
}
impl PathTracer {
   pub fn new(setup: &Setup, storage_texture_package: &StorageTexturePackage) -> Self {

      let constants = UniformPackageSingles::create(&setup, ShaderStages::COMPUTE, Constants::default());

      let shader = Self::load_shader(setup);

      let compute_pipeline_layout = setup.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
         label: Some("compute Pipeline Layout"),
         bind_group_layouts: &[
            &storage_texture_package.bind_group_layout,
            &constants.layout,
         ],
         push_constant_ranges: &[],
      });
      let compute_pipeline = setup.device.create_compute_pipeline(&ComputePipelineDescriptor {
         label: Some("compute path trace pipeline"),
         layout: Some(&compute_pipeline_layout),
         module: &shader,
         entry_point: "main",
      });

      Self {
         pipeline: compute_pipeline,
         constants,
      }
   }

   pub fn remake_pipeline(&mut self, setup: &Setup, storage_texture_package: &StorageTexturePackage) {
      let shader = Self::load_shader(setup);

      let compute_pipeline_layout = setup.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
         label: Some("compute Pipeline Layout"),
         bind_group_layouts: &[
            &storage_texture_package.bind_group_layout,
            &self.constants.layout,
         ],
         push_constant_ranges: &[],
      });

      self.pipeline = setup.device.create_compute_pipeline(&ComputePipelineDescriptor {
         label: Some("compute path trace pipeline"),
         layout: Some(&compute_pipeline_layout),
         module: &shader,
         entry_point: "main",
      });
   }

   pub fn load_shader(setup: &Setup) -> ShaderModule {
      let main_path = Box::from(Path::new("src/path_tracer/shaders/test_compute.glsl"));
      let out_path = Box::from(Path::new("src/path_tracer/shader_output/shader_output.glsl"));

      GlslPreprocessor::do_the_thing(&main_path, &out_path);

      let source = fs::read_to_string("src/path_tracer/shader_output/shader_output.glsl").unwrap();

      let shader_mod = ShaderModuleDescriptor {
         label: None,
         source: ShaderSource::Glsl {
            shader: Cow::Owned(source),
            stage: ShaderStage::Compute,
            defines: FastHashMap::default(), // Adjust as needed for your shader
         }
      };
      setup.device.create_shader_module(shader_mod)
   }

   pub fn update(&mut self, setup: &Setup, storage_texture_package: &mut StorageTexturePackage, time_package: &TimePackage, input_manager: &InputManager, resized: bool) {
      let constants = &mut self.constants;

      if resized {
         let size = Vector2::new(setup.size.width as f32, setup.size.height as f32);
         storage_texture_package.remake(setup, size.into());
         println!("remade texture");

         constants.data.last_clear = 0;
      }

      constants.data.time = time_package.start_time.elapsed().as_secs_f32();
      constants.data.aspect = setup.aspect();
      constants.data.frame += 1;
      constants.data.last_clear += 1;

      if input_manager.is_key_just_pressed(KeyCode::Space) {
         constants.data.last_clear = 0;
      }


      constants.update_with_data(&setup.queue);
   }

   pub fn gui(&mut self, ui: &mut Ui) {
      self.constants.data.ui(ui);
   }

   pub fn compute_pass(&self, encoder: &mut CommandEncoder, render_texture: &StorageTexturePackage) {
      let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("Compute pass"), timestamp_writes: None });

      compute_pass.set_pipeline(&self.pipeline);

      // bind groups
      compute_pass.set_bind_group(0, &render_texture.bind_group, &[]);

      compute_pass.set_bind_group(1, &self.constants.bind_group, &[]);

      let wg = 16;
      compute_pass.dispatch_workgroups(
         (render_texture.size.width as f32 / wg as f32).ceil() as u32,
         (render_texture.size.height as f32 / wg as f32).ceil() as u32,
         1);
   }
}

defaults_only_gui!(
   Constants,
   time: f32 = 0.0,
   frame: i32 = 0,
   aspect: f32 = 0.0,
   last_clear: i32 = 0
);