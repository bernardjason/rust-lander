use cgmath::{Matrix4, Matrix, Vector3, Array};
use crate::gl;
use std::ffi::CString;

pub(crate) mod texture;
pub(crate) mod shader;
pub(crate) mod model;
pub mod instance_model;
pub mod draw_text;
pub(crate) mod loading_screen;
//pub(crate) mod sprite;
//pub(crate) mod vertex;

pub fn gl_matrix4(gl: &gl::Gl, shader_program:u32,mat4:Matrix4<f32>, name:&str) {
    unsafe {
        let location = gl.GetUniformLocation(shader_program, CString::new(name).unwrap().as_ptr());
        gl.UniformMatrix4fv(
            location,
            1,
            gl::FALSE,
            mat4.as_ptr(),
        );
    }
}

pub fn gl_vec3(gl: &gl::Gl, shader_program:u32,vec3:Vector3<f32>, name:&str) {
    unsafe {
        let location = gl.GetUniformLocation(shader_program, CString::new(name).unwrap().as_ptr());
        gl.Uniform3fv(location, 1, vec3.as_ptr());
    }
}
pub fn gl_int(gl: &gl::Gl, shader_program:u32,value:i32, name:&str) {
    unsafe {
        let location = gl.GetUniformLocation(shader_program, CString::new(name).unwrap().as_ptr());
        gl.Uniform1i(location, value);
    }
}

#[allow(dead_code)]
pub fn gl_float(gl: &gl::Gl, shader_program:u32,value:f32, name:&str) {
    unsafe {
        let location = gl.GetUniformLocation(shader_program, CString::new(name).unwrap().as_ptr());
        gl.Uniform1f(location, value);
    }
}
