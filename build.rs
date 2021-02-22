use std::{path::PathBuf, str::FromStr};

use glob::glob;

fn main() {
    let mut compiler = shaderc::Compiler::new().expect("Failed to create shader compiler");

    let patterns: Result<Vec<glob::Paths>, _> =
        vec![glob("./src/shaders/*.vert"), glob("./src/shaders/*.frag")]
            .into_iter()
            .collect();

    let patterns = patterns.expect("Failed to create glob pattern");
    let entries = patterns.into_iter().flatten();

    let output_dir = std::env::var("OUT_DIR").expect("Cannot read OUT_DIR env var");
    let output_dir = PathBuf::from_str(&output_dir).unwrap();

    for entry in entries {
        if let Ok(path) = entry {
            println!("cargo:rerun-if-changed={}", path.display());

            let path_str = path.to_str().expect("Failed to read path as str");

            let extension = path
                .extension()
                .expect("Failed to extract extension")
                .to_str()
                .expect("Failed to convert extension to str");

            let shader_kind = match extension {
                "vert" => shaderc::ShaderKind::Vertex,
                "frag" => shaderc::ShaderKind::Fragment,
                other => panic!("Failed to guess shader kind with extension {}", other),
            };

            let shader_source_code =
                std::fs::read_to_string(&path).expect("Failed to read shader source code");

            let filename = path.file_name().expect("Failed to extract filename");
            let spirv_extension = format!("{}.spv", extension);

            let output_path = output_dir.join(filename).with_extension(spirv_extension);

            let artifact = compiler
                .compile_into_spirv(&shader_source_code, shader_kind, path_str, "main", None)
                .expect("Failed to compiler shader");

            std::fs::write(output_path, artifact.as_binary_u8())
                .expect("Failed to write compiled artifact to disk")
        }
    }
}
