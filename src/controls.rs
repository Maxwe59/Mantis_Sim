use crate::mantis::Mantis;
use crate::{MovementMode, WorldOptions};
use bevy::prelude::*;


pub struct ArcLengthCurve {
    samples: Vec<(f32, f32)>,  // (distance, t) pairs
    total_length: f32,
}

impl ArcLengthCurve {
    pub fn new<F: Fn(f32) -> Vec3>(curve: F, resolution: usize) -> Self {
        let mut samples = vec![(0.0, 0.0)];
        let mut total_length = 0.0;
        let mut prev_pos = curve(0.0);
        
        for i in 1..=resolution {
            let t = i as f32 / resolution as f32;
            let pos = curve(t);
            total_length += prev_pos.distance(pos);
            samples.push((total_length, t));
            prev_pos = pos;
        }
        
        Self { samples, total_length }
    }
    
    fn sample_at_distance<F: Fn(f32) -> Vec3>(&self, distance: f32, curve: F) -> Vec3 {
        let d = distance.clamp(0.0, self.total_length);
        
        // Binary search for the right segment
        let idx = self.samples
            .partition_point(|(dist, _)| *dist < d)
            .min(self.samples.len() - 1);
        
        if idx == 0 {
            return curve(0.0);
        }
        
        let (d0, t0) = self.samples[idx - 1];
        let (d1, t1) = self.samples[idx];
        
        // Interpolate t between samples
        let frac = (d - d0) / (d1 - d0);
        let t = t0 + (t1 - t0) * frac;
        
        curve(t)
    }
}

pub fn lemniscate(t: f32) -> Vec3 {
    return Vec3::new(((2.0 * t).cos()).sqrt()* t.cos(), 0.0, ((2.0 * t).cos()).sqrt()  * t.sin());
}

pub fn keyboard_controls(
    mut mantis: Single<(&mut Transform, &Mantis)>,
    input: Res<ButtonInput<KeyCode>>,
    mode: Res<WorldOptions>,
    time: Res<Time>,
) {
    if mode.movement_mode != MovementMode::Keyboard {
        return;
    }
    let mut transform = (0.0, 0.0);
    if input.pressed(KeyCode::KeyW) {
        transform.1 += 1.0;
    } else if input.pressed(KeyCode::KeyS) {
        transform.1 -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        transform.0 -= 1.0;
    } else if input.pressed(KeyCode::KeyD) {
        transform.0 += 1.0;
    }
    let speed = mantis.1.speed;
    mantis.0.translation.x += transform.0 * speed * time.delta_secs();
    mantis.0.translation.z += transform.1 * speed * time.delta_secs();
}

pub fn mouse_controls(
    mut mantis: Single<(&mut Transform, &Mantis)>,
    input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
) {
}

pub fn auto_movement(mut mantis: Single<&mut Transform, With<Mantis>>, mut mode: ResMut<WorldOptions>) {

    if mode.movement_mode != MovementMode::Auto {
        return;
    }
    if mode.auto_t > std::f32::consts::PI {
        mode.auto_t = -std::f32::consts::PI;
    }
    mode.auto_t += 0.01;
    let t: f32 = mode.auto_t;

    mantis.translation.x = ((2.0 * t).cos()).sqrt() * mode.auto_scale * t.cos();
    mantis.translation.z = ((2.0 * t).cos()).sqrt() * mode.auto_scale * t.sin();
}

pub fn switch_movement_mode(mut mode: ResMut<WorldOptions>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyM) {
        match mode.movement_mode {
            MovementMode::Mouse => {
                mode.movement_mode = MovementMode::Keyboard;
            }
            MovementMode::Keyboard => {
                mode.movement_mode = MovementMode::Auto;
            }
            MovementMode::Auto => {
                mode.movement_mode = MovementMode::Mouse;
            }
        }
    }
}
