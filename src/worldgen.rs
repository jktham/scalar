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
    (b - a) * (3.0 - w * 2.0) * w * w + a
}

/// deterministic random gradient for each grid coordinate
fn random_gradient(ix: i32, iy: i32) -> Vec2 {
    let seed: u64 = (ix as u64).strict_shl(32) | iy as u64;
    let mut prng = Rng::with_seed(seed);

    let r = prng.f32() * PI * 2.0 - PI; // [-pi, pi)

    Vec2::new(fast::cos(r), fast::sin(r))
}

fn dot_grid_gradient(ix: i32, iy: i32, x: f32, y: f32) -> f32 {
    let gradient = random_gradient(ix, iy);
    let dx = x - ix as f32;
    let dy = y - iy as f32;
    dx * gradient.x + dy * gradient.y
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

    smoothstep(ix0, ix1, sy) * 0.5 + 0.5 // [0, 1]
}

/// returns height in \[0, 1\]
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
pub fn bilinear_interp<T>(x: f32, z: f32, arr: &[T]) -> T
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

    fx0 * (1.0 - dz) + fx1 * dz
}

pub const TERRAIN_N: usize = 2000; // n*n array size
pub const TERRAIN_RESOLUTION: f32 = 1.0; // pixels per meter
pub const TERRAIN_SIZE: f32 = TERRAIN_N as f32 / TERRAIN_RESOLUTION;

#[derive(Copy, Clone)]
pub enum Ground {
    Grass,
    Dirt,
    Sand,
}

impl Ground {
    pub fn color(&self) -> Color {
        match self {
            Ground::Grass => Color::srgb(0.098, 0.718, 0.18),
            Ground::Dirt => Color::srgb(0.584, 0.361, 0.102),
            Ground::Sand => Color::srgb(0.906, 0.937, 0.447),
        }
    }
}

#[derive(Resource)]
pub struct WorldGen {
    height: Vec<f32>,
    normal: Vec<Vec3>,
    ground: Vec<Ground>,
}

impl WorldGen {
    pub fn new() -> Self {
        Self {
            height: vec![0.0; TERRAIN_N * TERRAIN_N],
            normal: vec![Vec3::Y; TERRAIN_N * TERRAIN_N],
            ground: vec![Ground::Dirt; TERRAIN_N * TERRAIN_N],
        }
    }

    pub fn generate() -> Self {
        let t0 = Instant::now();
        println!("running worldgen, {TERRAIN_N}x{TERRAIN_N}");

        let mut worldgen = WorldGen::new();

        worldgen.generate_maps();

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());

        worldgen.dump(std::path::Path::new("./output"));

        worldgen
    }

    pub fn generate_maps(&mut self) {
        // terrain height
        let xz_scale = 1.0 / 200.0;
        let offset = 1000.0;
        let y_scale = 300.0;

        let compute_height = |x: f32, z: f32| -> f32 {
            let dist = f32::max(
                (x - TERRAIN_SIZE / 2.0).abs(),
                (z - TERRAIN_SIZE / 2.0).abs(),
            ) / (TERRAIN_SIZE / 2.0);
            let falloff = dist.powf(2.0) * 200.0;

            let height = perlin_octaves(
                (x * xz_scale + offset) / TERRAIN_RESOLUTION,
                (z * xz_scale + offset) / TERRAIN_RESOLUTION,
                5,
                1.8,
                0.5,
            ) * y_scale
                - falloff;

            if height < 0.0 {
                height / 2.0 // flatten off underwater
            } else {
                height
            }
        };

        self.height.par_iter_mut().enumerate().for_each(|(i, v)| {
            let x = (i / TERRAIN_N) as f32;
            let z = (i % TERRAIN_N) as f32;

            *v = compute_height(x, z);
        });

        // terrain normal
        self.normal.par_iter_mut().enumerate().for_each(|(i, v)| {
            let x = (i / TERRAIN_N) as f32;
            let z = (i % TERRAIN_N) as f32;

            let h = 0.5;
            let dx = (compute_height(x + h, z) - compute_height(x - h, z))
                / (2.0 * h / TERRAIN_RESOLUTION);
            let dz = (compute_height(x, z + h) - compute_height(x, z - h))
                / (2.0 * h / TERRAIN_RESOLUTION);

            let tx = Vec3::new(1.0, dx, 0.0);
            let tz = Vec3::new(0.0, dz, 1.0);

            let n = tz.cross(tx).normalize();

            *v = n;
        });

        // ground type
        self.ground.par_iter_mut().enumerate().for_each(|(i, v)| {
            if self.height[i] < 3.0 {
                // coast and under ocean
                *v = Ground::Sand;
            } else if self.normal[i].dot(Vec3::Y) > 0.8 {
                // flat plateaus
                *v = Ground::Grass;
            } else {
                // steep inclines
                *v = Ground::Dirt;
            }
        });
    }

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());

        worldgen.dump(std::path::Path::new("./output"));

        worldgen
    }

    /// interpolated terrain height at meters x, z
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        // map [-t/2, t/2] -> [0, t]
        let ix = (x * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;
        let iz = (z * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;

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
        let iz = (z * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;

        // check bounds
        if ix < 0.0 || ix >= TERRAIN_N as f32 - 1.0 || iz < 0.0 || iz >= TERRAIN_N as f32 - 1.0 {
            return Vec3::Y;
        }

        // interpolate normal vector
        bilinear_interp::<Vec3>(ix, iz, &self.normal).normalize()
    }

    /// closest ground type at meters x, z
    pub fn get_ground(&self, x: f32, z: f32) -> Ground {
        // map [-t/2, t/2] -> [0, t]
        let ix = (x * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;
        let iz = (z * TERRAIN_RESOLUTION) + TERRAIN_N as f32 / 2.0;

        // check bounds
        if ix < 0.0 || ix >= TERRAIN_N as f32 - 1.0 || iz < 0.0 || iz >= TERRAIN_N as f32 - 1.0 {
            return Ground::Dirt;
        }

        self.ground[ix as usize * TERRAIN_N + iz as usize]
    }

    /// dump data as pngs in path
    pub fn dump(&self, path: &Path) {
        let t0 = Instant::now();
        println!("dumping worldgen data to {}", path.display());

        // height
        let min_height = 0.0;
        let max_height = 200.0;

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
                .flat_map(|v| {
                    [
                        (v.x * 255.0) as u8,
                        (v.y * 255.0) as u8,
                        (v.z * 255.0) as u8,
                    ]
                })
                .collect::<Vec<u8>>()
                .as_slice(),
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            ColorType::Rgb8,
        )
        .unwrap();

        // ground
        image::save_buffer(
            path.join("ground.png"),
            self.ground
                .iter()
                .flat_map(|g| {
                    let color = g.color();
                    [
                        (color.to_srgba().red * 255.0) as u8,
                        (color.to_srgba().green * 255.0) as u8,
                        (color.to_srgba().blue * 255.0) as u8,
                    ]
                })
                .collect::<Vec<u8>>()
                .as_slice(),
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            ColorType::Rgb8,
        )
        .unwrap();

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());
    }
}
