pub mod state;

pub mod path_tracer {
   pub mod path_tracer;
}


pub mod inbuilt {
   pub mod setup;
   pub mod vertex_library;
   pub mod vertex_package;
   pub mod event_loop;
   pub mod gui;
}

pub mod packages {
   pub mod glsl_preprocessor;
   pub mod time_package;
   pub mod input_manager_package;
}

pub mod pipelines {
   pub mod render_texture_pipeline;
}

pub mod utility {
   pub mod macros;
   pub mod functions;
   pub mod structs;
}

pub mod sdf_editor {
   pub mod sdf_editor;
   // pub mod sdf_editor_old;
   pub mod primitives;
   pub mod data_structures;
   pub mod containers;
}
