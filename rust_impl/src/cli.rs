use clap::{Parser, Subcommand};
use crate::types::SimulationParams;
use crate::sim_runner::{run_sequential_simulation, run_parallel_simulation};
use crate::metrics::analyze_energy_conservation;
use std::path::Path;

#[derive(Parser)]
#[command(name = "nbody-rs")]
#[command(about = "N-body simulation in Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run sequential simulation
    RunSeq {
        /// Number of bodies
        #[arg(short, long)]
        n: usize,
        
        /// Number of simulation steps
        #[arg(short, long)]
        steps: usize,
        
        /// Time step size
        #[arg(long)]
        dt: f64,
        
        /// Softening parameter
        #[arg(long, default_value = "0.01")]
        eps: f64,
        
        /// Gravitational constant
        #[arg(short = 'G', long, default_value = "1.0")]
        g: f64,
        
        /// Random seed for reproducibility
        #[arg(long, default_value = "42")]
        seed: u64,
        
        /// Save state every N steps
        #[arg(long, default_value = "100")]
        dump_every: usize,
        
        /// Integration algorithm
        #[arg(long, default_value = "verlet")]
        integrator: String,
        
        /// Force calculation algorithm
        #[arg(long, default_value = "naive")]
        algo: String,
        
        /// Output directory
        #[arg(short, long)]
        out: String,
    },
    
    /// Run parallel simulation with threads
    RunPar {
        /// Number of bodies
        #[arg(short, long)]
        n: usize,
        
        /// Number of simulation steps
        #[arg(short, long)]
        steps: usize,
        
        /// Time step size
        #[arg(long)]
        dt: f64,
        
        /// Softening parameter
        #[arg(long, default_value = "0.01")]
        eps: f64,
        
        /// Gravitational constant
        #[arg(short = 'G', long, default_value = "1.0")]
        g: f64,
        
        /// Random seed for reproducibility
        #[arg(long, default_value = "42")]
        seed: u64,
        
        /// Save state every N steps
        #[arg(long, default_value = "100")]
        dump_every: usize,
        
        /// Integration algorithm
        #[arg(long, default_value = "verlet")]
        integrator: String,
        
        /// Force calculation algorithm
        #[arg(long, default_value = "naive")]
        algo: String,
        
        /// Number of threads
        #[arg(short, long)]
        threads: Option<usize>,
        
        /// Output directory
        #[arg(short, long)]
        out: String,
    },
    
    /// Analyze simulation results
    Analyze {
        /// Simulation output directory
        #[arg(short, long)]
        dir: String,
    },
}

pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::RunSeq { 
            n, steps, dt, eps, g, seed, dump_every, integrator, algo, out 
        } => {
            let params = SimulationParams {
                n,
                steps,
                dt,
                eps,
                g,
                seed,
                dump_every,
                output_dir: out,
                threads: None,
                algorithm: algo,
                integrator,
            };
            
            match run_sequential_simulation(&params) {
                Ok(execution_time) => {
                    println!("\nSequential simulation completed successfully!");
                    println!("Execution time: {:.2} seconds", execution_time);
                    println!("Results saved to: {}", params.output_dir);
                }
                Err(e) => {
                    eprintln!("Error running sequential simulation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::RunPar { 
            n, steps, dt, eps, g, seed, dump_every, integrator, algo, threads, out 
        } => {
            let params = SimulationParams {
                n,
                steps,
                dt,
                eps,
                g,
                seed,
                dump_every,
                output_dir: out,
                threads,
                algorithm: algo,
                integrator,
            };
            
            match run_parallel_simulation(&params) {
                Ok(execution_time) => {
                    println!("\nParallel simulation completed successfully!");
                    println!("Execution time: {:.2} seconds", execution_time);
                    println!("Threads used: {}", threads.unwrap_or_else(|| num_cpus::get()));
                    println!("Results saved to: {}", params.output_dir);
                }
                Err(e) => {
                    eprintln!("Error running parallel simulation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Analyze { dir } => {
            let energy_file = Path::new(&dir).join("energy.csv");
            let metadata_file = Path::new(&dir).join("run_meta.json");
            
            if energy_file.exists() {
                match analyze_energy_conservation(energy_file.to_str().unwrap()) {
                    Ok(analysis) => {
                        println!("Energy Conservation Analysis:");
                        println!("  Initial energy: {:.6}", analysis.initial_energy);
                        println!("  Final energy: {:.6}", analysis.final_energy);
                        println!("  Energy drift: {:.6}", analysis.energy_drift);
                        println!("  Relative drift: {:.4}%", analysis.relative_drift_percent);
                        println!("  Max deviation: {:.4}%", analysis.max_relative_deviation_percent);
                    }
                    Err(e) => {
                        eprintln!("Error analyzing energy: {}", e);
                    }
                }
            } else {
                println!("Energy file not found: {}", energy_file.display());
            }
            
            if metadata_file.exists() {
                match crate::metrics::analyze_performance(metadata_file.to_str().unwrap()) {
                    Ok(perf) => {
                        println!("\nPerformance Analysis:");
                        println!("  Execution time: {:.2} seconds", perf.execution_time_seconds);
                        println!("  Time per step: {:.6} seconds", perf.time_per_step);
                        println!("  Operations per second: {:.0}", perf.operations_per_second);
                    }
                    Err(e) => {
                        eprintln!("Error analyzing performance: {}", e);
                    }
                }
            } else {
                println!("Metadata file not found: {}", metadata_file.display());
            }
        }
    }
    
    Ok(())
}
