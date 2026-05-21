use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn smoothstep(a: f32, b: f32, w: f32) -> f32 {
    return (b - a) * (3.0 - w * 2.0) * w * w + a;
}

fn random_gradient(ix: i32, iy: i32) -> Vec2 {
    let seed: u64 = (ix as u64).strict_shl(32) | iy as u64;
    let mut prng = StdRng::seed_from_u64(seed);

    let r = prng.random::<f32>() * PI * 2.0; // [0, 2*pi)
    return Vec2::new(f32::cos(r), f32::sin(r)); // [-1, 1]
}

fn dot_grid_gradient(ix: i32, iy: i32, x: f32, y: f32) -> f32 {
    let gradient = random_gradient(ix, iy);
    let dx = x - ix as f32;
    let dy = y - iy as f32;
    return dx * gradient.x + dy * gradient.y;
}

/// perlin noise implementation stolen from https://en.wikipedia.org/w/index.php?title=Perlin_noise&oldid=1230993513 <3
// TODO: negative coordinates
pub fn perlin(x: f32, y: f32) -> f32 {
    // grid points
    let x0 = x.floor() as i32;
    let x1 = x0 + 1;
    let y0 = y.floor() as i32;
    let y1 = y0 + 1;

    // interpolation weights
    let sx = x - x0 as f32;
    let sy = y - y0 as f32;

    // interpolate between grid point gradients
    let n0 = dot_grid_gradient(x0, y0, x, y);
    let n1 = dot_grid_gradient(x1, y0, x, y);
    let ix0 = smoothstep(n0, n1, sx);

    let n2 = dot_grid_gradient(x0, y1, x, y);
    let n3 = dot_grid_gradient(x1, y1, x, y);
    let ix1 = smoothstep(n2, n3, sx);

    return smoothstep(ix0, ix1, sy) * 0.5 + 0.5; // [0, 1]
}

pub fn perlin_octaves(x: f32, y: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += perlin(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;

        amplitude *= persistence; // reduce contribution each octave
        frequency *= lacunarity; // increase detail each octave
    }

    value / max_value // [0, 1]
}

pub fn get_terrain_height(x: f32, z: f32) -> f32 {
    let scale = 0.01;
    let offset = 1000.0; // to avoid negative coordinates
    let height = 100.0;
    perlin_octaves(x * scale + offset, z * scale + offset, 3, 2.0, 0.5) * height
}
