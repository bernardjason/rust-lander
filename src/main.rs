use crate::game::Runtime;
use cgmath::{Vector3, Point3, Matrix4, Vector4};
//use std::time::Instant;

#[macro_use]
extern crate lazy_static;

mod game;
mod gl;
mod cube;
mod gl_helper;
mod flying_camera;
mod handle_javascript;
mod landscape;
mod openglshadowalldirections;
mod shadow_shaders;
mod openglshadow;
mod ground;
mod lander_main_player;
mod launchpad;
mod special_effects;
mod rescue;
mod high_score_table;
mod end_screen;
mod sound;

pub const WIDTH:u32=800;
pub const HEIGHT:u32=600;
pub const SCALE_TO_SCREEN: f32 = 0.043;

static mut GLOBAL_ID: u128 = 1;
pub fn get_next_id() -> u128 {
    unsafe {
        let next = GLOBAL_ID;
        GLOBAL_ID = GLOBAL_ID + 1;
        next
    }
}

pub fn vec2point(vector:Vector3<f32>) -> Point3<f32> {
    let p = Point3::new(vector.x,vector.y,vector.z);
    return p
}
pub fn point2vec(point:Point3<f32>) -> Vector3<f32> {
    let v = Vector3::new(point.x,point.y,point.z);
    return v
}

pub fn print_matrix(m:Matrix4<f32>) {
    println!("x= {}",get_vector4_as_string(m.x));
    println!("y= {}",get_vector4_as_string(m.y));
    println!("z= {}",get_vector4_as_string(m.z));
    println!("w= {}",get_vector4_as_string(m.w));

}
pub fn get_vector4_as_string(v:Vector4<f32>) -> String {
   let s = format!("{},{},{},{}",v.x,v.y,v.z,v.w);
    return s;
}

fn main() {
    let runtime = Runtime::new();

    emscripten_main_loop::run(runtime);

}
