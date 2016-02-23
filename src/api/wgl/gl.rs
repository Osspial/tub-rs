pub mod wgl {
    include!(concat!(env!("OUT_DIR"), "/wgl.rs"));
}

pub mod wgl_ex {
    include!(concat!(env!("OUT_DIR"), "/wgl_ex.rs"));
}

#[link(name="opengl32")] extern {}