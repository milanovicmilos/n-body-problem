use clap::{Arg, ArgAction, Command};
use std::time::Instant;
use nbody::{compute_acc, compute_acc_par, parse_bodies, random_bodies, step_euler, write_state_csv};

fn main() -> anyhow::Result<()> {
    let matches = Command::new("nbody")
        .about("N-body simulation in Rust")
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
    Ok(())
}
