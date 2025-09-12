import os
from typing import List
from .model import Body


def write_state_csv(path: str, iteration: int, bodies: List[Body], create_header: bool) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    mode = "w" if create_header else "a"
    with open(path, mode, encoding="utf-8") as f:
        if create_header:
            f.write("iteration,id,m,x,y,z,vx,vy,vz\n")
        for i, b in enumerate(bodies):
            f.write(f"{iteration},{i},{b.m},{b.x},{b.y},{b.z},{b.vx},{b.vy},{b.vz}\n")
