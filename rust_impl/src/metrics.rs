use std::fs;
use std::error::Error;
use csv::Reader;
use serde_json;
use crate::types::SimulationMetadata;

#[derive(Debug)]
pub struct EnergyAnalysis {
    pub initial_energy: f64,
    pub final_energy: f64,
    pub energy_drift: f64,
    pub relative_drift_percent: f64,
    pub energy_variance: f64,
    pub energy_std: f64,
    pub max_deviation: f64,
    pub max_relative_deviation_percent: f64,
}

#[derive(Debug)]
pub struct PerformanceAnalysis {
    pub execution_time_seconds: f64,
    pub total_operations: u64,
    pub operations_per_second: f64,
    pub time_per_step: f64,
    pub time_per_particle_per_step: f64,
    pub n_bodies: usize,
    pub n_steps: usize,
}

/// Analyze energy conservation from energy CSV file
pub fn analyze_energy_conservation(energy_file: &str) -> Result<EnergyAnalysis, Box<dyn Error>> {
    let mut rdr = Reader::from_path(energy_file)?;
    let mut energies = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        let total_energy: f64 = record[3].parse()?; // total energy is column 3
        energies.push(total_energy);
    }
    
    if energies.is_empty() {
        return Err("No energy data found".into());
    }
    
    let initial_energy = energies[0];
    let final_energy = energies[energies.len() - 1];
    
    // Energy drift
    let energy_drift = final_energy - initial_energy;
    let relative_drift = if initial_energy != 0.0 {
        (energy_drift.abs() / initial_energy.abs()) * 100.0
    } else {
        0.0
    };
    
    // Energy variance and standard deviation
    let mean_energy = energies.iter().sum::<f64>() / energies.len() as f64;
    let energy_variance = energies.iter()
        .map(|e| (e - mean_energy).powi(2))
        .sum::<f64>() / energies.len() as f64;
    let energy_std = energy_variance.sqrt();
    
    // Maximum deviation
    let max_deviation = energies.iter()
        .map(|e| (e - initial_energy).abs())
        .fold(0.0, f64::max);
    let max_relative_deviation = if initial_energy != 0.0 {
        (max_deviation / initial_energy.abs()) * 100.0
    } else {
        0.0
    };
    
    Ok(EnergyAnalysis {
        initial_energy,
        final_energy,
        energy_drift,
        relative_drift_percent: relative_drift,
        energy_variance,
        energy_std,
        max_deviation,
        max_relative_deviation_percent: max_relative_deviation,
    })
}

/// Compare two simulation results
pub fn compare_simulations(energy_file1: &str, energy_file2: &str) -> Result<ComparisonResult, Box<dyn Error>> {
    let mut rdr1 = Reader::from_path(energy_file1)?;
    let mut rdr2 = Reader::from_path(energy_file2)?;
    
    let mut energies1 = Vec::new();
    let mut energies2 = Vec::new();
    
    for result in rdr1.records() {
        let record = result?;
        let kinetic: f64 = record[1].parse()?;
        let potential: f64 = record[2].parse()?;
        let total: f64 = record[3].parse()?;
        energies1.push((kinetic, potential, total));
    }
    
    for result in rdr2.records() {
        let record = result?;
        let kinetic: f64 = record[1].parse()?;
        let potential: f64 = record[2].parse()?;
        let total: f64 = record[3].parse()?;
        energies2.push((kinetic, potential, total));
    }
    
    let min_len = energies1.len().min(energies2.len());
    
    let mut total_diff_sq = 0.0;
    let mut kinetic_diff_sq = 0.0;
    let mut potential_diff_sq = 0.0;
    let mut max_total_diff: f64 = 0.0;
    let mut total_diff_sum = 0.0;
    
    for i in 0..min_len {
        let total_diff = energies1[i].2 - energies2[i].2;
        let kinetic_diff = energies1[i].0 - energies2[i].0;
        let potential_diff = energies1[i].1 - energies2[i].1;
        
        total_diff_sq += total_diff * total_diff;
        kinetic_diff_sq += kinetic_diff * kinetic_diff;
        potential_diff_sq += potential_diff * potential_diff;
        
        max_total_diff = max_total_diff.max(total_diff.abs());
        total_diff_sum += total_diff;
    }
    
    let total_energy_rmse = (total_diff_sq / min_len as f64).sqrt();
    let kinetic_energy_rmse = (kinetic_diff_sq / min_len as f64).sqrt();
    let potential_energy_rmse = (potential_diff_sq / min_len as f64).sqrt();
    let mean_total_diff = total_diff_sum / min_len as f64;
    
    // Calculate standard deviation of differences
    let mut diff_var = 0.0;
    for i in 0..min_len {
        let total_diff = energies1[i].2 - energies2[i].2;
        diff_var += (total_diff - mean_total_diff).powi(2);
    }
    let std_total_diff = (diff_var / min_len as f64).sqrt();
    
    Ok(ComparisonResult {
        total_energy_rmse,
        kinetic_energy_rmse,
        potential_energy_rmse,
        max_total_diff,
        mean_total_diff,
        std_total_diff,
    })
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub total_energy_rmse: f64,
    pub kinetic_energy_rmse: f64,
    pub potential_energy_rmse: f64,
    pub max_total_diff: f64,
    pub mean_total_diff: f64,
    pub std_total_diff: f64,
}

/// Extract performance metrics from metadata file
pub fn analyze_performance(metadata_file: &str) -> Result<PerformanceAnalysis, Box<dyn Error>> {
    let content = fs::read_to_string(metadata_file)?;
    let metadata: SimulationMetadata = serde_json::from_str(&content)?;
    
    let execution_time = metadata.execution_time_seconds.unwrap_or(0.0);
    let n = metadata.parameters.n;
    let steps = metadata.parameters.steps;
    
    // Calculate performance metrics
    let total_operations = (n * (n - 1) * steps) as u64; // O(N^2) per step
    let operations_per_second = if execution_time > 0.0 {
        total_operations as f64 / execution_time
    } else {
        0.0
    };
    let time_per_step = if steps > 0 {
        execution_time / steps as f64
    } else {
        0.0
    };
    let time_per_particle_per_step = if n > 0 {
        time_per_step / n as f64
    } else {
        0.0
    };
    
    Ok(PerformanceAnalysis {
        execution_time_seconds: execution_time,
        total_operations,
        operations_per_second,
        time_per_step,
        time_per_particle_per_step,
        n_bodies: n,
        n_steps: steps,
    })
}

/// Validate two-body orbital mechanics (for testing)
pub fn validate_two_body_orbit(states_dir: &str, body1_id: usize, body2_id: usize) -> Result<OrbitValidation, Box<dyn Error>> {
    use std::path::Path;
    use glob::glob;
    
    let pattern = Path::new(states_dir).join("states_iter_*.csv").to_string_lossy().to_string();
    let state_files: Vec<_> = glob(&pattern)?.collect();
    
    let mut distances = Vec::new();
    
    for state_file_result in state_files {
        let state_file = state_file_result?;
        let mut rdr = Reader::from_path(state_file)?;
        
        let mut body1_pos = None;
        let mut body2_pos = None;
        
        for result in rdr.records() {
            let record = result?;
            let id: usize = record[1].parse()?;
            
            if id == body1_id {
                let x: f64 = record[2].parse()?;
                let y: f64 = record[3].parse()?;
                let z: f64 = record[4].parse()?;
                body1_pos = Some((x, y, z));
            } else if id == body2_id {
                let x: f64 = record[2].parse()?;
                let y: f64 = record[3].parse()?;
                let z: f64 = record[4].parse()?;
                body2_pos = Some((x, y, z));
            }
        }
        
        if let (Some(pos1), Some(pos2)) = (body1_pos, body2_pos) {
            let dx = pos2.0 - pos1.0;
            let dy = pos2.1 - pos1.1;
            let dz = pos2.2 - pos1.2;
            let distance = (dx*dx + dy*dy + dz*dz).sqrt();
            distances.push(distance);
        }
    }
    
    if distances.is_empty() {
        return Err("No valid two-body data found".into());
    }
    
    let mean_distance = distances.iter().sum::<f64>() / distances.len() as f64;
    let distance_variance = distances.iter()
        .map(|d| (d - mean_distance).powi(2))
        .sum::<f64>() / distances.len() as f64;
    let max_distance: f64 = distances.iter().fold(0.0, |a, &b| a.max(b));
    let min_distance: f64 = distances.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    
    // Eccentricity estimate (simplified)
    let eccentricity = (max_distance - min_distance) / (max_distance + min_distance);
    
    Ok(OrbitValidation {
        mean_separation: mean_distance,
        distance_variance,
        max_separation: max_distance,
        min_separation: min_distance,
        estimated_eccentricity: eccentricity,
        separation_stability: distance_variance / (mean_distance * mean_distance),
    })
}

#[derive(Debug)]
pub struct OrbitValidation {
    pub mean_separation: f64,
    pub distance_variance: f64,
    pub max_separation: f64,
    pub min_separation: f64,
    pub estimated_eccentricity: f64,
    pub separation_stability: f64,
}
