use compute_path_tracer::inbuilt::event_loop::run;


fn main() {
    pollster::block_on(run())
}