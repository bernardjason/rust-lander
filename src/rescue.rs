use crate::gl_helper::model::Model;
use cgmath::{Matrix4, Vector3, };
use crate::gl_helper::instance_model::ModelInstance;
use crate::game::{MovementAndCollision, };
use crate::{gl, };
//use crate::ground::{Ground, BY};
//use crate::flying_camera::Flying_Camera;
//use rand::Rng;
//use crate::landscape::{SQUARE_COLUMNS, SQUARE_SIZE, MAX};

const MODEL_HEIGHT: f32 = 0.16;
pub const RESCUE_RADIUS: f32 = 0.25;


pub struct RescueInstances {
    pub(crate) model_instance: ModelInstance,
    pub(crate) movement_collision: MovementAndCollision,
    pub landed:bool,
    countdown_to_die_seconds:f32,
    pub remove:bool,
}

impl RescueInstances {
    pub fn new(gl: &gl::Gl,position:Vector3<f32>,model:&Model) -> RescueInstances {
        let model_instance = ModelInstance::new(gl,model.clone(), 0.018,None);
        RescueInstances {
            model_instance,
            movement_collision:MovementAndCollision::new(0.4, position),
            landed:false,
            countdown_to_die_seconds:80.0,
            remove:false,
        }
    }
    pub fn update(&mut self, _delta: f32, level:i32,ground_height:f32 ) {
        self.countdown_to_die_seconds=self.countdown_to_die_seconds - 1.0/(30.0 - (level * 3) as f32) * 0.10 ; //_delta;
        if self.countdown_to_die_seconds < 3.0 {
            self.movement_collision.position.y = self.movement_collision.position.y - 0.10 * _delta;

            if self.countdown_to_die_seconds < 0.0 {
                self.remove = true;
            }
        }
        if ! self.landed {
            if self.movement_collision.position.y - MODEL_HEIGHT <  ground_height  {
                //println!("landed y={} ground_y={}",self.movement_collision.position.y + MODEL_HEIGHT , ground_height);
                self.landed = true;
            } else {
                self.movement_collision.position.y = self.movement_collision.position.y - 0.10 * _delta;
            }
        }

    }
    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>,relative_to:Vector3<f32>,_rel_matrix: &Matrix4<f32>,our_shader:u32) {

        self.model_instance.matrix = Matrix4::from_translation(self.movement_collision.position + relative_to );
        self.model_instance.render(gl, &view, &projection,our_shader,false);
    }
}