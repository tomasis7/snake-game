use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let shaders = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../shaders");
    for name in ["quad.vert", "quad.frag"] {
        let src = shaders.join(name);
        println!("cargo:rerun-if-changed={}", src.display());
        let dst = out.join(format!("{name}.spv"));
        let status = Command::new("glslc")
            .arg(&src)
            .arg("-o")
            .arg(&dst)
            .status()
            .expect("failed to run glslc");
        assert!(status.success(), "glslc failed for {name}");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
