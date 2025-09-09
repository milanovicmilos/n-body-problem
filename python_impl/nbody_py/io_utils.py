"""
Input/Output utilities for N-body simulation
"""
import os
import json
import csv
import platform
import time
from typing import Dict, Any
import pandas as pd
from .physics import NBodySystem


def create_output_directory(output_dir: str):
    """Create output directory if it doesn't exist"""
    os.makedirs(output_dir, exist_ok=True)


def save_system_state(system: NBodySystem, iteration: int, output_dir: str):
    """Save current system state to CSV file"""
    filename = f"states_iter_{iteration:06d}.csv"
    filepath = os.path.join(output_dir, filename)
    
    data = []
    for i in range(system.n):
        data.append({
            'iter': iteration,
            'id': i,
            'x': system.x[i],
            'y': system.y[i], 
            'z': system.z[i],
            'vx': system.vx[i],
            'vy': system.vy[i],
            'vz': system.vz[i],
            'm': system.m[i]
        })
    
    df = pd.DataFrame(data)
    df.to_csv(filepath, index=False)


def save_energy_data(iteration: int, kinetic: float, potential: float, output_dir: str):
    """Append energy data to CSV file"""
    filepath = os.path.join(output_dir, "energy.csv")
    
    # Check if file exists to write header
    write_header = not os.path.exists(filepath)
    
    with open(filepath, 'a', newline='') as csvfile:
        fieldnames = ['iter', 'kinetic', 'potential', 'total']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        if write_header:
            writer.writeheader()
        
        writer.writerow({
            'iter': iteration,
            'kinetic': kinetic,
            'potential': potential,
            'total': kinetic + potential
        })


def save_metadata(params: Dict[str, Any], output_dir: str, execution_time: float = None):
    """Save simulation metadata to JSON file"""
    filepath = os.path.join(output_dir, "run_meta.json")
    
    # Add system information
    metadata = {
        'parameters': params,
        'system_info': {
            'hostname': platform.node(),
            'os': platform.system(),
            'os_version': platform.version(),
            'cpu': platform.processor(),
            'python_version': platform.python_version(),
            'timestamp': time.time()
        }
    }
    
    if execution_time is not None:
        metadata['execution_time_seconds'] = execution_time
    
    with open(filepath, 'w') as f:
        json.dump(metadata, f, indent=2)


def load_initial_conditions(filepath: str) -> NBodySystem:
    """Load initial conditions from CSV file"""
    df = pd.read_csv(filepath)
    n = len(df)
    
    system = NBodySystem(n)
    
    system.x = df['x'].values
    system.y = df['y'].values
    system.z = df['z'].values
    system.vx = df['vx'].values
    system.vy = df['vy'].values
    system.vz = df['vz'].values
    system.m = df['m'].values
    
    return system


def save_initial_conditions(system: NBodySystem, output_dir: str):
    """Save initial conditions to CSV file for reproducibility"""
    filepath = os.path.join(output_dir, "initial_conditions.csv")
    
    data = []
    for i in range(system.n):
        data.append({
            'id': i,
            'x': system.x[i],
            'y': system.y[i],
            'z': system.z[i],
            'vx': system.vx[i],
            'vy': system.vy[i],
            'vz': system.vz[i],
            'm': system.m[i]
        })
    
    df = pd.DataFrame(data)
    df.to_csv(filepath, index=False)
