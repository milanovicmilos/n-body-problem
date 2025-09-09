"""
Parallel N-body simulation implementation using multiprocessing
"""
import time
import math
import multiprocessing as mp
import numpy as np
from typing import Dict, Any, List, Tuple
from .physics import NBodySystem, velocity_verlet_step
from .io_utils import (
    create_output_directory, save_system_state, save_energy_data, 
    save_metadata, save_initial_conditions
)


def compute_accelerations_chunk(args: Tuple) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """
    Compute accelerations for a chunk of particles
    
    Args:
        args: Tuple containing (start_idx, end_idx, positions, masses, eps, G)
        
    Returns:
        Tuple of acceleration arrays (ax, ay, az) for the chunk
    """
    start_idx, end_idx, x, y, z, m, eps, G = args
    n = len(x)
    chunk_size = end_idx - start_idx
    
    # Initialize acceleration arrays for this chunk
    ax_chunk = np.zeros(chunk_size, dtype=np.float64)
    ay_chunk = np.zeros(chunk_size, dtype=np.float64)
    az_chunk = np.zeros(chunk_size, dtype=np.float64)
    
    # Compute accelerations for particles in this chunk
    for i in range(chunk_size):
        global_i = start_idx + i
        
        for j in range(n):
            if global_i == j:
                continue
                
            # Distance vector
            dx = x[j] - x[global_i]
            dy = y[j] - y[global_i]
            dz = z[j] - z[global_i]
            
            # Distance squared with softening
            r2 = dx*dx + dy*dy + dz*dz + eps*eps
            r = math.sqrt(r2)
            
            # Force magnitude factor
            inv_r3 = G / (r2 * r)
            force_factor = m[j] * inv_r3
            
            # Accumulate accelerations
            ax_chunk[i] += dx * force_factor
            ay_chunk[i] += dy * force_factor
            az_chunk[i] += dz * force_factor
    
    return ax_chunk, ay_chunk, az_chunk


def compute_accelerations_parallel(system: NBodySystem, num_processes: int, eps: float = 1e-2, G: float = 1.0):
    """
    Compute gravitational accelerations using parallel processing
    """
    n = system.n
    
    # For small N, use sequential computation to avoid overhead
    if n < 500 or num_processes == 1:
        from .physics import compute_accelerations_naive
        compute_accelerations_naive(system, eps, G)
        return
    
    # Determine chunk sizes for each process
    chunk_size = max(1, n // num_processes)
    chunks = []
    
    for p in range(num_processes):
        start_idx = p * chunk_size
        if p == num_processes - 1:
            end_idx = n  # Last process gets remaining particles
        else:
            end_idx = min((p + 1) * chunk_size, n)
        
        if start_idx < end_idx:  # Only add valid chunks
            chunks.append((start_idx, end_idx, system.x, system.y, system.z, system.m, eps, G))
    
    # Use multiprocessing pool to compute accelerations
    with mp.Pool(processes=len(chunks)) as pool:
        results = pool.map(compute_accelerations_chunk, chunks)
    
    # Combine results back into system arrays
    system.ax.fill(0.0)
    system.ay.fill(0.0)
    system.az.fill(0.0)
    
    for i, (ax_chunk, ay_chunk, az_chunk) in enumerate(results):
        start_idx = i * chunk_size
        if i == len(results) - 1:
            end_idx = n
        else:
            end_idx = min((i + 1) * chunk_size, n)
        
        chunk_len = end_idx - start_idx
        system.ax[start_idx:end_idx] = ax_chunk[:chunk_len]
        system.ay[start_idx:end_idx] = ay_chunk[:chunk_len]
        system.az[start_idx:end_idx] = az_chunk[:chunk_len]


def velocity_verlet_step_parallel(system: NBodySystem, dt: float, num_processes: int, eps: float = 1e-2, G: float = 1.0):
    """
    Perform one step of Velocity Verlet integration with parallel force computation
    """
    # Step 1: v(t + dt/2) = v(t) + (dt/2) * a(t)
    system.vx += 0.5 * dt * system.ax
    system.vy += 0.5 * dt * system.ay
    system.vz += 0.5 * dt * system.az
    
    # Step 2: x(t + dt) = x(t) + dt * v(t + dt/2)
    system.x += dt * system.vx
    system.y += dt * system.vy
    system.z += dt * system.vz
    
    # Step 3: compute a(t + dt) from new positions (parallel)
    compute_accelerations_parallel(system, num_processes, eps, G)
    
    # Step 4: v(t + dt) = v(t + dt/2) + (dt/2) * a(t + dt)
    system.vx += 0.5 * dt * system.ax
    system.vy += 0.5 * dt * system.ay
    system.vz += 0.5 * dt * system.az


def run_parallel_simulation(params: Dict[str, Any]) -> float:
    """
    Run parallel N-body simulation using multiprocessing
    
    Args:
        params: Dictionary containing simulation parameters
        
    Returns:
        execution_time: Total execution time in seconds
    """
    # Extract parameters
    n = params['n']
    steps = params['steps']
    dt = params['dt']
    eps = params.get('eps', 1e-2)
    G = params.get('G', 1.0)
    seed = params.get('seed', 42)
    dump_every = params.get('dump_every', 10)
    output_dir = params['output_dir']
    num_processes = params.get('procs', mp.cpu_count())
    
    print(f"Starting parallel simulation:")
    print(f"  N bodies: {n}")
    print(f"  Steps: {steps}")
    print(f"  Time step: {dt}")
    print(f"  Softening: {eps}")
    print(f"  Processes: {num_processes}")
    print(f"  Output: {output_dir}")
    
    # Create output directory
    create_output_directory(output_dir)
    
    # Initialize system
    print("Initializing system...")
    system = NBodySystem(n)
    system.initialize_plummer_sphere(seed)
    
    # Save initial conditions
    save_initial_conditions(system, output_dir)
    
    # Compute initial accelerations
    compute_accelerations_parallel(system, num_processes, eps, G)
    
    # Save initial state and energy
    save_system_state(system, 0, output_dir)
    kinetic = system.compute_kinetic_energy()
    potential = system.compute_potential_energy(eps)
    save_energy_data(0, kinetic, potential, output_dir)
    
    print(f"Initial energy: K={kinetic:.6f}, U={potential:.6f}, Total={kinetic+potential:.6f}")
    
    # Start simulation timer
    start_time = time.time()
    
    print("Running simulation...")
    for step in range(1, steps + 1):
        # Perform integration step
        velocity_verlet_step_parallel(system, dt, num_processes, eps, G)
        
        # Save state and energy if needed
        if step % dump_every == 0:
            save_system_state(system, step, output_dir)
            kinetic = system.compute_kinetic_energy()
            potential = system.compute_potential_energy(eps)
            save_energy_data(step, kinetic, potential, output_dir)
            
            if step % (dump_every * 10) == 0:
                total_energy = kinetic + potential
                print(f"Step {step:6d}: K={kinetic:.6f}, U={potential:.6f}, Total={total_energy:.6f}")
    
    # End simulation timer
    execution_time = time.time() - start_time
    
    # Final energy check
    final_kinetic = system.compute_kinetic_energy()
    final_potential = system.compute_potential_energy(eps)
    final_total = final_kinetic + final_potential
    
    print(f"Simulation completed in {execution_time:.2f} seconds")
    print(f"Final energy: K={final_kinetic:.6f}, U={final_potential:.6f}, Total={final_total:.6f}")
    
    # Save metadata
    params_with_procs = params.copy()
    params_with_procs['processes_used'] = num_processes
    save_metadata(params_with_procs, output_dir, execution_time)
    
    return execution_time


if __name__ == "__main__":
    # Test run
    test_params = {
        'n': 100,
        'steps': 1000,
        'dt': 0.001,
        'eps': 0.01,
        'seed': 42,
        'dump_every': 100,
        'procs': 4,
        'output_dir': '../data/outputs/test_parallel'
    }
    
    execution_time = run_parallel_simulation(test_params)
    print(f"Test completed in {execution_time:.2f} seconds")
