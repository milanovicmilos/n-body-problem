use crate::types::*;
use crate::io::*;
use crate::integrators::*;
use crate::force_naive::compute_accelerations_naive_sequential;
use std::time::Instant;

/// Run sequential N-body simulation
pub fn run_sequential_simulation(params: &SimulationParams) -> Result<f64, Box<dyn std::error::Error>> {
    println!("Starting sequential simulation:");
    println!("  N bodies: {}", params.n);
    println!("  Steps: {}", params.steps);
    println!("  Time step: {}", params.dt);
    println!("  Softening: {}", params.eps);
    println!("  Output: {}", params.output_dir);
    
    // Create output directory
    create_output_directory(&params.output_dir)?;
    
    // Initialize system
    println!("Initializing system...");
    let mut system = NBodySystem::new(params.n);
    system.initialize_plummer_sphere(params.seed);
    
    // Save initial conditions
    save_initial_conditions(&system, &params.output_dir)?;
    
    // Compute initial accelerations
    compute_accelerations_naive_sequential(&mut system, params.eps, params.g);
    
    // Save initial state and energy
    save_system_state(&system, 0, &params.output_dir)?;
    let kinetic = system.compute_kinetic_energy();
    let potential = system.compute_potential_energy(params.eps);
    save_energy_data(0, kinetic, potential, &params.output_dir)?;
    
    println!("Initial energy: K={:.6}, U={:.6}, Total={:.6}", 
             kinetic, potential, kinetic + potential);
    
    // Start simulation timer
    let start_time = Instant::now();
    
    println!("Running simulation...");
    for step in 1..=params.steps {
        // Perform integration step based on integrator type
        match params.integrator.as_str() {
            "verlet" | "leapfrog" => {
                velocity_verlet_step_sequential(&mut system, params.dt, params.eps, params.g);
            }
            "euler" => {
                euler_step_sequential(&mut system, params.dt, params.eps, params.g);
            }
            "rk4" => {
                rk4_step_sequential(&mut system, params.dt, params.eps, params.g);
            }
            _ => {
                return Err(format!("Unknown integrator: {}", params.integrator).into());
            }
        }
        
        // Save state and energy if needed
        if step % params.dump_every == 0 {
            save_system_state(&system, step, &params.output_dir)?;
            let kinetic = system.compute_kinetic_energy();
            let potential = system.compute_potential_energy(params.eps);
            save_energy_data(step, kinetic, potential, &params.output_dir)?;
            
            if step % (params.dump_every * 10) == 0 {
                let total_energy = kinetic + potential;
                println!("Step {:6}: K={:.6}, U={:.6}, Total={:.6}", 
                         step, kinetic, potential, total_energy);
            }
        }
    }
    
    // End simulation timer
    let execution_time = start_time.elapsed().as_secs_f64();
    
    // Final energy check
    let final_kinetic = system.compute_kinetic_energy();
    let final_potential = system.compute_potential_energy(params.eps);
    let final_total = final_kinetic + final_potential;
    
    println!("Simulation completed in {:.2} seconds", execution_time);
    println!("Final energy: K={:.6}, U={:.6}, Total={:.6}", 
             final_kinetic, final_potential, final_total);
    
    // Save metadata
    save_metadata(params, &params.output_dir, Some(execution_time))?;
    
    Ok(execution_time)
}

/// Run parallel N-body simulation using threads
pub fn run_parallel_simulation(params: &SimulationParams) -> Result<f64, Box<dyn std::error::Error>> {
    let num_threads = params.threads.unwrap_or_else(|| num_cpus::get());
    
    println!("Starting parallel simulation:");
    println!("  N bodies: {}", params.n);
    println!("  Steps: {}", params.steps);
    println!("  Time step: {}", params.dt);
    println!("  Softening: {}", params.eps);
    println!("  Threads: {}", num_threads);
    println!("  Output: {}", params.output_dir);
    
    // Create output directory
    create_output_directory(&params.output_dir)?;
    
    // Initialize system
    println!("Initializing system...");
    let mut system = NBodySystem::new(params.n);
    system.initialize_plummer_sphere(params.seed);
    
    // Save initial conditions
    save_initial_conditions(&system, &params.output_dir)?;
    
    // Compute initial accelerations
    crate::force_naive::compute_accelerations_naive_parallel_chunked(&mut system, params.eps, params.g, num_threads);
    
    // Save initial state and energy
    save_system_state(&system, 0, &params.output_dir)?;
    let kinetic = system.compute_kinetic_energy();
    let potential = system.compute_potential_energy(params.eps);
    save_energy_data(0, kinetic, potential, &params.output_dir)?;
    
    println!("Initial energy: K={:.6}, U={:.6}, Total={:.6}", 
             kinetic, potential, kinetic + potential);
    
    // Start simulation timer
    let start_time = Instant::now();
    
    println!("Running simulation...");
    for step in 1..=params.steps {
        // Perform integration step based on integrator type
        match params.integrator.as_str() {
            "verlet" | "leapfrog" => {
                velocity_verlet_step_parallel_safe(&mut system, params.dt, params.eps, params.g, num_threads);
            }
            "euler" => {
                // For now, use sequential for other integrators
                euler_step_sequential(&mut system, params.dt, params.eps, params.g);
            }
            "rk4" => {
                rk4_step_sequential(&mut system, params.dt, params.eps, params.g);
            }
            _ => {
                return Err(format!("Unknown integrator: {}", params.integrator).into());
            }
        }
        
        // Save state and energy if needed
        if step % params.dump_every == 0 {
            save_system_state(&system, step, &params.output_dir)?;
            let kinetic = system.compute_kinetic_energy();
            let potential = system.compute_potential_energy(params.eps);
            save_energy_data(step, kinetic, potential, &params.output_dir)?;
            
            if step % (params.dump_every * 10) == 0 {
                let total_energy = kinetic + potential;
                println!("Step {:6}: K={:.6}, U={:.6}, Total={:.6}", 
                         step, kinetic, potential, total_energy);
            }
        }
    }
    
    // End simulation timer
    let execution_time = start_time.elapsed().as_secs_f64();
    
    // Final energy check
    let final_kinetic = system.compute_kinetic_energy();
    let final_potential = system.compute_potential_energy(params.eps);
    let final_total = final_kinetic + final_potential;
    
    println!("Simulation completed in {:.2} seconds", execution_time);
    println!("Final energy: K={:.6}, U={:.6}, Total={:.6}", 
             final_kinetic, final_potential, final_total);
    
    // Save metadata with thread info
    let mut params_with_threads = params.clone();
    params_with_threads.threads = Some(num_threads);
    save_metadata(&params_with_threads, &params.output_dir, Some(execution_time))?;
    
    Ok(execution_time)
}
