"""
Metrics and analysis utilities for N-body simulation
"""
import pandas as pd
import numpy as np
from typing import Dict, Any, Tuple


def analyze_energy_conservation(energy_file: str) -> Dict[str, float]:
    """
    Analyze energy conservation from energy CSV file
    
    Args:
        energy_file: Path to energy.csv file
        
    Returns:
        Dictionary with energy conservation metrics
    """
    df = pd.read_csv(energy_file)
    
    initial_total = df['total'].iloc[0]
    final_total = df['total'].iloc[-1]
    
    # Energy drift
    energy_drift = final_total - initial_total
    relative_drift = abs(energy_drift) / abs(initial_total) * 100
    
    # Energy variance
    energy_variance = df['total'].var()
    energy_std = df['total'].std()
    
    # Maximum deviation
    max_deviation = (df['total'] - initial_total).abs().max()
    max_relative_deviation = max_deviation / abs(initial_total) * 100
    
    return {
        'initial_energy': initial_total,
        'final_energy': final_total,
        'energy_drift': energy_drift,
        'relative_drift_percent': relative_drift,
        'energy_variance': energy_variance,
        'energy_std': energy_std,
        'max_deviation': max_deviation,
        'max_relative_deviation_percent': max_relative_deviation
    }


def compare_simulations(energy_file1: str, energy_file2: str) -> Dict[str, Any]:
    """
    Compare two simulation results
    
    Args:
        energy_file1: Path to first energy.csv file
        energy_file2: Path to second energy.csv file
        
    Returns:
        Dictionary with comparison metrics
    """
    df1 = pd.read_csv(energy_file1)
    df2 = pd.read_csv(energy_file2)
    
    # Ensure same number of steps
    min_len = min(len(df1), len(df2))
    df1 = df1.iloc[:min_len]
    df2 = df2.iloc[:min_len]
    
    # Energy differences
    total_diff = df1['total'] - df2['total']
    kinetic_diff = df1['kinetic'] - df2['kinetic']
    potential_diff = df1['potential'] - df2['potential']
    
    return {
        'total_energy_rmse': np.sqrt(np.mean(total_diff**2)),
        'kinetic_energy_rmse': np.sqrt(np.mean(kinetic_diff**2)),
        'potential_energy_rmse': np.sqrt(np.mean(potential_diff**2)),
        'max_total_diff': total_diff.abs().max(),
        'mean_total_diff': total_diff.mean(),
        'std_total_diff': total_diff.std()
    }


def analyze_performance(metadata_file: str) -> Dict[str, Any]:
    """
    Extract performance metrics from metadata file
    
    Args:
        metadata_file: Path to run_meta.json file
        
    Returns:
        Dictionary with performance metrics
    """
    import json
    
    with open(metadata_file, 'r') as f:
        metadata = json.load(f)
    
    params = metadata['parameters']
    execution_time = metadata.get('execution_time_seconds', 0)
    
    n = params['n']
    steps = params['steps']
    
    # Calculate performance metrics
    total_operations = n * (n - 1) * steps  # O(N^2) per step
    operations_per_second = total_operations / execution_time if execution_time > 0 else 0
    time_per_step = execution_time / steps if steps > 0 else 0
    time_per_particle_per_step = time_per_step / n if n > 0 else 0
    
    return {
        'execution_time_seconds': execution_time,
        'total_operations': total_operations,
        'operations_per_second': operations_per_second,
        'time_per_step': time_per_step,
        'time_per_particle_per_step': time_per_particle_per_step,
        'n_bodies': n,
        'n_steps': steps
    }


def validate_two_body_orbit(states_dir: str, body1_id: int = 0, body2_id: int = 1) -> Dict[str, float]:
    """
    Validate two-body orbital mechanics (for testing)
    
    Args:
        states_dir: Directory containing state files
        body1_id: ID of first body
        body2_id: ID of second body
        
    Returns:
        Dictionary with orbital validation metrics
    """
    import os
    import glob
    
    # Load all state files
    state_files = sorted(glob.glob(os.path.join(states_dir, "states_iter_*.csv")))
    
    distances = []
    relative_positions = []
    
    for state_file in state_files:
        df = pd.read_csv(state_file)
        
        # Get positions of the two bodies
        body1 = df[df['id'] == body1_id].iloc[0]
        body2 = df[df['id'] == body2_id].iloc[0]
        
        # Calculate distance
        dx = body2['x'] - body1['x']
        dy = body2['y'] - body1['y']
        dz = body2['z'] - body1['z']
        distance = np.sqrt(dx*dx + dy*dy + dz*dz)
        distances.append(distance)
        relative_positions.append((dx, dy, dz))
    
    distances = np.array(distances)
    
    # Analyze orbital characteristics
    mean_distance = distances.mean()
    distance_variance = distances.var()
    max_distance = distances.max()
    min_distance = distances.min()
    
    # Eccentricity estimate (simplified)
    eccentricity = (max_distance - min_distance) / (max_distance + min_distance)
    
    return {
        'mean_separation': mean_distance,
        'distance_variance': distance_variance,
        'max_separation': max_distance,
        'min_separation': min_distance,
        'estimated_eccentricity': eccentricity,
        'separation_stability': distance_variance / mean_distance**2
    }
