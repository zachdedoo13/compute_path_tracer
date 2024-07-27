use std::fs;
use std::path::Path;
use std::time::Instant;

pub struct GlslPreprocessor;
impl GlslPreprocessor {
   pub fn do_the_thing(path: &Box<Path>, output: &Box<Path>, exceptions: Vec<&str>) {
      let st = Instant::now();
      let mut shader_code = fs::read_to_string(path)
          .expect("couldn't find input");

      shader_code = Self::handle_hash(shader_code, path);
      println!("handled hash");

      shader_code = Self::handle_orderless_glory(shader_code, exceptions);
      println!("handled orderless_glory");

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

                  out.push_str(format!("// included {:?}", &target_file_path).as_str());
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

   fn handle_orderless_glory(shader_code: String, exceptions: Vec<&str>) -> String {
      let mut funcs: String = String::new();
      let mut structs: String = String::new();

      let mut struct_names = vec![];

      let mut back = String::new();

      for line in shader_code.lines() {
         let words: Vec<&str> = line.split_whitespace().collect();

         if let Some(word) = words.get(0) {

            if word == &"struct" {
               structs.push_str(line);
               structs.push_str("\n");

               struct_names.push(words[1]);
               continue;
            }

            if Consts::GLSL_TYPES.contains(word) || struct_names.contains(word) {
               'oawowonalaonamwioawidjoiawjodiajwoidawoirjiawr: loop {
                  if let Some(name) = words.get(1) {
                     if !name.contains(&"(") {
                        break;
                     }
                     for exception in exceptions.iter() {
                        if name.contains(exception) {
                           break 'oawowonalaonamwioawidjoiawjodiajwoidawoirjiawr;
                        }
                     }
                  }

                  let mut func = String::new();
                  for word in words.iter() {
                     if word != &"{" {
                        func.push_str(word);
                        if !word.contains(")") { func.push_str(" "); }
                     }
                  }
                  func.push_str(";\n");
                  funcs.push_str(func.as_str());
                  break;
               }
            }
         }

         back.push_str(line);
         back.push_str("\n");
      }

      let mut lines: Vec<&str> = back.lines().collect();
      lines.insert(1, &structs);
      lines.insert(2, &funcs);
      let the_back = lines.join("\n");

      return the_back;
   }
}


struct Consts;
impl Consts {
   const GLSL_TYPES: [&'static str; 100] = [
      "void", "bool", "int", "float", "double", "uint",
      "vec2", "vec3", "vec4",
      "bvec2", "bvec3", "bvec4",
      "ivec2", "ivec3", "ivec4",
      "uvec2", "uvec3", "uvec4",
      "dvec2", "dvec3", "dvec4",
      "mat2", "mat3", "mat4",
      "mat2x2", "mat2x3", "mat2x4",
      "mat3x2", "mat3x3", "mat3x4", "mat4x2", "mat4x3", "mat4x4",
      "sampler1D", "sampler2D", "sampler3D", "samplerCube",
      "sampler1DShadow", "sampler2DShadow", "samplerCubeShadow",
      "sampler1DArray", "sampler2DArray",
      "sampler1DArrayShadow", "sampler2DArrayShadow",
      "sampler2DMS", "sampler2DMSArray",
      "samplerCubeArray", "samplerCubeArrayShadow",
      "isampler1D", "isampler2D", "isampler3D", "isamplerCube",
      "isampler1DArray", "isampler2DArray",
      "isampler2DMS", "isampler2DMSArray",
      "isamplerCubeArray",
      "usampler1D", "usampler2D", "usampler3D", "usamplerCube",
      "usampler1DArray", "usampler2DArray",
      "usampler2DMS", "usampler2DMSArray",
      "usamplerCubeArray",
      "image1D", "image2D", "image3D", "imageCube",
      "image2DRect", "imageBuffer",
      "image1DArray", "image2DArray",
      "imageCubeArray",
      "image2DMS", "image2DMSArray",
      "iimage1D", "iimage2D", "iimage3D", "iimageCube",
      "iimage2DRect", "iimageBuffer",
      "iimage1DArray", "iimage2DArray",
      "iimageCubeArray",
      "iimage2DMS", "iimage2DMSArray",
      "uimage1D", "uimage2D", "uimage3D", "uimageCube",
      "uimage2DRect", "uimageBuffer",
      "uimage1DArray", "uimage2DArray",
      "uimageCubeArray",
      "uimage2DMS", "uimage2DMSArray",
      "atomic_uint"
   ];
}
