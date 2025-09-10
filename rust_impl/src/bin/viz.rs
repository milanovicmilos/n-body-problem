use plotters::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::collections::HashMap;

fn read_energy<P: AsRef<Path>>(p: P) -> Result<Vec<(usize, f64, f64, f64)>, Box<dyn Error>> {
    let f = File::open(p)?;
    let mut rdr = csv::Reader::from_reader(f);
    let mut out = Vec::new();
    for result in rdr.records() {
        let rec = result?;
        let iter: usize = rec.get(0).unwrap().parse()?;
        let k: f64 = rec.get(1).unwrap().parse()?;
        let u: f64 = rec.get(2).unwrap().parse()?;
        let t: f64 = rec.get(3).unwrap().parse()?;
        out.push((iter, k, u, t));
    }
    Ok(out)
}

fn read_states_sample<P: AsRef<Path>>(p: P, sample_ids: &[usize]) -> Result<Vec<Vec<(f64, f64)>>, Box<dyn Error>> {
    // returns per-sample-id vectors of (x,y) across iterations in files named states_iter_*.csv
    let mut results: Vec<Vec<(f64, f64)>> = vec![Vec::new(); sample_ids.len()];
    let dir = p.as_ref();
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if !path.is_file() { continue; }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if !name.starts_with("states_iter_") { continue; }
        }
        let f = File::open(&path)?;
        let rdr = BufReader::new(f);
        for line in rdr.lines().skip(1) {
            let s = line?;
            let parts: Vec<&str> = s.split(',').collect();
            let id: usize = parts[1].parse()?;
            if let Some(pos) = sample_ids.iter().position(|&x| x == id) {
                let x: f64 = parts[2].parse()?;
                let y: f64 = parts[3].parse()?;
                results[pos].push((x, y));
            }
        }
    }
    Ok(results)
}

fn plot_energy(data: &[(usize, f64, f64, f64)], out: &Path) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(out, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    let max_iter = data.last().map(|d| d.0).unwrap_or(1) as i32;
    let y_vals: Vec<f64> = data.iter().map(|d| d.3).collect();
    let min_y = y_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = y_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Total Energy vs Iteration", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0..max_iter, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().map(|d| (d.0 as i32, d.3)),
        &RED,
    ))?;

    Ok(())
}

fn plot_trajectories(samples: &[Vec<(f64, f64)>], out: &Path) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(out, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    // compute bounds
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for s in samples {
        for (x, y) in s {
            if *x < min_x { min_x = *x }
            if *y < min_y { min_y = *y }
            if *x > max_x { max_x = *x }
            if *y > max_y { max_y = *y }
        }
    }
    if min_x==f64::INFINITY { min_x= -1.0; max_x=1.0; min_y=-1.0; max_y=1.0 }

    let mut chart = ChartBuilder::on(&root)
        .caption("Sample Trajectories (x-y)", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    let colors = &[&BLUE, &GREEN, &MAGENTA, &CYAN, &BLACK];
    for (i, sample) in samples.iter().enumerate() {
        chart.draw_series(LineSeries::new(
            sample.iter().map(|(x,y)| (*x,*y)),
            colors[i % colors.len()],
        ))?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: nbody-viz <path-to-output-dir> [--scaling-csv <file>]");
        std::process::exit(1);
    }
    let out_dir = Path::new(&args[1]);

    // optional: --scaling-csv <file>
    let mut scaling_csv: Option<String> = None;
    let mut idx = 2;
    while idx + 1 < args.len() {
        if args[idx] == "--scaling-csv" {
            scaling_csv = Some(args[idx + 1].clone());
            idx += 2;
            continue;
        }
        idx += 1;
    }

    let energy = read_energy(out_dir.join("energy.csv"))?;
    let samples = read_states_sample(out_dir, &[0,1,2])?;

    let report_dir = out_dir.join("viz");
    std::fs::create_dir_all(&report_dir)?;

    plot_energy(&energy, &report_dir.join("energy.png"))?;
    plot_trajectories(&samples, &report_dir.join("trajectories.png"))?;

    // If user supplied a scaling CSV, produce Amdahl & Gustafson plots
    if let Some(csv_path) = scaling_csv {
        if Path::new(&csv_path).exists() {
            if let Err(e) = plot_scaling(&csv_path, &report_dir) {
                eprintln!("Failed to generate scaling plots: {}", e);
            }
        } else {
            eprintln!("Scaling CSV not found: {}", csv_path);
        }
    }

    println!("Visualization generated at {}", report_dir.display());
    Ok(())
}

/// Parse a scaling CSV and draw Amdahl (speedup) and Gustafson (efficiency) plots.
fn plot_scaling(csv_path: &str, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    // CSV expected columns: lang,mode,processes,n,steps,dt,eps,time
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut groups: HashMap<String, Vec<(usize, f64)>> = HashMap::new();
    for result in rdr.records() {
        let rec = result?;
        let lang = rec.get(0).unwrap_or("");
        let mode = rec.get(1).unwrap_or("");
        let procs: usize = rec.get(2).unwrap_or("1").parse().unwrap_or(1);
        let time: f64 = rec.get(7).unwrap_or("0").parse().unwrap_or(0.0);
        let key = format!("{}::{}", lang, mode);
        groups.entry(key).or_default().push((procs, time));
    }

    if groups.is_empty() {
        return Ok(());
    }

    let mut series: Vec<(String, Vec<(usize, f64)>)> = Vec::new();
    let mut max_procs = 1usize;
    let mut max_speedup = 1.0f64;
    for (k, mut v) in groups.into_iter() {
        v.sort_by_key(|t| t.0);
        if let Some(m) = v.iter().map(|(p, _)| *p).max() { if m > max_procs { max_procs = m } }
        series.push((k, v));
    }

    // compute global max speedup for y-axis
    for (_k, v) in &series {
        if let Some(base) = v.iter().find(|(p, _)| *p == 1).map(|(_, t)| *t) {
            if base > 0.0 {
                for (_p, t) in v {
                    let s = base / *t;
                    if s > max_speedup { max_speedup = s }
                }
            }
        }
    }

    let amd_out = out_dir.join("scaling_amdal.png");
    let root = BitMapBackend::new(&amd_out, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;
    let x_max = max_procs as f64 + 1.0;
    let y_max = (max_speedup * 1.1).max(1.0);
    let mut chart = ChartBuilder::on(&root)
        .caption("Scaling (Amdahl) - Speedup vs Processes", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..x_max, 0f64..y_max)?;
    chart.configure_mesh().x_desc("Processes").y_desc("Speedup").draw()?;

    let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN, &BLACK];
    for (i, (_label, v)) in series.iter().enumerate() {
        let base = v.iter().find(|(p, _)| *p == 1).map(|(_, t)| *t).unwrap_or(0.0);
        if base <= 0.0 { continue; }
        let points: Vec<(f64, f64)> = v.iter().map(|(p, t)| (*p as f64, base / *t)).collect();
        chart.draw_series(LineSeries::new(points.clone(), colors[i % colors.len()]))?;
        chart.draw_series(points.into_iter().map(|(x,y)| Circle::new((x,y), 3, colors[i % colors.len()].filled())))?;
    }
    root.present()?;

    // Gustafson: efficiency = (base) / (time * p)
    let gust_out = out_dir.join("scaling_gustafson.png");
    let root2 = BitMapBackend::new(&gust_out, (1024, 768)).into_drawing_area();
    root2.fill(&WHITE)?;
    let mut chart2 = ChartBuilder::on(&root2)
        .caption("Scaling (Gustafson) - Parallel Efficiency vs Processes", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..x_max, 0f64..1.05f64)?;
    chart2.configure_mesh().x_desc("Processes").y_desc("Efficiency").draw()?;
    for (i, (_label, v)) in series.iter().enumerate() {
        let base = v.iter().find(|(p, _)| *p == 1).map(|(_, t)| *t).unwrap_or(0.0);
        if base <= 0.0 { continue; }
        let points: Vec<(f64, f64)> = v.iter().map(|(p, t)| (*p as f64, base / (*t * (*p as f64)))).collect();
        chart2.draw_series(LineSeries::new(points.clone(), colors[i % colors.len()]))?;
        chart2.draw_series(points.into_iter().map(|(x,y)| Circle::new((x,y), 3, colors[i % colors.len()].filled())))?;
    }
    root2.present()?;

    Ok(())
}
