# N-Body Simulation: Python vs Rust — Strong and Weak Scaling Report

## Overview
This repository contains N-body simulations implemented in Python and Rust, each with sequential and parallel modes. We performed strong and weak scaling experiments, computed statistical summaries over repeated runs (~30), fitted parallel fractions (Amdahl for strong, Gustafson for weak), and generated plots comparing measured speedups with ideal and fitted models.

## Project Structure
- `python/` — Python package with `nbody` module and CLI (`python/main.py`).
- `rust/` — Rust crate with library (`src/lib.rs`) and CLI (`src/main.rs`).
- `benchmarks/bench.py` — Benchmark harness for strong/weak scaling, plots, and summaries.
- `output/` — Results: CSVs, plots, system info, and fitted parameters.

## How to Run
PowerShell examples (correct flag names are shown):

Recommended pretty demo (balanced motion, clean visuals):

1) Run Rust simulation (50 bodies, moderate params):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --random 50 --steps 300 --dt 0.005 --G 3.0 --softening 0.15 --pos-range -10 10 --vel-range -0.2 0.2 --mode seq --output .\output\rs_seq_50.csv`

2) Visualize the Rust CSV (per-frame bounds, trails, faster GIF):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\rs_seq_50.csv --vis-size 3.0 --vis-trails 6 --gif-ms 60 --vis-bounds per-frame --vis-pad 0.05`

Optional: Python simulation with the same settings and visualization:

3) Run Python simulation (50 bodies, moderate params):
`python .\python\main.py --random 50 --steps 300 --dt 0.005 --G 3.0 --softening 0.15 --pos-range -10 10 --vel-range -0.2 0.2 --mode seq --output .\output\py_seq_50.csv`

4) Visualize the Python CSV:
`cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\py_seq_50.csv --vis-size 3.0 --vis-trails 6 --gif-ms 60 --vis-bounds per-frame --vis-pad 0.05`

Other examples:

1) Run Python simulation (sequential):
`python .\python\main.py --random 200 --steps 100 --dt 0.002 --mode seq --output .\output\py_seq.csv`

2) Run Python simulation (multiprocessing):
`python .\python\main.py --random 200 --steps 100 --dt 0.002 --mode mp --workers 4 --output .\output\py_mp.csv`

3) Run Rust simulation (sequential):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --random 200 --steps 100 --dt 0.002 --mode seq --output .\output\rs_seq.csv`

4) Run Rust simulation (threaded with rayon):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --random 200 --steps 100 --dt 0.002 --mode threads --output .\output\rs_threads.csv`

5) Run benchmarks (default settings):
`python .\benchmarks\bench.py`

6) Run benchmarks (custom):
`python .\benchmarks\bench.py --repeats 30 --workers 1 2 4 8 --problem-n 800 --base-n 200 --steps 200 --dt 0.002`

Outputs are written to `output/`.

7) Visualize an existing CSV (generate per-iteration PNG frames + animated GIF):
`cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\rs_seq.csv`

	- Visualization now draws bodies as filled colored circles (color assigned per body id) rather than simple markers. You can control the visual size of bodies with `--vis-size` (float multiplier; larger -> bigger circles). Example:

	`cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\rs_seq.csv --vis-size 4.0`

---

## Command-line arguments
This project exposes three entry points with command-line options: the Python simulation CLI (`python/main.py`), the benchmark harness (`benchmarks/bench.py`) and the Rust binary (`cargo run` invoking `rust` crate). Below are the available flags, defaults, and example usages.

- Python simulation (`python/main.py`)

	- Description: runs the Python N-body simulation; either load bodies from a JSON string (`--bodies`) or generate random bodies (`--random`).
	- Common flags:
		- `--mode` (choices: `seq`, `mp`) — execution mode. Default: `seq`.
		- `--steps` (int) — number of iterations. Default: `100`.
		- `--dt` (float) — timestep size. Default: `0.01`.
		- `--G` (float) — gravitational constant. Default: `1.0`.
		- `--softening` (float) — softening to avoid singularities. Default: `1e-9`.
		- `--output` (str) — output CSV path. Default: `output/nbody_python_seq.csv` (if `--mode mp` and the default is used, the program writes `output/nbody_python_mp.csv`).
		- `--workers` (int) — number of worker processes for `mp` mode (0 = auto). Default: `0`.
		- `--bodies` (str) — a JSON array string with body objects: `[{"m":..,"x":..,"y":..,"z":..,"vx":..,"vy":..,"vz":..}]`.
		- `--random` (int) — generate this many random bodies (use instead of `--bodies`). Default: `0`.
		- `--seed` (int) — RNG seed for `--random`. Default: `42`.
		- `--mass-range` (two floats) — mass range for random bodies. Default: `1.0 10.0`.
		- `--pos-range` (two floats) — position range for random bodies. Default: `-1.0 1.0`.
		- `--vel-range` (two floats) — velocity range for random bodies. Default: `-0.1 0.1`.

	- Example (multiprocessing, 500 bodies, 200 steps):
		`python .\python\main.py --random 500 --mode mp --workers 8 --steps 200 --dt 0.005 --output .\output\py_mp.csv`

- Benchmark harness (`benchmarks/bench.py`)

	- Description: runs strong and weak scaling experiments by calling the Python and Rust entry points repeatedly, collecting timings and plotting summaries.
	- Common flags:
		- `--steps` (int) — steps for each run. Default: `120`.
		- `--dt` (float) — timestep size. Default: `0.002`.
		- `--workers` (list of ints) — worker counts to test. Default: `1 2 4 8`.
		- `--problem-n` (int) — N for strong scaling. Default: `200`.
		- `--base-n` (int) — base N for weak scaling, actual N = `base_n * workers`. Default: `50`.
		- `--repeats` (int) — repeat each configuration this many times. Default: `5` (the published report used `30`).
		- `--no-python` — skip Python benchmarks.
		- `--no-rust` — skip Rust benchmarks.

	- Notes: the harness uses `RAYON_NUM_THREADS` to control Rust's Rayon thread pool when testing threaded runs. The script assumes `cargo` on PATH and will write CSV/plots under `output/`.
		- Notes:
			- The harness measures a sequential baseline (w=1) and then parallel runs for workers ≥ 2; speedups are normalized so S(1)=1.
			- For Rust, the harness builds once (`cargo build --release`) and then runs the compiled binary to avoid repeated `cargo run` overhead.
			- For small N, multiprocessing/threading overhead on Windows can dominate; prefer larger N/steps for meaningful scaling (e.g., `--problem-n 1200 --base-n 300 --steps 120`).

	- Example (run full report with 30 repeats):
			`python .\benchmarks\bench.py --repeats 30 --workers 2 4 8 --problem-n 1200 --base-n 300 --steps 120 --dt 0.002`

- Rust binary (`cargo run` / `rust` crate)

	- Description: the Rust program supports sequential and threaded (Rayon) execution, CSV output per iteration, and a visualization helper that converts an existing CSV into PNG frames and a GIF.
	- Common flags (passed after `--` when using `cargo run`):
	- `--visualize <path>` — path to an existing CSV to create per-iteration PNG frames and an animated GIF. If provided the program will run the visualization helper and exit.
	- `--vis-size <float>` — visualization size multiplier for body radii (default: `2.0`). Larger values make bodies appear bigger in the PNG/GIF frames.
	- `--vis-trails <usize>` — draw motion trails by overlaying previous frames with fading alpha (default: `0`, meaning disabled).
	- `--gif-ms <u16>` — GIF delay in milliseconds per frame (default: `100`). Smaller values speed up playback.
	- `--vis-bounds {per-frame|global|initial}` — sets axis bounds per frame (auto), from the whole run, or only from iteration 0 (default: `per-frame`).
	- `--vis-pad <float>` — padding fraction for bounds (e.g., `0.05` adds 5% margin; default: `0.05`).
		- `--mode` (choices: `seq`, `threads`) — execution mode. Default: `seq`.
		- `--steps` (usize) — number of iterations. Default: `100`.
		- `--dt` (f64) — timestep. Default: `0.01`.
		- `--G` (f64) — gravitational constant. Default: `1.0`.
		- `--softening` (f64) — softening term. Default: `1e-9`.
		- `--output` (str) — output CSV path. Default: `output/nbody_rust_seq.csv` (when `--mode threads` and the default is used, the program writes `output/nbody_rust_threads.csv`).
		- `--bodies` (str) — JSON array string with initial bodies.
		- `--random` (usize) — generate this many random bodies. Default: `0`.
		- `--seed` (u64) — RNG seed for `--random`. Default: `42`.
		- `--mass-range`, `--pos-range`, `--vel-range` (two floats each) — ranges for random bodies. Defaults:
		  - `--mass-range 1.0 10.0`
		  - `--pos-range -50.0 50.0`
		  - `--vel-range -0.1 0.1`
		- `--quiet` — suppress the per-run printed timing output (useful for harnesses).

	- Controlling threads: Set `RAYON_NUM_THREADS` to limit Rayon worker threads for `threads` mode, e.g. in PowerShell:
		`$env:RAYON_NUM_THREADS = "8"; cargo run --release --manifest-path .\rust\Cargo.toml -- --mode threads --random 200 --steps 100 --dt 0.002`

	- Example (visualize CSV):
		`cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\rs_seq.csv`

Visuals with more apparent motion: try running the simulation with a slightly larger timestep, stronger gravity, tighter position range, and higher initial velocities; then render with trails and faster GIF:

```
# Run sim (more dynamic motion)
cargo run --release --manifest-path .\rust\Cargo.toml -- --random 200 --steps 300 --dt 0.01 --G 10.0 --pos-range -10 10 --vel-range -0.5 0.5 --mode seq --output .\output\rs_seq.csv

# Visualize with trails, faster playback, and per-frame bounds
cargo run --release --manifest-path .\rust\Cargo.toml -- --visualize .\output\rs_seq.csv --vis-size 3.0 --vis-trails 5 --gif-ms 60 --vis-bounds per-frame --vis-pad 0.05
```

If you notice any mismatch between the examples above and the flags you prefer, the code in `python/main.py`, `benchmarks/bench.py` and `rust/src/main.rs` is the authoritative source for flag names and defaults.

## System Details
Collected automatically in `output/system_info.json`. Example from this machine:
- Python: see `python`
- Platform: see `platform`
- CPU: see `cpu` (cores/logical threads, caches, frequency)
- RAM: see `ram`
- OS: see `os`
- Rust toolchain: `cargo_version`, `rustc_version`

## Methodology
- Integrator: velocity Verlet (symplectic) for better energy behavior; gravitational interactions with softening; O(N^2) per step.
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

