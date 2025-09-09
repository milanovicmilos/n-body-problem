"""
N-body simulation physics calculations and data structures
"""
import numpy as np
import math
import random
from typing import Tuple, List


class NBodySystem:
    """Structure of Arrays (SoA) representation for N-body system"""
    
    def __init__(self, n: int):
        self.n = n
        # Positions
        self.x = np.zeros(n, dtype=np.float64)
        self.y = np.zeros(n, dtype=np.float64) 
        self.z = np.zeros(n, dtype=np.float64)
        
        # Velocities
        self.vx = np.zeros(n, dtype=np.float64)
        self.vy = np.zeros(n, dtype=np.float64)
        self.vz = np.zeros(n, dtype=np.float64)
        
        # Masses
        self.m = np.zeros(n, dtype=np.float64)
        
        # Accelerations (computed)
        self.ax = np.zeros(n, dtype=np.float64)
        self.ay = np.zeros(n, dtype=np.float64)
        self.az = np.zeros(n, dtype=np.float64)
    
    def initialize_plummer_sphere(self, seed: int = 42):
        """Initialize system with Plummer sphere distribution"""
        random.seed(seed)
        np.random.seed(seed)
        
        # Plummer sphere parameters
        total_mass = 1.0
        plummer_radius = 1.0
        
        for i in range(self.n):
            # Mass: equal for all bodies
            self.m[i] = total_mass / self.n
            
            # Position: sample from Plummer sphere
            # Using rejection sampling for simplicity
            while True:
                r = np.random.uniform(0, 10 * plummer_radius)
                density = (3 * total_mass / (4 * math.pi * plummer_radius**3)) * \
                         (1 + (r/plummer_radius)**2)**(-5/2)
                if np.random.uniform(0, 1) < density / (3 * total_mass / (4 * math.pi * plummer_radius**3)):
                    break
            
            # Random direction
            theta = np.random.uniform(0, 2 * math.pi)
            phi = np.random.uniform(0, math.pi)
            
            self.x[i] = r * math.sin(phi) * math.cos(theta)
            self.y[i] = r * math.sin(phi) * math.sin(theta)
            self.z[i] = r * math.cos(phi)
            
            # Velocity: simple random initialization
            self.vx[i] = np.random.normal(0, 0.1)
            self.vy[i] = np.random.normal(0, 0.1) 
            self.vz[i] = np.random.normal(0, 0.1)
    
    def compute_kinetic_energy(self) -> float:
        """Compute total kinetic energy"""
        return 0.5 * np.sum(self.m * (self.vx**2 + self.vy**2 + self.vz**2))
    
    def compute_potential_energy(self, eps: float = 1e-2) -> float:
        """Compute total gravitational potential energy"""
        G = 1.0  # Gravitational constant
        potential = 0.0
        
        for i in range(self.n):
            for j in range(i + 1, self.n):
                dx = self.x[j] - self.x[i]
                dy = self.y[j] - self.y[i]
                dz = self.z[j] - self.z[i]
                r2 = dx*dx + dy*dy + dz*dz + eps*eps
                r = math.sqrt(r2)
                potential -= G * self.m[i] * self.m[j] / r
        
        return potential
    
    def compute_center_of_mass(self) -> Tuple[float, float, float]:
        """Compute center of mass"""
        total_mass = np.sum(self.m)
        cm_x = np.sum(self.m * self.x) / total_mass
        cm_y = np.sum(self.m * self.y) / total_mass
        cm_z = np.sum(self.m * self.z) / total_mass
        return cm_x, cm_y, cm_z
    
    def compute_total_momentum(self) -> Tuple[float, float, float]:
        """Compute total momentum"""
        px = np.sum(self.m * self.vx)
        py = np.sum(self.m * self.vy)
        pz = np.sum(self.m * self.vz)
        return px, py, pz


def compute_accelerations_naive(system: NBodySystem, eps: float = 1e-2, G: float = 1.0):
    """
    Compute gravitational accelerations using naive O(N^2) algorithm
    """
    # Reset accelerations
    system.ax.fill(0.0)
    system.ay.fill(0.0)
    system.az.fill(0.0)
    
    n = system.n
    
    for i in range(n):
        for j in range(n):
            if i == j:
                continue
                
            # Distance vector
            dx = system.x[j] - system.x[i]
            dy = system.y[j] - system.y[i] 
            dz = system.z[j] - system.z[i]
            
            # Distance squared with softening
            r2 = dx*dx + dy*dy + dz*dz + eps*eps
            r = math.sqrt(r2)
            
            # Force magnitude factor
            inv_r3 = G / (r2 * r)
            force_factor = system.m[j] * inv_r3
            
            # Accumulate accelerations
            system.ax[i] += dx * force_factor
            system.ay[i] += dy * force_factor
            system.az[i] += dz * force_factor


def velocity_verlet_step(system: NBodySystem, dt: float, eps: float = 1e-2, G: float = 1.0):
    """
    Perform one step of Velocity Verlet integration
    """
    # Step 1: v(t + dt/2) = v(t) + (dt/2) * a(t)
    system.vx += 0.5 * dt * system.ax
    system.vy += 0.5 * dt * system.ay
    system.vz += 0.5 * dt * system.az
    
    # Step 2: x(t + dt) = x(t) + dt * v(t + dt/2)
    system.x += dt * system.vx
    system.y += dt * system.vy
    system.z += dt * system.vz
    
    # Step 3: compute a(t + dt) from new positions
    compute_accelerations_naive(system, eps, G)
    
    # Step 4: v(t + dt) = v(t + dt/2) + (dt/2) * a(t + dt)
    system.vx += 0.5 * dt * system.ax
    system.vy += 0.5 * dt * system.ay
    system.vz += 0.5 * dt * system.az
