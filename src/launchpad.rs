use crate::gl_helper::model::Model;
use cgmath::{Matrix4, Vector3, };
use crate::gl_helper::instance_model::ModelInstance;
use crate::game::{MovementAndCollision, };
use crate::{gl, };
//use crate::ground::{Ground, };
//use crate::flying_camera::Flying_Camera;
//use std::ops::{AddAssign, Add, Mul};
//use crate::landscape::{SQUARE_COLUMNS, SQUARE_SIZE};
//use crate::gl_helper::texture::create_texture;

pub struct LaunchPad {
    pub(crate) model_instance: ModelInstance,
    pub(crate) movement_collision: MovementAndCollision,
    matrix:Matrix4<f32>,
}
impl LaunchPad {
    pub fn new(gl: &gl::Gl,position:Vector3<f32>,launchpad_model:&Model) -> LaunchPad {

        let model = launchpad_model.clone();
        let model_instance = ModelInstance::new(gl,model, 0.01,None);
        LaunchPad {
            model_instance,
            movement_collision:MovementAndCollision::new(0.4, position),
            matrix:Matrix4::from_translation(position),
        }
    }
    //pub fn update(&mut self, _delta: f32, _ground:&Ground, _camera: &Flying_Camera) { }
    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>,our_shader:u32,wrapped_position:Vector3<f32>) {

        self.model_instance.matrix = self.matrix *Matrix4::from_translation(wrapped_position);
        self.model_instance.render(gl, &view, &projection,our_shader,false);
    }
}