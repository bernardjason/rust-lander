extern crate cgmath;

/*
use std::fs;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

use cgmath::*;
use cgmath::{vec3};
use cgmath::prelude::*;

use crate::{gl, WIDTH, HEIGHT};
use crate::shadow_shaders::*;

pub const SHADOW_WIDTH: i32 = 1024;
pub const SHADOW_HEIGHT: i32 = 1024;

pub struct OpenglShadowInstance {
    pub id: u128,
    pub matrix: Matrix4<f32>,
}

impl PartialEq for OpenglShadowInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct OpenglShadowPointAllDirections {
    depth_map_fbo: u32,
    depth_cube_map: u32,
    pub simple_depth_shader: u32,
    pub shader: u32,
    light_pos: Vector3<f32>,
    far_plane:f32,

}

impl OpenglShadowPointAllDirections {
    pub fn new(gl: &gl::Gl) -> OpenglShadowPointAllDirections {

        let shader = create_shader(&gl, POINT_SHADOWS_VS, POINT_SHADOWS_FS, None);
        let simple_depth_shader = create_shader(&gl,
                                                POINT_SHADOWS_DEPTH_VS, POINT_SHADOWS_DEPTH_FS, Some(POINT_SHADOWS_DEPTH_GS));


        let (depth_map_fbo, depth_cube_map) = {
            let mut depth_map_fbo: u32 = 0;
            let mut depth_cube_map: u32 = 0;
            unsafe {
                gl.GenFramebuffers(1, &mut depth_map_fbo); //    glGenFramebuffers(1, &depth_map_fbo);

                gl.GenTextures(1, &mut depth_cube_map); //     glGenTextures(1, &depthCubemap);
                gl.BindTexture(gl::TEXTURE_CUBE_MAP, depth_cube_map); //     glBindTexture(GL_TEXTURE_CUBE_MAP, depthCubemap);

                for i in 0..6 {
                    gl.TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X + i, 0, gl::DEPTH_COMPONENT as i32,
                             SHADOW_WIDTH, SHADOW_HEIGHT, 0, gl::DEPTH_COMPONENT, gl::FLOAT, ptr::null_mut());
                    // glTexImage2D(GL_TEXTURE_CUBE_MAP_POSITIVE_X + i, 0, GL_DEPTH_COMPONENT, SHADOW_WIDTH, SHADOW_HEIGHT, 0, GL_DEPTH_COMPONENT, GL_FLOAT, NULL);
                }
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32); //    glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32); //     glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32); //     glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32); //     glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
                gl.TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32); //     glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_R, GL_CLAMP_TO_EDGE);

                gl.BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo); //     glBindFramebuffer(GL_FRAMEBUFFER, depth_map_fbo);
                gl.FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_cube_map, 0); //     glFramebufferTexture(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, depthCubemap, 0);
                gl.DrawBuffer(gl::NONE); //     glDrawBuffer(GL_NONE);
                gl.ReadBuffer(gl::NONE); //     glReadBuffer(GL_NONE);
                gl.BindFramebuffer(gl::FRAMEBUFFER, 0); //     glBindFramebuffer(GL_FRAMEBUFFER, 0);
            }
            (depth_map_fbo, depth_cube_map)
        };

        unsafe {
            gl.UseProgram(shader); //     shader.use();
            gl_int(gl, shader, 0, "diffuseTexture"); //     shader.setInt("diffuseTexture", 0);
            gl_int(gl, shader, 1, "depthMap"); //     shader.setInt("depthMap", 1);
        }
        OpenglShadowPointAllDirections {
            depth_map_fbo: depth_map_fbo,
            depth_cube_map: depth_cube_map,
            simple_depth_shader: simple_depth_shader,
            shader,
            //lightSpaceMatrix: Matrix4::<f32>::from_scale(0.0),
            light_pos: vec3(0.0, 6.0, 0.0),
            far_plane:50.0,
        }
    }
    pub fn start_render_shadow(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            gl.Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);

        }

        let near_plane: f32 = 1.0;
        let shadow_proj = perspective(Deg(90.0), SHADOW_WIDTH as f32 / SHADOW_HEIGHT as f32, near_plane, self.far_plane);

        let mut shadow_transforms: Vec<Matrix4<f32>> = Vec::new();
        shadow_transforms.push(shadow_proj * Matrix4::look_at(p2v(self.light_pos), p2v(self.light_pos + vec3(1.0f32, 0.0, 0.0)), vec3(0.0, -1.0, 0.0)));
        shadow_transforms.push(shadow_proj * Matrix4::look_at(p2v(self.light_pos), p2v(self.light_pos + vec3(-1.0f32, 0.0, 0.0)), vec3(0.0, -1.0, 0.0)));
        shadow_transforms.push(shadow_proj * Matrix4::look_at(p2v(self.light_pos), p2v(self.light_pos + vec3(0.0f32, 1.0, 0.0)), vec3(0.0, 0.0, 1.0)));
        shadow_transforms.push(shadow_proj * Matrix4::look_at(p2v(self.light_pos), p2v(self.light_pos + vec3(0.0f32, -1.0, 0.0)), vec3(0.0, 0.0, -1.0)));
        shadow_transforms.push(shadow_proj * Matrix4::look_at(p2v(self.light_pos), p2v(self.light_pos + vec3(0.0f32, 0.0, 1.0)), vec3(0.0, -1.0, 0.0)));
        shadow_transforms.push(shadow_proj * Matrix4::look_at(p2v(self.light_pos), p2v(self.light_pos + vec3(0.0f32, 0.0, -1.0)), vec3(0.0, -1.0, 0.0)));


        //let lightView = Matrix4::look_at( Point3::new(self.light_pos.x, self.light_pos.y, self.light_pos.z), Point3::new(0.0, 0.0, 0.0), vec3(0.0f32, 1.0, 0.0));

        //self.lightSpaceMatrix = lightProjection * lightView;

        unsafe {

            gl.Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT);
            gl.BindFramebuffer(gl::FRAMEBUFFER, self.depth_map_fbo);
            gl.Clear(gl::DEPTH_BUFFER_BIT);
            gl.UseProgram(self.simple_depth_shader);

            for i in 0..6 {
                let name = format!("shadowMatrices[{}]",i);
                gl_matrix4(gl, self.simple_depth_shader, shadow_transforms[i], name.as_str());
            }
            gl_vec3(gl, self.simple_depth_shader, self.light_pos, "light_pos");
            gl_int(gl, self.simple_depth_shader, 0, "reverse_normals");
            gl_float(gl, self.simple_depth_shader, self.far_plane, "far_plane");

            //gl_matrix4(gl, self.shadow_shader, self.lightSpaceMatrix, "lightSpaceMatrix");
            //gl.Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT);
            //gl.BindFramebuffer(gl::FRAMEBUFFER, self.depth_map_fbo);
            //gl.Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn after_rendersceneshadow(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
    pub fn before_renderscenenormal(&mut self, gl: &gl::Gl, camera:Vector3<f32>) {
        unsafe {
            // 2. then render scene as normal with shadow mapping (using depth map)
            gl.Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.UseProgram(self.shader);

            gl_int(gl, self.shader, 0, "reverse_normals");
            //gl.BindTexture(gl::TEXTURE_CUBE_MAP, self.depth_cube_map);
            gl_vec3(gl, self.shader, self.light_pos, "light_pos");
            gl_vec3(gl, self.shader, vec3(camera.x, camera.y, camera.z), "viewPos");
            gl_int(gl, self.shader, 0, "shadows");
            gl_float(gl, self.shader, self.far_plane, "far_plane");
            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_CUBE_MAP, self.depth_cube_map);
        }
    }
}
fn p2v(vector:Vector3<f32>) -> Point3 {
    let p = Point3::new(vector.x,vector.y,vector.z);
    return p
}
*/