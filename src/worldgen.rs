use std::{
    f32::consts::PI,
    ops::{Add, Mul},
    path::Path,
    time::Instant,
};

use bevy::prelude::*;
use fastapprox::fast;
use fastrand::Rng;
use image::{ImageBuffer, Luma, Rgb, imageops};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::inventory::{Item, ItemStack};

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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Ground {
    Sand,
    Grass,
    Dirt,
    Stone,
}

impl Ground {
    pub fn color(&self) -> Color {
        match self {
            Ground::Sand => Color::srgb(0.906, 0.937, 0.447),
            Ground::Grass => Color::srgb(0.098, 0.718, 0.18),
            Ground::Dirt => Color::srgb(0.584, 0.361, 0.102),
            Ground::Stone => Color::srgb(0.608, 0.639, 0.663),
        }
    }
}

#[derive(Resource)]
pub struct WorldGen {
    height: Vec<f32>,
    normal: Vec<Vec3>,
    ground: Vec<Ground>,
    pub ore_nodes: Vec<(Transform, ItemStack)>,
    pub tree_nodes: Vec<(Transform, ItemStack)>,
    pub rock_nodes: Vec<(Transform, ItemStack, i32 /* variant */)>,
}

impl WorldGen {
    pub fn new() -> Self {
        Self {
            height: vec![0.0; TERRAIN_N * TERRAIN_N],
            normal: vec![Vec3::Y; TERRAIN_N * TERRAIN_N],
            ground: vec![Ground::Dirt; TERRAIN_N * TERRAIN_N],
            ore_nodes: Vec::new(),
            tree_nodes: Vec::new(),
            rock_nodes: Vec::new(),
        }
    }

    pub fn generate() -> Self {
        let mut worldgen = WorldGen::new();

        worldgen.generate_maps();
        worldgen.place_nodes();

        worldgen.dump(std::path::Path::new("./assets/map"));

        worldgen
    }

    pub fn generate_maps(&mut self) {
        let t0 = Instant::now();
        println!("generating worldgen maps, {TERRAIN_N}x{TERRAIN_N}");

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
                // coast and ocean
                *v = Ground::Sand;
            } else if self.normal[i].dot(Vec3::Y).abs().acos() < f32::to_radians(35.0) {
                // flat plateau
                *v = Ground::Grass;
            } else if self.normal[i].dot(Vec3::Y).abs().acos() < f32::to_radians(50.0) {
                // mild incline
                *v = Ground::Dirt;
            } else {
                // steep incline
                *v = Ground::Stone;
            }
        });

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());
    }

    pub fn place_nodes(&mut self) {
        let t0 = Instant::now();
        println!(
            "generating worldgen nodes, {} ores, {} trees, {} rocks",
            N_ORES, N_TREES, N_ROCKS
        );

        let mut rng = Rng::with_seed(67);

        // ore
        const N_ORES: usize = 1_000;
        self.ore_nodes.reserve(N_ORES);
        for _ in 0..N_ORES {
            let mut pos = vec3(
                rng.f32() * TERRAIN_SIZE - TERRAIN_SIZE / 2.0,
                0.0,
                rng.f32() * TERRAIN_SIZE - TERRAIN_SIZE / 2.0,
            );
            pos.y = self.get_height(pos.x, pos.z);

            if self.get_ground(pos.x, pos.z) == Ground::Sand {
                continue;
            }

            let rot = Quat::from_rotation_y(rng.f32() * std::f32::consts::TAU);
            let normal = self.get_normal(pos.x, pos.z);
            let normal_rot =
                Quat::from_axis_angle(normal.cross(Vec3::Y), -f32::acos(normal.dot(Vec3::Y)));

            let transform = Transform::from_translation(pos).with_rotation(normal_rot * rot);

            let variant = rng.i32(0..3);
            let stack = match variant {
                0 => ItemStack {
                    item: Item::Iron,
                    count: rng.i32(100..1000),
                },
                1 => ItemStack {
                    item: Item::Copper,
                    count: rng.i32(100..1000),
                },
                _ => ItemStack {
                    item: Item::Coal,
                    count: rng.i32(100..1000),
                },
            };

            self.ore_nodes.push((transform, stack));
        }

        // trees
        const N_TREES: usize = 30_000;
        self.tree_nodes.reserve(N_TREES);
        for _ in 0..N_TREES {
            let mut pos = vec3(
                rng.f32() * TERRAIN_SIZE - TERRAIN_SIZE / 2.0,
                0.0,
                rng.f32() * TERRAIN_SIZE - TERRAIN_SIZE / 2.0,
            );
            pos.y = self.get_height(pos.x, pos.z);

            if self.get_ground(pos.x, pos.z) == Ground::Sand
                || self.get_ground(pos.x, pos.z) == Ground::Stone
            {
                continue; // no trees
            }
            if self.get_ground(pos.x, pos.z) == Ground::Dirt && rng.f32() > 0.3 {
                continue; // less trees
            }

            let rot = Quat::from_rotation_y(rng.f32() * std::f32::consts::TAU);

            let transform = Transform::from_translation(pos).with_rotation(rot);

            let stack = ItemStack {
                item: Item::Wood,
                count: 5,
            };

            self.tree_nodes.push((transform, stack));
        }

        // rocks
        const N_ROCKS: usize = 8_000;
        self.rock_nodes.reserve(N_ROCKS);
        for _ in 0..N_ROCKS {
            let mut pos = vec3(
                rng.f32() * TERRAIN_SIZE - TERRAIN_SIZE / 2.0,
                0.0,
                rng.f32() * TERRAIN_SIZE - TERRAIN_SIZE / 2.0,
            );
            pos.y = self.get_height(pos.x, pos.z);

            let rot = Quat::from_rotation_y(rng.f32() * std::f32::consts::TAU);
            let normal = self.get_normal(pos.x, pos.z);
            let normal_rot =
                Quat::from_axis_angle(normal.cross(Vec3::Y), -f32::acos(normal.dot(Vec3::Y)));

            let transform = Transform::from_translation(pos).with_rotation(normal_rot * rot);

            let mut variant = rng.i32(0..2);
            let mut stack = ItemStack {
                item: Item::Stone,
                count: 1,
            };
            if rng.f32() < 0.02 {
                // rare big rock :)
                variant = 2;
                stack.count = 10;
            }

            self.rock_nodes.push((transform, stack, variant));
        }

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());
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
            return Ground::Sand;
        }

        self.ground[ix as usize * TERRAIN_N + iz as usize]
    }

    /// dump data as pngs in path
    pub fn dump(&self, path: &Path) {
        let t0 = Instant::now();
        println!("dumping worldgen data to {}", path.display());

        // create dir if not exists
        std::fs::create_dir_all(path).unwrap();

        // height
        let min_height = self.height.iter().copied().reduce(f32::min).unwrap_or(0.0);
        let max_height = self.height.iter().copied().reduce(f32::max).unwrap_or(1.0);

        fn normalize_float(f: f32, min: f32, max: f32) -> f32 {
            if min == max {
                return 0.0;
            }
            (f - min) / (max - min)
        }

        let height_img = ImageBuffer::<Luma<u8>, Vec<_>>::from_raw(
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            self.height
                .iter()
                .map(|f| (normalize_float(*f, min_height, max_height) * 255.0) as u8)
                .collect(),
        )
        .unwrap();
        imageops::flip_vertical(&height_img)
            .save(path.join("height.png"))
            .unwrap();

        // normal
        let normal_img = ImageBuffer::<Rgb<u8>, Vec<_>>::from_raw(
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            self.normal
                .iter()
                .flat_map(|v| v.to_array().map(|f| (f * 255.0) as u8))
                .collect(),
        )
        .unwrap();
        imageops::flip_vertical(&normal_img)
            .save(path.join("normal.png"))
            .unwrap();

        // ground
        let ground_img = ImageBuffer::<Rgb<u8>, Vec<_>>::from_raw(
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            self.ground
                .iter()
                .flat_map(|g| {
                    g.color()
                        .to_srgba()
                        .to_f32_array_no_alpha()
                        .map(|f| (f * 255.0) as u8)
                })
                .collect(),
        )
        .unwrap();
        imageops::flip_vertical(&ground_img)
            .save(path.join("ground.png"))
            .unwrap();

        // nodes
        fn pos_to_index(pos: Vec3) -> usize {
            let p = pos + Vec3::new(TERRAIN_SIZE / 2.0, 0.0, TERRAIN_SIZE / 2.0);
            let x = p.x.floor() as usize;
            let z = p.z.floor() as usize;
            let i = x * TERRAIN_N + z;
            i
        }

        let mut nodes = vec![Vec3::ZERO; TERRAIN_N * TERRAIN_N];
        for (transform, stack) in &self.ore_nodes {
            nodes[pos_to_index(transform.translation)] = match stack.item {
                Item::Iron => Color::srgb(0., 0.451, 1.).to_srgba().to_vec3(),
                Item::Copper => Color::srgb(1., 0.2, 0.).to_srgba().to_vec3(),
                _ => Color::srgb(0.204, 0.204, 0.204).to_srgba().to_vec3(),
            }
        }
        for (transform, _) in &self.tree_nodes {
            nodes[pos_to_index(transform.translation)] =
                Color::srgb(0., 1., 0.).to_srgba().to_vec3();
        }
        for (transform, _, _) in &self.rock_nodes {
            nodes[pos_to_index(transform.translation)] =
                Color::srgb(0.871, 0.871, 0.871).to_srgba().to_vec3();
        }

        let nodes_img = ImageBuffer::<Rgb<u8>, Vec<_>>::from_raw(
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            nodes
                .iter()
                .flat_map(|v| v.to_array().map(|f| (f * 255.0) as u8))
                .collect(),
        )
        .unwrap();
        imageops::flip_vertical(&nodes_img)
            .save(path.join("nodes.png"))
            .unwrap();

        // relief
        let mut relief = vec![Vec3::ZERO; TERRAIN_N * TERRAIN_N];
        for i in 0..relief.len() {
            let normal = self.normal[i].normalize();
            let light_dir = Vec3::new(1.0, 1.0, -1.0).normalize(); // from ground to light (points to top left)
            let diffuse = normal.dot(light_dir) / 2.0 + 0.5; // [0, 1]

            let height = self.height[i];
            let height_normalized = normalize_float(height, min_height, max_height); // [0, 1]

            let ground_col = self.ground[i].color().to_srgba().to_vec3();
            let node_col = nodes[i];
            let water_height = 0.0;
            let water_col = Color::srgba(0.039, 0.161, 0.392, 0.8).to_srgba().to_vec4();

            let base_col = match node_col {
                Vec3::ZERO => {
                    if height <= water_height {
                        ground_col.lerp(water_col.xyz(), water_col.w)
                    } else {
                        ground_col
                    }
                }
                _ => node_col,
            };
            relief[i] = base_col * diffuse * (height_normalized + 0.25 / 1.25);
        }

        let relief_img = ImageBuffer::<Rgb<u8>, Vec<_>>::from_raw(
            TERRAIN_N as u32,
            TERRAIN_N as u32,
            relief
                .iter()
                .flat_map(|v| v.to_array().map(|f| (f * 255.0) as u8))
                .collect(),
        )
        .unwrap();
        imageops::flip_vertical(&relief_img)
            .save(path.join("relief.png"))
            .unwrap();

        let t1 = Instant::now();
        println!("done, {:.2}s", (t1 - t0).as_secs_f32());
    }
}
