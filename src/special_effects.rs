use cgmath::{Matrix4, vec3, Vector3, Zero};

use crate::{get_next_id, gl};
use crate::cube::Cube;
use crate::game::{GROUND, MovementAndCollision, Render, Update};
use crate::ground::Ground;
use rand::Rng;
use crate::gl_helper::texture::create_texture_png;
use std::time::Instant;

pub struct SpecialEffects {
    cube: Cube,
    pub instances: Vec<SpecialInstance>,
    yellow:u32,
}

pub struct SpecialInstance {
    pub id: u128,
    pub collision: MovementAndCollision,
    direction: Vector3<f32>,
    scale:f32,
    ticks: i32,
    speed:f32,
    tex_index:usize,
    textures:Vec<u32>,
}

impl SpecialEffects {
    pub fn new(gl: &gl::Gl) -> SpecialEffects {
        let start = Instant::now();
        let cube = Cube::new(&gl, "resources/fire.png", vec3(0.001, 0.001, 0.001), 1.0);
        let yellow = create_texture_png(&gl, "resources/yellow.png");

        let duration = start.elapsed();
        println!("Time elapsed in special effects new () is: {:?}", duration);
        SpecialEffects {
            cube,
            instances: Vec::new(),
            yellow,
        }
    }
    pub fn explosion(&mut self, position: Vector3<f32>) {
        let mut rng = rand::thread_rng();

        for _i in 0..30 {
            let direction: Vector3<f32> = vec3(
                rng.gen_range(-0.1, 0.1),
                rng.gen_range(0.01, 0.15),
                rng.gen_range(-0.1, 0.1));

            let pos = position - direction;
            let instance = SpecialInstance {
                id: get_next_id(),
                direction,
                collision: MovementAndCollision::new(0.0, pos),
                ticks: rng.gen_range(50,150),
                scale:rng.gen_range(48.0,96.0),
                speed: rng.gen_range(0.15, 0.4),
                tex_index:0,
                textures:vec![self.cube.texture,self.yellow],
            };
            self.instances.push(instance);
        }
    }
    pub fn thrust(&mut self, position: Vector3<f32>, direction: Vector3<f32>, _delta: f32) {

        let mut rng = rand::thread_rng();
        let instance = SpecialInstance {
            id: get_next_id(),
            direction,
            collision: MovementAndCollision::new(0.0, position),
            ticks: 15,
            scale:rng.gen_range(8.0,12.0),
            speed:0.0,
            tex_index:0,
            textures:vec![self.cube.texture,self.yellow],
        };
        self.instances.push(instance);
    }

    pub fn _fire(&mut self, mut position: Vector3<f32>, direction: Vector3<f32>, delta: f32, radius: f32,speed:f32) {
        position += direction * delta * speed;

        let instance = SpecialInstance {
            id: get_next_id(),
            direction,
            collision: MovementAndCollision::new(radius, position),
            ticks: 300,
            scale:2.0,
            speed:0.08,
            tex_index:0,
            textures:vec![self.cube.texture,self.cube.texture],
        };
        self.instances.push(instance);
    }
}

impl Update for SpecialEffects {
    fn update(&mut self, delta: f32,_ground:&Ground) {
        for i in (0..self.instances.len()).rev() {
            let change = self.instances.get_mut(i).unwrap();
            change.tex_index = change.tex_index +1;

            if change.speed.is_zero() {
                let mut rng = rand::thread_rng();
               change.scale =  change.scale * rng.gen_range(0.9,1.2);

            } else {
                change.collision.position += change.direction * delta * change.speed;
            }

            if change.collision.position.y <= GROUND {
                change.collision.been_hit = true;
            }
            change.ticks = change.ticks - 1;
            if change.ticks <= 0 || change.collision.been_hit  {
                self.instances.remove(i);
            }
            //change.matrix = Matrix4::<f32>::from_translation(change.collision.position);
        }
    }
}

impl Render for SpecialEffects {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>,our_shader:u32) {
        for i in &self.instances {
            let scale =  Matrix4::<f32>::from_scale(i.scale);
            let matrix = Matrix4::<f32>::from_translation(i.collision.position) * scale;

            self.cube.render(gl, &matrix, view, projection,our_shader,i.textures[i.tex_index % i.textures.len()]);
        }
    }
}
