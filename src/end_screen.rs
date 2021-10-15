use crate::high_score_table::{HighScoreTable, };
use cgmath::{Matrix4, vec3};
use crate::{HEIGHT, WIDTH};
use crate::gl;
use crate::gl_helper::draw_text::DrawText;
use emscripten_main_loop::MainLoopEvent;
use sdl2::event::Event;
use sdl2::EventPump;
use std::collections::HashSet;
use sdl2::keyboard::Keycode;


pub struct EndScreen{
    high_score_table:HighScoreTable,
    draw_text: DrawText,
    tick:i32,
    flash:usize,
    new_entry:bool,
    prev_keys:HashSet<sdl2::keyboard::Keycode>,
    keyboard_input:String,
    keyboard_input_done:bool,
    pub(crate) space:bool,
}
//const DATE_FORMAT_STR: &'static str = "%Y-%m-%d][%H:%M:%S";

impl EndScreen{
    pub fn new(gl: &gl::Gl) -> EndScreen {
        EndScreen{
            high_score_table:HighScoreTable::new(),
            draw_text: DrawText::new(&gl),
            tick:0,
            flash:0,
            new_entry:false,
            prev_keys:HashSet::new(),
            keyboard_input:String::new(),
            keyboard_input_done:true,
            space:false,
        }
    }
    pub fn game_over(&mut self,score:i32,level:i32) {
        self.space = false;
        self.keyboard_input.clear();
        self.keyboard_input_done = true;
        if self.high_score_table.add_score(score,level) {
            self.new_entry = true;
            self.keyboard_input_done = false;
        } else {
            self.new_entry = false;
        }
    }
    pub(crate) fn render(&mut self, gl: &gl::Gl, _view: &Matrix4<f32>, _projection: &Matrix4<f32>,_our_shader:u32)  {
        self.tick += 1;
        if self.tick % 60 == 0 {
            self.flash=self.flash +1;
        }
        if self.new_entry == true {
            if self.keyboard_input_done || self.keyboard_input.len() >= 6 {
                self.keyboard_input.truncate(5);
                self.high_score_table.set_name(&self.keyboard_input);
                self.new_entry = false;
            }
        }
        if self.new_entry == false {
            let mut y = 1.0;
            let mut i=self.flash;
            let colours = vec![vec3(1.0,0.0,0.0),vec3(0.0,1.0,1.0),vec3(0.0,1.0,0.0),vec3(0.0,0.0,1.0)];
            for high_score in self.high_score_table.table.iter() {
                let entry = format!("score({}) level({}) by {}",high_score.score,high_score.level,high_score.by);
                y = y + 1.0;
                i=i+1;
                if self.high_score_table.is_current_score(high_score.id) {
                    let white_or_grey = if self.flash %2 == 0 { 1.0 } else { 0.5 };
                    self.draw_text.draw_text(gl, entry.as_str(), WIDTH as f32 * 0.05 , HEIGHT as f32 - y * 48.0,
                                             vec3(white_or_grey, white_or_grey, white_or_grey));
                } else {
                    self.draw_text.draw_text(gl, entry.as_str(), WIDTH as f32 * 0.05 , HEIGHT as f32 - y * 48.0,
                                             colours[i % colours.len()]);
                }
                self.draw_text.draw_text(gl, &high_score.formatted, WIDTH as f32 *0.65 , HEIGHT as f32 - y * 48.0, vec3(1.0, 1.0, 1.0));
            }
            self.draw_text.draw_text(gl, "Press space for another game", -(WIDTH as f32 * 0.6) + (self.tick as f32) % (WIDTH  as f32 *1.6)  , HEIGHT as f32 - 48.0, vec3(1.0, 1.0, 0.0));
            //self.draw_text.draw_text(gl, "HELLO Game over", -200.0 + (self.tick % (WIDTH + 200) as i32) as f32, HEIGHT as f32 - 164.0, vec3(1.0, 1.0, 0.0));
        } else {
            self.draw_text.draw_text(gl, "New entry enter your name", 10.0  , HEIGHT as f32 - 48.0, vec3(1.0, 1.0, 0.0));
            self.draw_text.draw_text(gl, self.keyboard_input.as_str(), 10.0  , HEIGHT as f32 - 96.0, vec3(1.0, 1.0, 0.0));
            self.draw_text.draw_text(gl, "_____", 10.0  , HEIGHT as f32 - 100.0, vec3(1.0, 1.0, 0.0));

        }

    }

    pub fn handle_keyboard(&mut self, events: &mut EventPump) -> MainLoopEvent {
        let mut return_status = emscripten_main_loop::MainLoopEvent::Continue;

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return_status = emscripten_main_loop::MainLoopEvent::Terminate;
                }
                _ => {}
            }
        }

        let keys = events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let new_keys = &keys - &self.prev_keys;
        if !new_keys.is_empty() {
            //println!("Keyboard {:?}",new_keys);
            for k in new_keys.iter() {
                let what = k.name();
                if *k == Keycode::Return {
                   self.keyboard_input_done = true;
                } else if *k == Keycode::Space  {
                    self.keyboard_input.push_str(" ");
                    if ! self.new_entry  {
                        self.space = true;
                    }
                } else if *k == Keycode::Backspace  {
                    if self.keyboard_input.len() > 0 {
                        self.keyboard_input.truncate(self.keyboard_input.len() - 1);
                    }
                } else if what.len() == 1 {
                    self.keyboard_input.push_str(k.name().as_str());
                }
            }
        }

        self.prev_keys = keys;

        return_status
    }
}