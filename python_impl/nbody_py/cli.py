"""
Command Line Interface for N-body simulation
"""
import argparse
import os
import sys
from .simulate_seq import run_sequential_simulation
from .simulate_mp import run_parallel_simulation
from .metrics import analyze_energy_conservation, analyze_performance


def main():
    parser = argparse.ArgumentParser(description='N-body simulation in Python')
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Sequential simulation command
    seq_parser = subparsers.add_parser('run-seq', help='Run sequential simulation')
    seq_parser.add_argument('--n', type=int, required=True, help='Number of bodies')
    seq_parser.add_argument('--steps', type=int, required=True, help='Number of simulation steps')
    seq_parser.add_argument('--dt', type=float, required=True, help='Time step size')
    seq_parser.add_argument('--eps', type=float, default=1e-2, help='Softening parameter')
    seq_parser.add_argument('--seed', type=int, default=42, help='Random seed')
    seq_parser.add_argument('--dump-every', type=int, default=10, help='Save state every N steps')
    seq_parser.add_argument('--out', type=str, required=True, help='Output directory')
    
    # Parallel simulation command
    mp_parser = subparsers.add_parser('run-mp', help='Run parallel simulation with multiprocessing')
    mp_parser.add_argument('--n', type=int, required=True, help='Number of bodies')
    mp_parser.add_argument('--steps', type=int, required=True, help='Number of simulation steps')
    mp_parser.add_argument('--dt', type=float, required=True, help='Time step size')
    mp_parser.add_argument('--eps', type=float, default=1e-2, help='Softening parameter')
    mp_parser.add_argument('--seed', type=int, default=42, help='Random seed')
    mp_parser.add_argument('--dump-every', type=int, default=10, help='Save state every N steps')
    mp_parser.add_argument('--procs', type=int, default=None, help='Number of processes')
    mp_parser.add_argument('--out', type=str, required=True, help='Output directory')
    
    # Analysis command
    analyze_parser = subparsers.add_parser('analyze', help='Analyze simulation results')
    analyze_parser.add_argument('--dir', type=str, required=True, help='Simulation output directory')
    
    args = parser.parse_args()
    
    if args.command == 'run-seq':
        params = {
            'n': args.n,
            'steps': args.steps,
            'dt': args.dt,
            'eps': args.eps,
            'seed': args.seed,
            'dump_every': args.dump_every,
            'output_dir': args.out
        }
        
        try:
            execution_time = run_sequential_simulation(params)
            print(f"\nSequential simulation completed successfully!")
            print(f"Execution time: {execution_time:.2f} seconds")
            print(f"Results saved to: {args.out}")
        except Exception as e:
            print(f"Error running sequential simulation: {e}")
            sys.exit(1)
    
    elif args.command == 'run-mp':
        import multiprocessing as mp
        
        params = {
            'n': args.n,
            'steps': args.steps,
            'dt': args.dt,
            'eps': args.eps,
            'seed': args.seed,
            'dump_every': args.dump_every,
            'procs': args.procs if args.procs else mp.cpu_count(),
            'output_dir': args.out
        }
        
        try:
            execution_time = run_parallel_simulation(params)
            print(f"\nParallel simulation completed successfully!")
            print(f"Execution time: {execution_time:.2f} seconds")
            print(f"Processes used: {params['procs']}")
            print(f"Results saved to: {args.out}")
        except Exception as e:
            print(f"Error running parallel simulation: {e}")
            sys.exit(1)
    
    elif args.command == 'analyze':
        try:
            energy_file = os.path.join(args.dir, 'energy.csv')
            metadata_file = os.path.join(args.dir, 'run_meta.json')
            
            if os.path.exists(energy_file):
                energy_analysis = analyze_energy_conservation(energy_file)
                print("Energy Conservation Analysis:")
                print(f"  Initial energy: {energy_analysis['initial_energy']:.6f}")
                print(f"  Final energy: {energy_analysis['final_energy']:.6f}")
                print(f"  Energy drift: {energy_analysis['energy_drift']:.6f}")
                print(f"  Relative drift: {energy_analysis['relative_drift_percent']:.4f}%")
                print(f"  Max deviation: {energy_analysis['max_relative_deviation_percent']:.4f}%")
            
            if os.path.exists(metadata_file):
                perf_analysis = analyze_performance(metadata_file)
                print("\nPerformance Analysis:")
                print(f"  Execution time: {perf_analysis['execution_time_seconds']:.2f} seconds")
                print(f"  Time per step: {perf_analysis['time_per_step']:.6f} seconds")
                print(f"  Operations per second: {perf_analysis['operations_per_second']:.0f}")
        
        except Exception as e:
            print(f"Error analyzing results: {e}")
            sys.exit(1)
    
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
