# Projekat: N-Body simulacija u Python-u i Rust-u (HPC) - Miloš Milanović SV32/2021

## 0) Ciljna ocena i opis problema

Ciljna ocena: 10 

Tema koju sam odabrao je N-body problem, koji predstavlja jedan od najpoznatijih i najvažnijih izazova u oblasti fizike i računarskih simulacija.

N-body problem opisuje situaciju u kojoj više tela u prostoru međusobno deluje gravitacionim silama. Svako telo svojim prisustvom utiče na svako drugo, a kako broj tela raste, tako raste i složenost problema. Za mali broj tela problem se može rešiti analitički, ali za veći broj (stotine, hiljade ili više) potrebno je koristiti numeričke metode i simulacije. Upravo zato je N-body problem odličan primer za ispitivanje efikasnosti algoritama i optimizacija u računarstvu.

U ovom radu koristiću različite metode za rešavanje problema. Najpre ću implementirati naivni pristup (brute-force), gde se računa interakcija svake čestice sa svakom drugom, što ima vremensku složenost O(n²). Nakon toga ću primeniti RSUT (Barnes–Hut) algoritam, koji koristi hijerarhijsku dekompoziciju prostora i aproksimacije da bi značajno smanjio broj potrebnih računanja, čime se složenost svodi na O(n log n). Na kraju ću razmotriti i mogućnosti paralelizacije i optimizacije korišćenjem modernih tehnologija poput programiranja na GPU-u (npr. CUDA) ili višeprocesorskog računanja.

Na ovaj način projekat ne samo da rešava jedan klasičan matematičko-fizički problem, već i pokazuje kako se kroz algoritamski dizajn i računarske metode mogu prevazići izazovi skalabilnosti i efikasnosti.

---

## 1) Opis problema (N-body)

Simulirati kretanje N čestica/tela koja međusobno deluju gravitacionom silom. Svako telo ima masu $m_i$, poziciju $\mathbf{x}_i(t)$ i brzinu $\mathbf{v}_i(t)$.

Za svaka dva tela $i \neq j$, gravitaciona sila je:

$$
\mathbf{F}_{ij} = G\,\frac{m_i\,m_j}{(r_{ij}^2 + \varepsilon^2)^{3/2}}\, (\mathbf{x}_j - \mathbf{x}_i), \qquad r_{ij} = \|\mathbf{x}_j - \mathbf{x}_i\|
$$

Gde je $\varepsilon$ (softening) mali parametar radi izbegavanja singularnosti pri malim rastojanjima.

### Jednačine kretanja

$$
\mathbf{a}_i = \frac{1}{m_i} \sum_{j \ne i} \mathbf{F}_{ij}, \qquad
\frac{d\mathbf{x}_i}{dt} = \mathbf{v}_i, \qquad
\frac{d\mathbf{v}_i}{dt} = \mathbf{a}_i.
$$

### Zašto je primer dobar za HPC i Rust?

- Računski intenzivan: naivno računanje je $O(N^2)$ po koraku.
- Embarrassingly parallel: akceleracije po telima se računaju nezavisno → idealno za niti/procese.
- Numerička dinamika: integratori, očuvanje energije/impulsa → merenje kvaliteta rešenja.
- Rust nudi C-like performanse + sigurnu paralelizaciju (bez data-race) i odličnu kontrolu memorije.

---

## 2) Metoda rešenja

### 2.1 Integrator (numerička integracija u vremenu)

Podrazumevani integrator: Velocity Verlet / Leapfrog (simplekstni, dobro čuva energiju):

\[\text{Velocity Verlet:}\]

$$
\mathbf{v}_i\left(t+\tfrac{\Delta t}{2}\right) = \mathbf{v}_i(t) + \tfrac{\Delta t}{2}\,\mathbf{a}_i(t)
$$

$$
\mathbf{x}_i(t+\Delta t) = \mathbf{x}_i(t) + \Delta t\,\mathbf{v}_i\left(t+\tfrac{\Delta t}{2}\right)
$$

Izračunati $\mathbf{a}_i(t+\Delta t)$ iz novih pozicija, pa:

$$
\mathbf{v}_i(t+\Delta t) = \mathbf{v}_i\left(t+\tfrac{\Delta t}{2}\right) + \tfrac{\Delta t}{2}\,\mathbf{a}_i(t+\Delta t).
$$

Alternativa (opciono za poređenje): RK4 (veća lokalna tačnost, ali ne simplekstni) i (semi)implicit Euler (brz, ali lošije očuvanje energije).

Parametri stabilnosti:

- $\Delta t$ (korak vremena): mali za bliske susrete; u projektu fiksni $\Delta t$ (jednostavnije i dovoljno), opciono adaptivni.
- $\varepsilon$ (softening): tipično $10^{-3} \dots 10^{-2}$ u normalizovanim jedinicama.

### 2.2 Algoritam obračuna sila

- Naivni: $O(N^2)$ — jednostavan, dovoljan za male i srednje $N$ (npr. do 10–50k uz dovoljno jezgra).
- Barnes–Hut: $O(N\log N)$ (opciono „plus"): octree (3D) ili quadtree (2D), kriterijum otvaranja $s/d < \theta$ (tipično $\theta\sim0.5\!–\!0.7$).

Barnes–Hut nije obavezno za ocenu 10, ali donosi veliki speedup za vrlo velika $N$; može se predstaviti kao proširenje.

### 2.3 Memorijski raspored i preciznost

- SoA (Structure of Arrays): `x[]`, `y[]`, `z[]`, `vx[]`, `vy[]`, `vz[]`, `m[]` → bolje iskorišćenje keša i SIMD.
- Preciznost: `f64` (preporučeno za stabilnost); `f32` (opciono za masivne $N$ uz verifikaciju greške).

---

## 3) Arhitektura softvera

### 3.1 Struktura repozitorijuma

```
nbody/
  README.md
  LICENSE
  configs/
    default.toml
    strong_scaling.toml
    weak_scaling.toml
  data/
    initial_conditions/
    outputs/       # CSV/JSON/PNG izveštaji i rezultati
  scripts/
    run_strong_scaling.py
    run_weak_scaling.py
    analyze_results.py
  python_impl/
    requirements.txt
    nbody_py/
      __init__.py
      physics.py         # sile, integratori
      simulate_seq.py    # sekvencijalno
      simulate_mp.py     # multiprocessing
      io_utils.py
      metrics.py
      cli.py
  rust_impl/
    Cargo.toml
    src/
      main.rs
      cli.rs             # clap args
      io.rs
      metrics.rs
      types.rs           # SoA strukture, inicijalizacija
      force_naive.rs
      integrators.rs
      sim_runner.rs      # orkestracija
      parallel.rs        # niti / rayon
      viz.rs             # plotters grafici
      # opcionalno barnes_hut.rs
  report/
    figures/             # PNG/SVG grafici
    tables/              # CSV tabele merenja
    report.md            # izveštaj (može i .pdf kasnije)
```

### 3.2 Ključni moduli i odgovornosti

- `physics/force_naive`: obračun sila; sumira akceleracije u privremene bafer-e (bez data race).
- `integrators`: Velocity Verlet (default), RK4/Euler (opciono).
- `sim_runner`: glavna petlja: init → za svaku iteraciju: `accel()` → `step()` → metrics/log.
- `io`: čitanje/pisanje `*.toml`, `*.csv`, `meta.json`, rotacija fajlova (dump na svakih k koraka).
- `metrics`: energija (kinetička, potencijalna), ukupni impuls, centar mase, detekcija outlier-a u merenjima.

Paralelizacija:

- Python: `multiprocessing` pool, podela tela po blokovima (chunked ranges), prenos read-only pozicija u shared memory / memmap (po mogućnosti), vraćanje parcijalnih akceleracija kanalom.
- Rust: `std::thread::scope` + thread-local akumulacija → barijera → upis u zajedničke bafer-e; alternativno `rayon` (`par_chunks_mut`).
- `viz`: Plotters grafici (energija v. vreme, skaliranje, teorijske krive).

---

## 4) Interfejs komandne linije (CLI)

### 4.1 Rust binarni

```text
nbody-rs run \
  --n 20000 --steps 5000 --dt 1e-3 --eps 1e-2 \
  --algo naive \
  --integrator verlet \
  --threads 8 \
  --dump-every 10 \
  --seed 42 \
  --out data/outputs/run_rs/
```

Argumenti:

- `--n` (broj tela), `--steps`, `--dt`, `--eps`
- `--algo` `naive|barnes-hut` (opciono)
- `--integrator` `verlet|leapfrog|rk4|euler`
- `--threads` (broj niti; 1,2,4,8,…)
- `--dump-every k` (snimi stanja na svaka k koraka)
- `--seed` (determinističnost)
- `--out` (folder izlaza)

### 4.2 Python CLI

```text
python -m nbody_py.cli run-seq \
  --n 10000 --steps 2000 --dt 1e-3 --eps 1e-2 \
  --dump-every 10 \
  --seed 42 \
  --out data/outputs/run_py_seq/

python -m nbody_py.cli run-mp \
  --n 10000 --steps 2000 --dt 1e-3 --eps 1e-2 \
  --procs 8 \
  --dump-every 10 \
  --seed 42 \
  --out data/outputs/run_py_mp/
```

---

## 5) Formati podataka (I/O)

### 5.1 Ulaz (inicijalni uslovi)

`initial_conditions/*.csv`:

```
id,x,y,z,vx,vy,vz,m
```

(Ako se ne prosledi, generišemo pseudo-nasumično prema izabranoj distribuciji, npr. Plummer.)

`configs/*.toml`: parametri simulacije i merenja (`N`, `steps`, `dt`, `eps`, `seed`, `dump_every`, `scenario`, …).

### 5.2 Izlaz

- `states_iter_000010.csv`:

```
iter,id,x,y,z,vx,vy,vz,m
```

- `energy.csv`:

```
iter,kinetic,potential,total
```

- `run_meta.json`:

```json
{ "n":..., "steps":..., "dt":..., "eps":..., "algo":"naive", "integrator":"verlet", "seed":..., "threads":..., "hostname":..., "cpu":..., "ram_gb":..., "os":..., "lib_versions":{...} }
```

- `scaling_results.csv` (po skriptama u `scripts/`):

```
lang,mode,p,n,steps,dt,eps,duration_s,speedup,efficiency,repeats,mean,std,outliers
```

---

## 6) Validacija i merenje kvaliteta

### 6.1 Fizičke invarijante i test primeri

- 2 tela (kružna/eliptična orbita): provera perioda i udaljenosti (greška < par % kroz duge simulacije).
- Sunce–Zemlja–Mesec: precesija, stabilnost energije.
- Centar mase ~ konstantan (drift ~ 0).
- Ukupan impuls ~ konstantan.
- Energija: manji drift uz Verlet nego uz Euler; grafički prikaz trenda.

### 6.2 Metrički izlazi

- Graf: `E_total` vs. iter (očekuje se stabilnost).
- Tabele: srednje vreme izvršavanja, SD, outlier-i (zapisati seeds).

---

## 7) Eksperimenti (skaliranje i izveštaj)

### 7.1 Jakog skaliranja (Amdahl)

Fiksiraj `N, steps, dt, eps`, menjaj paralelizam: $p\in\{1,2,4,8,16,\dots\}$.

Ubrzanje:

$$
S_p = \frac{T_1}{T_p}, \qquad E_p = \frac{S_p}{p}.
$$

Procena sekvencijalnog dela $f$: instrumentacijom vremena segmenata koda (npr. I/O, konstrukcija struktura, redukcije).

Amdahl-ova teorijska kriva:

$$
S_{\max}(p) = \frac{1}{f + (1-f)/p}.
$$

### 7.2 Slabog skaliranja (Gustafson)

Menjati $N$ proporcionalno $p$ (posao po jezgri ~ konstantan).

Gustafson-ova formula:

$$
S_G(p) = p - f (p - 1).
$$

Objasniti kako je posao skaliran (definicija posla po jezgri).

### 7.3 Protokol merenja

- Svaku tačku (kombinaciju parametara) izvršiti ~30 puta; izračunati mean, std, označiti outlier-e.
- Logovati hardver/softver: CPU model, takt, keš, #fizičkih/#logičkih jezgara, NUMA, RAM tip/veličina, OS, verzije biblioteka.

Grafici (obavezna 4):

1. Jakog skaliranja — Python + Amdahl kriva (x: #jezgara, y: speedup, uključiti idealnu liniju).
2. Jakog skaliranja — Rust + Amdahl kriva.
3. Slabog skaliranja — Python + Gustafson kriva.
4. Slabog skaliranja — Rust + Gustafson kriva.

> Napomena: Za ocenu 10 grafike praviti u Rust-u (Plotters) iz `scaling_results.csv`.

---

## 8) Paralelizacija i bezbednost podataka

### 8.1 Python (multiprocessing)

- Podela domena po indeksima tela (blokovi).
- Pozicije/masse kao read-only (deljene putem shared memory/memmap ili slanje kopije po procesu — balansiranje overhead-a).
- Svaki proces računa lokalne akceleracije → povratna redukcija (sumiranje) u roditelj procesu.
- Izbegavati česte IPC prelaske (grupisati iteracije ili koristiti veće blokove).

### 8.2 Rust (threads)

- Thread-local akceleracioni bafer-i → posle izračuna svih parova radi se redukcija u glavni bafer bez data race-a.
- Double-buffering pozicija/ brzina (write-new, read-old).
- Sinhronizacija barijerom po fazi (npr. `std::sync::Barrier` ili implicitno kroz `rayon`).
- Izbegavanje false sharing-a (poravnati strukture ili dodeliti kontinuirane blokove po niti).

---

## 9) Vizualizacija (Rust)

- Plotters izlaz: PNG/SVG.

Grafovi:

- Putanje (x-y projekcija) iz snimljenih stanja (sample svakih k).
- Energija kroz iteracije (K, U, T).
- 4 grafika skaliranja (Python/Rust × Amdahl/Gustafson) sa tačkama iz merenja, idealnom linijom i teorijskom krivom prema procenjenom $f$.

(Opciono) `egui/eframe`: interaktivni pregled simulacije u realnom vremenu (nije uslov).

---

## 10) Tehnologije i biblioteke

### Rust

- CLI: `clap`
- Paralela: `std::thread` + Barrier; opciono `rayon`
- Vizualizacija: `plotters`
- Serijalizacija: `serde`, `serde_json`, `toml`
- (Opciono) SIMD: `std::simd`
- Test/bench: `criterion` (opciono)

### Python

- Paralela: `multiprocessing`
- Numerika: `numpy` (vektorizacija), `pandas` (analiza CSV)
- Grafika za interne provere: `matplotlib` (neobavezno — finalne grafikone radi Rust)

---

## 11) Performanse i optimizacije

- SoA raspored podataka.
- Cache-friendly petlje (spoljna petlja po `i`, unutrašnja po `j>i` uz simetriju sila → prepolovljeni obračun; u paralelnom režimu voditi računa o konfliktima).
- SIMD vektorizacija (Rust) za unutrašnju petlju (opciono).
- Smanjenje I/O overheada: `--dump-every k`, binarni format za velika N (opciono `.npy/.npz` u Python delu).
- `f64` default; meriti odnos tačnost/performanse ako se pređe na `f32`.

---

## 12) Rizici i mitigacije

- Numerički drift energije: koristiti Verlet; validacija na 2-tela; smanjiti `\Delta t` ili uvesti softening.
- IPC overhead (Python): dovoljno krupni blokovi; manje sinhronizacionih tačaka.
- Data-race (Rust): striktno thread-local akumulacije i jednosmerne faze pisanja.
- Preveliki izlazni fajlovi: `--dump-every`, kompresija, sampling.
- Nedeterminističnost: setovati `--seed` i fiksirati generatore.

---

## 13) Plan testiranja

### 13.1 Jedinični i integracioni testovi

- Test 2-tela (kružna orbita): odstupanje radijusa < 2–3% tokom X perioda.
- Poređenje Python vs. Rust na istom seed-u i parametrima (RMSE pozicija < tolerancije zbog razlika u zaokruživanju).
- Test očuvanja centra mase i impulsa (maks. drift po iteraciji).

### 13.2 System/bench testovi

Skripte `run_strong_scaling.py`, `run_weak_scaling.py`:

- Automatizuju pokretanje (p = 1,2,4,8,16…)
- 30 ponavljanja po tački
- Snimaju CSV sa vremenima i metapodacima

`analyze_results.py`:

- Računa speedup/efficiency, fituje $f$, generiše agregacione tabele.

---

## 14) Reproducibilnost i dokumentacija

- Fiksirati verzije (Rust `Cargo.lock`, Python `requirements.txt`).
- U `run_meta.json` beležiti hardver/softver i parametre.
- `README.md`: jasna uputstva za pokretanje, primeri komandi, očekivani izlazi, granice resursa (RAM/CPU).

---

## 15) Kako pokrenuti

### 15.1 Rust

```powershell
cd rust_impl
cargo build --release
./target/release/nbody-rs run --n 20000 --steps 5000 --dt 1e-3 --eps 1e-2 --threads 8 --out ../data/outputs/run_rs/
```

### 15.2 Python

```powershell
cd python_impl
python -m venv .venv; .\.venv\Scripts\activate
pip install -r requirements.txt

python -m nbody_py.cli run-seq --n 10000 --steps 2000 --dt 1e-3 --eps 1e-2 --out ../data/outputs/run_py_seq/
python -m nbody_py.cli run-mp  --n 10000 --steps 2000 --dt 1e-3 --eps 1e-2 --procs 8 --out ../data/outputs/run_py_mp/
```

---

## 16) Kriterijumi prihvatanja (mapiranje na ocene)

- 6: Python sekvencijalno + multiprocessing, validni izlazni fajlovi stanja.
- 7: Rust sekvencijalno + threads paralela, validni izlazi.
- 8: Jak/Slab scaling eksperimenti za Python, izveštaj + graf(ovi).
- 9: Jak/Slab scaling eksperimenti za Rust, izveštaj + graf(ovi).
- 10: Vizualizacija rezultata u Rust-u (Plotters) + kompletan izveštaj (4 obavezna grafika, tabele, Amdahl/Gustafson, opis hardvera/softvera, metodologija 30 ponavljanja, analiza outlier-a).

---

## 17) Dodatne funkcionalnosti (opciono, „plus")

- Barnes–Hut za velika N (octree, `theta` kao parametar).
- Interaktivni prikaz (`egui`) — pauza/nastavak, zoom, snimanje GIF/sekvenci.
- SIMD ubrzanja (Rust).
- Adaptivni `\Delta t` po maksimalnoj akceleraciji.
- Kahan summation u obračunu potencijalne energije (smanjenje numeričke greške).

---

## 18) Pseudokôd jezgra (naivni $O(N^2)$, Verlet)

```text
# Data: SoA arrays x[], y[], z[], vx[], vy[], vz[], m[], ax[], ay[], az[]
# Step 0: compute_accel() for initial a(t)

for iter in 1..steps:
    # v(t + dt/2)
    for i in 0..N-1:
        vx[i] += 0.5*dt*ax[i]
        vy[i] += 0.5*dt*ay[i]
        vz[i] += 0.5*dt*az[i]

    # x(t + dt)
    for i in 0..N-1:
        x[i] += dt*vx[i]
        y[i] += dt*vy[i]
        z[i] += dt*vz[i]

    # a(t + dt)   <-- parallelizable
    zero(ax, ay, az)
    for i in 0..N-1 parallel:     # threads/procs compute disjoint i-ranges
        for j in 0..N-1:
            if i == j: continue
            dx = x[j]-x[i]; dy = y[j]-y[i]; dz = z[j]-z[i]
            r2 = dx*dx + dy*dy + dz*dz + eps*eps
            inv = G / (r2 * sqrt(r2))
            s  = m[j] * inv
            ax_local[i] += dx * s
            ay_local[i] += dy * s
            az_local[i] += dz * s
    reduce_local_accels_into(ax, ay, az)

    # v(t + dt)
    for i in 0..N-1:
        vx[i] += 0.5*dt*ax[i]
        vy[i] += 0.5*dt*ay[i]
        vz[i] += 0.5*dt*az[i]

    if iter % dump_every == 0:
        dump_states(iter)
        log_energy_momentum(iter)
```

U paraleli: svaka nit/proces računa svoje `ax_local[i]`, `ay_local[i]`, `az_local[i]` pa se radi redukcija.
Alternativa (optimizacija): petlju po parovima (`i<j`) i simetrično dodavanje, ali tada je sinhronizacija kompleksnija; u prvoj fazi držimo jednostavno i bezbedno.

---

## 19) Plan rada (faze)

- Arhitektura, konfiguracija, inicijalizacija (TOML, CLI, SoA, generator početnih uslova).
- Python: sekvencijalno → multiprocessing, validacija (2-tela, energija).
- Rust: sekvencijalno → niti (std::thread ili rayon), validacija.
- Merenja: strong/weak (Python i Rust), skripte, 30× ponavljanja, log hardvera.
- Vizualizacija (Rust/Plotters): 4 obavezna grafika + energija/putanje.
- Izveštaj: Amdahl/Gustafson, tabele (mean, std, outliers), zaključci.
- (Opciono): Barnes–Hut, egui, SIMD, adaptivni dt.

---

## 20) Zaključak (što se radi i zašto je vredno)

Ovaj projekat ti daje kompletan HPC ciklus na realnom fizičkom problemu: od modelovanja i numerike, preko paralelizacije i optimizacija, do rigoroznog merenja skaliranja i vizualizacije. Rust je izabran jer spaja performanse i bezbednost (thread-safety), a Python omogućava brzu iteraciju i demonstraciju uticaja GIL-a i potrebe za multiprocessing pristupom.

Rezultat je merljiv, reproduktiv (seed + meta-logovi), sa jasnim kriterijima za ocenu 10/10.
