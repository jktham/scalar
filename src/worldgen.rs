use std::{
    f32::consts::PI,
    ops::{Add, Mul},
    path::Path,
    time::Instant,
};

use bevy::prelude::*;
use fastapprox::fast;
use fastrand::Rng;
use image::ColorType;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

fn smoothstep(a: f32, b: f32, w: f32) -> f32 {
    return (b - a) * (3.0 - w * 2.0) * w * w + a;
}

/// deterministic random gradient for each grid coordinate
fn random_gradient(ix: i32, iy: i32) -> Vec2 {
    let seed: u64 = (ix as u64).strict_shl(32) | iy as u64;
    let mut prng = Rng::with_seed(seed);

    let r = prng.f32() * PI * 2.0 - PI; // [-pi, pi)
    let gradient = Vec2::new(fast::cos(r), fast::sin(r));

    gradient
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
        let t0 = Instant::now();
        println!("running worldgen, {TERRAIN_N}x{TERRAIN_N}");

        let mut terrain = WorldGen::new();

        // terrain height
        let scale = 0.01;
        let offset = 1000.0;
        let height = 100.0;

        terrain
            .height
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, v)| {
                let x = i / TERRAIN_N;
                let z = i % TERRAIN_N;

                *v = perlin_octaves(
                    (x as f32 * scale + offset) / TERRAIN_RESOLUTION,
                    (z as f32 * scale + offset) / TERRAIN_RESOLUTION,
                    3,
                    2.0,
                    0.5,
                ) * height;
            });

        // terrain normal
        terrain
            .normal
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, v)| {
                let x = i / TERRAIN_N;
                let z = i % TERRAIN_N;

                if x >= TERRAIN_N - 1 || z >= TERRAIN_N - 1 {
                    *v = Vec3::Y;
                    return;
                }

                let dx = (terrain.height[(x + 1) * TERRAIN_N + z]
                    - terrain.height[(x - 0) * TERRAIN_N + z])
                    / (1.0 / TERRAIN_RESOLUTION);
                let dz = (terrain.height[x * TERRAIN_N + (z + 1)]
                    - terrain.height[x * TERRAIN_N + (z - 0)])
                    / (1.0 / TERRAIN_RESOLUTION);

                let tx = Vec3::new(1.0, dx, 0.0);
                let tz = Vec3::new(0.0, dz, 1.0);

                let n = -tx.cross(tz).normalize();

                *v = n;
            });

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());

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

    /// dump data as pngs in path
    pub fn dump(&self, path: &Path) {
        // height
        let min_height = self.height.iter().copied().reduce(f32::min).unwrap_or(0.0);
        let max_height = self.height.iter().copied().reduce(f32::max).unwrap_or(1.0);

        fn normalize_float(f: f32, min: f32, max: f32) -> f32 {
            if min == max {
                return 0.0;
            }
            (f - min) / (max - min)
        }

        image::save_buffer(
            path.join("height.png"),
            self.height
                .iter()
                .map(|f| (normalize_float(*f, min_height, max_height) * 255.0) as u8)
                .collect::<Vec<u8>>()
                .as_slice(),
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            ColorType::L8,
        )
        .unwrap();

        // normal
        image::save_buffer(
            path.join("normal.png"),
            self.normal
                .iter()
                .map(|v| {
                    [
                        (v.x * 255.0) as u8,
                        (v.y * 255.0) as u8,
                        (v.z * 255.0) as u8,
                    ]
                })
                .flatten()
                .collect::<Vec<u8>>()
                .as_slice(),
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            ColorType::Rgb8,
        )
        .unwrap();
    }
}
