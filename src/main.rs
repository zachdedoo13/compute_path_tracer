use std::any::Any;
use compute_path_tracer::if_is_type;
use compute_path_tracer::inbuilt::event_loop::run;

fn main() {
    pollster::block_on(run());
}