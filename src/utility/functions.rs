use egui::{Context, Pos2};

#[allow(dead_code)]
pub fn round_to_x_decimals(num: f32, decimals: u32) -> f32 {
   let multiplier = 10f32.powi(decimals as i32);
   (num * multiplier).round() / multiplier
}


#[allow(dead_code)]
pub fn wait(ms: u64) {
   std::thread::sleep(std::time::Duration::from_millis(ms));
}


#[allow(dead_code)]
pub fn get_mouse_position(ctx: &Context) -> Pos2 {
   ctx.input(|i| i.pointer.hover_pos().unwrap_or(Pos2::new(0.0, 0.0)))
}





