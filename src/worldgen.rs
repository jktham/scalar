use std::{f32::consts::PI, ops::Add, ops::Mul};

use bevy::prelude::*;

fn smoothstep(a: f32, b: f32, w: f32) -> f32 {
    return (b - a) * (3.0 - w * 2.0) * w * w + a;
}

/// deterministic random gradient for same inputs
fn random_gradient(ix: i32, iy: i32) -> Vec2 {
    let seed: u64 = (ix as u64).strict_shl(32) | iy as u64;
    let mut prng = fastrand::Rng::with_seed(seed);

    let r = prng.f32() * PI * 2.0; // [0, 2*pi)
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

/// get value at float coordinate in array via bilinear interpolation
pub fn bilinear_interp<T>(x: f32, z: f32, arr: &Vec<T>) -> T
where
    T: Copy,
    T: Mul<f32, Output = T>,
    T: Add<T, Output = T>,
{
    let x0 = x.floor();
    let x1 = x0 + 1.0;
    let z0 = z.floor();
    let z1 = z0 + 1.0;

    let dx = (x - x0) / (x1 - x0);
    let dz = (z - z0) / (z1 - z0);

    let f00 = arr[x0 as usize * TERRAIN_N + z0 as usize];
    let f01 = arr[x0 as usize * TERRAIN_N + z1 as usize];
    let f10 = arr[x1 as usize * TERRAIN_N + z0 as usize];
    let f11 = arr[x1 as usize * TERRAIN_N + z1 as usize];

    let fx0 = f00 * (1.0 - dx) + f10 * dx;
    let fx1 = f01 * (1.0 - dx) + f11 * dx;
    let fxz = fx0 * (1.0 - dz) + fx1 * dz;

    fxz
}

pub const TERRAIN_N: usize = 1000; // n*n array size
pub const TERRAIN_RESOLUTION: f32 = 1.0; // pixels per meter

#[derive(Resource)]
pub struct WorldGen {
    height: Vec<f32>,
    normal: Vec<Vec3>,
}

impl WorldGen {
    pub fn new() -> Self {
        Self {
            height: vec![0.0; TERRAIN_N * TERRAIN_N],
            normal: vec![Vec3::Y; TERRAIN_N * TERRAIN_N],
        }
    }

    pub fn generate() -> Self {
        let mut terrain = WorldGen::new();

        // terrain height
        let scale = 0.01;
        let offset = 1000.0;
        let height = 100.0;

        for x in 0..TERRAIN_N {
            for z in 0..TERRAIN_N {
                terrain.height[x * TERRAIN_N + z] = perlin_octaves(
                    (x as f32 * scale + offset) / TERRAIN_RESOLUTION,
                    (z as f32 * scale + offset) / TERRAIN_RESOLUTION,
                    3,
                    2.0,
                    0.5,
                ) * height;
            }
        }

        // terrain normal
        for x in 0..TERRAIN_N - 1 {
            for z in 0..TERRAIN_N - 1 {
                let dx = (terrain.height[(x + 1) * TERRAIN_N + z]
                    - terrain.height[(x - 0) * TERRAIN_N + z])
                    / (1.0 / TERRAIN_RESOLUTION);
                let dz = (terrain.height[x * TERRAIN_N + (z + 1)]
                    - terrain.height[x * TERRAIN_N + (z - 0)])
                    / (1.0 / TERRAIN_RESOLUTION);

                let tx = Vec3::new(1.0, dx, 0.0);
                let tz = Vec3::new(0.0, dz, 1.0);

                let normal = tx.cross(tz).normalize();

                terrain.normal[x * TERRAIN_N + z] = normal;
            }
        }

        terrain
    }

    /// interpolated terrain height at meters x, z
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        // map [-t/2, t/2] -> [0, t]
        let ix = (x * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;
        let iz: f32 = (z * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;

        // check bounds
        if ix < 0.0 || ix >= TERRAIN_N as f32 - 1.0 || iz < 0.0 || iz >= TERRAIN_N as f32 - 1.0 {
            return 0.0;
        }

        // interpolate height value
        bilinear_interp::<f32>(ix, iz, &self.height)
    }

    /// interpolated terrain normal at meters x, z
    pub fn get_normal(&self, x: f32, z: f32) -> Vec3 {
        // map [-t/2, t/2] -> [0, t]
        let ix = (x * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;
        let iz: f32 = (z * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;

        // check bounds
        if ix < 0.0 || ix >= TERRAIN_N as f32 - 1.0 || iz < 0.0 || iz >= TERRAIN_N as f32 - 1.0 {
            return Vec3::Y;
        }

        // interpolate normal vector
        bilinear_interp::<Vec3>(ix, iz, &self.normal).normalize()
    }
}
