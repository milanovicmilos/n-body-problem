"""Run strong-scaling experiments for Python and Rust implementations.

Generates `scripts/results/strong_scaling.csv` with aggregated timings.
"""
import argparse
from pathlib import Path
import json
from experiment_utils import run_command, write_rows_csv, aggregate_results


def build_rust_binary(rust_dir: Path):
    cmd = ["cargo", "build", "--release"]
    rc, out, err, dur = run_command(cmd, cwd=str(rust_dir))
    if rc != 0:
        raise RuntimeError(f"Cargo build failed: {err}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=1000)
    parser.add_argument("--steps", type=int, default=1000)
    parser.add_argument("--dt", type=float, default=0.001)
    parser.add_argument("--eps", type=float, default=0.01)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--out", type=str, default="scripts/results/strong_scaling.csv")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    rust_dir = repo_root / "rust_impl"
    py_dir = repo_root / "python_impl"

    # Build rust release
    print("Building Rust release...")
    build_rust_binary(rust_dir)

    procs_list = [1, 2, 4, 8]
    perf_map = {}

    for p in procs_list:
        # Python mp: use --procs
        for rep in range(args.repeats):
            out_dir = repo_root / f"data/outputs/strong_py_p{p}_n{args.n}_r{rep}"
            cmd = [sys_executable(), "-m", "nbody_py.cli", "run-mp",
                   "--n", str(args.n), "--steps", str(args.steps), "--dt", str(args.dt),
                   "--eps", str(args.eps), "--procs", str(p), "--dump-every", "100",
                   "--seed", "42", "--out", str(out_dir)]
            print("Running Python MP: p=", p, "rep=", rep)
            rc, out, err, dur = run_command(cmd, cwd=str(py_dir), timeout=3600)
            key = ("python", "mp", p, args.n, args.steps, args.dt, args.eps)
            perf_map.setdefault(key, []).append(dur)

        # Rust parallel: use --threads when running run-par; for p=1 run run-seq
        for rep in range(args.repeats):
            out_dir = repo_root / f"data/outputs/strong_rs_p{p}_n{args.n}_r{rep}"
            if p == 1:
                cmd = [str(rust_dir / "target" / "release" / "nbody-rs.exe"), "run-seq",
                       "--n", str(args.n), "--steps", str(args.steps), "--dt", str(args.dt),
                       "--eps", str(args.eps), "--dump-every", "100", "--seed", "42",
                       "--out", str(out_dir)]
            else:
                cmd = [str(rust_dir / "target" / "release" / "nbody-rs.exe"), "run-par",
                       "--n", str(args.n), "--steps", str(args.steps), "--dt", str(args.dt),
                       "--eps", str(args.eps), "--threads", str(p), "--dump-every", "100",
                       "--seed", "42", "--out", str(out_dir)]
            print("Running Rust: p=", p, "rep=", rep)
            rc, out, err, dur = run_command(cmd, cwd=str(repo_root), timeout=3600)
            key = ("rust", "par", p, args.n, args.steps, args.dt, args.eps)
            perf_map.setdefault(key, []).append(dur)

    # Aggregate and write CSV
    rows = []
    for rec in aggregate_results(perf_map):
        rows.append([rec["lang"], rec["mode"], rec["p"], rec["n"], rec["steps"], rec["dt"], rec["eps"], rec["mean_s"], rec["std_s"], rec["repeats"]])

    write_rows_csv(Path(args.out), rows, header=["lang","mode","p","n","steps","dt","eps","mean_s","std_s","repeats"])
    print("Done. Results written to", args.out)


def sys_executable():
    # prefer python from PATH
    return "python"


if __name__ == "__main__":
    main()
