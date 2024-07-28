use std::fs;
use std::path::Path;
use std::time::Instant;

pub struct GlslPreprocessor;
impl GlslPreprocessor {
   pub fn do_the_thing(path: &Box<Path>, output: &Box<Path>) {
      let st = Instant::now();
      let mut shader_code = fs::read_to_string(path)
          .expect("couldn't find input");

      shader_code = Self::handle_hash(shader_code, path);
      println!("handled directives");

      fs::write(output, shader_code).expect("failed to write to output path");

      println!("processed {:?} in {:?}\n////////////////////\n", path, st.elapsed());
   }

   fn handle_hash(shader_code: String, start_path: &Box<Path>) -> String {
      let mut out = String::new();

      let mut past_includes = vec![];
      for line in shader_code.lines() {
         if let Some(hash_pos) = line.find("#") {

            let directive_full = &line[hash_pos+1..].trim();
            let parts: Vec<&str> = directive_full.split_whitespace().collect();

            let directive = parts[0];
            let following_text = parts.get(1).unwrap_or(&"");


            if directive == "test" {
               out.push_str("// Found #test\n");
               continue;
            }

            else if directive == "include" {
               if past_includes.contains(following_text) {
                  out.push_str(format!("// ignored repeat include {following_text}").as_str());
                  out.push('\n');
                  continue;
               }

               let target_dir = start_path.parent().unwrap();

               let following_path = following_text.replace("\"", "");
               let target_file_path = target_dir.join(following_path);

               if fs::metadata(&target_file_path).is_ok() {
                  let target_code = fs::read_to_string(&target_file_path).unwrap();

                  out.push_str(format!("// included {:?}\n", &target_file_path).as_str());
                  out.push_str(target_code.as_str());
                  out.push_str("\n// end include\n");

                  past_includes.push(following_text);
                  println!("handled include {} for {:?}", following_text, target_file_path)

               } else {
                  panic!("could not find target include, {:?}", target_file_path);
               }

            }

            else {
               out.push_str(line);
               out.push('\n')
            }

         }
         else {
            out.push_str(line);
            out.push('\n')
         }
      }

      out
   }
}