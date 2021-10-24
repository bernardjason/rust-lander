use std::ffi::CString;
use std::time::Instant;

use cgmath::{Basis3, Deg, Matrix4, MetricSpace, perspective, Point3, Rotation, Rotation3, vec3, Vector3, Zero};
use emscripten_main_loop::MainLoopEvent;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, Window};

use crate::{gl, HEIGHT, vec2point, WIDTH};
use crate::flying_camera::{Flying_Camera, PERSPECTIVE_ANGLE};
//use crate::gl_helper::shader::create_shader;
use crate::ground::Ground;
#[cfg(target_os = "emscripten")]
use crate::handle_javascript::end_game;
#[cfg(target_os = "emscripten")]
use crate::handle_javascript::start_game;
#[cfg(target_os = "emscripten")]
use crate::handle_javascript::start_javascript_play_sound;
use crate::handle_javascript::write_stats_data;
use crate::lander_main_player::LanderMainPlayer;

use crate::openglshadow::OpenglShadow;
use crate::gl_helper::draw_text::DrawText;
use crate::gl_helper::loading_screen::LoadingScreen;
use crate::special_effects::SpecialEffects;
use crate::end_screen::EndScreen;
use crate::sound::{load_sound, stop, ENGINE};


pub const GROUND: f32 = 0.01;
const TARGET_FPS: u128 = 40;

pub struct Runtime {
    //opengl_shadow: OpenglShadowPointAllDirections,
    opengl_shadow: OpenglShadow,
    loaded: bool,
    now: Instant,
    last_time_called: u128,
    rate_debug: String,
    sdl: Sdl,
    _video: VideoSubsystem,
    window: Window,
    _gl_context: GLContext,
    pub gl: std::rc::Rc<gl::Gl>,
    pub camera: Flying_Camera,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub forward: bool,
    pub backward: bool,
    pub space: bool,
    pub ground: Option<Ground>,
    score: i32,
    level: i32,
    change_level_pause: i32,
    rescued: i32,
    tick: i32,
    player_avitar: LanderMainPlayer,
    camera_angle: f32,
    draw_text: Option<DrawText>,
    special_effects: SpecialEffects,
    flash_message: String,
    flash_message_countdown: i32,
    game_over: bool,
    end_screen: EndScreen,
    bernard: i32,
    slow_loading_items: bool,
    loading_screen1:LoadingScreen,
    loading_screen2:LoadingScreen,
}


static mut GLOBAL_ID: u128 = 0;

fn get_next_id() -> u128 {
    unsafe {
        GLOBAL_ID = GLOBAL_ID + 1;
        GLOBAL_ID
    }
}

#[derive(Clone)]
pub struct MovementAndCollision {
    pub id: u128,
    pub radius: f32,
    pub position: Vector3<f32>,
    pub been_hit: bool,
    pub moved: bool,
}

impl Default for MovementAndCollision {
    fn default() -> Self {
        MovementAndCollision {
            id: get_next_id(),
            radius: 0.0,
            position: Vector3::zero(),
            been_hit: false,
            moved: false,
        }
    }
}


impl MovementAndCollision {
    pub fn new(radius: f32, position: Vector3<f32>) -> MovementAndCollision {
        MovementAndCollision {
            radius,
            position,
            been_hit: false,
            moved: false,
            ..MovementAndCollision::default()
        }
    }
    pub fn hit_other(&self, other: &MovementAndCollision) -> bool {
        self.position.distance(other.position) < self.radius
    }
}

pub(crate) trait Render {
    fn render(&mut self, gl: &gl::Gl, view: &Matrix4<f32>, projection: &Matrix4<f32>, our_shader: u32);
}

pub(crate) trait Update {
    fn update(&mut self, delta: f32, ground: &Ground);
}

impl Runtime {
    pub(crate) fn new() -> Runtime {
        let sdl = sdl2::init().unwrap();

        let video = sdl.video().unwrap();

        #[cfg(not(target_os = "emscripten"))]
            let context_params = (sdl2::video::GLProfile::Core, 3, 0);
        #[cfg(target_os = "emscripten")]
            let context_params = (sdl2::video::GLProfile::GLES, 3, 0);


        video.gl_attr().set_context_profile(context_params.0);
        video.gl_attr().set_context_major_version(context_params.1);
        video.gl_attr().set_context_minor_version(context_params.2);

        // Create a window
        let window = video
            .window("rust-lander", WIDTH, HEIGHT)
            .resizable()
            .opengl()
            .position_centered()
            .build().unwrap();


        let gl_context = window.gl_create_context().unwrap();
        let gl_orig: std::rc::Rc<gl::Gl> = std::rc::Rc::new(gl::Gl::load_with(|s| { video.gl_get_proc_address(s) as *const _ }));

        let gl = std::rc::Rc::clone(&gl_orig);

        //let ground = Ground::new(&gl);

        let start = Instant::now();
        let camera = Flying_Camera {
            Position: Point3::new(0.25, 8.00, 0.25),
            ..Flying_Camera::default()
        };

        let special_effects = SpecialEffects::new(&gl);

        unsafe { gl.Enable(gl::BLEND); }

        #[cfg(not(target_os = "emscripten"))]
            load_sound(&sdl);

        let player = LanderMainPlayer::new(&gl);

        let start_block = Instant::now();
        let opengl_shadow = OpenglShadow::new(&gl);
        let duration = start_block.elapsed();
        println!("Time elapsed in openglshadow is: {:?}", duration);

        //let draw_text = Runtime::setup_text_if_not_loaded(&gl);


        //let opengl_shadow = OpenglShadowPointAllDirections::new(&gl);
        let runtime = Runtime {
            opengl_shadow,
            loaded: false,
            now: Instant::now(),
            last_time_called: 0,
            sdl,
            _video: video,
            window,
            _gl_context: gl_context,
            gl: gl_orig,
            camera,
            ground: None,
            left: false,
            right: false,
            up: false,
            down: false,
            forward: false,
            backward: false,
            space: false,
            score: 0,
            level: 1,
            change_level_pause: 0,
            rescued: 0,
            tick: 0,
            player_avitar: player,
            camera_angle: 0.0,
            draw_text: None,
            special_effects,
            flash_message: String::new(),
            flash_message_countdown: 0,
            game_over: false,
            end_screen: EndScreen::new(&gl),
            rate_debug: "".to_string(),
            slow_loading_items: true,
            bernard: 0,
            loading_screen1 : LoadingScreen::new(&gl,"resources/loading.png"),
            loading_screen2 : LoadingScreen::new(&gl,"resources/loading2.png"),
        };
        let duration = start.elapsed();
        println!("Time elapsed in game() is: {:?}", duration);
        runtime
    }
}


impl emscripten_main_loop::MainLoop for Runtime {
    fn main_loop(&mut self) -> emscripten_main_loop::MainLoopEvent {
        self.tick = self.tick + 1;


        let debug_start = Instant::now();

        let time_now = self.now.elapsed().as_millis();
        let diff = time_now - self.last_time_called;

        if diff < 1000 / TARGET_FPS {
            self.bernard = self.bernard + 1;
            return MainLoopEvent::Continue;
        }

        self.last_time_called = time_now;

        let delta = (diff as f32) / 1000.0;

        let fps = 1.0 / delta as f32;

        // just for browser, big drop in rate on first load
        let update_delta = delta; //if fps > 5.0 { 1.0 } else { fps };

        if self.tick % 20 == 0 {
            self.rate_debug = format!("{} - {:2.2}", self.bernard, fps);
        }


        if self.loaded == false {
            self.loaded = true;
            #[cfg(target_os = "emscripten")]
                unsafe {
                start_game();
            }
        }
        unsafe {
            self.gl.Enable(gl::DEPTH_TEST);
        }

        let end_status = if ! self.slow_loading_items {
            self.game_playing_loop(debug_start, update_delta)
        } else {
            unsafe {
                self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
                self.gl.Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
            }
            if self.draw_text.is_none() {
                self.loading_screen1.render(&self.gl);
            } else {
                self.loading_screen2.render(&self.gl);

            }
            self.window.gl_swap_window();

            if self.tick > 10 && self.draw_text.is_none() {
                self.setup_text_if_not_loaded();
            }

            if self.tick > 20 && self.ground.is_none()  {
                self.ground = Some(Ground::new(&self.gl));
                self.slow_loading_items = false;
            }
            MainLoopEvent::Continue
        };


        match end_status {
            MainLoopEvent::Terminate => {
                #[cfg(target_os = "emscripten")]
                    unsafe {
                    end_game();
                }
            }
            MainLoopEvent::Continue => {}
        }

        end_status
    }
}


impl Runtime {
    fn setup_text_if_not_loaded(&mut self) {
        let start_block = Instant::now();
        let draw_text = DrawText::new(&self.gl);
        let duration = start_block.elapsed();
        println!("Time elapsed in drawtext is: {:?}", duration);
        self.draw_text = Some(draw_text);
    }
    fn reset_game(&mut self) {
        self.player_avitar.reset();
        self.forward = false;
        self.player_avitar.lives = 3;
        self.score = 0;
        self.level = 1;
        self.rescued = 0;
        self.game_over = false;
    }
    fn position_camera_matrix(&self) -> Matrix4<f32> {
        let rotation: Basis3<f32> = Rotation3::from_angle_y(Deg(self.camera_angle));

        let away: Vector3<f32> = rotation.rotate_vector(vec3(0.0, 0.0, 2.0));
        let mut here = self.player_avitar.movement_collision.position.clone() + away;
        here.y = here.y + 0.5;
        let matrix =
            Matrix4::look_at(vec2point(here),
                             vec2point(self.player_avitar.movement_collision.position),
                             vec3(0.0, 1.0, 0.0));
        matrix
    }

    fn handle_keyboard(&mut self) -> MainLoopEvent {
        let mut return_status = emscripten_main_loop::MainLoopEvent::Continue;
        let mut events = self.sdl.event_pump().unwrap();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return_status = emscripten_main_loop::MainLoopEvent::Terminate;
                }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    self.left = true;
                    self.right = false;
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    self.right = true;
                    self.left = false;
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    self.up = true;
                    self.down = false
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    self.down = true;
                    self.up = false
                }
                Event::KeyDown { keycode: Some(Keycode::LShift), .. } => {
                    self.forward = true;
                }
                Event::KeyDown { keycode: Some(Keycode::RShift), .. } => {
                    self.backward = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    self.space = true;
                }
                Event::KeyDown { keycode: Some(Keycode::N), .. } => { self.camera_angle = 0.0; }
                Event::KeyDown { keycode: Some(Keycode::S), .. } => { self.camera_angle = 180.0; }
                Event::KeyDown { keycode: Some(Keycode::E), .. } => { self.camera_angle = 90.0; }
                Event::KeyDown { keycode: Some(Keycode::W), .. } => { self.camera_angle = 270.0; }
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => { self.left = false; }
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => { self.right = false; }
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => { self.up = false }
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => { self.down = false }
                Event::KeyUp { keycode: Some(Keycode::LShift), .. } => { self.forward = false }
                Event::KeyUp { keycode: Some(Keycode::RShift), .. } => { self.backward = false }
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => { self.space = false; }

                _ => {}
            }
        }

        return_status
    }

    fn game_playing_loop(&mut self, _debug_start: Instant, update_delta: f32) -> MainLoopEvent {
        let projection: Matrix4<f32> =
            perspective(Deg(PERSPECTIVE_ANGLE), WIDTH as f32 / HEIGHT as f32, 0.01, 100.0);

        let view = self.position_camera_matrix();

            self.ground.as_mut().unwrap().set_player_position(self.player_avitar.movement_collision.position.x, self.player_avitar.movement_collision.position.z);

            if !self.game_over {
                self.ground.as_mut().unwrap().update(&self.gl, self.player_avitar.movement_collision.position, self.level, self.camera_angle, update_delta);
            }

            if !self.game_over {
                let rescue = self.ground.as_mut().unwrap().handle_player_rescue(self.player_avitar.movement_collision.position);
                self.rescued += rescue.0;
                self.score += rescue.1;
            }

        self.change_level_pause += -1;
        self.flash_message_countdown += -1;
        if self.change_level_pause <= 0 {
            if self.rescued >= self.level * 2 {
                self.level = self.level + 1;
                self.flash_message = format!("well done next level {}. You rescued {}", self.level, self.rescued);
                self.flash_message_countdown = 240;
                self.change_level_pause = 120;
                self.rescued = 0;
            }
        }


        self.opengl_shadow.update_light_pos(
            self.player_avitar.movement_collision.position.x, 6.0, self.player_avitar.movement_collision.position.z,
            self.camera_angle);

            self.special_effects.update(update_delta, &self.ground.as_ref().unwrap());

        self.opengl_shadow.start_render_shadow(&self.gl);

        self.player_avitar.render(&self.gl, &view, &projection, self.opengl_shadow.simple_depth_shader);
        self.special_effects.render(&self.gl, &view, &projection, self.opengl_shadow.simple_depth_shader);
        //self.rescue.render(&self.gl, &view, &projection,self.opengl_shadow.simple_depth_shader);

        self.opengl_shadow.after_rendersceneshadow(&self.gl);

        self.opengl_shadow.before_renderscenenormal(&self.gl, vec3(self.camera.Position.x, self.camera.Position.y, self.camera.Position.z));
            self.ground.as_mut().unwrap().render(&self.gl, &view, &projection, self.player_avitar.movement_collision.position, self.camera_angle, self.opengl_shadow.shader);
        self.player_avitar.render(&self.gl, &view, &projection, self.opengl_shadow.shader);
        self.special_effects.render(&self.gl, &view, &projection, self.opengl_shadow.shader);

        if !self.game_over {
            if self.player_avitar.lives <= 0 {
                self.forward = false;
                //if 1 == 1 { self.score = thread_rng().gen_range(10, 100); }

                self.end_screen.game_over(self.score, self.level);
                self.game_over = true;
            } else {
                if self.draw_text.is_some() {
                    let status = format!("Fuel={:05.2} rescued {} score {} level {} lives {}",
                                         //self.rate_debug,
                                         self.player_avitar.fuel, self.rescued, self.score, self.level, self.player_avitar.lives);//self.player_avitar.rotation_x,self.player_avitar.rotation_y);
                    self.draw_text.as_ref().unwrap().draw_text(&self.gl, &status, 2.0, HEIGHT as f32 - 30.0, vec3(1.0, 1.0, 0.0));
                    if self.flash_message_countdown > 0 {
                        self.draw_text.as_ref().unwrap().draw_text(&self.gl, self.flash_message.as_str(), 2.0, HEIGHT as f32 - 64.0, vec3(1.0, 1.0, 0.0));
                    }
                }
            }
        }
        if self.game_over {
            self.end_screen.render(&self.gl, &view, &projection, self.opengl_shadow.shader);
            if self.end_screen.space {
                self.reset_game();
                self.space = false;
            }
        }

        self.window.gl_swap_window();


        // update here to remove flicker.
            if !self.game_over {
                let msg = self.player_avitar.update(update_delta, &self.ground.as_ref().unwrap(), &self.camera, self.tick, &mut self.special_effects);
                if msg.len() > 0 {
                    self.flash_message = msg.to_string();
                    self.flash_message_countdown = 60;

                }
            }

        self.camera.save_position();


        let change = 70.0 * update_delta;
        if self.left { self.player_avitar.rotation_y_constant(-change) } else if self.right { self.player_avitar.rotation_y_constant(change) } else {
            self.player_avitar.rotation_y_constant(0.0)
        }

            if self.up {
                self.player_avitar.change_pitch(change, &self.ground.as_mut().unwrap())
            } else if self.down {
                self.player_avitar.change_pitch(-change, &self.ground.as_mut().unwrap())
            } else {
                self.player_avitar.change_pitch(0.0, &self.ground.as_mut().unwrap())
            }


            let thrust_by = 1.5 * update_delta;

            if self.forward { self.player_avitar.forward(thrust_by, &self.ground.as_ref().unwrap()); } else {
                if self.player_avitar.thrusting {
                    stop(ENGINE);
                }
                self.player_avitar.thrusting = false;
            }


        let end_status = if self.game_over {
            self.end_screen.handle_keyboard(&mut self.sdl.event_pump().unwrap())
        } else {
            self.handle_keyboard()
        };


        let mut list: Vec<String> = Vec::new();
        list.push(format!("level {} score {} rescued {} {}", self.level, self.score, self.rescued, self.player_avitar.msg));
        let update: String = list.join("\n");

        write_stats_data(CString::new(update).to_owned().unwrap().as_ptr());

        /*
            COULD ALL BE QUICKER BUT MAKE SURE ALL MODELS LOADED OUTSIDE OF LOOP.
         */

        /*
        let duration = debug_start.elapsed().as_millis();
        if duration > 40 {
            self.rate_debug = format!("{:?} {}",duration,self.rate_debug);
        } else {
            //self.rate_debug = format!("");
        }
        //println!("Time elapsed in expensive_ground() is: {:?}", duration);
         */
        end_status
    }
}
