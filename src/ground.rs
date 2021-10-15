use cgmath::{Matrix4, vec3, Vector3, Vector2, vec2, MetricSpace};
use crate::{gl};

//use crate::game::Render;
use crate::landscape::{AtCell, Landscape, SQUARE_COLUMNS, SQUARE_ROWS, SQUARE_SIZE, };
use rand::Rng;
use std::collections::HashMap;
//use std::ops::Deref;
use crate::rescue::RescueInstances;
//use std::borrow::BorrowMut;
use crate::sound::{SCOOP, play};
//use crate::handle_javascript::write_stats_data;
use std::time::Instant;
use crate::gl_helper::model::Model;

pub struct Ground {
    pub land: Vec<Vec<Landscape>>,
    pub player_pos: Vector3<f32>,
    to_display: HashMap<Vector2<i32>, Vector2<i32>>,
    rescue_model:Model,
}

pub(crate) const BY: usize = 3;

impl Ground {
    //const  MUL:f32 = 3.40;
    //pub(crate) const  MUL:f32 = 3.00;
    pub(crate) const MUL: f32 = SQUARE_SIZE * SQUARE_COLUMNS as f32;

    pub fn new(gl: &gl::Gl) -> Ground {
        let mut land: Vec<Vec<Landscape>> = vec![vec![]];
        let mut height_map: Vec<Vec<AtCell>> = vec![vec![AtCell { height: 0.0 }; SQUARE_COLUMNS * BY]; SQUARE_COLUMNS * BY];
        Landscape::simple_height(&mut height_map, SQUARE_COLUMNS * BY, SQUARE_ROWS * BY);
        Landscape::work_up(&mut height_map, SQUARE_COLUMNS * BY, SQUARE_ROWS * BY);

        let offset_x = BY as f32 * Ground::MUL * 0.5 - SQUARE_COLUMNS as f32 * SQUARE_SIZE * 0.5;
        let offset_z = BY as f32 * Ground::MUL * 0.5 - SQUARE_ROWS as f32 * SQUARE_SIZE * 0.5;

        let start = Instant::now();
        let launchpad_model = Model::new(gl, "resources/models/launchpad.obj", "resources/models/launchpad.png");
        for y in 0..BY {
            land.push(vec![]);

            for x in 0..BY {
                let mut cell_height_map: Vec<Vec<AtCell>> = vec![vec![AtCell { height: 7.9 }; SQUARE_COLUMNS]; SQUARE_COLUMNS];
                for cell_y in 0..SQUARE_ROWS {
                    for cell_x in 0..SQUARE_COLUMNS {
                        let source_x = cell_x + x * SQUARE_COLUMNS;
                        let source_y = cell_y + y * SQUARE_ROWS;
                        let copy = height_map[source_y][source_x].clone();
                        cell_height_map[cell_y][cell_x] = copy;
                    }
                }
                //Ground::random_debug(&mut cell_height_map);
                let here = vec3(x as f32 * Ground::MUL - offset_x, 0.0, y as f32 * Ground::MUL - offset_z);
                let land_cell = Landscape::new(&gl, "resources/ground.png", here, 1.0, &mut cell_height_map,&launchpad_model);
                //println!("MUL={}  X={} Y={}   xyz={},{} ", Ground::MUL, x, y, land_cell.xyz.x, land_cell.xyz.z);

                land[y].push(land_cell);
                assert_eq!(land[y][x].xyz, here);
            }
        }
        //let model = Model::new(gl, "resources/models/rescue.obj", "resources/models/rescue.png");

        let duration = start.elapsed();
        println!("Time elapsed in expensive_ground() is: {:?}", duration);

        Ground {
            land,
            player_pos: vec3(0.0, 0.0, 0.0),
            to_display: HashMap::new(),
            rescue_model :Model::new(gl, "resources/models/rescue.obj", "resources/models/rescue.png"),
        }
    }

    fn _random_debug(cell_height_map: &mut Vec<Vec<AtCell>>) {
        let centre = SQUARE_COLUMNS as i32 / 2 - 1;
        let mut rng = rand::thread_rng();
        let add_x = rng.gen_range(-centre, centre) as i32;
        let add_y = rng.gen_range(-centre, centre) as i32;
        //let add_x=0;
        //let add_y=0;
        cell_height_map[(centre + add_y) as usize][(centre + add_x) as usize] = AtCell { height: 0.5 };
        cell_height_map[(centre + add_y) as usize][(centre + add_x + 1) as usize] = AtCell { height: 0.5 };
        cell_height_map[(centre + add_y) as usize][(centre + add_x + 2) as usize] = AtCell { height: 0.5 };
    }

    pub fn set_player_position(&mut self, x: f32, z: f32) {
        self.player_pos.x = x;
        self.player_pos.z = z;
    }
    pub fn currently_under_landscape(&self, x: f32, z: f32) -> &Landscape {
        let (xx, zz) = Ground::get_current_cell(x, z);
        &self.land[zz][xx]
    }

    pub fn position_height(&self, x: f32, z: f32) -> f32 {
        let (xx, zz) = Ground::get_current_cell(x, z);
        //print!("xx is {} zz is {} MUL={}   ",xx,zz,Ground::MUL);
        let height = self.land[zz][xx].position_height(x, z);

        height
    }

    pub(crate) fn get_current_cell(x: f32, z: f32) -> (usize, usize) {
        let mut xx = ((x + Ground::MUL * BY as f32 * 0.5) / Ground::MUL) as usize;
        let mut zz = ((z + Ground::MUL * BY as f32 * 0.5) / Ground::MUL) as usize;
        if xx >= BY { xx = BY - 1 }
        if zz >= BY { zz = BY - 1 }
        (xx, zz)
    }
    pub fn update(&mut self, gl: &gl::Gl, player_position: Vector3<f32>, level:i32,camera_angle: f32, delta: f32) {
        self.sort_out_what_to_display(player_position, camera_angle);

        self.process_rescue_logic(gl,level,delta)
    }

    pub fn total_to_rescue(&self) -> usize {
       let mut total = 0;
        for xx in 0..BY {
            for yy in 0..BY{
                total += self.land[yy][xx].rescue.len();
            }
        }
        return total;
    }

    fn process_rescue_logic(&mut self, gl: &gl::Gl,level:i32, delta: f32) {
        let mut landing = 0;
        for xz in self.to_display.values() {
            let yyy = wrap_value(xz.y);
            let xxx = wrap_value(xz.x);
            for i in 0..self.land[yyy as usize][xxx as usize].rescue.len() {
                let r = &self.land[yyy as usize][xxx as usize].rescue[i];
                let ground_height = if r.landed {
                    0.0
                } else {
                    landing = landing + 1;
                    let position = self.land[yyy as usize][xxx as usize].xyz * 0.5 + r.movement_collision.position;
                    self.position_height(position.x, position.z)
                };
                self.land[yyy as usize][xxx as usize].rescue[i].update(delta, level,ground_height);
            }
        }
        if landing == 0  && self.total_to_rescue() < 10 {
            let mut rng = rand::thread_rng();

            let xx = rng.gen_range(0, BY );
            let yy = rng.gen_range(0, BY );

            let range = (SQUARE_COLUMNS as i32) / 2 * level;
            let mut position = vec3(rng.gen_range(-range, range) as f32 * SQUARE_SIZE, 4.0, rng.gen_range(-range, range) as f32 * SQUARE_SIZE);

            position.x = position.x % (SQUARE_COLUMNS as i32 / 2) as f32;
            position.z = position.z % (SQUARE_COLUMNS as i32 / 2) as f32;

            let r = RescueInstances::new(gl, position,&self.rescue_model);
            self.land[yy ][xx ].rescue.push(r);
        }
    }

    fn sort_out_what_to_display(&mut self, player_position: Vector3<f32>, camera_angle: f32) {
        let (current_xx, current_zz) = Ground::get_current_cell(player_position.x, player_position.z);

        let xx = current_xx as f32;
        let zz = current_zz as f32;

        //let mut to_display: HashMap<Vector2<i32>, Vector2<i32>> = HashMap::new();
        self.to_display.clear();

        let v = vec2(xx as i32, zz as i32);
        self.to_display.insert(v, v);
        let debug = false;

        if debug {
            let v = vec2(xx as i32 +1, zz as i32); self.to_display.insert(v, v);
            let v = vec2(xx as i32 +1, zz as i32 +1); self.to_display.insert(v, v);
            let v = vec2(xx as i32 , zz as i32 +1); self.to_display.insert(v, v);
            let v = vec2(xx as i32 -1, zz as i32); self.to_display.insert(v, v);
            let v = vec2(xx as i32 , zz as i32 -1); self.to_display.insert(v, v);
            let v = vec2(xx as i32 -1, zz as i32 -1); self.to_display.insert(v, v);

        } else {

            // -1 so make sure when low we still show cell
            for going_away in -1..5 {
                for a in (-120..130).step_by(5) {
                    let apply = Vector3 {
                        x: (a as f32 - camera_angle).to_radians().sin() * going_away as f32,
                        y: 0.0,
                        z: (a as f32 - camera_angle).to_radians().cos() * -going_away as f32,
                    };
                    let v = vec2((xx + apply.x) as i32, (zz + apply.z) as i32);
                    self.to_display.insert(v, v);
                }
            }
        }
    }

    pub fn handle_player_rescue(&mut self, player_position: Vector3<f32>) -> (i32,i32) {
        //let (xx, zz) = Ground::get_current_cell(player_position.x, player_position.z);
        // ideally should be able to limit this to current landscape player over. Something wrong
        // with me or it....
        //println!("{},{} items {}",xx,zz,self.land[zz][xx].rescue.len());
        let mut rescued = 0;
        let mut score = 0;

        for zz in (0..BY).rev() {
            for xx in (0..BY).rev() {
                for i in (0..self.land[zz][xx].rescue.len()).rev() {
                    let r = &self.land[zz][xx].rescue[i];
                    let rescue_position = &self.land[zz][xx].xyz * 0.5 + r.movement_collision.position;

                    let dist = rescue_position.distance(player_position);
                    //if dist < 3.0 { println!("DISTANCE {}", dist); }
                    if dist < crate::lander_main_player::PLAYER_RADIUS + crate::rescue::RESCUE_RADIUS {
                        //println!("WELL DONE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                        rescued+=1;
                        score+=1;
                        if r.landed == false {
                            score+=10;
                        }
                        play(SCOOP);

                        self.land[zz][xx].rescue.remove(i);
                    } else if r.remove {
                        self.land[zz][xx].rescue.remove(i);
                    }
                }
            }
        }
        return (rescued,score);
    }


    pub fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, _player_position: Vector3<f32>, _camera_angle: f32, our_shader: u32) {
        for xz in self.to_display.values() {
            let yyy = wrap_value(xz.y);
            let xxx = wrap_value(xz.x);
            let offset = BY as f32 * Ground::MUL / 2.0 - Ground::MUL / 2.0;
            let position = vec3(xz.x as f32 * (Ground::MUL) - offset, 0.0, xz.y as f32 * (Ground::MUL) - offset);
            let here = Matrix4::<f32>::from_translation(position);
            //print!("yyy={},xxx={}  here {},{}      {},{} ",yyy,xxx,position.x,position.z,self.land[yyy][xxx].xyz.x,self.land[yyy][xxx].xyz.z);
            self.land[yyy as usize][xxx as usize].render(gl, view, projection, here, position, our_shader);

            let p = vec3(position.x, 0.0, position.z) - self.land[yyy as usize][xxx as usize].xyz * 0.5;

            for r in self.land[yyy as usize][xxx as usize].rescue.iter_mut() {
                r.render(&gl, view, projection, p, &here, our_shader);
            }
        }
    }
}

fn wrap_value(v: i32) -> i32 {
    let mut r = v;
    if r >= BY as i32 {
        while r >= BY as i32 {
            r = r - BY as i32;
        }
    }
    if r < 0 {
        while r < 0 {
            r = r + BY as i32;
        }
    }
    if r < 0 { r = -99 };
    if r >= BY as i32 { r = -99 };
    return r;
}