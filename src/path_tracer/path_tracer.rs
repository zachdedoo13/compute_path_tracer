use std::borrow::Cow;
use wgpu::{CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages};
use wgpu::naga::{FastHashMap, ShaderStage};
use crate::defaults_only_gui;
use crate::inbuilt::setup::Setup;
use crate::packages::time_package::TimePackage;
use crate::utility::structs::{StorageTexturePackage, UniformPackageSingles};

pub struct PathTracer {
   pub pipeline: ComputePipeline,
   pub constants: UniformPackageSingles<Constants>
}
impl PathTracer {
   pub fn new(setup: &Setup, storage_texture_package: &StorageTexturePackage) -> Self {

      let shader = Self::load_shader(setup);


      let constants = UniformPackageSingles::create(&setup, ShaderStages::COMPUTE, Constants::default());


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

   pub fn load_shader(setup: &Setup) -> ShaderModule {
      let source = include_str!("test_compute.glsl");

      let code = source;

      let shader_mod = ShaderModuleDescriptor {
         label: None,
         source: ShaderSource::Glsl {
            shader: Cow::Owned(code.to_string()),
            stage: ShaderStage::Compute,
            defines: FastHashMap::default(), // Adjust as needed for your shader
         }
      };
      setup.device.create_shader_module(shader_mod)
   }

   pub fn update(&mut self, setup: &Setup, storage_texture_package: &StorageTexturePackage, time_package: &TimePackage) {
      let mut constants = &mut self.constants;

      constants.data.time = time_package.start_time.elapsed().as_secs_f32();
      constants.data.aspect = setup.aspect();

      constants.update_with_data(&setup.queue);
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
   aspect: f32 = 0.0
);