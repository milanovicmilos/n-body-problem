"""
Sequential N-body simulation implementation
"""
import time
from typing import Dict, Any
from .physics import NBodySystem, compute_accelerations_naive, velocity_verlet_step
from .io_utils import (
    create_output_directory, save_system_state, save_energy_data, 
    save_metadata, save_initial_conditions
)


def run_sequential_simulation(params: Dict[str, Any]) -> float:
    """
    Run sequential N-body simulation
    
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
    
    print(f"Starting sequential simulation:")
    print(f"  N bodies: {n}")
    print(f"  Steps: {steps}")
    print(f"  Time step: {dt}")
    print(f"  Softening: {eps}")
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
    compute_accelerations_naive(system, eps, G)
    
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
        velocity_verlet_step(system, dt, eps, G)
        
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
    save_metadata(params, output_dir, execution_time)
    
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
        'output_dir': '../data/outputs/test_sequential'
    }
    
    execution_time = run_sequential_simulation(test_params)
    print(f"Test completed in {execution_time:.2f} seconds")
