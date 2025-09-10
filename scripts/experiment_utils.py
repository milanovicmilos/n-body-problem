"""Utilities used by the scaling experiment drivers.

Provides functions to run external simulation commands (Python and Rust),
measure wall time, and aggregate results into CSV.
"""
import csv
import subprocess
import sys
import time
from statistics import mean, stdev
from pathlib import Path


def run_command(cmd, cwd=None, timeout=None):
    """Run command (list) and return (returncode, stdout, stderr, duration_s)."""
    start = time.perf_counter()
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        rc = p.returncode
        out = p.stdout
        err = p.stderr
    except subprocess.TimeoutExpired as e:
        rc = -1
        out = e.stdout or ""
        err = (e.stderr or "") + f"\nTIMEOUT after {timeout}s"
    duration = time.perf_counter() - start
    return rc, out, err, duration


def write_rows_csv(path: Path, rows, header=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        if header:
            w.writerow(header)
        for r in rows:
            w.writerow(r)


def aggregate_results(perf_map):
    """Aggregate a mapping (key -> list of durations) into summary rows.

    perf_map keys: tuples (lang, mode, p, n, steps, dt, eps)
    returns list of summary dicts
    """
    summary = []
    for key, durations in perf_map.items():
        lang, mode, p, n, steps, dt, eps = key
        if not durations:
            continue
        m = mean(durations)
        s = stdev(durations) if len(durations) > 1 else 0.0
        summary.append({
            "lang": lang,
            "mode": mode,
            "p": p,
            "n": n,
            "steps": steps,
            "dt": dt,
            "eps": eps,
            "mean_s": m,
            "std_s": s,
            "repeats": len(durations),
        })
    return summary
