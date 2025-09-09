mod types;
mod force_naive;
mod integrators;
mod io;
mod sim_runner;
mod cli;
mod metrics;

use cli::run_cli;

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
