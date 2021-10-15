extern crate cgmath;

use std::mem;
use std::os::raw::c_void;
use std::ptr;

use cgmath::*;

use crate::gl_helper::texture::{create_texture_jpg, create_texture_png};
use crate::gl_helper::gl_matrix4;
use crate::{gl, };
use rand::Rng;
use std::collections::HashMap;
use crate::launchpad::LaunchPad;
use crate::game::MovementAndCollision;
use rand::prelude::ThreadRng;
use crate::rescue::RescueInstances;
use crate::gl_helper::model::Model;

static mut TEXTURE_LOADED:i32 = -1;

pub struct LandscapeInstance {
    pub id: u128,
    pub matrix: Matrix4<f32>,
}

impl PartialEq for LandscapeInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub const SQUARE_SIZE: f32 = 0.2;
pub const SQUARE_ROWS: usize = 32;
pub const SQUARE_COLUMNS: usize = 32;
pub const MAX: f32 = 0.12;
const FUDGE_PAD_ABOVE_GROUND:f32 = 0.05;

#[derive(Clone)]
pub struct AtCell {
    pub(crate) height: f32,
}

pub struct Landscape {
    //id:u128,
    texture: u32,
    vao: u32,
    height_map: Vec<Vec<AtCell>>,
    pub xyz: Vector3<f32>,
    vertices_count: usize,
    landing_pad:Vec<LaunchPad>,
    pub rescue:Vec<RescueInstances>,
}
struct SideVertice {
    height:i32,
    vertices:Vec<f32>,
}
const ROUND_Y:f32  = 10000.0;
pub const MAX_HEIGHT: f32 = 2.5;

impl Landscape {
    pub fn new(gl: &gl::Gl, image_file: &str, xyz: Vector3<f32>, _texture_end: f32, height_map: &mut Vec<Vec<AtCell>>,launchpad_model:&Model) -> Landscape {
        let rng = rand::thread_rng();

        //let id=get_next_id();
        let (landing_pad,added_pads) = Landscape::add_landing_pads(&gl, xyz, &height_map, rng,launchpad_model);

        for pad in added_pads.iter() {
            let y =  height_map[pad.z][pad.x].height;
            for zz in pad.z-2..pad.z+2 {
                for xx in pad.x-2..pad.x+2 {
                    height_map[zz][xx].height = y;
                }
            }
        }

        let (_vbo, vao, texture, vertices_count) = unsafe {
            let mut vertices: Vec<f32> = vec![];
            let mut sidex_vertices:HashMap<String,SideVertice> = HashMap::new();
            let mut sidez_vertices:HashMap<String,SideVertice> = HashMap::new();
            let mut tex_x = 0.0;
            let mut tex_y = 0.0;
            let tex_size = 0.125;
            let tey_size = 1.0;

            Landscape::ground_base_if_all_else_fails(&mut vertices);


            for row in 0..SQUARE_ROWS {
                let zero_height = AtCell { height: 0.0 };
                tex_x = tex_x + tex_size;
                for column in 0..SQUARE_COLUMNS {
                    tex_x = tex_x + tex_size;
                    if tex_x >= 0.5 {
                        tex_x=0.0;
                    }
                    let me = &height_map[row][column];
                    tex_y = tex_y + me.height;
                    let x = column as f32 * SQUARE_SIZE - (SQUARE_SIZE * SQUARE_COLUMNS as f32 / 2.0);
                    let z = row as f32 * SQUARE_SIZE - (SQUARE_SIZE * SQUARE_ROWS as f32 / 2.0);
                    let left = if column > 0 && column < SQUARE_COLUMNS - 1 { &height_map[row][column - 1] } else { &zero_height };
                    let right = if column > 0 && column < SQUARE_COLUMNS - 1 { &height_map[row][column + 1] } else { &zero_height };
                    let below = if row > 0 && row < SQUARE_ROWS - 1 { &height_map[row + 1][column] } else { &zero_height };
                    let up = if row > 0 && row < SQUARE_ROWS - 1 { &height_map[row - 1][column] } else { &zero_height };

                    Landscape::one(&mut vertices, me, x, z);
                    vertices.push(tex_x); vertices.push(tex_y);
                    Landscape::two(&mut vertices, me, x, z);
                    vertices.push(tex_x); vertices.push(tex_y + tey_size);
                    Landscape::three(&mut vertices, me, x, z);
                    vertices.push(tex_x + tex_size); vertices.push(tex_y);

                    Landscape::three(&mut vertices, me, x, z); //four
                    vertices.push(tex_x + tex_size); vertices.push(tex_y);
                    Landscape::two(&mut vertices, me, x, z); //five
                    vertices.push(tex_x); vertices.push(tex_y + tey_size);
                    Landscape::four(&mut vertices, me, x, z);
                    vertices.push(tex_x + tex_size); vertices.push(tex_y + tey_size);

                    if right.height < me.height {
                        // right
                        let mut side_vertices: Vec<f32> = vec![];
                        let tex_x = 0.5;
                        let repeat_tey_size = (me.height - right.height).abs() ;//tey_size ;/// (me.height + zero_height.height);

                        Landscape::four(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::three(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::three(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x); side_vertices.push(tex_y);

                        Landscape::three(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x); side_vertices.push(tex_y);
                        Landscape::four(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y);
                        Landscape::four(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y + repeat_tey_size);

                        Landscape::add_if_not_duplicate(&mut sidex_vertices,  &mut side_vertices, x + SQUARE_SIZE , me.height, z)
                    }

                    if left.height > me.height {
                        //left
                        let tex_x = 0.5 + 0.125;
                        let mut side_vertices: Vec<f32> = vec![];
                        let repeat_tey_size = (me.height - left.height).abs() * 5.0;
                        Landscape::one(&mut side_vertices, me, x, z,);
                        side_vertices.push(tex_x); side_vertices.push(tex_y);
                        Landscape::two(&mut side_vertices, me, x, z,);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y);
                        //Landscape::two(&mut side_vertices, left, x, z,zfight,0.0);
                        Landscape::two(&mut side_vertices, &zero_height, x, z,);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::two(&mut side_vertices, &zero_height, x, z,);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::one(&mut side_vertices, &zero_height, x, z,);
                        side_vertices.push(tex_x); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::one(&mut side_vertices, me, x, z,);
                        side_vertices.push(tex_x); side_vertices.push(tex_y);

                        Landscape::add_if_not_duplicate(&mut sidex_vertices,  &mut side_vertices, x , me.height, z)
                    }
                    if below.height != me.height { // flicker
                        // below
                        let tex_x = 0.5 + 0.125 * 2.0;
                        let mut side_vertices: Vec<f32> = vec![];
                        let repeat_tey_size = (me.height - below.height).abs() * 5.0 ;
                        Landscape::two(&mut side_vertices, me, x, z,); // five
                        side_vertices.push(tex_x); side_vertices.push(tex_y);
                        Landscape::four(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y);
                        Landscape::two(&mut side_vertices, &zero_height, x, z,); //five
                        side_vertices.push(tex_x); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::two(&mut side_vertices, &zero_height, x, z,);//five
                        side_vertices.push(tex_x); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::four(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::four(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y);

                        Landscape::add_if_not_duplicate(&mut sidez_vertices, &mut side_vertices, x, me.height, z+SQUARE_SIZE)
                    }
                    if  up.height != me.height {
                        // above
                        let tex_x = 0.5 + 0.125 * 3.0;
                        let mut side_vertices: Vec<f32> = vec![];
                        let repeat_tey_size = (me.height - up.height).abs(); //tey_size * (me.height *5.0);
                        Landscape::one(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x); side_vertices.push(tex_y);
                        Landscape::three(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y);
                        Landscape::one(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::one(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::three(&mut side_vertices, &zero_height, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y + repeat_tey_size);
                        Landscape::three(&mut side_vertices, me, x, z);
                        side_vertices.push(tex_x + tex_size); side_vertices.push(tex_y);

                        Landscape::add_if_not_duplicate(&mut sidez_vertices, &mut side_vertices, x, me.height, z)
                    }
                }
            }
            for side in sidez_vertices.values_mut(){
                vertices.append(&mut side.vertices);
            }
            for side in sidex_vertices.values_mut(){
                vertices.append(&mut side.vertices);
            }


            let (mut vbo, mut vao) = (0, 0);
            gl.GenVertexArrays(1, &mut vao);
            gl.GenBuffers(1, &mut vbo);

            gl.BindVertexArray(vao);

            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(gl::ARRAY_BUFFER,
                          (vertices.len() * mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                          &vertices[0] as *const f32 as *const c_void,
                          gl::STATIC_DRAW);

            let stride = 5 * mem::size_of::<gl::types::GLfloat>() as gl::types::GLsizei;
            gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<gl::types::GLfloat>()) as *const c_void);
            gl.EnableVertexAttribArray(1);

            let texture = if TEXTURE_LOADED == -1 {
                if image_file.ends_with(".png") {
                    TEXTURE_LOADED = create_texture_png(&gl, image_file) as i32;
                } else {
                    TEXTURE_LOADED = create_texture_jpg(&gl, image_file) as i32;
                };
                TEXTURE_LOADED as u32
            } else {
                TEXTURE_LOADED as u32
            };


            (vbo, vao, texture, (vertices.len() as f32 * 0.2) as usize)
        };


        Landscape {
            //id:id,
            texture,
            vao,
            height_map: height_map.clone(),
            xyz,
            vertices_count,
            landing_pad,
            rescue:vec![],
        }
    }



    fn ground_base_if_all_else_fails(vertices: &mut Vec<f32>) {
        let y=-0.05;
        vertices.push(0.0);
        vertices.push(y);
        vertices.push(0.0);
        vertices.push(0.0); vertices.push(0.0);
        vertices.push(SQUARE_SIZE * SQUARE_COLUMNS as f32);
        vertices.push(y);
        vertices.push(0.0);
        vertices.push(1.0); vertices.push(0.0);
        vertices.push(SQUARE_SIZE * SQUARE_COLUMNS as f32);
        vertices.push(y);
        vertices.push(SQUARE_SIZE * SQUARE_ROWS as f32);
        vertices.push(1.0); vertices.push(1.0);

        vertices.push(SQUARE_SIZE * SQUARE_COLUMNS as f32);
        vertices.push(y);
        vertices.push(SQUARE_SIZE * SQUARE_ROWS as f32);
        vertices.push(1.0); vertices.push(1.0);
        vertices.push(0.0);
        vertices.push(y);
        vertices.push(SQUARE_SIZE * SQUARE_ROWS as f32);
        vertices.push(0.0); vertices.push(1.0);
        vertices.push(0.0);
        vertices.push(y);
        vertices.push(0.0);
        vertices.push(0.0); vertices.push(0.0);
    }

    fn add_if_not_duplicate(thesidevertices: &mut HashMap<String,SideVertice> , side_vertices: &mut Vec<f32>, x: f32, y: f32, z: f32) {

        let round_y = (y * ROUND_Y) as i32;
        let round_x = (x * ROUND_Y) as i32;
        let round_z = (z * ROUND_Y) as i32;
        let key = format!("{},{}",round_x,round_z);
        if thesidevertices.contains_key(&*key) {

            let side = thesidevertices.get(&*key).unwrap();
            if side.height >= round_y {
                return;
            }
        }
        let side = SideVertice{ height: round_y, vertices: side_vertices.clone() };
        thesidevertices.insert(key,side);

    }



    fn three(vertices: &mut Vec<f32>, me: &AtCell, x: f32, z: f32) {
        vertices.push(x + SQUARE_SIZE  );
        vertices.push(me.height);
        vertices.push(z); //3
    }

    fn two(vertices: &mut Vec<f32>, me: &AtCell, x: f32, z: f32, ) {
        vertices.push(x );
        vertices.push(me.height);
        vertices.push(z + SQUARE_SIZE );//2
    }

    fn one(vertices: &mut Vec<f32>, me: &AtCell, x: f32, z: f32) {
        vertices.push(x);
        vertices.push(me.height);
        vertices.push(z); //1
    }

    fn four(vertices: &mut Vec<f32>, me: &AtCell, x: f32, z: f32) {
        vertices.push(x + SQUARE_SIZE  );
        vertices.push(me.height);
        vertices.push(z + SQUARE_SIZE ); //6
    }




    pub(crate) fn work_up(height_map: &mut Vec<Vec<AtCell>>, square_columns: usize, square_rows: usize) {
        let get_sine = |radius: f32| -> f32  { (radius).to_radians().sin() };
        let get_cosine = |radius: f32| -> f32  { (radius).to_radians().cos() };
        let mut rng = rand::thread_rng();

        let max_radius = SQUARE_COLUMNS/2 -1;
        let mut skip_for_a_while: i32 = 0;
        for zz in max_radius..square_rows - max_radius {
            for xx in max_radius..square_columns - max_radius {
                let v: f64 = rng.gen_range(0.0, 100.0);
                skip_for_a_while = skip_for_a_while - 1;
                if skip_for_a_while <= 0 && v > 25.0 {
                    skip_for_a_while = max_radius as i32 * 20;
                    let mut centre_start_height = 26.0 / rng.gen_range(9.0, 27.0);
                    if centre_start_height > MAX_HEIGHT {
                        centre_start_height = MAX_HEIGHT;
                    }

                    //println!("Start height {} x,z={},{} col/row={},{} max_radius={}", centre_start_height, xx, zz, square_columns, square_rows, max_radius);
                    height_map[zz][xx] = AtCell { height: centre_start_height };
                    let go_up = centre_start_height / (max_radius as f32 - 1.0);
                    for angle in (0..360).step_by(2) {
                        let mut start_height = centre_start_height;
                        for radius in 1..max_radius { //((square_columns+square_rows)/2) {
                            let x = get_cosine(angle as f32) * radius as f32 + xx as f32;
                            let y = get_sine(angle as f32) * radius as f32 + zz as f32;
                            height_map[y as usize][x as usize] = AtCell { height: start_height };
                            //if height_map[y as usize][x as usize].height > start_height {
                            start_height = start_height - go_up;
                            //}
                        }
                        //println!("Angle {} End height {}",angle,start_height);
                    }
                }
            }
        }
    }

    pub fn simple_height(height_map: &mut Vec<Vec<AtCell>>, square_columns: usize, square_rows: usize) {
        let mut rng = rand::thread_rng();
        let mut addsub: f32 = 0.02;
        for row in 0..square_rows {
            for column in 1..square_columns {
                let previous = &height_map[row][column - 1];
                height_map[row][column] = AtCell { height: previous.height + rng.gen_range(0.0, 0.2) * addsub };
                if height_map[row][column].height > MAX {
                    addsub = addsub * -1.0;
                }
                if height_map[row][column].height <= 0.0 {
                    addsub = addsub * -1.0;
                }
            }
        }
    }

    pub fn position_height(&self, x: f32, z: f32) -> f32 {
        let col = ((x - self.xyz.x) / SQUARE_SIZE) + SQUARE_COLUMNS as f32 / 2.0;
        let row = ((z - self.xyz.z) / SQUARE_SIZE) + SQUARE_ROWS as f32 / 2.0;



        let height = if col as usize >= SQUARE_COLUMNS || col < 0.0 || row as usize >= SQUARE_ROWS || row < 0.0 {
            0.0
        } else {
            let height = self.height_map[row as usize][col as usize].height;
            height
        };
        //println!("x,z={},{}   col={} row={}    height={} me={},{}", x, z, col as usize, row as usize, height, self.xyz.x, self.xyz.z);

        return height;
    }



    fn add_landing_pads(gl: &gl::Gl, xyz: Vector3<f32>, height_map: &Vec<Vec<AtCell>>, mut rng: ThreadRng,launchpad_model:&Model) -> (Vec<LaunchPad>, Vec<Vector3<usize>>) {
        let mut adding_to = Vec::new();
        let mut landing_pad: Vec<LaunchPad> = Vec::new();

        let x = rng.gen_range(3, SQUARE_COLUMNS - 3);
        let z = rng.gen_range(3, SQUARE_ROWS - 3);

        adding_to.push(vec3(x, 0, z));

        let y = height_map[z][x].height;


        let x_pos = (x as i32 - SQUARE_COLUMNS as i32 / 2) as f32 * SQUARE_SIZE;
        let z_pos = (z as i32 - SQUARE_ROWS as i32 / 2) as f32 * SQUARE_SIZE;
        landing_pad.push(LaunchPad::new(&gl,
                                        xyz + vec3(x_pos,
                                                   y + FUDGE_PAD_ABOVE_GROUND,
                                                   z_pos),launchpad_model));
        (landing_pad, adding_to)
    }


    pub fn handle_collision(&self, other:&MovementAndCollision) -> bool {
        let mut hit = false;
        for l in self.landing_pad.iter() {
            if l.movement_collision.hit_other(other) && other.position.y - l.movement_collision.position.y < other.radius  {
                hit = true;
            }
        }
        return hit
    }


    pub fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, here: Matrix4<f32>, wrapped_position:Vector3<f32>,our_shader: u32) {

        //let here = Matrix4::<f32>::from_translation(self.xyz);
        unsafe {
            //gl.UseProgram(our_shader);
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);
            gl.BindVertexArray(self.vao);

            gl_matrix4(gl, our_shader, here, "model");
            gl_matrix4(gl, our_shader, *view, "view");
            gl_matrix4(gl, our_shader, *projection, "projection");
            gl.DrawArrays(gl::TRIANGLES, 0, self.vertices_count as i32);
        }

        let adjust = wrapped_position - self.xyz;
        for l in self.landing_pad.iter_mut() {
            l.render(gl,view,projection,our_shader,adjust);
        }
    }
}

