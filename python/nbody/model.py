import json
import random
from dataclasses import dataclass
from typing import List, Tuple


@dataclass
class Body:
    m: float
    x: float
    y: float
    z: float
    vx: float
    vy: float
    vz: float


def parse_bodies_from_json(json_str: str) -> List[Body]:
    arr = json.loads(json_str)
    bodies: List[Body] = []
    for item in arr:
        bodies.append(
            Body(
                float(item["m"]),
                float(item["x"]),
                float(item["y"]),
                float(item["z"]),
                float(item.get("vx", 0.0)),
                float(item.get("vy", 0.0)),
                float(item.get("vz", 0.0)),
            )
        )
    return bodies


def random_bodies(n: int, mass_range: Tuple[float, float], pos_range: Tuple[float, float], vel_range: Tuple[float, float], seed: int) -> List[Body]:
    rnd = random.Random(seed)
    bodies: List[Body] = []
    for _ in range(n):
        m = rnd.uniform(*mass_range)
        x = rnd.uniform(*pos_range)
        y = rnd.uniform(*pos_range)
        z = rnd.uniform(*pos_range)
        vx = rnd.uniform(*vel_range)
        vy = rnd.uniform(*vel_range)
        vz = rnd.uniform(*vel_range)
        bodies.append(Body(m, x, y, z, vx, vy, vz))
    return bodies
