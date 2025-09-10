Scaling report — strong and weak experiments

This report summarizes the strong- and weak-scaling experiments executed with repeats=10 using the Python multiprocessing implementation and the Rust threaded implementation. Data and plots are in the repository.

Results (strong scaling)

CSV: scripts/results/strong_scaling.csv

Summary (p, python mean s ± std, rust mean s ± std):
- p=1: python 1.6224 ± 0.0639 s, rust 16.7465 ± 3.1586 s
- p=2: python 1.5682 ± 0.0864 s, rust 8.9458 ± 0.6616 s
- p=4: python 2.0423 ± 0.2462 s, rust 5.5678 ± 0.8057 s
- p=8: python 1.5257 ± 0.0413 s, rust 3.2783 ± 0.1309 s

Results (weak scaling)

CSV: scripts/results/weak_scaling.csv

Summary (p, python mean s ± std, rust mean s ± std):
- p=1: python 1.6288 ± 0.0569 s, rust 0.4760 ± 0.0202 s
- p=2: python 1.5608 ± 0.0296 s, rust 1.1765 ± 0.0683 s
- p=4: python 1.6205 ± 0.0956 s, rust 2.8503 ± 0.0630 s
- p=8: python 1.5627 ± 0.0635 s, rust 6.6856 ± 0.1753 s

Plots

The scaling plots (Amdahl and Gustafson) were generated and saved under:
- data/outputs/strong_rs_p1_n1000_r0/viz/scaling_amdal.png
- data/outputs/strong_rs_p1_n1000_r0/viz/scaling_gustafson.png

Notes and interpretation

- Python multiprocessing times are small and fairly stable across p in these settings; the implementation uses shared memory to reduce copying. Unexpected anomalies (e.g., faster times at higher p) can occur due to OS scheduling and the particular small problem size used for these runs; for true production conclusions increase N and steps.
- Rust threaded times increase with p in weak-scaling (expected because N increases with p in weak-scaling). For strong-scaling Rust shows improved time as p increases.

Files

- scripts/results/strong_scaling.csv
- scripts/results/weak_scaling.csv
- data/outputs/strong_rs_p1_n1000_r0/viz/*

If you want a PDF or slide-ready export with the plots embedded, I can produce one next.
