use crate::types::*;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use csv::Writer;
use serde_json;

/// Create output directory if it doesn't exist
pub fn create_output_directory(output_dir: &str) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    Ok(())
}

/// Save current system state to CSV file
pub fn save_system_state(system: &NBodySystem, iteration: usize, output_dir: &str) -> io::Result<()> {
    let filename = format!("states_iter_{:06}.csv", iteration);
    let filepath = Path::new(output_dir).join(filename);
    
    let mut wtr = Writer::from_path(filepath)?;
    
    // Write header
    wtr.write_record(&["iter", "id", "x", "y", "z", "vx", "vy", "vz", "m"])?;
    
    // Write data
    for i in 0..system.n {
        wtr.write_record(&[
            iteration.to_string(),
            i.to_string(),
            system.x[i].to_string(),
            system.y[i].to_string(),
            system.z[i].to_string(),
            system.vx[i].to_string(),
            system.vy[i].to_string(),
            system.vz[i].to_string(),
            system.m[i].to_string(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Append energy data to CSV file
pub fn save_energy_data(iteration: usize, kinetic: f64, potential: f64, output_dir: &str) -> io::Result<()> {
    let filepath = Path::new(output_dir).join("energy.csv");
    
    // Check if file exists to determine if we need to write header
    let write_header = !filepath.exists();
    
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)?;
    
    let mut wtr = Writer::from_writer(file);
    
    if write_header {
        wtr.write_record(&["iter", "kinetic", "potential", "total"])?;
    }
    
    let total = kinetic + potential;
    wtr.write_record(&[
        iteration.to_string(),
        kinetic.to_string(),
        potential.to_string(),
        total.to_string(),
    ])?;
    
    wtr.flush()?;
    Ok(())
}

/// Save simulation metadata to JSON file
pub fn save_metadata(params: &SimulationParams, output_dir: &str, execution_time: Option<f64>) -> io::Result<()> {
    let filepath = Path::new(output_dir).join("run_meta.json");
    
    let system_info = SystemInfo::new();
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    let metadata = SimulationMetadata {
        parameters: params.clone(),
        system_info,
        execution_time_seconds: execution_time,
        timestamp,
    };
    
    let json_data = serde_json::to_string_pretty(&metadata)?;
    let mut file = File::create(filepath)?;
    file.write_all(json_data.as_bytes())?;
    file.flush()?;
    
    Ok(())
}

/// Save initial conditions to CSV file for reproducibility
pub fn save_initial_conditions(system: &NBodySystem, output_dir: &str) -> io::Result<()> {
    let filepath = Path::new(output_dir).join("initial_conditions.csv");
    let mut wtr = Writer::from_path(filepath)?;
    
    // Write header
    wtr.write_record(&["id", "x", "y", "z", "vx", "vy", "vz", "m"])?;
    
    // Write data
    for i in 0..system.n {
        wtr.write_record(&[
            i.to_string(),
            system.x[i].to_string(),
            system.y[i].to_string(),
            system.z[i].to_string(),
            system.vx[i].to_string(),
            system.vy[i].to_string(),
            system.vz[i].to_string(),
            system.m[i].to_string(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Load initial conditions from CSV file
pub fn load_initial_conditions(filepath: &str) -> io::Result<NBodySystem> {
    let mut rdr = csv::Reader::from_path(filepath)?;
    let mut records = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        records.push(record);
    }
    
    let n = records.len();
    let mut system = NBodySystem::new(n);
    
    for (i, record) in records.iter().enumerate() {
        // Parse CSV: id,x,y,z,vx,vy,vz,m
        system.x[i] = record[1].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        system.y[i] = record[2].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        system.z[i] = record[3].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        system.vx[i] = record[4].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        system.vy[i] = record[5].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        system.vz[i] = record[6].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        system.m[i] = record[7].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    }
    
    Ok(system)
}

/// Load simulation parameters from TOML file
pub fn load_params_from_toml(filepath: &str) -> io::Result<SimulationParams> {
    let content = fs::read_to_string(filepath)?;
    let params: SimulationParams = toml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(params)
}

/// Save simulation parameters to TOML file
pub fn save_params_to_toml(params: &SimulationParams, filepath: &str) -> io::Result<()> {
    let toml_data = toml::to_string_pretty(params)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut file = File::create(filepath)?;
    file.write_all(toml_data.as_bytes())?;
    file.flush()?;
    Ok(())
}
