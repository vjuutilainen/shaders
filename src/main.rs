// Mostly combined from examples at:
// https://github.com/tomaka/glutin
// https://github.com/brendanzab/gl-rs/blob/master/gl/examples/triangle.rs

#![feature(duration_as_u128)]

use gl::types::*;
use glutin::dpi::LogicalSize;
use glutin::{ContextBuilder, EventsLoop, GlWindow, WindowBuilder};
use shaders::utils::compile_shader;
use shaders::utils::link_program;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::time::Instant;

static VERTEX_DATA: [GLfloat; 8] = [
    -1.0, -1.0, // bottom left
    1.0, -1.0, // bottom right
    1.0, 1.0, // top right
    -1.0, 1.0, // top left
];

static VS_SRC: &'static str = "
#version 150

in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}

";

static FS_SRC: &'static str = "
#version 150

uniform float u_time;
out vec4 out_color;

void main() {
    float time = abs(sin(u_time * 0.001));
    out_color = vec4(0.0, time, 1.0, 1.0);
}

";

fn main() {
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Project")
        .with_dimensions(LogicalSize::new(1024.0, 768.0));
    let context = ContextBuilder::new().with_vsync(true);
    let gl_window = GlWindow::new(window_builder, context, &events_loop).unwrap();

    unsafe {
        use gl;
        use glutin::GlContext;

        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    }

    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;

    let u_time;

    unsafe {
        use gl;

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&VERTEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        gl::UseProgram(program);
        gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

        let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            0,
            ptr::null(),
        );

        u_time = gl::GetUniformLocation(program, CString::new("u_time").unwrap().as_ptr());
        gl::Uniform1f(u_time, 1.0);
    }

    let mut running = true;

    let now = Instant::now();

    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => running = false,
                glutin::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    gl_window.resize(logical_size.to_physical(dpi_factor));
                }
                _ => (),
            },
            _ => (),
        });

        let timepassed = now.elapsed().as_millis();

        unsafe {
            gl::Uniform1f(u_time, timepassed as GLfloat);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }

        gl_window.swap_buffers().unwrap();
    }

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
