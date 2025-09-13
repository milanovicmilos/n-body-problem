use clap::{Arg, ArgAction, Command};
use std::time::Instant;
use nbody::{compute_acc, compute_acc_par, parse_bodies, random_bodies, write_state_csv};

// Visualization helpers
fn visualize_csv<P: AsRef<std::path::Path>>(
    csv_path: P,
    vis_size: f64,
    vis_trails: usize,
    gif_ms: u16,
    vis_bounds_mode: &str,
    vis_pad: f64,
) -> anyhow::Result<()> {
    use csv::Reader;
    use plotters::prelude::*;
    use std::fs::create_dir_all;
    use image::{Frame, codecs::gif::GifEncoder, Delay};
    use std::fs::File;

    let csv_path = csv_path.as_ref();
    let file_stem = csv_path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| anyhow::anyhow!("Invalid csv file name"))?;
    // Place visualization folder next to the CSV in output/visualisation/<basename>/
    let out_dir = csv_path.parent().unwrap_or(std::path::Path::new(".")).join("visualisation").join(file_stem);
    create_dir_all(&out_dir)?;

    // Read CSV and group bodies by iteration
    let mut rdr = Reader::from_path(csv_path)?;
    // store (id, x, y, mass) per iteration so we can color bodies consistently
    let mut frames: std::collections::BTreeMap<usize, Vec<(usize,f64,f64,f64)>> = std::collections::BTreeMap::new();
    for result in rdr.records() {
        let rec = result?;
        // iteration,id,m,x,y,z,vx,vy,vz
        let it: usize = rec.get(0).unwrap().parse()?;
        let id: usize = rec.get(1).unwrap().parse()?;
        let x: f64 = rec.get(3).unwrap().parse()?;
        let y: f64 = rec.get(4).unwrap().parse()?;
        let m: f64 = rec.get(2).unwrap().parse()?;
        frames.entry(it).or_default().push((id,x,y,m));
    }

    if frames.is_empty() { anyhow::bail!("No data found in CSV"); }
    // Precompute global and initial bounds
    let mut g_xmin = std::f64::INFINITY; let mut g_xmax = std::f64::NEG_INFINITY;
    let mut g_ymin = std::f64::INFINITY; let mut g_ymax = std::f64::NEG_INFINITY;
    for (_it, bodies) in &frames {
        for (_id,x,y,_m) in bodies {
            if *x < g_xmin { g_xmin = *x; }
            if *x > g_xmax { g_xmax = *x; }
            if *y < g_ymin { g_ymin = *y; }
            if *y > g_ymax { g_ymax = *y; }
        }
    }
    // Initial bounds from iteration 0 (or first key)
    let (i_xmin, i_xmax, i_ymin, i_ymax) = {
    let (_first_it, bodies0) = frames.iter().next().unwrap();
        let mut ixmin = std::f64::INFINITY; let mut ixmax = std::f64::NEG_INFINITY;
        let mut iymin = std::f64::INFINITY; let mut iymax = std::f64::NEG_INFINITY;
        for (_id,x,y,_m) in bodies0 {
            if *x < ixmin { ixmin = *x; }
            if *x > ixmax { ixmax = *x; }
            if *y < iymin { iymin = *y; }
            if *y > iymax { iymax = *y; }
        }
        (ixmin, ixmax, iymin, iymax)
    };

    // Helper to pad bounds
    let pad_bounds = |mut xmin:f64, mut xmax:f64, mut ymin:f64, mut ymax:f64| {
        // guard against degenerate ranges
        let mut dx = (xmax - xmin).abs();
        let mut dy = (ymax - ymin).abs();
        if dx < 1e-9 { xmin -= 0.5; xmax += 0.5; dx = 1.0; }
        if dy < 1e-9 { ymin -= 0.5; ymax += 0.5; dy = 1.0; }
        let px = dx * vis_pad.abs();
        let py = dy * vis_pad.abs();
        xmin -= px; xmax += px; ymin -= py; ymax += py;
        (xmin, xmax, ymin, ymax)
    };

    // Create PNG frames using plotters
    let mut png_paths = Vec::new();
    let size = (800, 800);
    // simple palette to cycle through body colors (as RGB tuples)
    let palette: Vec<(u8,u8,u8)> = vec![
        (255,0,0), (0,0,255), (0,128,0), (255,0,255), (0,255,255), (255,165,0), (128,0,128), (0,0,0)
    ];

    for (it, bodies) in &frames {
        // Determine bounds for this frame
        let (xmin, xmax, ymin, ymax) = match vis_bounds_mode {
            "global" => pad_bounds(g_xmin, g_xmax, g_ymin, g_ymax),
            "initial" => pad_bounds(i_xmin, i_xmax, i_ymin, i_ymax),
            _ => { // per-frame (default)
                let mut xmin = std::f64::INFINITY; let mut xmax = std::f64::NEG_INFINITY;
                let mut ymin = std::f64::INFINITY; let mut ymax = std::f64::NEG_INFINITY;
                for (_id,x,y,_m) in bodies {
                    if *x < xmin { xmin = *x; }
                    if *x > xmax { xmax = *x; }
                    if *y < ymin { ymin = *y; }
                    if *y > ymax { ymax = *y; }
                }
                pad_bounds(xmin, xmax, ymin, ymax)
            }
        };
        let file_name = format!("frame_{:05}.png", it);
        let out_path = out_dir.join(&file_name);
        let out_path_owned = out_path.clone();
        {
            let root = BitMapBackend::new(&out_path_owned, size).into_drawing_area();
            root.fill(&WHITE)?;
            let mut chart = ChartBuilder::on(&root)
                .caption(format!("Iteration {}", it), ("sans-serif", 20).into_font())
                .margin(10)
                .set_all_label_area_size(40)
                .build_cartesian_2d(xmin..xmax, ymin..ymax)?;
            chart.configure_mesh().disable_mesh().draw()?;
            for (id,x,y,m) in bodies {
                // Draw motion trails from previous frames (if requested)
                if vis_trails > 0 {
                    for t in 1..=vis_trails {
                        if *it >= t {
                            if let Some(prev_bodies) = frames.get(&(*it - t)) {
                                if let Some((_, px, py, _pm)) = prev_bodies.iter().find(|(pid,_,_,_)| pid == id) {
                                    let (r,g,b) = palette[id % palette.len()];
                                    let alpha = ((vis_trails - t + 1) as f64 / (vis_trails as f64 + 1.0) * 0.6).min(0.6).max(0.1);
                                    let radius_f = (m.sqrt() * vis_size * 0.7_f64.powi(t as i32)).max(1.0);
                                    let radius = radius_f as u32;
                                    chart.draw_series(std::iter::once(Circle::new((*px,*py), radius, plotters::style::RGBAColor(r,g,b, alpha).filled())))?;
                                }
                            }
                        }
                    }
                }
                // radius in pixels: scale sqrt(mass) by vis_size (CLI-controlled)
                let radius_f = (m.sqrt() * vis_size).max(1.0);
                let radius = radius_f as u32;
                let (r,g,b) = palette[id % palette.len()];
                chart.draw_series(std::iter::once(Circle::new((*x,*y), radius, RGBColor(r,g,b).filled())))?;
            }
            root.present()?;
        }
        png_paths.push(out_path_owned);
    }

    // Encode GIF
    let gif_path = out_dir.join(format!("{}.gif", file_stem));
    let gif_file = File::create(&gif_path)?;
    let mut encoder = GifEncoder::new(gif_file);
    for p in &png_paths {
        let img = image::open(p)?;
        let rgba = img.to_rgba8();
        let delay = Delay::from_numer_denom_ms(gif_ms.into(), 1); // configurable ms per frame
        let frame = Frame::from_parts(rgba.clone(), 0, 0, delay);
        encoder.encode_frame(frame)?;
    }

    println!("Visualization written to {}", out_dir.display());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let matches = Command::new("nbody")
        .about("N-body simulation in Rust")
        .arg(Arg::new("visualize").long("visualize").value_parser(clap::value_parser!(String)).help("Path to CSV to visualize (produced by this program)."))
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["seq", "threads"]) // threads uses rayon
                .default_value("seq"),
        )
        .arg(Arg::new("steps").long("steps").value_parser(clap::value_parser!(usize)).default_value("100"))
        .arg(Arg::new("dt").long("dt").value_parser(clap::value_parser!(f64)).default_value("0.01"))
        .arg(Arg::new("G").long("G").value_parser(clap::value_parser!(f64)).default_value("1.0"))
        .arg(Arg::new("softening").long("softening").value_parser(clap::value_parser!(f64)).default_value("1e-9"))
        .arg(Arg::new("output").long("output").value_parser(clap::value_parser!(String)).default_value("output/nbody_rust_seq.csv"))
        .arg(Arg::new("bodies").long("bodies").value_parser(clap::value_parser!(String)))
        .arg(Arg::new("random").long("random").value_parser(clap::value_parser!(usize)).default_value("0"))
        .arg(Arg::new("seed").long("seed").value_parser(clap::value_parser!(u64)).default_value("42"))
        .arg(Arg::new("mass_range").long("mass-range").num_args(2).value_parser(clap::value_parser!(f64)).allow_hyphen_values(true).default_values(["1.0", "10.0"]))
        .arg(Arg::new("pos_range").long("pos-range").num_args(2).value_parser(clap::value_parser!(f64)).allow_hyphen_values(true).default_values(["-50.0", "50.0"]))
        .arg(Arg::new("vel_range").long("vel-range").num_args(2).value_parser(clap::value_parser!(f64)).allow_hyphen_values(true).default_values(["-0.1", "0.1"]))
    .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
    .arg(Arg::new("vis_size").long("vis-size").value_parser(clap::value_parser!(f64)).default_value("2.0").help("Visualization size multiplier for body radii (larger -> bigger circles)"))
    .arg(Arg::new("vis_trails").long("vis-trails").value_parser(clap::value_parser!(usize)).default_value("0").help("Number of previous frames to draw as fading trails for each body"))
    .arg(Arg::new("gif_ms").long("gif-ms").value_parser(clap::value_parser!(u16)).default_value("100").help("GIF frame delay in milliseconds (smaller = faster animation)"))
    .arg(Arg::new("vis_bounds").long("vis-bounds").value_parser(["per-frame","global","initial"]).default_value("per-frame").help("How to choose plot bounds: per-frame (auto), global (all frames), or initial (iteration 0)"))
    .arg(Arg::new("vis_pad").long("vis-pad").value_parser(clap::value_parser!(f64)).default_value("0.05").help("Padding fraction added to bounds on each side (e.g., 0.05 = 5%)"))
    .arg(Arg::new("write_every").long("write-every").value_parser(clap::value_parser!(usize)).default_value("1").help("Write CSV every K steps (0 = disable all writes)"))
        .get_matches();

    // If user asked only for visualization, run that and exit early
    let vis_size = *matches.get_one::<f64>("vis_size").unwrap();
    let vis_trails = *matches.get_one::<usize>("vis_trails").unwrap();
    let gif_ms = *matches.get_one::<u16>("gif_ms").unwrap();
    let vis_bounds_mode = matches.get_one::<String>("vis_bounds").unwrap();
    let vis_pad = *matches.get_one::<f64>("vis_pad").unwrap();
    if let Some(csv_to_vis) = matches.get_one::<String>("visualize") {
        visualize_csv(csv_to_vis, vis_size, vis_trails, gif_ms, vis_bounds_mode, vis_pad)?;
        return Ok(());
    }

    let mode = matches.get_one::<String>("mode").unwrap().as_str();
    let steps = *matches.get_one::<usize>("steps").unwrap();
    let dt = *matches.get_one::<f64>("dt").unwrap();
    let g = *matches.get_one::<f64>("G").unwrap();
    let softening = *matches.get_one::<f64>("softening").unwrap();
    let mut output = matches.get_one::<String>("output").unwrap().to_owned();
    let quiet = matches.get_flag("quiet");
    let write_every = *matches.get_one::<usize>("write_every").unwrap();

    let bodies: Vec<nbody::Body> = if let Some(bodies_json) = matches.get_one::<String>("bodies") {
        parse_bodies(bodies_json)?
    } else {
        let n = *matches.get_one::<usize>("random").unwrap();
        if n == 0 { anyhow::bail!("Provide either --bodies JSON or --random N > 0"); }
        let seed = *matches.get_one::<u64>("seed").unwrap();
        let mr = matches.get_many::<f64>("mass_range").unwrap().cloned().collect::<Vec<_>>();
        let pr = matches.get_many::<f64>("pos_range").unwrap().cloned().collect::<Vec<_>>();
        let vr = matches.get_many::<f64>("vel_range").unwrap().cloned().collect::<Vec<_>>();
        random_bodies(n, (mr[0], mr[1]), (pr[0], pr[1]), (vr[0], vr[1]), seed)
    };

    if output == "output/nbody_rust_seq.csv" && mode == "threads" {
        output = "output/nbody_rust_threads.csv".to_string();
    }

    let mut bodies_mut = bodies.clone();
    let start = Instant::now();
    // Velocity Verlet integrator for better energy behavior
    // 1) a(t) from current positions
    let mut acc_prev: Vec<(f64,f64,f64)> = match mode {
        "seq" => compute_acc(&bodies_mut, g, softening),
        "threads" => compute_acc_par(&bodies_mut, g, softening),
        _ => unreachable!(),
    };
    if write_every != 0 { write_state_csv(&output, 0, &bodies_mut, true)?; }
    for it in 1..=steps {
        // 2) update positions using x += v*dt + 0.5*a*dt^2
        for (i, b) in bodies_mut.iter_mut().enumerate() {
            let (ax, ay, az) = acc_prev[i];
            b.x += b.vx * dt + 0.5 * ax * dt * dt;
            b.y += b.vy * dt + 0.5 * ay * dt * dt;
            b.z += b.vz * dt + 0.5 * az * dt * dt;
        }
        // 3) compute a(t+dt) from new positions
        let acc_new: Vec<(f64,f64,f64)> = match mode {
            "seq" => compute_acc(&bodies_mut, g, softening),
            "threads" => compute_acc_par(&bodies_mut, g, softening),
            _ => unreachable!(),
        };
        // 4) update velocities using v += 0.5*(a_old + a_new)*dt
        for (i, b) in bodies_mut.iter_mut().enumerate() {
            let (ax_prev, ay_prev, az_prev) = acc_prev[i];
            let (ax_new, ay_new, az_new) = acc_new[i];
            b.vx += 0.5 * (ax_prev + ax_new) * dt;
            b.vy += 0.5 * (ay_prev + ay_new) * dt;
            b.vz += 0.5 * (az_prev + az_new) * dt;
        }
        acc_prev = acc_new;
        if write_every != 0 && (it % write_every == 0 || it == steps) {
            write_state_csv(&output, it, &bodies_mut, false)?;
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    if !quiet {
        println!(
            "Mode={} Bodies={} Steps={} dt={} ElapsedSeconds={:.6}",
            mode,
            bodies_mut.len(),
            steps,
            dt,
            elapsed
        );
    }

    // If visualization requested, run it on finished CSV
    if let Some(csv_to_vis) = matches.get_one::<String>("visualize") {
        visualize_csv(csv_to_vis, vis_size, vis_trails, gif_ms, vis_bounds_mode, vis_pad)?;
    }
    Ok(())
}
