#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use cgmath::*;
use winit::event::*;

use crate::camera;
use crate::qone_bsp;

const MOVEMENT_JUMP: i32 = 1 << 1;
pub const MOVEMENT_JUMP_THIS_FRAME: i32 = 1 << 2;
pub const MOVEMENT_JUMPING: i32 = 1 << 3;

//pub const DELTA: f32 = 0.006944;
pub const DELTA: f32 = 16.666666666666 * 0.001;
//pub const DELTA: f32 = 0.016666 * 0.2;

const GRAVITY: f32 = 800.0;
//const STOP_SPEED: f32 = 200.0;
const MAX_SPEED: f32 = 320.0;

const STEP_SIZE: f32 = 18.0;

const MAX_CLIP_PLANES: usize = 5;
const OVERCLIP: f32 = 1.001;

const STOP_EPSILON: f32 = 0.1;
const STOP_SPEED: f32 = 100.0;
const FRICTION: f32 = 6.0;
const ACCELERATE: f32 = 10.0;
const AIR_ACCELERATE: f32 = 0.7;

const RUNNING: f32 = 2.0;
const WALKING: f32 = 1.0;

const FORWARD_SPEED: f32 = 200.0;
const BACKWARD_SPEED: f32 = 200.0;
const SIDE_SPEED: f32 = 350.0;
const JUMP_VELOCITY: f32 = 270.0;

const WATER_ACCELERATE: f32 = 10.0;

const BUTTON_JUMP: i32 = 2;

pub const PLAYER_MINS: [f32; 3] = [-16.0, -16.0, -24.0];
pub const PLAYER_MAXS: [f32; 3] = [16.0, 16.0, 32.0];

//Quake World values
//cl_forwardspeed = 200
//cl_upspeed = 200
//cl_backspeed = 200
//cl_sidespeed = 350

pub struct Player {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub noclip: bool,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    speed: f32,
    pub wish_dir: Vector3<f32>,
    pub movement: i32,
    mins: Vector3<f32>,
    maxs: Vector3<f32>,
    ground_normal: Vector3<f32>,
    forward_speed: f32,
    side_speed: f32,
    direction: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    on_ground: i32,
    running: f32,
    pub delta_time: f32,
    water_level: i32,
    water_type: i32,
}

impl Player {

    pub fn new() -> Player {
        Player { position: Vector3::new(0.0, 0.0, 0.0), velocity: Vector3::new(0.0, 0.0, 0.0), noclip: true, amount_left: 0.0, amount_right: 0.0, amount_forward: 0.0, amount_backward: 0.0, amount_up: 0.0, amount_down: 0.0, speed: 4.0,
            wish_dir: Vector3::new(0.0, 0.0, 0.0), movement: 0,
            mins: Vector3::new(-15.0, -15.0, -24.0),
            maxs: Vector3::new(15.0, 15.0, 32.0),
            ground_normal: Vector3::new(0.0, 0.0, 0.0),
            forward_speed: FORWARD_SPEED,
            side_speed: SIDE_SPEED,
            direction: Vector3::new(1.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            on_ground: -1,
            running: RUNNING,
            delta_time: 0.0,
            water_level: 0,
            water_type: qone_bsp::CONTENTS_EMPTY,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {

        match key {
            VirtualKeyCode::W => {
                if state == ElementState::Pressed {
                    self.wish_dir.x = self.forward_speed * self.running;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.x > 0.0 {
                        self.wish_dir.x = 0.0;
                    }
                }
            }
            VirtualKeyCode::S => {
                if state == ElementState::Pressed {
                    self.wish_dir.x = -self.forward_speed * self.running;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.x < 0.0 {
                        self.wish_dir.x = 0.0;
                    }
                }
            }
            VirtualKeyCode::A => {
                if state == ElementState::Pressed {
                    self.wish_dir.y = -self.side_speed * self.running;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.y < 0.0 {
                        self.wish_dir.y = 0.0;
                    }
                }
            }
            VirtualKeyCode::D => {
                if state == ElementState::Pressed {
                    self.wish_dir.y = self.side_speed * self.running;
                }
                else if state == ElementState::Released {
                    if self.wish_dir.y > 0.0 {
                        self.wish_dir.y = 0.0;
                    }
                }
            }
            VirtualKeyCode::Space => {
                if state == ElementState::Pressed {
                    self.movement |= BUTTON_JUMP;
                }
                else if state == ElementState::Released {
                    self.movement &= !BUTTON_JUMP;
                }
            }
            _ => {},
        }
    }

    pub fn update(&mut self, camera: &camera::Camera) {
        self.yaw = camera.yaw;
        self.pitch = camera.pitch;
    }

    pub fn air_move(&mut self, bsp: &mut qone_bsp::Bsp) {

        let fmove = self.wish_dir.x;
        let smove = self.wish_dir.y;

        let mut forward = cgmath::Vector3::new(self.yaw.cos() * self.pitch.cos(), self.yaw.sin() * self.pitch.cos(), self.pitch.sin());
        let mut right = forward.cross(cgmath::Vector3::unit_z());

        forward[2] = 0.0;
        right[2] = 0.0;
        forward = forward.normalize();
        right = right.normalize();

        let mut wish_vel = cgmath::Vector3::new(0.0, 0.0, 0.0);
        for i in 0..2 {
            wish_vel[i] = forward[i] * fmove + right[i] * smove;
        }

        let mut wish_dir = wish_vel;
        let mut wish_speed = wish_dir.magnitude();
        if wish_speed != 0.0 {
            wish_dir = wish_dir.normalize();
        }

        if wish_speed > MAX_SPEED {
            wish_vel = wish_vel * (MAX_SPEED / wish_speed);
            wish_speed = MAX_SPEED;
        }

        if self.on_ground != -1 {
            self.velocity.z = 0.0;
            self.accelerate(wish_dir, wish_speed, ACCELERATE);
            self.velocity.z -= GRAVITY * self.delta_time;
            self.ground_move(bsp);
        }
        else {
            self.air_accelerate(wish_dir, wish_speed, ACCELERATE);
            self.velocity.z -= GRAVITY * self.delta_time;
            self.fly_move(bsp);
        }
    }

    fn air_accelerate(&mut self, wish_dir: cgmath::Vector3<f32>, wish_speed: f32, accel: f32) {

        let mut wish_spd = wish_speed;

        if wish_spd > 30.0 {
            wish_spd = 30.0;
        }

        let current_speed = cgmath::dot(self.velocity, wish_dir);
        let add_speed = wish_spd - current_speed;
        if add_speed <= 0.0 {
            return;
        }
        let mut accel_speed = accel * wish_speed * self.delta_time;
        if accel_speed > add_speed {
            accel_speed = add_speed;
        }

        for i in 0..3 {
            self.velocity[i] += accel_speed * wish_dir[i];
        }
    }

    fn accelerate(&mut self, wish_dir: cgmath::Vector3<f32>, wish_speed: f32, accel: f32) {

        let current_speed = cgmath::dot(self.velocity, wish_dir);
        let add_speed = wish_speed - current_speed;

        if add_speed <= 0.0 {
            return;
        }
        let mut accel_speed = accel * self.delta_time * wish_speed;
        if accel_speed > add_speed {
            accel_speed = add_speed;
        }
        for i in 0..3 {
            self.velocity[i] += accel_speed * wish_dir[i];
        }
    }

    fn jump_button(&mut self) {

        if self.water_level >= 2 {
            self.on_ground = -1;
            self.velocity.z = 100.0;
            return;
        }

        if self.on_ground == -1 {
            return;
        }

        self.on_ground = -1;
        self.velocity.z = JUMP_VELOCITY;
        self.movement |= BUTTON_JUMP;
    }

    pub fn player_move(&mut self, bsp: &mut qone_bsp::Bsp) {

        self.nudge_position(bsp);

        self.catagorize_position(bsp);

        if self.movement & BUTTON_JUMP != 0 {
            self.jump_button();
        }
        else {
            self.movement &= !BUTTON_JUMP;
        }

        self.friction();

        if self.water_level >= 2 {
            self.water_move(bsp);
        }
        else {
            self.air_move(bsp);
        }

        self.catagorize_position(bsp);
    }

    fn water_move(&mut self, bsp: &mut qone_bsp::Bsp) {

        let fmove = self.wish_dir.x;
        let smove = self.wish_dir.y;

        let mut wish_vel = cgmath::Vector3::new(0.0, 0.0, 0.0);

        let mut forward = cgmath::Vector3::new(self.yaw.cos() * self.pitch.cos(), self.yaw.sin() * self.pitch.cos(), self.pitch.sin());
        let mut right = forward.cross(cgmath::Vector3::unit_z());

        for i in 0..3 {
            wish_vel[i] = forward[i] * fmove + right[i] * smove;
        }

        if wish_vel[0] == 0.0 && wish_vel[1] == 0.0 && wish_vel[2] == 0.0 {
            wish_vel.z -= 60.0;
        }

        let mut wish_dir = wish_vel.normalize();
        let mut wish_speed = wish_vel.magnitude();
        
        if wish_speed >  MAX_SPEED {
            wish_vel = wish_vel * (MAX_SPEED / wish_speed);
            wish_speed = MAX_SPEED;
        }
        wish_speed *= 0.7;

        self.accelerate(wish_dir, wish_speed, WATER_ACCELERATE);

        let dest = self.position + self.delta_time * self.velocity;
        let mut start = dest;

        start.z += STEP_SIZE + 1.0;

        let trace = bsp.player_trace(start, dest);
        if !trace.starts_solid && !trace.all_solid {
            self.position = trace.end_pos;
            return;
        }

        self.fly_move(bsp);
    }

    fn nudge_position(&mut self, bsp: &mut qone_bsp::Bsp) {
        
        let base = self.position;

        let sign = [0.0, -1.0, 1.0];

        for i in 0..3 {
            self.position[i] = (((self.position[i] * 8.0) as i32) as f32) * 0.125;
        }

        for z in 0..3 {
            for x in 0..3 {
                for y in 0..3 {

                    self.position.x = base.x + (sign[x] * 1.0 / 8.0);
                    self.position.y = base.y + (sign[y] * 1.0 / 8.0);
                    self.position.z = base.z + (sign[z] * 1.0 / 8.0);
                    if bsp.test_player_position(self.position) {
                        return;
                    }
                }
            }
        }

        self.position = base;
    }

    fn friction(&mut self) {

        let mut vel = self.velocity;

        let speed = self.velocity.magnitude();
        if speed < 1.0 {
            self.velocity[0] = 0.0;
            self.velocity[1] = 0.0;
            return;
        }

        let mut drop = 0.0;
        if self.water_level >= 2 {
            drop += speed * 4.0 * self.water_level as f32 *  self.delta_time;
        }
        else if self.on_ground != -1 {
            let mut control = speed;
            if speed < STOP_SPEED {
                control = STOP_SPEED;
            }
            drop += control * FRICTION * self.delta_time;
        }

        let mut newspeed = speed - drop;
        if newspeed < 0.0 {
            newspeed = 0.0;
        }
        newspeed /= speed;

        self.velocity[0] = vel[0] * newspeed;
        self.velocity[1] = vel[1] * newspeed;
        self.velocity[2] = vel[2] * newspeed;
    }

    fn catagorize_position(&mut self, bsp: &mut qone_bsp::Bsp) {

        let mut point = self.position;
        point.z -= 1.0;
        if self.velocity.z > 180.0 {
            self.on_ground = -1;
        }
        else {
            let tr = bsp.player_trace(self.position, point);
            if tr.plane.normal[2] < 0.7 {
                self.on_ground = -1;
            }
            else {
                self.on_ground = tr.ent;
            }
            if self.on_ground != -1 {

                if !tr.starts_solid && !tr.all_solid {
                    self.position = tr.end_pos;
                }
            }

            if tr.ent > 0 {

            }
        }


        self.water_level = 0;
        self.water_type = qone_bsp::CONTENTS_WATER;
        point.z = self.position.z + PLAYER_MINS[2] + 1.0;
        let mut cont = bsp.point_contents(point);

        if cont <= qone_bsp::CONTENTS_WATER {
            self.water_type = cont;
            self.water_level = 1;
            point.z = self.position.z + (PLAYER_MINS[2] + PLAYER_MAXS[2]) * 0.5;
            cont = bsp.point_contents(point);
            if cont <= qone_bsp::CONTENTS_WATER {
                self.water_level = 2;
                point.z = self.position.z + 22.0;
                cont = bsp.point_contents(point);
                if cont <= qone_bsp::CONTENTS_WATER {
                    self.water_level = 3;
                }
            }
        }

        //println!("woa {}", cont);
    }

    pub fn fly_move(&mut self, bsp: &mut qone_bsp::Bsp) {
        
        let num_bumps = 4;
        let mut blocked = 0;
        let orginal_velocity = self.velocity;
        let primal_velocity = self.velocity;
        let mut num_planes = 0;

        let mut time_left = self.delta_time;

        let mut planes: [cgmath::Vector3<f32>; MAX_CLIP_PLANES] = [cgmath::Vector3::new(0.0, 0.0, 0.0); MAX_CLIP_PLANES];

        let mut end = cgmath::Vector3::new(0.0, 0.0, 0.0);

        for bump_count in 0..num_bumps {
            for i in 0..3 {
                end[i] = self.position[i] + time_left * self.velocity[i];
            }

            let trace = bsp.player_trace(self.position, end);
            if trace.starts_solid || trace.all_solid {
                self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
                return;
            }

            if trace.fraction > 0.0 {
                self.position = trace.end_pos;
                num_planes = 0;
            }

            if trace.fraction == 1.0 {
                break;
            }

            //Entity touch

            if trace.plane.normal[2] > 0.7 {
                blocked |= 1;
            }
            if trace.plane.normal[2] == 0.0 {
                blocked |= 2;
            }

            time_left -= time_left * trace.fraction;

            if num_planes >= MAX_CLIP_PLANES {
                self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
                break;
            }

            planes[num_planes] = qone_bsp::to_vec3(trace.plane.normal);
            num_planes += 1;

            let mut i_c = 0;

            for i in 0..num_planes {
                i_c = i;
                clip_velocity(self.velocity, planes[i], &mut self.velocity, 1.0);
                for j in 0..num_planes {
                    if j != i {
                        if cgmath::dot(self.velocity, planes[j]) < 0.0 {
                            break;
                        }
                    }
                    if j == num_planes {
                        break;
                    }
                }
            }

            if i_c != num_planes {

            }
            else {
                if num_planes != 2 {
                    self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
                    break;
                }
                let dir = planes[0].cross(planes[1]);
                let d = cgmath::dot(dir, self.velocity);
                self.velocity = cgmath::Vector3::new(dir.x * d, dir.y * d, dir.z * d);
            }

            if cgmath::dot(self.velocity, primal_velocity) <= 0.0 {
                self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
                break;
            }
        }
    }

    pub fn ground_move(&mut self, bsp: &mut qone_bsp::Bsp) {

        self.velocity[2] = 0.0;
        if self.velocity[0] == 0.0 && self.velocity[1] == 0.0 && self.velocity[2] == 0.0 {
            return;
        }

        let mut dest = cgmath::Vector3::new(self.position[0] + self.velocity[0] * self.delta_time, self.position[1] + self.velocity[1] * self.delta_time, self.position[2]);

        let start = dest;

        let mut trace = bsp.player_trace(self.position, dest);
 
        if trace.fraction == 1.0 {
            self.position = trace.end_pos;
            return;
        }

        let original = self.position;
        let original_vel = self.velocity;

        self.fly_move(bsp);

        let down = self.position;
        let down_vel = self.velocity;

        self.position = original;
        self.velocity = original_vel;
        dest = self.position;
        dest[2] += STEP_SIZE;
        trace = bsp.player_trace(self.position, dest);
        if !trace.starts_solid && !trace.all_solid {
            self.position = trace.end_pos;
        }

        self.fly_move(bsp);

        dest = self.position;
        dest[2] -= STEP_SIZE;
        trace = bsp.player_trace(self.position, dest);
        if trace.plane.normal[2] < 0.7 {
            self.position = down;
            self.velocity = down_vel;
            return;
        }
        if !trace.starts_solid && !trace.all_solid {
            self.position = trace.end_pos;
        }
        let up = self.position;

        let down_dist = (down[0] - original[0]) * (down[0] - original[0]) + (down[1] - original[1]) * (down[1] - original[1]);

        let up_dist = (up[0] - original[0]) * (up[0] - original[0]) + (up[1] - original[1]) * (up[1] - original[1]);
        
        if down_dist > up_dist {
            self.position = down;
            self.velocity = down_vel;
        }
        else {
            self.velocity[2] = down_vel[2];
        }
    }
}

pub fn clip_velocity(inv: cgmath::Vector3<f32>, normal: cgmath::Vector3<f32>, out: &mut cgmath::Vector3<f32>, overbounce: f32) {

    let mut blocked = 0;

    if normal[2] > 0.0 {
        blocked |= 1;
    }
    if normal[2] == 0.0 {
        blocked |= 2;
    }

    let backoff = cgmath::dot(inv, normal) * overbounce;

    for i in 0..3 {

        let change = normal[i] * backoff;
        out[i] = inv[i] - change;
        if out[i] > STOP_EPSILON && out[i] < STOP_EPSILON {
            out[i] = 0.0;
        }
    }
}