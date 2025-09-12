import argparse
import os
import sys
from typing import List

try:
    import multiprocessing as mp  # for Windows freeze_support
except Exception:
    mp = None  # type: ignore

from nbody.model import parse_bodies_from_json, random_bodies
from nbody.sim import simulate


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(description="N-body simulation (Python)")
    parser.add_argument("--mode", choices=["seq", "mp"], default="seq", help="Execution mode: sequential (seq) or multiprocessing (mp)")
    parser.add_argument("--steps", type=int, default=100, help="Number of iterations")
    parser.add_argument("--dt", type=float, default=0.01, help="Timestep size")
    parser.add_argument("--G", type=float, default=1.0, help="Gravitational constant")
    parser.add_argument("--softening", type=float, default=1e-9, help="Softening term to avoid singularities")
    parser.add_argument("--output", type=str, default=os.path.join("output", "nbody_python_seq.csv"), help="Output CSV path")
    parser.add_argument("--workers", type=int, default=0, help="Number of worker processes for mp mode (0 = auto)")

    # Initial conditions: either --bodies JSON or --random N with ranges
    parser.add_argument("--bodies", type=str, default="", help="JSON array of bodies: [{\"m\":..,\"x\":..,\"y\":..,\"z\":..,\"vx\":..,\"vy\":..,\"vz\":..}]")
    parser.add_argument("--random", type=int, default=0, help="If >0, generate this many random bodies")
    parser.add_argument("--seed", type=int, default=42, help="Random seed when using --random")
    parser.add_argument("--mass-range", type=float, nargs=2, default=[1.0, 10.0], help="Mass range for random bodies [min max]")
    parser.add_argument("--pos-range", type=float, nargs=2, default=[-1.0, 1.0], help="Position range for random bodies [min max]")
    parser.add_argument("--vel-range", type=float, nargs=2, default=[-0.1, 0.1], help="Velocity range for random bodies [min max]")

    args = parser.parse_args(argv)

    if args.bodies:
        bodies = parse_bodies_from_json(args.bodies)
    elif args.random > 0:
        bodies = random_bodies(
            args.random,
            (args.__dict__["mass_range"][0], args.__dict__["mass_range"][1]),
            (args.__dict__["pos_range"][0], args.__dict__["pos_range"][1]),
            (args.__dict__["vel_range"][0], args.__dict__["vel_range"][1]),
            args.seed,
        )
    else:
        print("Error: Provide either --bodies JSON or --random N", file=sys.stderr)
        return 2

    # Adjust default output name based on mode if user didn't override
    out_path = args.output
    if out_path == os.path.join("output", "nbody_python_seq.csv") and args.mode == "mp":
        out_path = os.path.join("output", "nbody_python_mp.csv")

    if args.mode == "mp":
        if mp is None:
            print("multiprocessing not available; falling back to seq", file=sys.stderr)
            args.mode = "seq"
        else:
            mp.freeze_support()  # Windows requirement

    elapsed = simulate(
        bodies=bodies,
        steps=args.steps,
        dt=args.dt,
        G=args.G,
        softening=args.softening,
        mode=args.mode,
        out_path=out_path,
        workers=args.workers,
    )

    print(f"Mode={args.mode} Bodies={len(bodies)} Steps={args.steps} dt={args.dt} ElapsedSeconds={elapsed:.6f}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
