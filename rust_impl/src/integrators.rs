use crate::types::NBodySystem;
use crate::force_naive::{compute_accelerations_naive_sequential, compute_accelerations_naive_parallel_chunked};

/// Velocity Verlet integration step - sequential version
pub fn velocity_verlet_step_sequential(system: &mut NBodySystem, dt: f64, eps: f64, g: f64) {
    let n = system.n;
    
    // Step 1: v(t + dt/2) = v(t) + (dt/2) * a(t)
    for i in 0..n {
        system.vx[i] += 0.5 * dt * system.ax[i];
        system.vy[i] += 0.5 * dt * system.ay[i];
        system.vz[i] += 0.5 * dt * system.az[i];
    }
    
    // Step 2: x(t + dt) = x(t) + dt * v(t + dt/2)
    for i in 0..n {
        system.x[i] += dt * system.vx[i];
        system.y[i] += dt * system.vy[i];
        system.z[i] += dt * system.vz[i];
    }
    
    // Step 3: compute a(t + dt) from new positions
    compute_accelerations_naive_sequential(system, eps, g);
    
    // Step 4: v(t + dt) = v(t + dt/2) + (dt/2) * a(t + dt)
    for i in 0..n {
        system.vx[i] += 0.5 * dt * system.ax[i];
        system.vy[i] += 0.5 * dt * system.ay[i];
        system.vz[i] += 0.5 * dt * system.az[i];
    }
}

/// Velocity Verlet integration step - parallel version
pub fn velocity_verlet_step_parallel(system: &mut NBodySystem, dt: f64, eps: f64, g: f64, num_threads: usize) {
    // Use the safe version instead of unsafe pointers
    velocity_verlet_step_parallel_safe(system, dt, eps, g, num_threads);
}

/// Safer parallel version using chunks to avoid unsafe code
pub fn velocity_verlet_step_parallel_safe(system: &mut NBodySystem, dt: f64, eps: f64, g: f64, num_threads: usize) {
    let n = system.n;
    
    // Step 1: v(t + dt/2) = v(t) + (dt/2) * a(t)
    for i in 0..n {
        system.vx[i] += 0.5 * dt * system.ax[i];
        system.vy[i] += 0.5 * dt * system.ay[i];
        system.vz[i] += 0.5 * dt * system.az[i];
    }
    
    // Step 2: x(t + dt) = x(t) + dt * v(t + dt/2)
    for i in 0..n {
        system.x[i] += dt * system.vx[i];
        system.y[i] += dt * system.vy[i];
        system.z[i] += dt * system.vz[i];
    }
    
    // Step 3: compute a(t + dt) from new positions (parallel)
    compute_accelerations_naive_parallel_chunked(system, eps, g, num_threads);
    
    // Step 4: v(t + dt) = v(t + dt/2) + (dt/2) * a(t + dt)
    for i in 0..n {
        system.vx[i] += 0.5 * dt * system.ax[i];
        system.vy[i] += 0.5 * dt * system.ay[i];
        system.vz[i] += 0.5 * dt * system.az[i];
    }
}

/// Euler integration step (for comparison) - sequential
pub fn euler_step_sequential(system: &mut NBodySystem, dt: f64, eps: f64, g: f64) {
    let n = system.n;
    
    // Compute accelerations at current positions
    compute_accelerations_naive_sequential(system, eps, g);
    
    // Update velocities: v(t + dt) = v(t) + dt * a(t)
    for i in 0..n {
        system.vx[i] += dt * system.ax[i];
        system.vy[i] += dt * system.ay[i];
        system.vz[i] += dt * system.az[i];
    }
    
    // Update positions: x(t + dt) = x(t) + dt * v(t + dt)
    for i in 0..n {
        system.x[i] += dt * system.vx[i];
        system.y[i] += dt * system.vy[i];
        system.z[i] += dt * system.vz[i];
    }
}

/// RK4 integration step (for comparison) - sequential
pub fn rk4_step_sequential(system: &mut NBodySystem, dt: f64, eps: f64, g: f64) {
    let n = system.n;
    
    // Save original state
    let x_orig = system.x.clone();
    let y_orig = system.y.clone();
    let z_orig = system.z.clone();
    let vx_orig = system.vx.clone();
    let vy_orig = system.vy.clone();
    let vz_orig = system.vz.clone();
    
    // k1
    compute_accelerations_naive_sequential(system, eps, g);
    let k1_vx = system.ax.clone();
    let k1_vy = system.ay.clone();
    let k1_vz = system.az.clone();
    let k1_x = system.vx.clone();
    let k1_y = system.vy.clone();
    let k1_z = system.vz.clone();
    
    // k2 - move to midpoint
    for i in 0..n {
        system.x[i] = x_orig[i] + 0.5 * dt * k1_x[i];
        system.y[i] = y_orig[i] + 0.5 * dt * k1_y[i];
        system.z[i] = z_orig[i] + 0.5 * dt * k1_z[i];
        system.vx[i] = vx_orig[i] + 0.5 * dt * k1_vx[i];
        system.vy[i] = vy_orig[i] + 0.5 * dt * k1_vy[i];
        system.vz[i] = vz_orig[i] + 0.5 * dt * k1_vz[i];
    }
    
    compute_accelerations_naive_sequential(system, eps, g);
    let k2_vx = system.ax.clone();
    let k2_vy = system.ay.clone();
    let k2_vz = system.az.clone();
    let k2_x = system.vx.clone();
    let k2_y = system.vy.clone();
    let k2_z = system.vz.clone();
    
    // k3 - move to midpoint with k2
    for i in 0..n {
        system.x[i] = x_orig[i] + 0.5 * dt * k2_x[i];
        system.y[i] = y_orig[i] + 0.5 * dt * k2_y[i];
        system.z[i] = z_orig[i] + 0.5 * dt * k2_z[i];
        system.vx[i] = vx_orig[i] + 0.5 * dt * k2_vx[i];
        system.vy[i] = vy_orig[i] + 0.5 * dt * k2_vy[i];
        system.vz[i] = vz_orig[i] + 0.5 * dt * k2_vz[i];
    }
    
    compute_accelerations_naive_sequential(system, eps, g);
    let k3_vx = system.ax.clone();
    let k3_vy = system.ay.clone();
    let k3_vz = system.az.clone();
    let k3_x = system.vx.clone();
    let k3_y = system.vy.clone();
    let k3_z = system.vz.clone();
    
    // k4 - move to endpoint with k3
    for i in 0..n {
        system.x[i] = x_orig[i] + dt * k3_x[i];
        system.y[i] = y_orig[i] + dt * k3_y[i];
        system.z[i] = z_orig[i] + dt * k3_z[i];
        system.vx[i] = vx_orig[i] + dt * k3_vx[i];
        system.vy[i] = vy_orig[i] + dt * k3_vy[i];
        system.vz[i] = vz_orig[i] + dt * k3_vz[i];
    }
    
    compute_accelerations_naive_sequential(system, eps, g);
    let k4_vx = system.ax.clone();
    let k4_vy = system.ay.clone();
    let k4_vz = system.az.clone();
    let k4_x = system.vx.clone();
    let k4_y = system.vy.clone();
    let k4_z = system.vz.clone();
    
    // Final RK4 combination
    for i in 0..n {
        system.x[i] = x_orig[i] + (dt / 6.0) * (k1_x[i] + 2.0 * k2_x[i] + 2.0 * k3_x[i] + k4_x[i]);
        system.y[i] = y_orig[i] + (dt / 6.0) * (k1_y[i] + 2.0 * k2_y[i] + 2.0 * k3_y[i] + k4_y[i]);
        system.z[i] = z_orig[i] + (dt / 6.0) * (k1_z[i] + 2.0 * k2_z[i] + 2.0 * k3_z[i] + k4_z[i]);
        system.vx[i] = vx_orig[i] + (dt / 6.0) * (k1_vx[i] + 2.0 * k2_vx[i] + 2.0 * k3_vx[i] + k4_vx[i]);
        system.vy[i] = vy_orig[i] + (dt / 6.0) * (k1_vy[i] + 2.0 * k2_vy[i] + 2.0 * k3_vy[i] + k4_vy[i]);
        system.vz[i] = vz_orig[i] + (dt / 6.0) * (k1_vz[i] + 2.0 * k2_vz[i] + 2.0 * k3_vz[i] + k4_vz[i]);
    }
    
    // Recompute final accelerations
    compute_accelerations_naive_sequential(system, eps, g);
}
