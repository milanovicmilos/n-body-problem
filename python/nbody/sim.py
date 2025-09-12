import math
import time
from typing import List, Tuple, Optional, Any

try:
    import multiprocessing as mp
except Exception:
    mp = None  # type: ignore

from .model import Body
from .io import write_state_csv


def compute_accelerations(bodies: List[Body], G: float, softening: float) -> List[Tuple[float, float, float]]:
    n = len(bodies)
    ax = [0.0] * n
    ay = [0.0] * n
    az = [0.0] * n
    for i in range(n):
        xi, yi, zi = bodies[i].x, bodies[i].y, bodies[i].z
        aix = aiy = aiz = 0.0
        for j in range(n):
            if i == j:
                continue
            dx = bodies[j].x - xi
            dy = bodies[j].y - yi
            dz = bodies[j].z - zi
            dist_sqr = dx * dx + dy * dy + dz * dz + softening
            inv_r3 = 1.0 / (dist_sqr * math.sqrt(dist_sqr))
            f = G * bodies[j].m * inv_r3
            aix += dx * f
            aiy += dy * f
            aiz += dz * f
        ax[i] = aix
        ay[i] = aiy
        az[i] = aiz
    return list(zip(ax, ay, az))


def step_euler(bodies: List[Body], acc: List[Tuple[float, float, float]], dt: float) -> None:
    for i, (ax, ay, az) in enumerate(acc):
        b = bodies[i]
        b.vx += ax * dt
        b.vy += ay * dt
        b.vz += az * dt
    for b in bodies:
        b.x += b.vx * dt
        b.y += b.vy * dt
        b.z += b.vz * dt


def _accel_chunk(args):
    bodies_flat, masses, idx_start, idx_end, G, softening = args
    n = len(masses)
    out = []
    for i in range(idx_start, idx_end):
        xi = bodies_flat[3 * i + 0]
        yi = bodies_flat[3 * i + 1]
        zi = bodies_flat[3 * i + 2]
        aix = aiy = aiz = 0.0
        for j in range(n):
            if i == j:
                continue
            dx = bodies_flat[3 * j + 0] - xi
            dy = bodies_flat[3 * j + 1] - yi
            dz = bodies_flat[3 * j + 2] - zi
            dist_sqr = dx * dx + dy * dy + dz * dz + softening
            inv_r3 = 1.0 / (dist_sqr * math.sqrt(dist_sqr))
            f = G * masses[j] * inv_r3
            aix += dx * f
            aiy += dy * f
            aiz += dz * f
        out.append((i, aix, aiy, aiz))
    return out


def compute_accelerations_mp(bodies: List[Body], G: float, softening: float, workers: int, pool: Optional[Any] = None) -> List[Tuple[float, float, float]]:
    if mp is None:
        raise RuntimeError("multiprocessing is not available on this platform")
    n = len(bodies)
    masses = [b.m for b in bodies]
    bodies_flat = []
    bodies_flat_extend = bodies_flat.extend
    for b in bodies:
        bodies_flat_extend([b.x, b.y, b.z])
    if workers <= 0:
        workers = max(1, mp.cpu_count() - 1)
    chunk_size = max(1, (n + workers - 1) // workers)
    tasks = []
    for start in range(0, n, chunk_size):
        end = min(n, start + chunk_size)
        tasks.append((bodies_flat, masses, start, end, G, softening))
    if pool is None:
        with mp.Pool(processes=workers) as p:
            results = p.map(_accel_chunk, tasks)
    else:
        results = pool.map(_accel_chunk, tasks)
    acc = [(0.0, 0.0, 0.0)] * n
    for part in results:
        for (i, ax, ay, az) in part:
            acc[i] = (ax, ay, az)
    return acc


def simulate(
    bodies: List[Body],
    steps: int,
    dt: float,
    G: float,
    softening: float,
    mode: str,
    out_path: str,
    workers: int,
) -> float:
    t0 = time.perf_counter()
    write_state_csv(out_path, 0, bodies, create_header=True)
    if mode == "seq":
        for it in range(1, steps + 1):
            acc = compute_accelerations(bodies, G, softening)
            step_euler(bodies, acc, dt)
            write_state_csv(out_path, it, bodies, create_header=False)
    elif mode == "mp":
        if mp is None:
            raise RuntimeError("multiprocessing is not available on this platform")
        if workers <= 0:
            workers = max(1, mp.cpu_count() - 1)
        with mp.Pool(processes=workers) as pool:
            for it in range(1, steps + 1):
                acc = compute_accelerations_mp(bodies, G, softening, workers, pool=pool)
                step_euler(bodies, acc, dt)
                write_state_csv(out_path, it, bodies, create_header=False)
    else:
        raise ValueError(f"Unknown mode: {mode}")
    t1 = time.perf_counter()
    return t1 - t0
