use crate::types::NBodySystem;

/// Compute gravitational accelerations using naive O(N^2) algorithm - sequential version
pub fn compute_accelerations_naive_sequential(system: &mut NBodySystem, eps: f64, g: f64) {
    // Reset accelerations
    for i in 0..system.n {
        system.ax[i] = 0.0;
        system.ay[i] = 0.0;
        system.az[i] = 0.0;
    }
    
    // Compute pairwise forces
    for i in 0..system.n {
        for j in 0..system.n {
            if i == j {
                continue;
            }
            
            // Distance vector
            let dx = system.x[j] - system.x[i];
            let dy = system.y[j] - system.y[i];
            let dz = system.z[j] - system.z[i];
            
            // Distance squared with softening
            let r2 = dx * dx + dy * dy + dz * dz + eps * eps;
            let r = r2.sqrt();
            
            // Force magnitude factor
            let inv_r3 = g / (r2 * r);
            let force_factor = system.m[j] * inv_r3;
            
            // Accumulate accelerations
            system.ax[i] += dx * force_factor;
            system.ay[i] += dy * force_factor;
            system.az[i] += dz * force_factor;
        }
    }
}

/// Compute gravitational accelerations using naive O(N^2) algorithm - parallel version
pub fn compute_accelerations_naive_parallel(system: &mut NBodySystem, eps: f64, g: f64, num_threads: usize) {
    use rayon::prelude::*;
    use std::sync::{Arc, Mutex};
    
    // Reset accelerations
    for i in 0..system.n {
        system.ax[i] = 0.0;
        system.ay[i] = 0.0;
        system.az[i] = 0.0;
    }
    
    // Create thread-safe containers for accelerations
    let ax_mutex = Arc::new(Mutex::new(&mut system.ax));
    let ay_mutex = Arc::new(Mutex::new(&mut system.ay));
    let az_mutex = Arc::new(Mutex::new(&mut system.az));
    
    // Clone read-only data for parallel access
    let x_data: Vec<f64> = system.x.clone();
    let y_data: Vec<f64> = system.y.clone();
    let z_data: Vec<f64> = system.z.clone();
    let m_data: Vec<f64> = system.m.clone();
    let n = system.n;
    
    // Set rayon thread pool size
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();
    
    // Use parallel iterator to compute accelerations
    pool.install(|| {
        (0..n).into_par_iter().for_each(|i| {
            let mut local_ax = 0.0;
            let mut local_ay = 0.0;
            let mut local_az = 0.0;
            
            // Compute accelerations for particle i
            for j in 0..n {
                if i == j {
                    continue;
                }
                
                // Distance vector
                let dx = x_data[j] - x_data[i];
                let dy = y_data[j] - y_data[i];
                let dz = z_data[j] - z_data[i];
                
                // Distance squared with softening
                let r2 = dx * dx + dy * dy + dz * dz + eps * eps;
                let r = r2.sqrt();
                
                // Force magnitude factor
                let inv_r3 = g / (r2 * r);
                let force_factor = m_data[j] * inv_r3;
                
                // Accumulate local accelerations
                local_ax += dx * force_factor;
                local_ay += dy * force_factor;
                local_az += dz * force_factor;
            }
            
            // Write back to shared arrays (with mutex protection)
            {
                let mut ax_guard = ax_mutex.lock().unwrap();
                ax_guard[i] = local_ax;
            }
            {
                let mut ay_guard = ay_mutex.lock().unwrap();
                ay_guard[i] = local_ay;
            }
            {
                let mut az_guard = az_mutex.lock().unwrap();
                az_guard[i] = local_az;
            }
        });
    });
}

/// Optimized parallel version using chunked processing to reduce lock contention
pub fn compute_accelerations_naive_parallel_chunked(system: &mut NBodySystem, eps: f64, g: f64, num_threads: usize) {
    use rayon::prelude::*;
    
    let n = system.n;
    let chunk_size = (n + num_threads - 1) / num_threads; // Ceiling division
    
    // Reset accelerations
    system.ax.fill(0.0);
    system.ay.fill(0.0);
    system.az.fill(0.0);
    
    // Clone read-only data
    let x_data = system.x.clone();
    let y_data = system.y.clone();
    let z_data = system.z.clone();
    let m_data = system.m.clone();
    
    // Process chunks in parallel
    let chunks: Vec<_> = (0..n).step_by(chunk_size).collect();
    let results: Vec<(Vec<f64>, Vec<f64>, Vec<f64>)> = chunks
        .into_par_iter()
        .map(|start_idx| {
            let end_idx = (start_idx + chunk_size).min(n);
            let chunk_len = end_idx - start_idx;
            
            let mut chunk_ax = vec![0.0; chunk_len];
            let mut chunk_ay = vec![0.0; chunk_len];
            let mut chunk_az = vec![0.0; chunk_len];
            
            for (local_i, i) in (start_idx..end_idx).enumerate() {
                for j in 0..n {
                    if i == j {
                        continue;
                    }
                    
                    // Distance vector
                    let dx = x_data[j] - x_data[i];
                    let dy = y_data[j] - y_data[i];
                    let dz = z_data[j] - z_data[i];
                    
                    // Distance squared with softening
                    let r2 = dx * dx + dy * dy + dz * dz + eps * eps;
                    let r = r2.sqrt();
                    
                    // Force magnitude factor
                    let inv_r3 = g / (r2 * r);
                    let force_factor = m_data[j] * inv_r3;
                    
                    // Accumulate accelerations
                    chunk_ax[local_i] += dx * force_factor;
                    chunk_ay[local_i] += dy * force_factor;
                    chunk_az[local_i] += dz * force_factor;
                }
            }
            
            (chunk_ax, chunk_ay, chunk_az)
        })
        .collect();
    
    // Combine results back into system arrays
    for (chunk_idx, (chunk_ax, chunk_ay, chunk_az)) in results.into_iter().enumerate() {
        let start_idx = chunk_idx * chunk_size;
        let end_idx = (start_idx + chunk_ax.len()).min(n);
        
        for (local_i, i) in (start_idx..end_idx).enumerate() {
            system.ax[i] = chunk_ax[local_i];
            system.ay[i] = chunk_ay[local_i];
            system.az[i] = chunk_az[local_i];
        }
    }
}
