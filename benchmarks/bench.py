import argparse
import csv
import math
import os
import re
import statistics
import subprocess
import sys
from typing import List, Tuple, Dict

try:
    import matplotlib.pyplot as plt  # type: ignore
    MATPLOTLIB_AVAILABLE = True
except Exception:
    MATPLOTLIB_AVAILABLE = False

# PowerShell friendly execution assumed. Use sys.executable for Python.

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PY = sys.executable or "python"
PY_MAIN = os.path.join(ROOT, "python", "main.py")
RUST_DIR = os.path.join(ROOT, "rust")
OUTPUT = os.path.join(ROOT, "output")
PLOTS = {
    "python_strong": os.path.join(OUTPUT, "python_strong.png"),
    "rust_strong": os.path.join(OUTPUT, "rust_strong.png"),
    "python_weak": os.path.join(OUTPUT, "python_weak.png"),
    "rust_weak": os.path.join(OUTPUT, "rust_weak.png"),
}


def _parse_elapsed(stdout: str) -> float:
    # Look for pattern ElapsedSeconds=number in the entire stdout, last occurrence wins
    matches = re.findall(r"ElapsedSeconds=([0-9]*\.?[0-9]+)", stdout)
    if not matches:
        raise RuntimeError(f"Could not parse elapsed time from output:\n{stdout}")
    return float(matches[-1])


def run_py(mode: str, n: int, steps: int, dt: float, workers: int = 0) -> float:
    cmd = [
        PY, PY_MAIN,
        "--mode", mode,
        "--random", str(n),
        "--steps", str(steps),
        "--dt", str(dt),
        "--output", os.path.join(OUTPUT, f"tmp_py_{mode}.csv"),
        "--write-every", "0",
    ]
    if mode == "mp":
        cmd += ["--workers", str(workers)]
    p = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return _parse_elapsed(p.stdout)


def run_rust(mode: str, n: int, steps: int, dt: float) -> float:
    exe = ["cargo", "run", "--release", "--", "--mode", mode, "--random", str(n),
        "--steps", str(steps), "--dt", str(dt), "--output", os.path.join("..", "output", f"tmp_rs_{mode}.csv"), "--quiet", "--write-every", "0"]
    p = subprocess.run(exe, cwd=RUST_DIR, capture_output=True, text=True, check=True)
    # Output suppressed; parse elapsed from stderr or ignore; instead run again with quiet off to get timing.
    exe_verbose = ["cargo", "run", "--release", "--", "--mode", mode, "--random", str(n),
             "--steps", str(steps), "--dt", str(dt), "--output", os.path.join("..", "output", f"tmp_rs_{mode}.csv"), "--write-every", "0"]
    p2 = subprocess.run(exe_verbose, cwd=RUST_DIR, capture_output=True, text=True, check=True)
    return _parse_elapsed(p2.stdout)


def strong_scaling(language: str, problem_n: int, steps: int, dt: float, worker_counts: List[int]) -> List[Tuple[int, float, float]]:
    results = []
    if language == "python":
        t_seq = run_py("seq", problem_n, steps, dt)
        for w in worker_counts:
            t_par = run_py("mp", problem_n, steps, dt, workers=w)
            speedup = t_seq / t_par if t_par > 0 else float("inf")
            results.append((w, t_par, speedup))
    elif language == "rust":
        t_seq = run_rust("seq", problem_n, steps, dt)
        for w in worker_counts:
            # Control rayon via env var; for simplicity, call with env override
            env = os.environ.copy()
            env["RAYON_NUM_THREADS"] = str(w)
            # We cannot pass env to helper directly; inline the call here
            exe_verbose = ["cargo", "run", "--release", "--", "--mode", "threads", "--random", str(problem_n),
                           "--steps", str(steps), "--dt", str(dt), "--output", os.path.join("..", "output", f"tmp_rs_threads.csv"), "--write-every", "0"]
            p2 = subprocess.run(exe_verbose, cwd=RUST_DIR, capture_output=True, text=True, check=True, env=env)
            t_par = _parse_elapsed(p2.stdout)
            speedup = t_seq / t_par if t_par > 0 else float("inf")
            results.append((w, t_par, speedup))
    else:
        raise ValueError("language must be 'python' or 'rust'")
    return results


def weak_scaling(language: str, base_n: int, steps: int, dt: float, worker_counts: List[int]) -> List[Tuple[int, int, float, float]]:
    # Keep work per worker roughly constant: N ~ base_n * workers
    results = []
    if language == "python":
        t_seq = run_py("seq", base_n, steps, dt)
        for w in worker_counts:
            n = max(1, base_n * w)
            t_par = run_py("mp", n, steps, dt, workers=w)
            speedup = (t_seq * w) / t_par if t_par > 0 else float("inf")
            results.append((w, n, t_par, speedup))
    elif language == "rust":
        t_seq = run_rust("seq", base_n, steps, dt)
        for w in worker_counts:
            n = max(1, base_n * w)
            env = os.environ.copy()
            env["RAYON_NUM_THREADS"] = str(w)
            exe_verbose = ["cargo", "run", "--release", "--", "--mode", "threads", "--random", str(n),
                           "--steps", str(steps), "--dt", str(dt), "--output", os.path.join("..", "output", f"tmp_rs_threads.csv"), "--write-every", "0"]
            p2 = subprocess.run(exe_verbose, cwd=RUST_DIR, capture_output=True, text=True, check=True, env=env)
            t_par = _parse_elapsed(p2.stdout)
            speedup = (t_seq * w) / t_par if t_par > 0 else float("inf")
            results.append((w, n, t_par, speedup))
    else:
        raise ValueError("language must be 'python' or 'rust'")
    return results


def write_csv(path: str, headers: List[str], rows: List[Tuple]):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(headers)
        for r in rows:
            w.writerow(list(r))


def fit_amdahl(workers: List[int], speedups: List[float]) -> float:
    # Fit parallel fraction p in Amdahl: S(N) = 1 / ((1-p) + p/N)
    # Simple grid search [0,1] step 1e-3 to avoid external deps
    best_p, best_err = 0.0, float("inf")
    for k in range(0, 1001):
        p = k / 1000.0
        err = 0.0
        for n, s in zip(workers, speedups):
            st = 1.0 / ((1.0 - p) + p / max(1, n))
            err += (s - st) ** 2
        if err < best_err:
            best_err = err
            best_p = p
    return best_p


def fit_gustafson(workers: List[int], speedups: List[float]) -> float:
    # Fit parallel fraction p in Gustafson: S(N) = (1-p) + p*N
    # Least squares closed form for linear regression S = a + b*N, with a=(1-p), b=p
    # But N includes 1,2,4,8 etc; do simple linear regression
    n = len(workers)
    x = workers
    y = speedups
    sx = sum(x)
    sy = sum(y)
    sxx = sum(xi * xi for xi in x)
    sxy = sum(xi * yi for xi, yi in zip(x, y))
    denom = n * sxx - sx * sx
    if denom == 0:
        return 0.0
    b = (n * sxy - sx * sy) / denom  # slope ~ p
    p = max(0.0, min(1.0, b))
    return p


def summarize_strong(language: str, problem_n: int, steps: int, dt: float, workers: List[int], repeats: int) -> Dict[str, str]:
    # Run sequential baseline repeats
    t_seq_runs = [run_py("seq", problem_n, steps, dt) if language == "python" else run_rust("seq", problem_n, steps, dt) for _ in range(repeats)]
    t_seq_mean = statistics.mean(t_seq_runs)
    t_seq_std = statistics.pstdev(t_seq_runs) if repeats > 1 else 0.0

    rows = []
    speedups = []
    for w in workers:
        runs = [
            (run_py("mp", problem_n, steps, dt, workers=w) if language == "python"
             else (lambda: (os.environ.setdefault("RAYON_NUM_THREADS", str(w)), run_rust("threads", problem_n, steps, dt))[1])())
            for _ in range(repeats)
        ]
        mean = statistics.mean(runs)
        std = statistics.pstdev(runs) if repeats > 1 else 0.0
        sp = t_seq_mean / mean if mean > 0 else float("inf")
        speedups.append(sp)
        # outliers via IQR
        q1 = statistics.quantiles(runs, n=4)[0] if repeats >= 4 else min(runs)
        q3 = statistics.quantiles(runs, n=4)[2] if repeats >= 4 else max(runs)
        iqr = q3 - q1
        lo, hi = q1 - 1.5 * iqr, q3 + 1.5 * iqr
        outliers = sum(1 for r in runs if r < lo or r > hi)
        rows.append((language, "strong", w, problem_n, repeats, mean, std, min(runs), max(runs), outliers, sp))

    # Fit Amdahl p
    p = fit_amdahl(workers, speedups)

    # Write summary CSV
    out_csv = os.path.join(OUTPUT, f"summary_{language}_strong.csv")
    write_csv(out_csv, [
        "language", "type", "workers", "n", "repeats", "t_mean", "t_std", "t_min", "t_max", "outliers", "speedup"
    ], rows)

    # Plot
    plot_path = PLOTS[f"{language}_strong"]
    if MATPLOTLIB_AVAILABLE:
        xs = workers
        ys = speedups
        ys_ideal = [n for n in xs]
        ys_amdahl = [1.0 / ((1.0 - p) + p / n) for n in xs]
        plt.figure()
        plt.plot(xs, ys, "o-", label="measured")
        plt.plot(xs, ys_amdahl, "--", label=f"Amdahl fit p={p:.2f}")
        plt.plot(xs, ys_ideal, ":", label="ideal")
        plt.xlabel("cores (workers)")
        plt.ylabel("speedup")
        plt.title(f"Strong scaling {language}")
        plt.grid(True, alpha=0.3)
        plt.legend()
        plt.savefig(plot_path, dpi=140, bbox_inches="tight")
        plt.close()

    return {
        "t_seq_mean": f"{t_seq_mean:.6f}",
        "t_seq_std": f"{t_seq_std:.6f}",
        "p_amdahl": f"{p:.4f}",
        "plot": plot_path,
    }


def summarize_weak(language: str, base_n: int, steps: int, dt: float, workers: List[int], repeats: int) -> Dict[str, str]:
    # Baseline sequential time at base_n
    t_seq_runs = [run_py("seq", base_n, steps, dt) if language == "python" else run_rust("seq", base_n, steps, dt) for _ in range(repeats)]
    t_seq_mean = statistics.mean(t_seq_runs)
    t_seq_std = statistics.pstdev(t_seq_runs) if repeats > 1 else 0.0

    rows = []
    speedups = []
    for w in workers:
        n = max(1, base_n * w)
        runs = [
            (run_py("mp", n, steps, dt, workers=w) if language == "python"
             else (lambda: (os.environ.setdefault("RAYON_NUM_THREADS", str(w)), run_rust("threads", n, steps, dt))[1])())
            for _ in range(repeats)
        ]
        mean = statistics.mean(runs)
        std = statistics.pstdev(runs) if repeats > 1 else 0.0
        sp = (t_seq_mean * w) / mean if mean > 0 else float("inf")
        speedups.append(sp)
        q1 = statistics.quantiles(runs, n=4)[0] if repeats >= 4 else min(runs)
        q3 = statistics.quantiles(runs, n=4)[2] if repeats >= 4 else max(runs)
        iqr = q3 - q1
        lo, hi = q1 - 1.5 * iqr, q3 + 1.5 * iqr
        outliers = sum(1 for r in runs if r < lo or r > hi)
        rows.append((language, "weak", w, n, repeats, mean, std, min(runs), max(runs), outliers, sp))

    # Fit Gustafson p
    p = fit_gustafson(workers, speedups)

    out_csv = os.path.join(OUTPUT, f"summary_{language}_weak.csv")
    write_csv(out_csv, [
        "language", "type", "workers", "n", "repeats", "t_mean", "t_std", "t_min", "t_max", "outliers", "speedup"
    ], rows)

    plot_path = PLOTS[f"{language}_weak"]
    if MATPLOTLIB_AVAILABLE:
        xs = workers
        ys = speedups
        ys_ideal = [n for n in xs]
        ys_gust = [(1.0 - p) + p * n for n in xs]
        plt.figure()
        plt.plot(xs, ys, "o-", label="measured")
        plt.plot(xs, ys_gust, "--", label=f"Gustafson fit p={p:.2f}")
        plt.plot(xs, ys_ideal, ":", label="ideal")
        plt.xlabel("cores (workers)")
        plt.ylabel("speedup")
        plt.title(f"Weak scaling {language}")
        plt.grid(True, alpha=0.3)
        plt.legend()
        plt.savefig(plot_path, dpi=140, bbox_inches="tight")
        plt.close()

    return {
        "t_seq_mean": f"{t_seq_mean:.6f}",
        "t_seq_std": f"{t_seq_std:.6f}",
        "p_gust": f"{p:.4f}",
        "plot": plot_path,
    }


def collect_system_info() -> Dict[str, str]:
    info: Dict[str, str] = {}
    try:
        import platform
        info["python"] = sys.version.split()[0]
        info["platform"] = platform.platform()
        info["machine"] = platform.machine()
        info["processor"] = platform.processor()
    except Exception:
        pass
    # Rust toolchain
    try:
        out = subprocess.run(["cargo", "--version"], capture_output=True, text=True, check=True)
        info["cargo_version"] = out.stdout.strip()
    except Exception:
        pass
    try:
        out = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True)
        info["rustc_version"] = out.stdout.strip()
    except Exception:
        pass
    # Windows-specific details via PowerShell CIM
    try:
        ps_cmd_cpu = [
            "powershell", "-NoProfile", "-Command",
            "(Get-CimInstance Win32_Processor | Select-Object -First 1 Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed,L2CacheSize,L3CacheSize | ConvertTo-Json -Compress)"
        ]
        out = subprocess.run(ps_cmd_cpu, capture_output=True, text=True, check=True)
        info["cpu"] = out.stdout.strip()
    except Exception:
        pass
    try:
        ps_cmd_os = [
            "powershell", "-NoProfile", "-Command",
            "(Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber | ConvertTo-Json -Compress)"
        ]
        out = subprocess.run(ps_cmd_os, capture_output=True, text=True, check=True)
        info["os"] = out.stdout.strip()
    except Exception:
        pass
    try:
        ps_cmd_ram = [
            "powershell", "-NoProfile", "-Command",
            "([Math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,2)).ToString() + ' GB'"
        ]
        out = subprocess.run(ps_cmd_ram, capture_output=True, text=True, check=True)
        info["ram"] = out.stdout.strip()
    except Exception:
        pass
    return info


def main():
    parser = argparse.ArgumentParser(description="N-body strong/weak scaling benchmarks for Python and Rust")
    parser.add_argument("--steps", type=int, default=120, help="Number of steps for all runs (default: 120)")
    parser.add_argument("--dt", type=float, default=0.002, help="Timestep size (default: 0.002)")
    parser.add_argument("--workers", type=int, nargs="+", default=[1, 2, 4, 8], help="Worker counts to test (default: 1 2 4 8)")
    parser.add_argument("--problem-n", type=int, default=200, help="N for strong scaling (default: 200)")
    parser.add_argument("--base-n", type=int, default=50, help="Base N for weak scaling (N = base_n * workers; default: 50)")
    parser.add_argument("--repeats", type=int, default=5, help="Repeat each configuration this many times (default: 5; set 30 for full report)")
    parser.add_argument("--no-python", action="store_true", help="Skip Python benchmarks")
    parser.add_argument("--no-rust", action="store_true", help="Skip Rust benchmarks")

    args = parser.parse_args()

    os.makedirs(OUTPUT, exist_ok=True)
    steps = args.steps
    dt = args.dt
    worker_counts = list(args.workers)
    problem_n = args.problem_n
    base_n = args.base_n
    repeats = args.repeats

    # Collect system info and write to file for README consumption
    sysinfo = collect_system_info()
    with open(os.path.join(OUTPUT, "system_info.json"), "w", encoding="utf-8") as f:
        import json
        json.dump(sysinfo, f, ensure_ascii=False, indent=2)

    fit_params: Dict[str, Dict[str, str]] = {}
    if not args.no_python:
        fit_params["python_strong"] = summarize_strong("python", problem_n, steps, dt, worker_counts, repeats)
        fit_params["python_weak"] = summarize_weak("python", base_n, steps, dt, worker_counts, repeats)

    if not args.no_rust:
        fit_params["rust_strong"] = summarize_strong("rust", problem_n, steps, dt, worker_counts, repeats)
        fit_params["rust_weak"] = summarize_weak("rust", base_n, steps, dt, worker_counts, repeats)

    # Persist fit parameters
    with open(os.path.join(OUTPUT, "fit_params.json"), "w", encoding="utf-8") as f:
        import json
        json.dump(fit_params, f, ensure_ascii=False, indent=2)

    print("Benchmarking finished. CSVs, fit_params.json, system_info.json and plots (if matplotlib available) written to output/.")


if __name__ == "__main__":
    main()
