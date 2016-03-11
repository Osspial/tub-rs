extern crate tub;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;

use gfx_core::{format, handle, tex};
use gfx_device_gl::Resources as R;

use tub::platform::Window;

gfx_vertex_struct!( Vertex {
    pos: [f32; 2] = "a_Pos",
    color: [f32; 3] = "a_Color",
});

gfx_pipeline!(pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    out: gfx::RenderTarget<gfx::format::Srgba8> = "o_Color",
});

fn main() {
    use gfx::traits::{Device, FactoryExt};

    let window = init_window::<gfx::format::Srgba8, gfx::format::Depth>(
        tub::config::WindowConfig::new()
            .name("Triangle".to_owned())
            .size(Some((500, 500)))
            .borderless(false),
        tub::config::PixelFormat::new()
            .multisampling(16)
        );

    let (context, mut device, mut factory, main_color, _) =
        init_context::<gfx::format::Srgba8, gfx::format::Depth>(&window);
    let mut encoder = factory.create_encoder();

    let pso = factory.create_pipeline_simple(
        TRIANGLE_VERT.as_bytes(),
        TRIANGLE_FRAG.as_bytes(),
        gfx::state::CullFace::Nothing,
        pipe::new()
        ).unwrap();

    let vertex_data = [
        Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] },
    ];
    let (vbuf, slice) = factory.create_vertex_buffer(&vertex_data);
    let data = pipe::Data {
        vbuf: vbuf,
        out: main_color,
    };

    window.show();
    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                tub::event::Event::KeyInput(_, tub::event::VKeyCode::Escape) |
                tub::event::Event::Closed => break 'main,
                _ => {},
            }
        }

        encoder.reset();
        encoder.clear(&data.out, [0.0, 0.0, 0.0, 0.0]);
        encoder.draw(&slice, &pso, &data);

        device.submit(encoder.as_buffer());
        context.swap_buffers();
        device.cleanup();
    }
}

const TRIANGLE_FRAG: &'static str = r#"
    #version 150 core

    in vec4 v_Color;
    out vec4 o_Color;

    void main() {
        o_Color = v_Color;
    }
"#;

const TRIANGLE_VERT: &'static str = r#"
    #version 150 core

    in vec2 a_Pos;
    in vec3 a_Color;
    out vec4 v_Color;

    void main() {
        v_Color = vec4(a_Color, 1.0);
        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
"#;

fn init_window<'w, Cf, Df>(window_config: tub::config::WindowConfig, pixel_format: tub::config::PixelFormat) -> tub::platform::Window<'w>
where
    Cf: format::RenderFormat,
    Df: format::DepthFormat,
{
    let color_format = Cf::get_format();
    let ds_format = Df::get_format();

    let pix_format = {
        let color_total_bits = color_format.0.get_total_bits();
        let alpha_bits = color_format.0.get_alpha_stencil_bits();
        let depth_total_bits = ds_format.0.get_total_bits();
        let stencil_bits = ds_format.0.get_alpha_stencil_bits();
        tub::config::PixelFormat {
            depth_bits: depth_total_bits - stencil_bits,
            stencil_bits: stencil_bits,
            color_bits: color_total_bits - alpha_bits,
            alpha_bits: alpha_bits,
            srgb: Some(color_format.1 == format::ChannelType::Srgb),
            ..pixel_format
        }
    };
    tub::platform::Window::new(window_config, pix_format).unwrap()
}

fn init_context<'w, 'c, Cf, Df>(window: &'w Window) -> 
    (tub::platform::GlContext<'w, 'c>, gfx_device_gl::Device, 
    gfx_device_gl::Factory, handle::RenderTargetView<R, Cf>, 
    handle::DepthStencilView<R, Df>)
where
    Cf: format::RenderFormat,
    Df: format::DepthFormat,
{
    use gfx_core::factory::Phantom;

    let color_format = Cf::get_format();
    let ds_format = Df::get_format();
    let context = tub::platform::GlContext::new(window, None).unwrap();

    unsafe { context.make_current().unwrap() };
    let (device, factory) = gfx_device_gl::create(|s|
        context.get_proc_address(s) as *const std::os::raw::c_void);

    // create the main color/depth targets
    let dim = get_window_dimensions(&window);
    let (color_view, ds_view) = gfx_device_gl::create_main_targets_raw(dim, color_format.0, ds_format.0);

    (context, device, factory, Phantom::new(color_view), Phantom::new(ds_view))
}

fn get_window_dimensions(window: &tub::platform::Window) -> tex::Dimensions {
    let (width, height) = window.get_inner_size().unwrap();
    let aa = window.get_pixel_format().multisampling as tex::NumSamples;
    (width as tex::Size, height as tex::Size, 1, aa.into())
}