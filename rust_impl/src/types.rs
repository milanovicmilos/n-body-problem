use serde::{Deserialize, Serialize};

/// Structure of Arrays (SoA) representation for N-body system
#[derive(Clone, Debug)]
pub struct NBodySystem {
    pub n: usize,
    
    // Positions
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    
    // Velocities
    pub vx: Vec<f64>,
    pub vy: Vec<f64>,
    pub vz: Vec<f64>,
    
    // Masses
    pub m: Vec<f64>,
    
    // Accelerations (computed)
    pub ax: Vec<f64>,
    pub ay: Vec<f64>,
    pub az: Vec<f64>,
}

impl NBodySystem {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            x: vec![0.0; n],
            y: vec![0.0; n],
            z: vec![0.0; n],
            vx: vec![0.0; n],
            vy: vec![0.0; n],
            vz: vec![0.0; n],
            m: vec![0.0; n],
            ax: vec![0.0; n],
            ay: vec![0.0; n],
            az: vec![0.0; n],
        }
    }
    
    /// Initialize system with Plummer sphere distribution
    pub fn initialize_plummer_sphere(&mut self, seed: u64) {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;
        
        let mut rng = StdRng::seed_from_u64(seed);
        
        let total_mass = 1.0;
        let plummer_radius = 1.0;
        
        for i in 0..self.n {
            // Mass: equal for all bodies
            self.m[i] = total_mass / self.n as f64;
            
            // Position: simplified Plummer sphere sampling
            let r = rng.gen::<f64>() * 10.0 * plummer_radius;
            let theta = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
            let phi = rng.gen::<f64>() * std::f64::consts::PI;
            
            self.x[i] = r * phi.sin() * theta.cos();
            self.y[i] = r * phi.sin() * theta.sin();
            self.z[i] = r * phi.cos();
            
            // Velocity: simple random initialization
            self.vx[i] = rng.gen_range(-0.1..0.1);
            self.vy[i] = rng.gen_range(-0.1..0.1);
            self.vz[i] = rng.gen_range(-0.1..0.1);
        }
    }
    
    /// Compute total kinetic energy
    pub fn compute_kinetic_energy(&self) -> f64 {
        let mut kinetic = 0.0;
        for i in 0..self.n {
            let v2 = self.vx[i] * self.vx[i] + self.vy[i] * self.vy[i] + self.vz[i] * self.vz[i];
            kinetic += 0.5 * self.m[i] * v2;
        }
        kinetic
    }
    
    /// Compute total gravitational potential energy
    pub fn compute_potential_energy(&self, eps: f64) -> f64 {
        let g = 1.0; // Gravitational constant
        let mut potential = 0.0;
        
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let dx = self.x[j] - self.x[i];
                let dy = self.y[j] - self.y[i];
                let dz = self.z[j] - self.z[i];
                let r2 = dx * dx + dy * dy + dz * dz + eps * eps;
                let r = r2.sqrt();
                potential -= g * self.m[i] * self.m[j] / r;
            }
        }
        potential
    }
    
    /// Compute center of mass
    pub fn compute_center_of_mass(&self) -> (f64, f64, f64) {
        let total_mass: f64 = self.m.iter().sum();
        let cm_x = self.x.iter().zip(&self.m).map(|(x, m)| x * m).sum::<f64>() / total_mass;
        let cm_y = self.y.iter().zip(&self.m).map(|(y, m)| y * m).sum::<f64>() / total_mass;
        let cm_z = self.z.iter().zip(&self.m).map(|(z, m)| z * m).sum::<f64>() / total_mass;
        (cm_x, cm_y, cm_z)
    }
    
    /// Compute total momentum
    pub fn compute_total_momentum(&self) -> (f64, f64, f64) {
        let px = self.vx.iter().zip(&self.m).map(|(vx, m)| vx * m).sum();
        let py = self.vy.iter().zip(&self.m).map(|(vy, m)| vy * m).sum();
        let pz = self.vz.iter().zip(&self.m).map(|(vz, m)| vz * m).sum();
        (px, py, pz)
    }
}

/// Simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    pub n: usize,
    pub steps: usize,
    pub dt: f64,
    pub eps: f64,
    pub g: f64,
    pub seed: u64,
    pub dump_every: usize,
    pub output_dir: String,
    pub threads: Option<usize>,
    pub algorithm: String,
    pub integrator: String,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            n: 1000,
            steps: 1000,
            dt: 0.001,
            eps: 0.01,
            g: 1.0,
            seed: 42,
            dump_every: 100,
            output_dir: "data/outputs/rust_default".to_string(),
            threads: None,
            algorithm: "naive".to_string(),
            integrator: "verlet".to_string(),
        }
    }
}

/// Energy data for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyData {
    pub iter: usize,
    pub kinetic: f64,
    pub potential: f64,
    pub total: f64,
}

/// Particle state for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleState {
    pub iter: usize,
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub m: f64,
}

/// System metadata for reproducibility
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationMetadata {
    pub parameters: SimulationParams,
    pub system_info: SystemInfo,
    pub execution_time_seconds: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub cpu_count: usize,
    pub rust_version: String,
    pub nbody_version: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self {
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
            os: std::env::consts::OS.to_string(),
            cpu_count: num_cpus::get(),
            rust_version: "unknown".to_string(),
            nbody_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

// Add these dependencies to Cargo.toml
// gethostname = "0.4"
// num_cpus = "1.16"
