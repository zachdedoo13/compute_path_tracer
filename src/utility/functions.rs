


#[allow(dead_code)]
pub fn round_to_x_decimals(num: f32, decimals: u32) -> f32 {
   let multiplier = 10f32.powi(decimals as i32);
   (num * multiplier).round() / multiplier
}


#[allow(dead_code)]
pub fn wait(ms: u64) {
   std::thread::sleep(std::time::Duration::from_millis(ms));
}