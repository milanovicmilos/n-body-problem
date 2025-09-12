use rayon::prelude::*;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub m: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

pub fn parse_bodies(json_str: &str) -> anyhow::Result<Vec<Body>> {
    let v: serde_json::Value = serde_json::from_str(json_str)?;
    let arr = v.as_array().ok_or_else(|| anyhow::anyhow!("Expected JSON array"))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let m = item.get("m").and_then(|x| x.as_f64()).ok_or_else(|| anyhow::anyhow!("m missing"))?;
        let x = item.get("x").and_then(|x| x.as_f64()).ok_or_else(|| anyhow::anyhow!("x missing"))?;
        let y = item.get("y").and_then(|x| x.as_f64()).ok_or_else(|| anyhow::anyhow!("y missing"))?;
        let z = item.get("z").and_then(|x| x.as_f64()).ok_or_else(|| anyhow::anyhow!("z missing"))?;
        let vx = item.get("vx").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let vy = item.get("vy").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let vz = item.get("vz").and_then(|x| x.as_f64()).unwrap_or(0.0);
        out.push(Body { m, x, y, z, vx, vy, vz });
    }
    Ok(out)
}

pub fn random_bodies(n: usize, mass_range: (f64, f64), pos_range: (f64, f64), vel_range: (f64, f64), seed: u64) -> Vec<Body> {
    use rand::prelude::*;
    use rand_pcg::Pcg64;
    let mut rng = Pcg64::seed_from_u64(seed);
    let mut out = Vec::with_capacity(n);
    let mass_dist = rand::distributions::Uniform::new_inclusive(mass_range.0, mass_range.1);
    let pos_dist = rand::distributions::Uniform::new_inclusive(pos_range.0, pos_range.1);
    let vel_dist = rand::distributions::Uniform::new_inclusive(vel_range.0, vel_range.1);
    for _ in 0..n {
        let m = rng.sample(mass_dist);
        let x = rng.sample(pos_dist);
        let y = rng.sample(pos_dist);
        let z = rng.sample(pos_dist);
        let vx = rng.sample(vel_dist);
        let vy = rng.sample(vel_dist);
        let vz = rng.sample(vel_dist);
        out.push(Body { m, x, y, z, vx, vy, vz });
    }
    out
}

pub fn compute_acc(bodies: &[Body], g: f64, softening: f64) -> Vec<(f64, f64, f64)> {
    let n = bodies.len();
    let mut acc = vec![(0.0, 0.0, 0.0); n];
    for i in 0..n {
        let (xi, yi, zi) = (bodies[i].x, bodies[i].y, bodies[i].z);
        let (mut aix, mut aiy, mut aiz) = (0.0, 0.0, 0.0);
        for j in 0..n {
            if i == j { continue; }
            let dx = bodies[j].x - xi;
            let dy = bodies[j].y - yi;
            let dz = bodies[j].z - zi;
            let dist_sqr = dx * dx + dy * dy + dz * dz + softening;
            let inv_r3 = 1.0 / (dist_sqr * dist_sqr.sqrt());
            let f = g * bodies[j].m * inv_r3;
            aix += dx * f; aiy += dy * f; aiz += dz * f;
        }
        acc[i] = (aix, aiy, aiz);
    }
    acc
}

pub fn compute_acc_par(bodies: &[Body], g: f64, softening: f64) -> Vec<(f64, f64, f64)> {
    let n = bodies.len();
    (0..n).into_par_iter().map(|i| {
        let (xi, yi, zi) = (bodies[i].x, bodies[i].y, bodies[i].z);
        let mut aix = 0.0; let mut aiy = 0.0; let mut aiz = 0.0;
        for j in 0..n {
            if i == j { continue; }
            let dx = bodies[j].x - xi;
            let dy = bodies[j].y - yi;
            let dz = bodies[j].z - zi;
            let dist_sqr = dx * dx + dy * dy + dz * dz + softening;
            let inv_r3 = 1.0 / (dist_sqr * dist_sqr.sqrt());
            let f = g * bodies[j].m * inv_r3;
            aix += dx * f; aiy += dy * f; aiz += dz * f;
        }
        (aix, aiy, aiz)
    }).collect()
}

pub fn step_euler(bodies: &mut [Body], acc: &[(f64, f64, f64)], dt: f64) {
    for (b, (ax, ay, az)) in bodies.iter_mut().zip(acc.iter()) {
        b.vx += ax * dt; b.vy += ay * dt; b.vz += az * dt;
    }
    for b in bodies.iter_mut() {
        b.x += b.vx * dt; b.y += b.vy * dt; b.z += b.vz * dt;
    }
}

pub fn write_state_csv<P: AsRef<Path>>(path: P, iteration: usize, bodies: &[Body], create_header: bool) -> anyhow::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() { create_dir_all(parent)?; }
    }
    let mut writer: Box<dyn Write> = if create_header {
        Box::new(BufWriter::new(File::create(&path)?))
    } else {
        Box::new(BufWriter::new(std::fs::OpenOptions::new().append(true).open(&path)?))
    };
    if create_header { writeln!(writer, "iteration,id,m,x,y,z,vx,vy,vz")?; }
    for (i, b) in bodies.iter().enumerate() {
        writeln!(writer, "{},{},{},{},{},{},{},{},{}", iteration, i, b.m, b.x, b.y, b.z, b.vx, b.vy, b.vz)?;
    }
    Ok(())
}
