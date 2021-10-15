use crate::gl_helper::model::Model;
use cgmath::{Matrix4, vec3, Deg, Vector3, Point3, Transform, EuclideanSpace, Zero, InnerSpace, Angle};
use crate::gl_helper::instance_model::ModelInstance;
use crate::game::{MovementAndCollision, };
use crate::{gl, point2vec, landscape};
use crate::ground::{Ground, BY};
use crate::flying_camera::Flying_Camera;
//use std::ops::{AddAssign, Add, Mul};
use crate::landscape::{SQUARE_COLUMNS, SQUARE_SIZE, };
//use crate::gl_helper::texture::create_texture;
//use std::ops::AddAssign;
use crate::special_effects::SpecialEffects;
use rand::Rng;
use crate::sound::{play, ENGINE, stop, EXPLOSION};
use std::time::Instant;

pub struct LanderMainPlayer {
    pub(crate) model_instance: ModelInstance,
    pub(crate) movement_collision: MovementAndCollision,
    matrix:Matrix4<f32>,
    rotation_y_axis:Matrix4<f32>,
    pub rotation_y:f32,
    rotation_x_axis:Matrix4<f32>,
    pub rotation_x:f32,
    thrust:Matrix4<f32>,
    ahead_thrust:Matrix4<f32>,
    pub thrusting:bool,
    applied_rotation:Matrix4<f32>,
    gravity:f32,
    dir:Vector3<f32>,
    crashed:bool,
    crashed_ticker:i32,
    pub fuel:f32,
    pub lives:i32,
    pub msg:String,
}

const MODEL_HEIGHT: f32 = 0.12;
pub const PLAYER_RADIUS: f32 = 0.12;
const THRUST_AHEAD: f32 = 3.0;
const GRAVITY_ADD:f32=0.0005;
const GRAVITY_MAX:f32=0.005;
const SCALE:f32 = 0.01;
const FUEL:f32=100.0;
const GRAVITY:bool = true;
fn start_position() -> Vector3<f32>  {
    vec3(0.0,2.00,0.0)
}

const FUEL_DOWN_BY: f32 = 0.05;

impl LanderMainPlayer {
    pub fn new(gl: &gl::Gl,) -> LanderMainPlayer {
        let start = Instant::now();

        let model = Model::new(gl, "resources/models/moon_lander.obj", "resources/models/moon_lander_texture.png");
        //let model_instance = ModelInstance::new(gl,model.clone(), SCALE,Some("resources/models/moon_lander_texture_thrust.png"));
        let model_instance = ModelInstance::new(gl,model.clone(), SCALE,None);

        let duration = start.elapsed();
        println!("Time elapsed in lander_main_player new () is: {:?}", duration);
        LanderMainPlayer {
            model_instance,
            movement_collision:MovementAndCollision::new(MODEL_HEIGHT * 1.25, start_position()),
            matrix:Matrix4::from_translation(start_position()),
            rotation_y_axis:Matrix4::from_angle_y(Deg(0.0)),
            rotation_y:0.0,
            rotation_x_axis:Matrix4::from_angle_x(Deg(0.0)),
            rotation_x:0.0,
            thrust:Matrix4::from_translation(vec3(0.0,0.0,0.0)),
            ahead_thrust:Matrix4::from_translation(vec3(0.0,0.0,0.0)),
            thrusting:false,
            applied_rotation:Matrix4::from_translation(vec3(0.0,0.0,0.0)),
            dir:vec3(0.0,0.00,0.0),
            gravity:GRAVITY_ADD,
            crashed:false,
            crashed_ticker:0,
            fuel:FUEL,
            lives:3,
            msg:"".to_string(),
        }
    }
    pub fn rotation_y_constant(&mut self, change_by:f32) {
        self.rotation_y_axis = Matrix4::from_angle_y(Deg(change_by));
        //self.rotation_y = self.rotation_y + change_by;
    }
    pub fn change_pitch(&mut self, change_by:f32, _ground:&Ground) {
        self.rotation_x_axis = Matrix4::from_angle_x(Deg(- change_by));
        //self.rotation_x = self.rotation_x - change_by;
    }
    pub fn forward(&mut self,mut forward_by:f32,_ground:&Ground) {
        if forward_by.is_zero() {
            self.thrusting = false;
        } else {
            self.thrusting = true;
        }
        if self.fuel<= 0.0 {
            forward_by  = 0.0;
            self.thrusting = false;
        }
        if self.thrusting {
            play(ENGINE);
        } else {
            stop(ENGINE);
        }
        self.thrust = Matrix4::from_translation(vec3(0.0,forward_by,0.0)) ;
        self.ahead_thrust = Matrix4::from_translation(vec3(0.0,forward_by.abs()* THRUST_AHEAD,0.0)) ;

        let mut dir = vec3(0.0,forward_by,0.0);
        dir = self.applied_rotation.transform_vector(dir) * 0.005;
        self.dir = self.dir + dir;

        self.gravity =  self.gravity - dir.y;
        //println!("dir is {},{},{} gavity={}",self.dir.x,self.dir.y,self.dir.z,self.gravity);

    }

    pub fn reset(&mut self){
        stop(ENGINE);
        self.crashed = false;
        self.crashed_ticker = 0;
        self.model_instance.scale = SCALE;
        //self.movement_collision.position = start_position();
        self.movement_collision.position.y = start_position().y;
        self.applied_rotation = Matrix4::from_translation(vec3(0.0,0.0,0.0));
        self.thrust = Matrix4::from_translation(vec3(0.0,0.0,0.0));
        self.gravity = GRAVITY_ADD;
        self.dir = Vector3::<f32>::zero();
        self.thrusting = false;
        self.fuel = FUEL;
    }

    pub fn update(&mut self, delta: f32, ground:&Ground, _camera: &Flying_Camera, tick:i32, special_effects: &mut SpecialEffects) -> &String {
        if self.crashed {
           if tick % 2 == 0 {
               self.model_instance.scale = self.model_instance.scale * 0.9;
           }
           self.crashed_ticker = self.crashed_ticker -1;
            if self.crashed_ticker < 0 {
                self.reset();
            }
        } else {
            self.update_all_ok(delta,ground,_camera,tick,special_effects);
        }
        return &self.msg;
    }

    pub fn update_all_ok(&mut self, delta: f32, ground:&Ground, _camera: &Flying_Camera, tick:i32, special_effects: &mut SpecialEffects) {

        //let mut msg:String = "".to_string();
        self.applied_rotation = self.applied_rotation * self.rotation_x_axis * self.rotation_y_axis;

        let rotated =  self.applied_rotation.transform_vector(vec3(0.0,0.0,1.0));
        let ang1 = rotated.angle(vec3(0.0,1.0,0.0)).sin_cos();
        let ang2 = rotated.angle(vec3(0.0,0.0,1.0)).sin_cos();

        self.rotation_x = (Deg::asin(ang1.1).0 ).round();
        self.rotation_y = (Deg::acos(ang2.1).0).round();

        self.matrix = Matrix4::from_translation(self.movement_collision.position) * self.applied_rotation;
        let original_matrix = self.matrix;


            self.matrix.w.x = self.matrix.w.x + self.dir.x;
            self.matrix.w.y = self.matrix.w.y + self.dir.y ;
            self.matrix.w.z = self.matrix.w.z + self.dir.z;

        if GRAVITY {
            self.matrix.w.y = self.matrix.w.y - self.gravity;
        }
        if tick % 60 <= 1 {
            self.dir = self.dir * 0.9;
            if self.gravity <= GRAVITY_MAX {
                self.gravity = self.gravity + GRAVITY_ADD;
            }
        }
        self.update_position();

        let half_width = (SQUARE_COLUMNS /2 ) as f32 * SQUARE_SIZE *BY as f32 ;

        //println!("{} {} {} {}",self.thrust.w.x,self.thrust.w.y,self.thrust.w.z,self.thrust.w.w);

        if self.movement_collision.position.x < -half_width {
            println!("b4 x< 0 Reset x={},z={} {}",self.movement_collision.position.x,self.movement_collision.position.z,self.thrust.w.y);
            self.flip_reset_the_matrix(1.0,0.0);
            println!("x< 0 Reset x={},z={}",self.movement_collision.position.x,self.movement_collision.position.z);
        } else if self.movement_collision.position.x > half_width {
            println!("b4 x> Reset x={},z={}  {}",self.movement_collision.position.x,self.movement_collision.position.z,self.thrust.w.y);
            self.flip_reset_the_matrix(-1.0,0.0);
            println!("x> Reset x={},z={}",self.movement_collision.position.x,self.movement_collision.position.z);
        }
        if self.movement_collision.position.z <= -half_width  {
            println!("b4 z<0 Reset x={},z={}  {}",self.movement_collision.position.x,self.movement_collision.position.z,self.thrust.w.y);
            self.flip_reset_the_matrix(0.0,1.0);
            println!("z<0 Reset x={},z={}",self.movement_collision.position.x,self.movement_collision.position.z);
        } else if self.movement_collision.position.z >= half_width  {
            println!("b4 z> Reset x={},z={}  {}",self.movement_collision.position.x,self.movement_collision.position.z,self.thrust.w.y);
            self.flip_reset_the_matrix(0.0,-1.0);
        }
        let ahead_matrix = self.matrix * self.rotation_y_axis * self.rotation_x_axis * self.ahead_thrust;
        let ahead = LanderMainPlayer::position_ahead(ahead_matrix);

        let ground_height = ground.position_height(self.movement_collision.position.x,self.movement_collision.position.z);
        let ground_height_ahead = ground.position_height(ahead.x,ahead.z);
        if ahead.y < ground_height_ahead + MODEL_HEIGHT  {
            self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis ;
            self.update_position();
            println!("AHEAD ROLLBACK ground={} ahead.y={}",ground_height_ahead,ahead.y);
            self.crashed(special_effects);
        } else if self.movement_collision.position.y < ground_height + MODEL_HEIGHT   {
            self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis ;
            self.update_position();
            println!("ROLLBACK height {} ",ground_height);
            self.crashed(special_effects);
        }


        let under = ground.currently_under_landscape(self.movement_collision.position.x,self.movement_collision.position.z);

        if under.handle_collision(&self.movement_collision) {
            self.matrix = original_matrix * self.rotation_y_axis * self.rotation_x_axis ;
            self.update_position();
            //println!("LANDED ROLLBACK height {}  hit launchpad I guess",ground_height);
            //msg = format!("LANDED ROLLBACK height {}  hit launchpad I guess",ground_height);
            self.fuel = FUEL;

        }

        if self.thrusting {
            self.fuel = self.fuel - FUEL_DOWN_BY;
        }
        if self.thrusting  && tick % 3 == 0 {
            let mut rng = rand::thread_rng();
            let mut dir = vec3(0.0,MODEL_HEIGHT,0.0) ;
            dir = self.applied_rotation.transform_vector(dir) ;
            for _i in 1..rng.gen_range(2,5) {
                let pos =self.movement_collision.position - dir * rng.gen_range(0.95,1.75) ;
                //println!("thrust dir is {},{},{} pos is {},{},{}",dir.x,dir.y,dir.z,pos.x,pos.y,pos.z);
                special_effects.thrust(pos,self.dir  ,delta);
            }
        }

        if self.movement_collision.position.y > landscape::MAX_HEIGHT *0.9 {
            self.movement_collision.position.y = landscape::MAX_HEIGHT *0.9;
            self.gravity = GRAVITY_ADD;
            self.dir.y = 0.0;
            self.msg = "upper atmosphere reached don't thrust up!!".to_string();
        } else {
            self.msg = "".to_string();
        }

    }

    fn crashed(&mut self, special_effects: &mut SpecialEffects) {
        self.thrusting = false;
        stop(ENGINE);
        self.lives = self.lives - 1;
        self.crashed = true;
        self.crashed_ticker = 160;
        special_effects.explosion(self.movement_collision.position);
        play(EXPLOSION);
    }
    fn flip_reset_the_matrix(&mut self,x:f32,z:f32) {

        let width = (SQUARE_COLUMNS  ) as f32 * SQUARE_SIZE *BY  as f32 ;

        if ! x.is_zero() {
            self.matrix.w.x = self.matrix.w.x + ( x * width);
        }
        if ! z.is_zero() {
            self.matrix.w.z = self.matrix.w.z + (z  * width);
        }
        self.update_position();
    }

    fn update_position(&mut self) {
        let mut point = Point3::from_vec(vec3(0.0, 0.0, 0.0));
        point = self.matrix.transform_point(point);
        self.movement_collision.position.x = point.x;
        self.movement_collision.position.y = point.y;
        self.movement_collision.position.z = point.z;
    }
    fn position_ahead(matrix:Matrix4<f32>) -> Vector3<f32>{
        let mut point = Point3::from_vec(vec3(0.0, 0.0, 0.0));
        point = matrix.transform_point(point);
        point2vec(point)
    }
    pub(crate) fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>,our_shader:u32) {
        self.model_instance.matrix = self.matrix;
        if ! self.crashed {
            //if self.thrusting {
                //self.model_instance.render(gl, &view, &projection,our_shader,true);
            //} else {
                self.model_instance.render(gl, &view, &projection,our_shader,false);
            //}
        } else {
            self.model_instance.render(gl, &view, &projection,our_shader,false);

        }
    }
}