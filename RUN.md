RUNBOOK — N-body simulation (production + defense)

This runbook is written so you can demonstrate and defend the project during a presentation.
It contains exact commands to run the production implementations (Python and Rust), what to show, the expected outputs, quick validation checks, and short talking points for each step.

Prerequisites
- Windows PowerShell
- Python 3.10+ (recommended) with pip
- Rust toolchain (rustup + cargo)
- Enough RAM for chosen N

Directory layout (relevant)
- `python_impl/nbody_py/` — Python production code (seq + multiprocessing)
- `rust_impl/` — Rust production code (seq + threads)
- `configs/default.toml` — default simulation parameters

1) Quick verification (5 minutes)

a) Python sequential smoke run (show simple correctness and outputs)

```powershell
cd python_impl
python -m nbody_py.cli run-seq --n 10 --steps 10 --dt 0.001 --eps 0.01 --dump-every 5 --seed 42 --out ..\data\outputs\py_demo_seq
```

What to show:
- Print the console output showing initial/final energy and "Sequential simulation completed successfully!".
- Open `data\outputs\py_demo_seq\energy.csv` and show the energy values across iterations.
- Open one `states_iter_*.csv` to show per-body states.

## RUN.md — production runbook

This file contains short, defense-ready instructions to reproduce production runs, run scaling experiments, and generate visualizations used for the project report.

Environment
- Python: use the interpreter on your machine. From the repository root run:
   - pip install -r python_impl/requirements.txt
- Rust: stable toolchain with cargo. Tested on Rust 1.70+.

Production runs (recommended)
- Rust (release): build and run the release binary. Example sequential production run:
   - cd rust_impl; cargo build --release
   - target/release/nbody-rs run-seq --n 1000 --steps 5000 --dt 0.001 --eps 0.01 --dump-every 100 --seed 42 --out data/outputs/prod_rs_run
- Rust parallel:
   - target/release/nbody-rs run-par --n 1000 --steps 5000 --dt 0.001 --eps 0.01 --procs 8 --dump-every 100 --seed 42 --out data/outputs/prod_rs_run_par
- Python sequential (stable fallback):
   - cd python_impl
   - python -m nbody_py.cli run-seq --n 1000 --steps 5000 --dt 0.001 --eps 0.01 --dump-every 100 --seed 42 --out ../data/outputs/prod_py_run_seq
- Python multiprocessing (shared-memory):
   - cd python_impl
   - python -m nbody_py.cli run-mp --n 1000 --steps 5000 --dt 0.001 --eps 0.01 --procs 8 --dump-every 100 --seed 42 --out ../data/outputs/prod_py_run_mp

Scaling experiments (strong & weak)
- Drivers: scripts/run_strong_scaling.py and scripts/run_weak_scaling.py
- Example (quick validation):
   - python scripts/run_strong_scaling.py --n 1000 --steps 1000 --repeats 1
   - python scripts/run_weak_scaling.py --base-n 250 --steps 500 --repeats 1
- Full statistical runs (defense):
   - increase --repeats to 10 or 30; be aware this is time-consuming.
   - output CSVs are written to scripts/results/strong_scaling.csv and scripts/results/weak_scaling.csv

Visualization
- Rust visualizer binary (adds scaling plot support):
   - cd rust_impl; cargo build --bin nbody-viz --release
   - target/release/nbody-viz <path-to-run-output> [--scaling-csv <scripts/results/strong_scaling.csv>]
- The tool writes PNGs into <run>/viz/: energy.png, trajectories.png, scaling_amdal.png, scaling_gustafson.png (when scaling CSV provided).

Practical tips
- Use --seed for reproducibility.
- Set dump_every to a value that balances I/O and checkpointing (e.g., 50–200).
- For very large N prefer Rust release builds and avoid Python MP for the largest experiments unless you have ample RAM and CPU.

Next steps (optional, recommended for defense)
- Run full experiments with repeats=10 for both strong and weak scaling, collect mean/std, and include scaling plots in your presentation.
- If Python MP remains unstable at your target N, switch to shared-memory memmap strategy (implemented in python_impl/nbody_py/simulate_mp.py) or run the Python sequential as a fallback.

Coverage
- This runbook covers:
   - producing production outputs (Rust and Python),
   - running scaling experiments,
   - generating visualizations used in the project report.

If you'd like, I can now run the full experiments (repeats=10) and produce the aggregated plots and a short Markdown report — confirm and I'll start (these runs are time-consuming).

Appendix: Files you will show during defense
- `python_impl/nbody_py/simulate_seq.py` and `simulate_mp.py` (briefly explain architecture).
- `rust_impl/src/sim_runner.rs`, `force_naive.rs`, `integrators.rs` (highlight parallel path).
- Example outputs: `data/outputs/.../energy.csv`, `states_iter_000100.csv`, `run_meta.json`.

---
I will now run the Python multiprocessing demo (small N) and the Rust demos were already executed successfully earlier; I will report back the output file locations when finished.
RUN instructions for current project state

This file explains how to run the current repository's Python and Rust implementations (minimal, smoke-test runs).

Python (smoke run)

1. From repository root, change into the Python implementation folder:

   cd python_impl

2. (Optional) Create and activate a virtual environment if you want isolation:

   python -m venv .venv
   .\.venv\Scripts\activate

3. Install required Python packages (one-time):

   pip install -r requirements.txt

4. Run a short sequential smoke simulation (small N/steps). This will create an output directory and write CSV files:

   python -m nbody_py.cli run-seq --n 10 --steps 10 --dt 0.001 --eps 0.01 --dump-every 5 --seed 42 --out ..\data\outputs\py_smoke_seq

5. Run a short multiprocessing smoke simulation:

   python -m nbody_py.cli run-mp --n 10 --steps 10 --dt 0.001 --eps 0.01 --procs 2 --dump-every 5 --seed 42 --out ..\data\outputs\py_smoke_mp

Outputs produced:
- CSV state dumps: `states_iter_000000.csv`, `states_iter_000005.csv`, ...
- `energy.csv` in output folder
- `run_meta.json` in output folder

Rust (smoke run)

1. From repository root, change into Rust folder and build release binary (recommended):

   cd rust_impl
   cargo build --release

2. Run a short sequential smoke simulation:

   ..\target\release\nbody-rs run-seq -n 10 -s 10 --dt 0.001 --eps 0.01 --dump-every 5 --seed 42 -o ..\data\outputs\rs_smoke_seq

3. Run a short threaded smoke simulation (2 threads):

   ..\target\release\nbody-rs run-par -n 10 -s 10 --dt 0.001 --eps 0.01 --threads 2 --dump-every 5 --seed 42 -o ..\data\outputs\rs_smoke_par

Notes
- The repository currently contains only production source code for Python and Rust implementations.
- If you run into permission or PATH issues, ensure the Python executable and Cargo are available in your shell environment.
- Large runs require adequate RAM; tune `--dump-every` to reduce I/O.

If you want, I will now perform the Python smoke run from `python_impl` and report the output.
# How to run the N-body simulation (production)

This repository contains two production-grade implementations of an N-body simulator:
- Python implementation: `python_impl/nbody_py`
- Rust implementation: `rust_impl`

This `RUN.md` describes minimal steps to build and run production simulations for both implementations.

## Python (production run)

Prerequisites:
- Python 3.10+ installed

Steps:
1. Enter the Python implementation folder:

```powershell
cd python_impl
```

2. Create and activate a virtual environment and install dependencies:

```powershell
python -m venv .venv
.\.venv\Scripts\activate
pip install -r requirements.txt
```

3. Run a sequential simulation (example):

```powershell
python -m nbody_py.cli run-seq --n 1000 --steps 1000 --dt 0.001 --eps 0.01 --dump-every 100 --seed 42 --out ../data/outputs/py_seq
```

4. Run a multiprocessing simulation (production):

```powershell
python -m nbody_py.cli run-mp --n 10000 --steps 2000 --dt 0.001 --eps 0.01 --procs 8 --dump-every 100 --seed 42 --out ../data/outputs/py_mp
```

Outputs: CSV files `states_iter_*.csv`, `energy.csv`, and `run_meta.json` in the chosen `--out` folder.

Notes:
- For very large `N`, consider increasing system memory and using `--dump-every` to reduce I/O.
- This build is intended for production runs; tests and development scripts were intentionally removed.

## Rust (production run)

Prerequisites:
- Rust toolchain (rustup, cargo)

Steps:
1. Build the release binary:

```powershell
cd rust_impl
cargo build --release
```

2. Run the binary (sequential):

```powershell
..\target\release\nbody-rs run-seq -n 10000 -s 2000 --dt 0.001 --eps 0.01 --dump-every 100 --seed 42 -o ..\data\outputs\rs_seq
```

3. Run the parallel build (threads):

```powershell
..\target\release\nbody-rs run-par -n 20000 -s 5000 --dt 0.001 --eps 0.01 --threads 8 --dump-every 100 --seed 42 -o ..\data\outputs\rs_par
```

Outputs: CSV files `states_iter_*.csv`, `energy.csv`, and `run_meta.json` in the chosen `-o` folder.

Notes:
- The Rust implementation is optimized for production workloads (release build with LTO and opt-level=3 in `Cargo.toml`).
- For very large runs, monitor memory and consider `--dump-every` to reduce I/O.

## Final notes
- I removed auxiliary test files and example scripts to keep the repository lean for production.
- If you need the development/test artifacts back, they can be restored from version control history.

## Scaling experiments (production)

This project includes drivers to run strong- and weak-scaling experiments that compare Python (multiprocessing) vs Rust (threaded) implementations using production parameters.

Files:
- `scripts/run_strong_scaling.py` — strong-scaling driver (vary p, keep N fixed)
- `scripts/run_weak_scaling.py` — weak-scaling driver (scale N with p)
- `scripts/experiment_utils.py` — helper utilities

Results are written to CSV under `scripts/results/` (e.g. `strong_scaling.csv`, `weak_scaling.csv`). Each row contains: lang, mode, p, n, steps, dt, eps, mean_s, std_s, repeats.

Run (example, single repeat):

```powershell
cd <repo-root>
python scripts/run_strong_scaling.py --n 1000 --steps 1000 --repeats 1
python scripts/run_weak_scaling.py --base-n 250 --steps 500 --repeats 1
```

Notes and interpretation:
- The drivers build the Rust release binary before running Rust experiments. Make sure `cargo` and MSVC toolchain are installed.
- Python experiments invoke `python -m nbody_py.cli run-mp` (multiprocessing) and write per-run outputs under `data/outputs/` alongside aggregated timings in `scripts/results/`.
- Use `scripts/results/*.csv` to produce Amdahl/Gustafson plots (these drivers generate the timing numbers; plotting is left for analysis).

If you want, I can run a full production strong-scaling sweep (p=1,2,4,8 with repeats=10) now and collect `scripts/results/strong_scaling.csv`. This may take a long time depending on your machine.
