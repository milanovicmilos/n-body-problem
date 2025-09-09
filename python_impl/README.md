# N-body Simulation - Python Implementation

Implementacija N-body simulacije u Python-u sa sekvencijalnom i paralelnom verzijom.

## Instalacija

1. Kreirajte Python virtual environment:
```powershell
python -m venv venv
.venv\Scripts\activate
```

2. Instalirajte zavisnosti:
```powershell
pip install -r requirements.txt
```

## Pokretanje simulacija

### Sekvencijalna simulacija

```powershell
python -m nbody_py.cli run-seq --n 1000 --steps 1000 --dt 0.001 --eps 0.01 --seed 42 --dump-every 100 --out "../data/outputs/sequential_run"
```

### Paralelna simulacija (multiprocessing)

```powershell
python -m nbody_py.cli run-mp --n 1000 --steps 1000 --dt 0.001 --eps 0.01 --seed 42 --dump-every 100 --procs 4 --out "../data/outputs/parallel_run"
```

### Analiza rezultata

```powershell
python -m nbody_py.cli analyze --dir "../data/outputs/sequential_run"
```

## Parametri

- `--n`: Broj tela u simulaciji
- `--steps`: Broj koraka simulacije  
- `--dt`: Vremenski korak
- `--eps`: Parametar omekšavanja (softening)
- `--seed`: Seme za reproduktivnost
- `--dump-every`: Čuvaj stanje svakih N koraka
- `--procs`: Broj procesa za paralelizaciju (samo za run-mp)
- `--out`: Izlazni direktorijum

## Izlazni fajlovi

Simulacija generiše sledeće fajlove:

1. `states_iter_XXXXXX.csv` - Stanja sistema po iteracijama
2. `energy.csv` - Energije sistema kroz vreme
3. `run_meta.json` - Metadata simulacije
4. `initial_conditions.csv` - Početni uslovi

## Testiranje

Za testiranje implementacije:

```powershell
python test_nbody.py
```

Za testiranje performansi:

```powershell
python test_performance.py
```

## Struktura podataka

Sistem koristi Structure of Arrays (SoA) format za optimalne performanse:

```python
class NBodySystem:
    def __init__(self, n: int):
        # Pozicije
        self.x, self.y, self.z = np.zeros(n), np.zeros(n), np.zeros(n)
        # Brzine  
        self.vx, self.vy, self.vz = np.zeros(n), np.zeros(n), np.zeros(n)
        # Mase
        self.m = np.zeros(n)
        # Akceleracije
        self.ax, self.ay, self.az = np.zeros(n), np.zeros(n), np.zeros(n)
```

## Fizička validacija

Implementacija koristi:
- Velocity Verlet integrator za stabilnost
- Plummer sphere inicijalizaciju
- Gravitacione sile sa softening parametrom
- Praćenje očuvanja energije

Rezultati simulacije:
- ✓ Dobro očuvanje energije (drift < 0.01%)
- ✓ Stabilnost numerička kroz dugotrajne simulacije
- ✓ Sekvencijalna i paralelna verzija daju identične rezultate
