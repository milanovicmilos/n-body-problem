use clap::{Arg, ArgAction, Command};
use std::time::Instant;
use std::path::Path;
use nbody::{compute_acc, compute_acc_par, parse_bodies, random_bodies, step_euler, write_state_csv};

// Visualization helpers
fn visualize_csv<P: AsRef<std::path::Path>>(csv_path: P) -> anyhow::Result<()> {
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
    let mut frames: std::collections::BTreeMap<usize, Vec<(f64,f64,f64)>> = std::collections::BTreeMap::new();
    for result in rdr.records() {
        let rec = result?;
        // iteration,id,m,x,y,z,vx,vy,vz
        let it: usize = rec.get(0).unwrap().parse()?;
        let x: f64 = rec.get(3).unwrap().parse()?;
        let y: f64 = rec.get(4).unwrap().parse()?;
        let m: f64 = rec.get(2).unwrap().parse()?;
        frames.entry(it).or_default().push((x,y,m));
    }

    // Determine bounds
    let mut xmin = std::f64::INFINITY; let mut xmax = std::f64::NEG_INFINITY;
    let mut ymin = std::f64::INFINITY; let mut ymax = std::f64::NEG_INFINITY;
    for (_it, bodies) in &frames {
        for (x,y,_m) in bodies {
            if *x < xmin { xmin = *x; }
            if *x > xmax { xmax = *x; }
            if *y < ymin { ymin = *y; }
            if *y > ymax { ymax = *y; }
        }
    }
    if xmin == std::f64::INFINITY { anyhow::bail!("No data found in CSV"); }
    // add small padding
    let dx = (xmax - xmin).abs().max(1.0) * 0.05;
    let dy = (ymax - ymin).abs().max(1.0) * 0.05;
    xmin -= dx; xmax += dx; ymin -= dy; ymax += dy;

    // Create PNG frames using plotters
    let mut png_paths = Vec::new();
    let size = (800, 800);
    for (it, bodies) in &frames {
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
            for (x,y,m) in bodies {
                let radius_f = (m.sqrt() * 0.02).max(1.0);
                let radius = radius_f as u32;
                chart.draw_series(std::iter::once(Circle::new((*x,*y), radius, RED.filled())))?;
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
        let delay = Delay::from_numer_denom_ms(100, 1); // 100 ms per frame
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
        .arg(Arg::new("mass_range").long("mass-range").num_args(2).value_parser(clap::value_parser!(f64)).default_values(["1.0", "10.0"]))
        .arg(Arg::new("pos_range").long("pos-range").num_args(2).value_parser(clap::value_parser!(f64)).default_values(["-1.0", "1.0"]))
        .arg(Arg::new("vel_range").long("vel-range").num_args(2).value_parser(clap::value_parser!(f64)).default_values(["-0.1", "0.1"]))
        .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
        .get_matches();

    // If user asked only for visualization, run that and exit early
    if let Some(csv_to_vis) = matches.get_one::<String>("visualize") {
        visualize_csv(csv_to_vis)?;
        return Ok(());
    }

    let mode = matches.get_one::<String>("mode").unwrap().as_str();
    let steps = *matches.get_one::<usize>("steps").unwrap();
    let dt = *matches.get_one::<f64>("dt").unwrap();
    let g = *matches.get_one::<f64>("G").unwrap();
    let softening = *matches.get_one::<f64>("softening").unwrap();
    let mut output = matches.get_one::<String>("output").unwrap().to_owned();
    let quiet = matches.get_flag("quiet");

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
    write_state_csv(&output, 0, &bodies_mut, true)?;
    for it in 1..=steps {
        let acc = match mode {
            "seq" => compute_acc(&bodies_mut, g, softening),
            "threads" => compute_acc_par(&bodies_mut, g, softening),
            _ => unreachable!(),
        };
        step_euler(&mut bodies_mut, &acc, dt);
        write_state_csv(&output, it, &bodies_mut, false)?;
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
        visualize_csv(csv_to_vis)?;
    }
    Ok(())
}
