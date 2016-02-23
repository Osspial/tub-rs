extern crate gl_generator;
extern crate khronos_api;

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out = PathBuf::from(&env::var("OUT_DIR").unwrap());

    println!("rerun-if-changed=build.rs");

    if target.contains("windows") {
        let mut file = File::create(&out.join(Path::new("wgl.rs"))).unwrap();
        gl_generator::generate_bindings(
            gl_generator::StaticGenerator,
            gl_generator::registry::Ns::Wgl,
            gl_generator::Fallbacks::All,
            khronos_api::WGL_XML,
            vec![],
            "1.0", "core", &mut file
        ).unwrap();

        let mut file = File::create(&out.join(Path::new("wgl_ex.rs"))).unwrap();
        gl_generator::generate_bindings(
            gl_generator::StructGenerator,
            gl_generator::registry::Ns::Wgl,
            gl_generator::Fallbacks::All,
            khronos_api::WGL_XML,
            vec![
                "WGL_ARB_pixel_format".to_string(),
                "WGL_ARB_extensions_string".to_string(),
                "WGL_ARB_pixel_format_float".to_string(),
                "WGL_ARB_framebuffer_sRGB".to_string(),
                "WGL_ARB_multisample".to_string()
            ],
            "1.0", "core", &mut file
        ).unwrap();
    }
}