# N-Body Simulation: Python vs Rust — Strong and Weak Scaling Report

## Overview
This repository contains N-body simulations implemented in Python and Rust, each with sequential and parallel modes. We performed strong and weak scaling experiments, computed statistical summaries over repeated runs (~30), fitted parallel fractions (Amdahl for strong, Gustafson for weak), and generated plots comparing measured speedups with ideal and fitted models.

## Project Structure
- `python/` — Python package with `nbody` module and CLI (`python/main.py`).
- `rust/` — Rust crate with library (`src/lib.rs`) and CLI (`src/main.rs`).
- `benchmarks/bench.py` — Benchmark harness for strong/weak scaling, plots, and summaries.
- `output/` — Results: CSVs, plots, system info, and fitted parameters.

## How to Run
PowerShell examples:

1) Run Python simulation (sequential):
`python .\python\main.py --n 200 --steps 100 --dt 0.002 --mode seq --out .\output\py_seq.csv`

2) Run Python simulation (multiprocessing):
`python .\python\main.py --n 200 --steps 100 --dt 0.002 --mode mp --workers 4 --out .\output\py_mp.csv`

3) Run Rust simulation (sequential):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --n 200 --steps 100 --dt 0.002 --mode seq --out .\output\rs_seq.csv`

4) Run Rust simulation (threaded with rayon):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --n 200 --steps 100 --dt 0.002 --mode threads --workers 4 --out .\output\rs_threads.csv`

5) Run benchmarks (default settings):
`python .\benchmarks\bench.py`

6) Run benchmarks (custom):
`python .\benchmarks\bench.py --repeats 30 --workers 1 2 4 8 --problem-n 800 --base-n 200 --steps 200 --dt 0.002`

Outputs are written to `output/`.

7) Visualize an existing CSV (generate per-iteration PNG frames + animated GIF):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\rs_seq.csv`

## System Details
Collected automatically in `output/system_info.json`. Example from this machine:
- Python: see `python`
- Platform: see `platform`
- CPU: see `cpu` (cores/logical threads, caches, frequency)
- RAM: see `ram`
- OS: see `os`
- Rust toolchain: `cargo_version`, `rustc_version`

## Methodology
- Integrator: explicit Euler; gravitational interactions with softening; O(N^2) per step.
- Modes:
	- Python: `seq` (single process), `mp` (multiprocessing pool, persistent across steps).
	- Rust: `seq` (single-thread), `threads` (Rayon parallel iterators, thread count via workers).
- Timing: each configuration repeated `--repeats` times (default 5; report used 30). We record elapsed seconds and compute statistics: mean, std, min/max, and outliers by IQR.
- Strong scaling: fixed workload (`--problem-n` bodies, `--steps`), vary workers.
- Weak scaling: workload grows with workers: bodies = `workers * --base-n`, fixed `--steps`.

## Theory and Fitted Parameters
- Amdahl’s Law (strong scaling): `S(w) = 1 / ((1-p) + p/w)`, where `p` is parallel fraction. We fit `p` to minimize squared error vs measured speedups from `w=1` baseline.
- Gustafson’s Law (weak scaling): `S(w) = (1-p) + p*w`. We fit `p` by linear regression of `S(w)` on `w`.

Fitted parameters and references to plots are saved in `output/fit_params.json`. Example fields:
- `python_strong.p_amdahl`
- `python_weak.p_gust`
- `rust_strong.p_amdahl`
- `rust_weak.p_gust`

Theoretical maxima (Amdahl): with fitted `p`, `S_max = 1/(1-p)` as `w→∞`. For Gustafson, speedup grows approximately linearly with `w` according to fitted `p`.

## Results Summary
CSV summaries:
- Strong scaling means/std/outliers: `output/summary_python_strong.csv`, `output/summary_rust_strong.csv`
- Weak scaling means/std/outliers: `output/summary_python_weak.csv`, `output/summary_rust_weak.csv`

Plots (measured vs ideal and fitted):
- Python strong: `output/python_strong.png`
- Python weak: `output/python_weak.png`
- Rust strong: `output/rust_strong.png`
- Rust weak: `output/rust_weak.png`

Weak-scaling workload explanation: for worker count `w`, number of bodies is `N = w * base_n`, keeping per-worker workload roughly constant (O((N/w)^2 * w) ≈ O(N^2/w) per worker, but total pairs O(N^2) scales with `w^2`; here we interpret weak scaling as increasing N linearly with `w` to observe how total time grows and how parallel efficiency trends with `w`). In practice, ideal weak scaling would keep time constant; deviations reflect overheads and non-parallel portions.

## Selected Fitted Values (from this run)
Refer to `output/fit_params.json` for authoritative values. Example snapshot:
- Python strong: `p ≈ 0.9360` (Amdahl) → `S_max ≈ 15.6`
- Python weak: `p ≈ 0.0000` (Gustafson fit near 0 with these parameters)
- Rust strong: `p ≈ 0.0000` (baseline so fast that threading provided limited extra benefit at selected sizes)
- Rust weak: `p ≈ 0.1578`

Important: These values depend on problem sizes and repeats. For larger N/steps, Python multiprocessing and Rust threading typically show higher `p`.

## Reproducibility and Tips
- Ensure `matplotlib` is installed to generate plots. If not, install: `python -m pip install matplotlib`.
- For Python mp, larger `N` and `steps` reduce overhead dominance.
- On Windows, Python multiprocessing uses spawn; `if __name__ == "__main__":` is properly guarded in our CLI.
- Control Rust threads via `--workers` (Rayon uses that as a global pool size for the run).

## Files Produced
- `output/bench_*` — raw per-run timings for each configuration.
- `output/summary_*` — aggregation across repeats (mean, std, outliers).
- `output/*_strong.png`, `output/*_weak.png` — plots.
- `output/system_info.json` — system and toolchain details.
- `output/fit_params.json` — fitted parameters and plot paths.

## License
This project is for educational purposes within course requirements. No external copyrighted datasets are included.

