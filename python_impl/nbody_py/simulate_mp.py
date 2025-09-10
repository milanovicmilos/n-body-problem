"""
Parallel N-body simulation implementation using multiprocessing with shared memory.

This version uses multiprocessing.shared_memory to avoid copying large numpy arrays
to worker processes. It's more stable for large N in production runs.
"""
import time
import math
import multiprocessing as mp
import numpy as np
from multiprocessing import shared_memory
from typing import Dict, Any, List, Tuple
from .physics import NBodySystem
from .io_utils import (
    create_output_directory, save_system_state, save_energy_data,
    save_metadata, save_initial_conditions
)

# Global references for worker processes (attached in initializer)
_X = None
_Y = None
_Z = None
_M = None
_N = 0


def _worker_init(x_name: str, y_name: str, z_name: str, m_name: str, n: int):
    """Initializer for Pool workers: attach to shared memory segments."""
    global _X, _Y, _Z, _M, _N
    _N = n
    _X = np.ndarray((n,), dtype=np.float64, buffer=shared_memory.SharedMemory(name=x_name).buf)
    _Y = np.ndarray((n,), dtype=np.float64, buffer=shared_memory.SharedMemory(name=y_name).buf)
    _Z = np.ndarray((n,), dtype=np.float64, buffer=shared_memory.SharedMemory(name=z_name).buf)
    _M = np.ndarray((n,), dtype=np.float64, buffer=shared_memory.SharedMemory(name=m_name).buf)


def compute_accelerations_chunk(args: Tuple) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Compute accelerations for a chunk using global shared arrays.

    Args:
        args: (start_idx, end_idx, eps, G)
    Returns:
        (ax_chunk, ay_chunk, az_chunk)
    """
    start_idx, end_idx, eps, G = args
    n = _N
    chunk_size = end_idx - start_idx
    ax_chunk = np.zeros(chunk_size, dtype=np.float64)
    ay_chunk = np.zeros(chunk_size, dtype=np.float64)
    az_chunk = np.zeros(chunk_size, dtype=np.float64)

    for i in range(chunk_size):
        gi = start_idx + i
        xi = _X[gi]
        yi = _Y[gi]
        zi = _Z[gi]
        for j in range(n):
            if gi == j:
                continue
            dx = _X[j] - xi
            dy = _Y[j] - yi
            dz = _Z[j] - zi
            r2 = dx * dx + dy * dy + dz * dz + eps * eps
            r = math.sqrt(r2)
            inv_r3 = G / (r2 * r)
            ff = _M[j] * inv_r3
            ax_chunk[i] += dx * ff
            ay_chunk[i] += dy * ff
            az_chunk[i] += dz * ff

    return ax_chunk, ay_chunk, az_chunk


def compute_accelerations_parallel(system: NBodySystem, num_processes: int, eps: float = 1e-2, G: float = 1.0):
    """Compute gravitational accelerations using multiprocessing + shared memory."""
    n = system.n

    # For small N or single process, fallback to naive
    if n < 500 or num_processes == 1:
        from .physics import compute_accelerations_naive
        compute_accelerations_naive(system, eps, G)
        return

    # Create shared memory buffers for x,y,z,m
    x_shm = shared_memory.SharedMemory(create=True, size=system.x.nbytes)
    y_shm = shared_memory.SharedMemory(create=True, size=system.y.nbytes)
    z_shm = shared_memory.SharedMemory(create=True, size=system.z.nbytes)
    m_shm = shared_memory.SharedMemory(create=True, size=system.m.nbytes)

    try:
        # Copy data into shared buffers
        x_buf = np.ndarray(system.x.shape, dtype=system.x.dtype, buffer=x_shm.buf)
        y_buf = np.ndarray(system.y.shape, dtype=system.y.dtype, buffer=y_shm.buf)
        z_buf = np.ndarray(system.z.shape, dtype=system.z.dtype, buffer=z_shm.buf)
        m_buf = np.ndarray(system.m.shape, dtype=system.m.dtype, buffer=m_shm.buf)
        np.copyto(x_buf, system.x)
        np.copyto(y_buf, system.y)
        np.copyto(z_buf, system.z)
        np.copyto(m_buf, system.m)

        # Determine chunks
        chunk_size = max(1, n // num_processes)
        chunks = []
        for p in range(num_processes):
            start_idx = p * chunk_size
            end_idx = n if p == num_processes - 1 else min((p + 1) * chunk_size, n)
            if start_idx < end_idx:
                chunks.append((start_idx, end_idx, eps, G))

        # Launch pool with initializer attaching shared memory
        init_args = (x_shm.name, y_shm.name, z_shm.name, m_shm.name, n)
        with mp.Pool(processes=len(chunks), initializer=_worker_init, initargs=init_args) as pool:
            results = pool.map(compute_accelerations_chunk, chunks)

        # Combine into system arrays
        system.ax.fill(0.0)
        system.ay.fill(0.0)
        system.az.fill(0.0)
        for i, (ax_chunk, ay_chunk, az_chunk) in enumerate(results):
            start_idx = i * chunk_size
            end_idx = n if i == len(results) - 1 else min((i + 1) * chunk_size, n)
            chunk_len = end_idx - start_idx
            system.ax[start_idx:end_idx] = ax_chunk[:chunk_len]
            system.ay[start_idx:end_idx] = ay_chunk[:chunk_len]
            system.az[start_idx:end_idx] = az_chunk[:chunk_len]

    finally:
        # Clean up shared memory
        x_shm.close(); x_shm.unlink()
        y_shm.close(); y_shm.unlink()
        z_shm.close(); z_shm.unlink()
        m_shm.close(); m_shm.unlink()


def velocity_verlet_step_parallel(system: NBodySystem, dt: float, num_processes: int, eps: float = 1e-2, G: float = 1.0):
    """Perform one Velocity-Verlet step with parallel acceleration compute."""
    system.vx += 0.5 * dt * system.ax
    system.vy += 0.5 * dt * system.ay
    system.vz += 0.5 * dt * system.az

    system.x += dt * system.vx
    system.y += dt * system.vy
    system.z += dt * system.vz

    compute_accelerations_parallel(system, num_processes, eps, G)

    system.vx += 0.5 * dt * system.ax
    system.vy += 0.5 * dt * system.ay
    system.vz += 0.5 * dt * system.az


def run_parallel_simulation(params: Dict[str, Any]) -> float:
    """Run parallel N-body simulation using multiprocessing with shared memory."""
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

    create_output_directory(output_dir)

    print("Initializing system...")
    system = NBodySystem(n)
    system.initialize_plummer_sphere(seed)

    save_initial_conditions(system, output_dir)

    compute_accelerations_parallel(system, num_processes, eps, G)

    save_system_state(system, 0, output_dir)
    kinetic = system.compute_kinetic_energy()
    potential = system.compute_potential_energy(eps)
    save_energy_data(0, kinetic, potential, output_dir)

    print(f"Initial energy: K={kinetic:.6f}, U={potential:.6f}, Total={kinetic+potential:.6f}")

    start_time = time.time()

    print("Running simulation...")
    for step in range(1, steps + 1):
        velocity_verlet_step_parallel(system, dt, num_processes, eps, G)

        if step % dump_every == 0:
            save_system_state(system, step, output_dir)
            kinetic = system.compute_kinetic_energy()
            potential = system.compute_potential_energy(eps)
            save_energy_data(step, kinetic, potential, output_dir)
            if step % (dump_every * 10) == 0:
                total_energy = kinetic + potential
                print(f"Step {step:6d}: K={kinetic:.6f}, U={potential:.6f}, Total={total_energy:.6f}")

    execution_time = time.time() - start_time

    final_kinetic = system.compute_kinetic_energy()
    final_potential = system.compute_potential_energy(eps)
    final_total = final_kinetic + final_potential

    print(f"Simulation completed in {execution_time:.2f} seconds")
    print(f"Final energy: K={final_kinetic:.6f}, U={final_potential:.6f}, Total={final_total:.6f}")

    params_with_procs = params.copy()
    params_with_procs['processes_used'] = num_processes
    save_metadata(params_with_procs, output_dir, execution_time)

    return execution_time


if __name__ == "__main__":
    test_params = {
        'n': 100,
        'steps': 100,
        'dt': 0.001,
        'eps': 0.01,
        'seed': 42,
        'dump_every': 50,
        'procs': 4,
        'output_dir': '../data/outputs/test_parallel'
    }
    execution_time = run_parallel_simulation(test_params)
    print(f"Test completed in {execution_time:.2f} seconds")
