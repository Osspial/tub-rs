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
    out: gfx::RenderTarget<gfx::format::Srgb8> = "o_Color",
});

fn main() {
    use gfx::traits::{Device, FactoryExt};

    let window = Window::new("Windowy shit", &Default::default()).unwrap();
    let (context, mut device, mut factory, main_color, _) =
        init::<gfx::format::Srgb8, gfx::format::Depth>(&window);
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
        encoder.clear(&data.out, [0.1, 0.2, 0.3]);
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


fn init<'w, Cf, Df>(window: &'w tub::platform::Window) ->
            (tub::platform::GlContext<'w>,
             gfx_device_gl::Device, gfx_device_gl::Factory,
             handle::RenderTargetView<R, Cf>, handle::DepthStencilView<R, Df>)
where
    Cf: format::RenderFormat,
    Df: format::DepthFormat,
{
    use gfx_core::factory::Phantom;
    let (context, device, factory, color_view, ds_view) = init_raw(window, Cf::get_format(), Df::get_format());
    (context, device, factory, Phantom::new(color_view), Phantom::new(ds_view))
}

fn init_raw<'w>(window: &'w tub::platform::Window,
                color_format: format::Format, ds_format: format::Format) ->
                (tub::platform::GlContext<'w>,
                gfx_device_gl::Device, gfx_device_gl::Factory,
                handle::RawRenderTargetView<R>, handle::RawDepthStencilView<R>)
{
    let context = {
        let color_total_bits = color_format.0.get_total_bits();
        let alpha_bits = color_format.0.get_alpha_stencil_bits();
        let depth_total_bits = ds_format.0.get_total_bits();
        let stencil_bits = ds_format.0.get_alpha_stencil_bits();
        let pix_format = tub::PixelFormat {
            depth_bits: depth_total_bits - stencil_bits,
            stencil_bits: stencil_bits,
            color_bits: color_total_bits - alpha_bits,
            alpha_bits: alpha_bits,
            srgb: color_format.1 == format::ChannelType::Srgb,
            ..Default::default()
        };

        let context = tub::platform::GlContext::new(&window, pix_format).unwrap();
        context
    };

    unsafe { context.make_current().unwrap() };
    let (device, factory) = gfx_device_gl::create(|s|
        context.get_proc_address(s) as *const std::os::raw::c_void);

    // create the main color/depth targets
    let dim = get_window_dimensions(&window);
    let (color_view, ds_view) = gfx_device_gl::create_main_targets_raw(dim, color_format.0, ds_format.0);

    // done
    (context, device, factory, color_view, ds_view)
}

fn get_window_dimensions(window: &tub::platform::Window) -> tex::Dimensions {
    let (width, height) = window.get_inner_size().unwrap();
    let aa = 0;
    (width as tex::Size, height as tex::Size, 1, aa.into())
}