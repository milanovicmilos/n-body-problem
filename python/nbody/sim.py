import math
import time
from typing import List, Tuple, Optional, Any

try:
    import multiprocessing as mp
    from multiprocessing import Array
except Exception:
    mp = None  # type: ignore

from .model import Body
from .io import write_state_csv

# Optional: shared memory acceleration for Python multiprocessing to avoid per-iteration pickling
try:
    import multiprocessing.shared_memory as shm
    HAVE_SHM = True
except Exception:
    HAVE_SHM = False


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


def _velocity_verlet_step(
    bodies: List[Body],
    acc_prev: List[Tuple[float, float, float]],
    dt: float,
    acc_fn,
) -> List[Tuple[float, float, float]]:
    # 1) update positions using current velocities and previous accelerations
    dt2 = dt * dt
    for i, (ax, ay, az) in enumerate(acc_prev):
        b = bodies[i]
        b.x += b.vx * dt + 0.5 * ax * dt2
        b.y += b.vy * dt + 0.5 * ay * dt2
        b.z += b.vz * dt + 0.5 * az * dt2
    # 2) compute new accelerations from updated positions
    acc_new = acc_fn()
    # 3) update velocities using average of old and new accelerations
    for i, (ax_new, ay_new, az_new) in enumerate(acc_new):
        ax_prev, ay_prev, az_prev = acc_prev[i]
        b = bodies[i]
        b.vx += 0.5 * (ax_prev + ax_new) * dt
        b.vy += 0.5 * (ay_prev + ay_new) * dt
        b.vz += 0.5 * (az_prev + az_new) * dt
    return acc_new


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


###################################################################################################
# Optimized multiprocessing (shared-memory) implementation
# -----------------------------------------------------------------------------------------------
# Original implementation (above) pickled the entire bodies array every iteration, which *dominates*
# runtime for modest N on Windows (spawn). Here we eliminate that overhead by:
#   1. Storing positions (x,y,z) and masses in shared, lock-free Array('d') buffers (SoA-like layout)
#   2. Passing ONLY (start,end,G,softening) to workers each iteration (tiny pickled payload)
#   3. Workers read the shared buffers directly (read-only) and compute per-body accelerations
#   4. Parent updates positions (and the shared buffer) + velocities via Velocity Verlet, then re-dispatches
# This drastically reduces serialization and improves scaling for moderate N.
###################################################################################################

_SHM_POS: Any = None  # shared array of length n*3 (x,y,z interleaved)
_SHM_MASS: Any = None  # shared array of length n (masses)
_SHM_N: int = 0


def _init_worker(pos_base, mass_base, n):  # type: ignore
    """Worker initializer: attach module-level globals to inherited shared memory.

    Args:
        pos_base: multiprocessing Array('d') length n*3
        mass_base: multiprocessing Array('d') length n
        n: number of bodies
    """
    global _SHM_POS, _SHM_MASS, _SHM_N
    _SHM_POS = pos_base
    _SHM_MASS = mass_base
    _SHM_N = n


def _accel_chunk_shm(args):
    """Compute accelerations for a contiguous index range using shared arrays.

    Args:
        args: (start, end, G, softening)
    Returns:
        list[(i, ax, ay, az)] for i in [start,end)
    """
    start, end, G, softening = args
    # Local references (fast attribute -> local var)
    pos = _SHM_POS  # type: ignore
    mass = _SHM_MASS  # type: ignore
    n = _SHM_N
    out = []
    for i in range(start, end):
        xi = pos[3 * i + 0]
        yi = pos[3 * i + 1]
        zi = pos[3 * i + 2]
        aix = aiy = aiz = 0.0
        for j in range(n):
            if i == j:
                continue
            dx = pos[3 * j + 0] - xi
            dy = pos[3 * j + 1] - yi
            dz = pos[3 * j + 2] - zi
            dist_sqr = dx * dx + dy * dy + dz * dz + softening
            inv_r3 = 1.0 / (dist_sqr * math.sqrt(dist_sqr))
            f = G * mass[j] * inv_r3
            aix += dx * f
            aiy += dy * f
            aiz += dz * f
        out.append((i, aix, aiy, aiz))
    return out


def _prepare_shared(bodies: List[Body]):
    """Allocate and populate shared arrays (idempotent per simulate() call)."""
    n = len(bodies)
    pos = Array('d', n * 3, lock=False)
    mass = Array('d', n, lock=False)
    for i, b in enumerate(bodies):
        pos[3 * i + 0] = b.x
        pos[3 * i + 1] = b.y
        pos[3 * i + 2] = b.z
        mass[i] = b.m
    return pos, mass


def compute_accelerations_mp(bodies: List[Body], G: float, softening: float, workers: int, pool: Optional[Any] = None, shared: Optional[Tuple[Any, Any]] = None) -> List[Tuple[float, float, float]]:
    """Shared-memory optimized acceleration computation.

    Backwards compatible signature keeps callers unchanged; internally we use shared Arrays.
    """
    if mp is None:
        raise RuntimeError("multiprocessing is not available on this platform")
    n = len(bodies)
    if workers <= 0:
        workers = max(1, mp.cpu_count() - 1)

    # shared contains (pos_array, mass_array)
    if shared is None:
        raise RuntimeError("Shared arrays not prepared (expected simulate() to supply them)")
    pos_arr, mass_arr = shared

    # Build tasks with minimal payload
    chunk_size = max(1, (n + workers - 1) // workers)
    tasks = []
    for start in range(0, n, chunk_size):
        end = min(n, start + chunk_size)
        tasks.append((start, end, G, softening))

    if pool is None:
        # One-off usage (not expected here but keep for API completeness)
        with mp.Pool(processes=workers, initializer=_init_worker, initargs=(pos_arr, mass_arr, n)) as p:
            results = p.map(_accel_chunk_shm, tasks)
    else:
        results = pool.map(_accel_chunk_shm, tasks)

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
    write_every: int = 1,
) -> float:
    t0 = time.perf_counter()
    if write_every != 0:
        write_state_csv(out_path, 0, bodies, create_header=True)
    if mode == "seq":
        # initial accelerations
        acc_prev = compute_accelerations(bodies, G, softening)
        for it in range(1, steps + 1):
            acc_prev = _velocity_verlet_step(
                bodies,
                acc_prev,
                dt,
                acc_fn=lambda: compute_accelerations(bodies, G, softening),
            )
            if write_every != 0 and (it % write_every == 0 or it == steps):
                write_state_csv(out_path, it, bodies, create_header=False)
    elif mode == "mp":
        if mp is None:
            raise RuntimeError("multiprocessing is not available on this platform")
        if workers <= 0:
            workers = max(1, mp.cpu_count() - 1)

        # Prepare shared arrays (positions & masses) once
        pos_arr, mass_arr = _prepare_shared(bodies)

        # Pool with initializer so workers attach to shared memory without copying data each iteration
        with mp.Pool(processes=workers, initializer=_init_worker, initargs=(pos_arr, mass_arr, len(bodies))) as pool:
            # Initial accelerations
            acc_prev = compute_accelerations_mp(bodies, G, softening, workers, pool=pool, shared=(pos_arr, mass_arr))
            for it in range(1, steps + 1):
                # Velocity Verlet position update (also mirror positions into shared array)
                dt2 = dt * dt
                for i, (ax, ay, az) in enumerate(acc_prev):
                    b = bodies[i]
                    b.x += b.vx * dt + 0.5 * ax * dt2
                    b.y += b.vy * dt + 0.5 * ay * dt2
                    b.z += b.vz * dt + 0.5 * az * dt2
                    pos_arr[3 * i + 0] = b.x
                    pos_arr[3 * i + 1] = b.y
                    pos_arr[3 * i + 2] = b.z
                # New accelerations (parallel)
                acc_new = compute_accelerations_mp(bodies, G, softening, workers, pool=pool, shared=(pos_arr, mass_arr))
                # Velocity update
                for i, (ax_new, ay_new, az_new) in enumerate(acc_new):
                    ax_prev, ay_prev, az_prev = acc_prev[i]
                    b = bodies[i]
                    b.vx += 0.5 * (ax_prev + ax_new) * dt
                    b.vy += 0.5 * (ay_prev + ay_new) * dt
                    b.vz += 0.5 * (az_prev + az_new) * dt
                acc_prev = acc_new
                if write_every != 0 and (it % write_every == 0 or it == steps):
                    write_state_csv(out_path, it, bodies, create_header=False)
    else:
        raise ValueError(f"Unknown mode: {mode}")
    t1 = time.perf_counter()
    return t1 - t0
