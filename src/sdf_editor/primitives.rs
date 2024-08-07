use std::collections::HashMap;
use std::ops::RangeInclusive;
use bytemuck::cast_slice;
use egui::{DragValue, Label, Ui};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ShaderStages};
use wgpu::util::DeviceExt;
use crate::inbuilt::setup::Setup;

fn gen_hash() -> u128 {
   let mut rng = StdRng::from_entropy();
   let random_value1: u128 = rng.gen();
   let random_value2: u128 = rng.gen();
   random_value1 ^ random_value2
}



pub struct CompData {
   pub checker: bool,

   pub data_array: DataArray,
   pub rec_update: RecUpdate,
}
impl CompData {
   pub fn new(setup: &Setup) -> Self {
      Self {
         checker: true,
         data_array: DataArray::new(setup),
         rec_update: RecUpdate::set_true(),
      }
   }

   pub fn check(&mut self) -> f32 {
      if self.checker {
         self.checker = !self.checker;
         1.0
      } else {
         self.checker = !self.checker;
         0.8
      }
   }

   pub fn reset_data_array(&mut self) {
      self.data_array.data = vec![6969.69];
      self.data_array.seen.clear();
   }
}

pub struct DataArray {
   pub data: Vec<f32>,
   pub counter: u32,
   pub seen: HashMap<u128, usize>,

   pub bind_group_layout: BindGroupLayout,
   pub bind_group: BindGroup,
   pub buffer: Buffer,
}
impl DataArray {
   pub fn new(setup: &Setup) -> Self {

      let data: Vec<f32> = vec![6969.69];

      let bind_group_layout = setup.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
         label: Some("Data bind group"),
         entries: &[
            BindGroupLayoutEntry {
               binding: 0,
               visibility: ShaderStages::COMPUTE,
               ty: BindingType::Buffer {
                  ty: BufferBindingType::Storage { read_only: false },
                  has_dynamic_offset: false,
                  min_binding_size: None,
               },
               count: None,
            }
         ],
      });

      let buffer = setup.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Data Buffer"),
         contents: cast_slice(&data),
         usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

      let bind_group = setup.device.create_bind_group(&wgpu::BindGroupDescriptor {
         layout: &bind_group_layout,
         entries: &[
            wgpu::BindGroupEntry {
               binding: 0,
               resource: buffer.as_entire_binding(),
            },
         ],
         label: Some("Data Bind Group"),
      });

      Self {
         data: vec![],
         counter: 0,
         seen: Default::default(),

         bind_group_layout,
         bind_group,
         buffer
      }
   }

   pub fn get_index(&mut self, in_data: f32, hash: u128) -> String {
      if self.seen.contains_key(&hash) {
         let i = self.seen.get(&hash).unwrap();
         format!("data[{i}]")
      }
      else {
         self.data.push(in_data);
         let i = self.data.len() - 1;
         self.seen.insert(hash, i);

         format!("data[{i}]")
      }
   }

   pub fn update(&mut self, setup: &Setup) {
      // Recreate the buffer with the updated data size
      self.buffer = setup.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Updated Data Buffer"),
         contents: cast_slice(&self.data),
         usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

      // Update the bind group to use the new buffer
      self.bind_group = setup.device.create_bind_group(&wgpu::BindGroupDescriptor {
         layout: &self.bind_group_layout,
         entries: &[
            wgpu::BindGroupEntry {
               binding: 0,
               resource: self.buffer.as_entire_binding(),
            },
         ],
         label: Some("Updated Data Bind Group"),
      });

      println!("{:?}", self.data)
   }

   pub fn refresh(&mut self, hash: u128, data: f32) {
      let entry = self.seen.get(&hash).expect("Something fucked up");
      self.data[*entry] = data;
   }
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RecUpdate {
   pub queue_compile: bool,
   pub queue_update: bool,
}
impl RecUpdate {
   pub fn set_false() -> Self {
      Self {
         queue_update: false,
         queue_compile: false,
      }
   }

   pub fn set_true() -> Self {
      Self {
         queue_update: true,
         queue_compile: true,
      }
   }

   pub fn reset(&mut self) {
      self.queue_compile = false;
      self.queue_update = false;
   }

   pub fn update(&mut self) { self.queue_update = true; }

   pub fn compile(&mut self) { self.queue_compile = true; }

   pub fn both(&mut self) { self.queue_compile = true; self.queue_update = true; }
}



// speeds
pub const S1: f32 = 0.001;
pub const S2: f32 = 0.01;
pub const S3: f32 = 0.1;


////////////////
// Primitives //
////////////////
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Float {
   val: f32,
   range: RangeInclusive<f32>,
   speed: f32,

   name: String,
   hash: u128,
}
impl Float {
   // creation
   pub fn new(name: &str, speed: f32, def: f32, range: RangeInclusive<f32>) -> Self {
      Self {
         val: def,
         range,
         speed,
         name: name.to_string(),
         hash: gen_hash(),
      }
   }
   pub fn inv(name: &str, speed: f32, def: f32) -> Self {
      Self {
         val: def,
         name: name.to_string(),
         range: -f32::MAX..=f32::MAX,
         speed,
         hash: gen_hash(),
      }
   }
   pub fn percent(name: &str, speed: f32, def: f32) -> Self {
      Self {
         val: def,
         name: name.to_string(),
         range: 0.0..=1.0,
         speed,
         hash: gen_hash(),
      }
   }

   // display
   pub fn ui(&mut self, ui: &mut Ui) {
      ui.add(
         DragValue::new(&mut self.val)
             .speed(self.speed)
             .clamp_range(self.range.clone())
             .prefix(format!("{}: ", self.name))
      );

   }

   // compile
   pub fn compile(&self, comp_data: &mut CompData) -> String {

      let index = comp_data.data_array.get_index(self.val, self.hash);

      return format!("{}", index)
   }

   pub fn refresh(&self, comp_data: &mut CompData) {
      comp_data.data_array.refresh(self.hash, self.val);
   }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct V3 {
   x: Float,
   y: Float,
   z: Float,
   name: String,
}
impl V3 {
   // creation
   pub fn xyz(name: &str, speed: f32, def: f32) -> Self {
      Self {
         x: Float::inv("X", speed, def),
         y: Float::inv("Y", speed, def),
         z: Float::inv("Z", speed, def),
         name: name.to_string(),
      }
   }
   pub fn rgb(name: &str) -> Self {
      Self {
         x: Float::inv("R", 1.0, 1.0),
         y: Float::inv("G", 1.0, 1.0),
         z: Float::inv("B", 1.0, 1.0),
         name: name.to_string(),
      }
   }

   // display
   pub fn separate_values(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         ui.add(Label::new(format!("{}", self.name)));
         ui.horizontal(|ui| {
            self.x.ui(ui);
            self.y.ui(ui);
            self.z.ui(ui);
         });
      });
   }
   pub fn color_ui(&mut self, ui: &mut Ui) {
      let mut col = [self.x.val, self.y.val, self.z.val];
      ui.horizontal(|ui| {
         ui.add(Label::new(format!("{}", self.name)));
         ui.color_edit_button_rgb(&mut col);
      });

      self.x.val = col[0]; self.y.val = col[1]; self.z.val = col[2];
   }

   // compile
   pub fn compile(&self, comp_data: &mut CompData) -> String {
      return format!("vec3({}, {}, {})", self.x.compile(comp_data), self.y.compile(comp_data), self.z.compile(comp_data))
   }
   pub fn refresh(&self, comp_data: &mut CompData) {
      self.x.refresh(comp_data);
      self.y.refresh(comp_data);
      self.z.refresh(comp_data);
   }
}


