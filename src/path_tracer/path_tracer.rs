use std::borrow::Cow;
use std::cmp::PartialEq;
use std::fs;
use std::path::Path;
use std::time::Instant;
use cgmath::Vector2;
use egui::Ui;
use wgpu::{BindGroup, BindGroupLayout, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, PipelineLayout, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};
use wgpu::naga::{FastHashMap, ShaderStage};
use winit::keyboard::KeyCode;
use crate::{defaults_and_sliders_gui, defaults_only_gui};
use crate::inbuilt::setup::Setup;
use crate::packages::glsl_preprocessor::GlslPreprocessor;
use crate::packages::input_manager_package::InputManager;
use crate::packages::time_package::TimePackage;
use crate::utility::structs::{StorageTexturePackage, UniformPackageSingles};

pub struct PathTracer {
   pub pipeline_layout: PipelineLayout,
   pub pipeline: ComputePipeline,
   pub constants: UniformPackageSingles<Constants>,
   pub settings: UniformPackageSingles<Settings>,

   pub changed: bool,
}

impl PathTracer {
   pub fn new(setup: &Setup, storage_texture_package: &StorageTexturePackage, map: String, data_array_layout: &BindGroupLayout) -> Self {

      let constants = UniformPackageSingles::create(&setup, ShaderStages::COMPUTE, Constants::default());
      let settings = UniformPackageSingles::create(&setup, ShaderStages::COMPUTE, Settings::default());

      let shader = Self::load_shader(setup, map);

      let compute_pipeline_layout = setup.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
         label: Some("compute Pipeline Layout"),
         bind_group_layouts: &[
            &storage_texture_package.bind_group_layout,
            &constants.layout,
            &settings.layout,
            &data_array_layout,
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
         pipeline_layout: compute_pipeline_layout,
         pipeline: compute_pipeline,
         constants,
         settings,

         changed: false,
      }
   }

   pub fn remake_pipeline(&mut self, setup: &Setup, map: String) {

      let shader = Self::load_shader(setup, map);

      let st = Instant::now();

      self.pipeline = setup.device.create_compute_pipeline(&ComputePipelineDescriptor {
         label: Some("compute path trace pipeline"),
         layout: Some(&self.pipeline_layout),
         module: &shader,
         entry_point: "main",
      });

      println!("{:?}", st.elapsed());
   }

   pub fn load_shader(setup: &Setup, map: String) -> ShaderModule {
      let main_path = Box::from(Path::new("assets/shaders/path_tracer/test_compute.glsl"));
      let out_path = Box::from(Path::new("assets/shaders/path_tracer/shader_out/test_compute.glsl"));

      GlslPreprocessor::do_the_thing(&main_path, &out_path, vec![("map".to_string(), map)]);

      let source = fs::read_to_string(out_path).unwrap();

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
      let settings = &mut self.settings;

      if resized || self.changed || input_manager.is_key_just_pressed(KeyCode::Space) {
         let size = Vector2::new(setup.size.width as f32 * settings.data.scale, setup.size.height as f32 * settings.data.scale);
         storage_texture_package.remake(setup, size.into());

         constants.data.last_clear = 0;
      }

      constants.data.time = time_package.start_time.elapsed().as_secs_f32();
      constants.data.aspect = setup.aspect();
      constants.data.frame += 1;
      constants.data.last_clear += 1;


      constants.update_with_data(&setup.queue);
      settings.update_with_data(&setup.queue);

      self.changed = false;
   }

   pub fn gui(&mut self, ui: &mut Ui) {
      let sc = self.settings.data.clone();
      self.constants.data.ui(ui);
      self.settings.data.ui(ui);

      if sc != self.settings.data { self.changed = true;}
   }

   pub fn compute_pass(&self, encoder: &mut CommandEncoder, render_texture: &StorageTexturePackage, data_array: &BindGroup) {
      let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("Compute pass"), timestamp_writes: None });

      compute_pass.set_pipeline(&self.pipeline);

      // bind groups
      compute_pass.set_bind_group(0, &render_texture.bind_group, &[]);

      compute_pass.set_bind_group(1, &self.constants.bind_group, &[]);
      compute_pass.set_bind_group(2, &self.settings.bind_group, &[]);

      compute_pass.set_bind_group(3, data_array, &[]);

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

defaults_and_sliders_gui!(
   Settings,
   debug: i32 = 1 => 0..=3,
   bounces: i32 = 8 => 0..=32,
   scale: f32 = 1.0 => 0.1..=1.0,
   fov: f32 = 1.0 => 0.0..=5.0,
   aabb: i32 = 0 => 0..=1
);