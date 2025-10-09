# N-body problem

## Specifikacija projekta

### 0. Ciljna ocena: 10

### 1. Uvod

N-body problem predstavlja klasičan problem u oblasti kompjuterske fizike i numeričkih simulacija koji se bavi modelovanjem gravitacionih interakcija između N tela u prostoru. Zadatak je simulirati kretanje sistema tela pod uticajem međusobne gravitacione privlačnosti.

#### 1.1. Opis problema

Sistem se sastoji od N tela, gde svako telo ima:
- Masu (m)
- Poziciju u 3D prostoru (x, y, z)
- Brzinu u 3D prostoru (vx, vy, vz)

Svako telo utiče gravitaciono na sva ostala tela prema Njutnovom zakonu gravitacije. Simulacija napreduje kroz diskretne vremenske korake (iteracije), pri čemu se u svakom koraku:
1. Izračunavaju gravitacione sile između svih parova tela
2. Ažuriraju brzine i pozicije svih tela

Složenost problema je O(N²) po vremenskom koraku jer je potrebno razmotriti sve parove tela.

#### 1.2. Cilj projekta

Implementirati N-body simulaciju u dva programska jezika (Python i Rust), sa sekvencijalnim i paralelnim verzijama, izvršiti analizu performansi paralelizacije, i vizualizovati rezultate.

### 2. Matematički model

#### 2.1. Gravitaciona sila

Sila između dva tela i i j računa se prema Njutnovom zakonu gravitacije:

```
F_ij = G × m_i × m_j × (r_j - r_i) / |r_j - r_i|³
```

Gde je:
- **G** - gravitaciona konstanta (parametar simulacije)
- **m_i, m_j** - mase tela i i j
- **r_i, r_j** - pozicioni vektori tela (u 3D prostoru)
- **|r_j - r_i|** - rastojanje između tela

**Napomena o singularitetu:** Kada se tela nađu veoma blizu (r → 0), sila teži beskonačnosti. Da bi se izbegao numerički problem, uvodi se **softening parametar** ε:

```
F_ij = G × m_i × m_j × (r_j - r_i) / (|r_j - r_i|² + ε)^(3/2)
```

#### 2.2. Integracija jednačina kretanja

Potrebno je koristiti numerički integrator koji će iz sila izračunati nova ubrzanja, pa potom ažurirati brzine i pozicije tela.

**Preporučeni metod:** Velocity Verlet algoritam (simplektički integrator drugog reda)

Prednosti ovog metoda:
- Bolja stabilnost od jednostavnog Euler metoda
- Očuvanje energije u sistemu (važno za fizičku tačnost)
- Vremenska reverzibilnost

**Napomena:** Alternativno se može koristiti i Euler metod, ali Velocity Verlet daje tačnije rezultate.

### 3. Zahtevi projekta

Projekat je organizovan u pet nivoa zahteva koji odgovaraju ocenama od 6 do 10:

#### 3.1. Ocena 6 - Python implementacija

Implementirati rešenje problema upotrebom programskog jezika **Python**.

**Zahtevi:**

1. **Sekvencijalna verzija:**
   - Implementirati algoritam koji simulira kretanje N tela kroz zadati broj vremenskih koraka
   - Rezultat: CSV datoteka koja sadrži stanje sistema za svaku iteraciju
   - Format CSV-a: `iteration,id,m,x,y,z,vx,vy,vz`
     - iteration: redni broj koraka simulacije
     - id: identifikator tela (0, 1, 2, ...)
     - m: masa tela
     - x, y, z: pozicija tela
     - vx, vy, vz: brzina tela

2. **Paralelna verzija (multiprocessing):**
   - Paralelizovati algoritam koristeći `multiprocessing` biblioteku
   - Omogućiti korisniku da specificira broj radnih procesa
   - Rezultat: CSV datoteka istog formata kao kod sekvencijalne verzije
   - **Bitno:** Rezultati sekvencijalne i paralelne verzije moraju biti identični (za iste početne uslove i parametre)

**Ulazni parametri (komandna linija):**
- Broj tela (N)
- Broj iteracija (koraka simulacije)
- Veličina vremenskog koraka (dt)
- Gravitaciona konstanta (G)
- Softening parametar
- Početni uslovi: iliJSON string sa telima ili generisanje nasumičnih tela
- Putanja do izlazne CSV datoteke

**Generisanje nasumičnih tela:**
Ako korisnik želi da generiše N nasumičnih tela, omogućiti specificiranje:
- Opsega masa [min, max]
- Opsega pozicija [min, max] za sve tri dimenzije
- Opsega brzina [min, max] za sve tri dimenzije
- Seed za generator slučajnih brojeva (reproduktivnost)

#### 3.2. Ocena 7 - Rust implementacija

Implementirati rešenje problema upotrebom programskog jezika **Rust**.

**Zahtevi:**

1. **Sekvencijalna verzija:**
   - Identična funkcionalnost kao Python sekvencijalna verzija
   - Isti format CSV izlaza (kompatibilnost)
   - Kompilacija sa release optimizacijama

2. **Paralelna verzija (threads):**
   - Paralelizovati algoritam koristeći niti (preporučena biblioteka: **Rayon**)
   - Omogućiti kontrolu broja niti (npr. preko environment varijable)
   - Isti format CSV izlaza

**Ulazni parametri:**
Identični kao kod Python verzije - omogućiti jednake opcije komandne linije.

**Napomena:** CSV datoteke generisane iz Python i Rust verzija moraju biti međusobno kompatibilne (isti format).

#### 3.3. Ocena 8 - Analiza performansi (Python)

Izvršiti eksperimente **jakog** i **slabog** skaliranja za Python implementaciju.

##### 3.3.1. Jako skaliranje (Strong Scaling)

**Definicija:** Testiranje ubrzanja programa sa povećanjem broja radnika, pri čemu je **veličina problema fiksna**.

**Eksperiment:**
- Fiksirati broj tela (npr. N = 1200)
- Pokrenuti simulaciju sa 1, 2, 4, 8 radnika (procesa)
- Za svaku konfiguraciju ponoviti merenje **minimum 30 puta** (zahtev kursa)
- Meriti vreme izvršavanja simulacionog dela (bez I/O operacija)

**Metrika:** Ubrzanje (Speedup)
```
S(w) = T_seq / T_parallel(w)
```
gde je:
- w = broj radnika
- T_seq = vreme izvršavanja sa 1 radnikom (baseline)
- T_parallel(w) = vreme izvršavanja sa w radnika

**Teorijska osnova:** Amdalov zakon
```
S(w) = 1 / ((1 - p) + p/w)
```
gde je p = paralelna frakcija koda (deo koda koji se može paralelizovati)

##### 3.3.2. Slabo skaliranje (Weak Scaling)

**Definicija:** Testiranje skalabilnosti programa kada **veličina problema raste proporcionalno** broju radnika.

**Eksperiment:**
- Veličina problema: N = base_N × w (npr. base_N = 300)
  - 1 radnik: 300 tela
  - 2 radnika: 600 tela
  - 4 radnika: 1200 tela
  - 8 radnika: 2400 tela
- Za svaku konfiguraciju ponoviti merenje **minimum 30 puta**
- Cilj: konstantan posao po radniku

**Metrika:** Ubrzanje
```
S(w) = (T_baseline × w) / T_parallel(w)
```

**Teorijska osnova:** Gustafsonov zakon
```
S(w) = (1 - p) + p × w
```

**Napomena:** Zbog O(N²) prirode problema, posao po radniku nije potpuno konstantan, već raste sa w. Ovo treba objasniti u izveštaju.

##### 3.3.3. Statistička analiza

Za svaku konfiguraciju (broj radnika) izračunati:
- **Srednju vrednost** vremena izvršavanja
- **Standardnu devijaciju**
- **Minimum i maksimum**
- **Outliere** (identifikovati pomoću IQR metoda)

IQR metod:
- Q1 = prvi kvartil, Q3 = treći kvartil
- IQR = Q3 - Q1
- Outlier ako: vrednost < Q1 - 1.5×IQR ili vrednost > Q3 + 1.5×IQR

##### 3.3.4. Fitovanje paralelne frakcije

Na osnovu izmerenih ubrzanja, fitovati paralelnu frakciju **p** koja najbolje odgovara podacima:
- Za jako skaliranje: fitovati p u Amdalov zakon
- Za slabo skaliranje: fitovati p u Gustafsonov zakon

Metoda: Minimizacija kvadratne greške između teorijskog i izmerenog ubrzanja.

##### 3.3.5. Izlazni fajlovi

Generisati:
1. **CSV tabele** sa statistikama (srednja vrednost, std, min/max, outlieri, speedup)
2. **Grafike** (PNG):
   - X-osa: broj radnika
   - Y-osa: ubrzanje
   - Tri linije na grafiku:
     - Izmereno ubrzanje (tačke sa error barovima)
     - Idealno ubrzanje (S = w)
     - Fitovani teorijski model (Amdalov ili Gustafsonov)

#### 3.4. Ocena 9 - Analiza performansi (Rust)

Izvršiti iste eksperimente kao u sekciji 3.3, ali za **Rust** implementaciju.

**Zahtevi:**
- Identična metodologija merenja
- Isti parametri eksperimenata (N, broj koraka, dt, broj radnika, ponavljanja)
- Isti format izlaznih fajlova (CSV tabele, grafici)
- Uporedna analiza sa Python rezultatima u izveštaju

#### 3.5. Ocena 10 - Vizualizacija

Implementirati vizualizaciju rezultata simulacije **u Rust okruženju**.

**Zahtevi:**

1. **Ulaz:** CSV datoteka sa rezultatima simulacije (može biti iz Python ili Rust verzije)

2. **Izlaz:**
   - PNG frameovi za svaku iteraciju simulacije
   - Animirani GIF koji prikazuje evoluciju sistema

3. **Funkcionalnosti:**
   - Prikazati tela kao obojene tačke/krugove na 2D projekciji (x-y ravan)
   - Veličina tela proporcionalna masi (ili √masa)
   - Različite boje za različita tela (radi lakšeg praćenja)
   - **Opciono:** Prikazivanje tragova kretanja (prethodne pozicije tela sa transparencijom)

4. **Tehnička implementacija:**
   - Koristiti grafičku biblioteku (npr. **Plotters**, **image**, ili sl.)
   - Generisanje GIF-a pomoću odgovarajuće biblioteke (npr. **gif** crate)

5. **Parametri (komandna linija):**
   - Putanja do ulazne CSV datoteke
   - Veličina tela (skalirajući faktor)
   - Brzina animacije (delay između frameova u ms)
   - Broj tragova (koliko prethodnih pozicija prikazati)
   - Režim određivanja granica prikaza (fiksni ili dinamički)

6. **Folder struktura izlaza:**
   ```
   output/visualisation/<ime_csv_fajla>/
   ├── frame_00000.png
   ├── frame_00001.png
   ├── ...
   └── <ime_csv_fajla>.gif
   ```

### 4. Izveštaj o eksperimentima

Napisati detaljan izveštaj koji sadrži:

#### 4.1. Tehnički detalji sistema

**Hardverska arhitektura:**
- Model procesora
- Radni takt
- Organizacija cache memorije (L1, L2, L3)
- Broj fizičkih jezgara
- Broj logičkih jezgara (ako postoji hyperthreading)
- Broj NUMA node-ova
- Količina RAM memorije
- Tip RAM-a (DDR4, DDR5, brzina)

**Softverska arhitektura:**
- Operativni sistem (tip, verzija, build broj)
- Python verzija i korišćene biblioteke sa verzijama
- Rust verzija (rustc, cargo) i korišćene biblioteke (crate-ovi) sa verzijama
- Ostale relevantne informacije koje mogu uticati na performanse

**Napomena:** Ove informacije automatski prikupiti i sačuvati u JSON fajlu tokom benchmark-a.

#### 4.2. Analiza paralelizacije

**Identifikovati:**

1. **Paralelni deo koda:**
   - Koje operacije se mogu izvršavati paralelno?
   - Koliki procenat ukupnog vremena čini paralelni deo?

2. **Sekvencijalni deo koda:**
   - Koje operacije se moraju izvršavati sekvencijalno?
   - Zašto se ne mogu paralelizovati?
   - Koliki procenat ukupnog vremena čini sekvencijalni deo?

3. **Overhead paralelizacije:**
   - Kreiranje i upravljanje procesima/nitima
   - Komunikacija između radnika
   - Sinhronizacija

#### 4.3. Teorijski maksimumi

Izračunati teorijski maksimum ubrzanja prema:

1. **Amdalov zakon** (strong scaling):
   ```
   S_max = 1 / (1 - p)
   ```
   gde je p fitovana paralelna frakcija.

2. **Gustafsonov zakon** (weak scaling):
   - Objasniti kako se posao skalira sa brojem radnika
   - Zašto problem O(N²) složenosti predstavlja izazov za idealno slabo skaliranje

#### 4.4. Grafici (minimum 4)

Generisati i priložiti sledeće grafike:

1. **Python - Strong Scaling**
   - X-osa: broj radnika (1, 2, 4, 8)
   - Y-osa: ubrzanje (speedup)
   - Tri linije: izmereno, idealno (S=w), fitovani Amdalov model

2. **Rust - Strong Scaling**
   - Isti format kao Python strong scaling

3. **Python - Weak Scaling**
   - X-osa: broj radnika
   - Y-osa: ubrzanje
   - Tri linije: izmereno, idealno (S=w), fitovani Gustafsonov model

4. **Rust - Weak Scaling**
   - Isti format kao Python weak scaling

**Zahtevi za grafike:**
- Jasne oznake osa
- Legenda
- Naslov
- Grid (opciono)
- Error barovi na izmerenim podacima (prikazati standardnu devijaciju ili confidence interval)

#### 4.5. Potporne tabele

Za svaki grafik priložiti CSV tabelu sa:
- Broj radnika
- Srednje vreme izvršavanja
- Standardna devijacija
- Minimum
- Maksimum
- Broj outlier-a (i koje su to vrednosti)
- Ostvareno ubrzanje
- Fitovana paralelna frakcija

#### 4.6. Diskusija

Analizirati i diskutovati:

1. **Kako se posao manipuliše u weak scaling eksperimentima?**
   - Objasniti strategiju skaliranja problema (N = base_N × w)
   - Objasniti zašto je posao po radniku O(base_N × w) a ne konstantan (zbog O(N²))

2. **Poređenje Python vs Rust:**
   - Apsolutna brzina izvršavanja
   - Kvalitet paralelizacije (ubrzanje)
   - Overhead paralelizacije
   - Praktične prednosti/mane oba pristupa

3. **Odstupanja od teorije:**
   - Gde je izmereno ubrzanje bolje/lošije od teorijskog?
   - Koji faktori utiču na odstupanja?
   - Da li postoje outlier-i i šta ih je prouzrokovalo?

4. **Preporuke:**
   - Za koje veličine problema se isplati paralelizacija?
   - Koliki je optimalan broj radnika?
   - Koji pristup (Python multiprocessing vs Rust threads) je bolji i u kojim situacijama?

### 5. Struktura projekta (preporučena)

```
n-body-problem/
├── python/                      # Python implementacija
│   ├── main.py                  # Entry point, CLI parser
│   └── nbody/                   # Glavni paket
│       ├── __init__.py
│       ├── model.py             # Definicija tela, parsing
│       ├── sim.py               # Algoritmi simulacije
│       └── io.py                # I/O operacije (CSV)
│
├── rust/                        # Rust implementacija
│   ├── Cargo.toml               # Dependencies
│   ├── src/
│   │   ├── main.rs              # Entry point, CLI, vizualizacija
│   │   └── lib.rs               # Algoritmi simulacije
│   └── target/release/          # Kompajlirani executable
│
├── benchmarks/
│   └── bench.py                 # Benchmark harness
│
├── output/                      # Rezultati
│   ├── *.csv                    # Simulacioni izlazi
│   ├── summary_*.csv            # Statistike eksperimenata
│   ├── *.png                    # Grafici
│   ├── system_info.json         # Sistemski detalji
│   ├── fit_params.json          # Fitovane vrednosti
│   └── visualisation/           # Frameovi i GIF-ovi
│
├── SPECIFICATION.md             # Ovaj dokument
├── REPORT.md                    # Izveštaj o eksperimentima
└── README.md                    # Uputstvo za pokretanje
```

### 6. Tehnički saveti i smernice

#### 6.1. Izbor algoritma integracije

**Velocity Verlet metod** (preporuka):

Koraci po iteraciji:
1. Izračunati ubrzanja a(t) iz trenutnih pozicija
2. Ažurirati pozicije: x(t+dt) = x(t) + v(t)×dt + 0.5×a(t)×dt²
3. Izračunati nova ubrzanja a(t+dt) iz novih pozicija
4. Ažurirati brzine: v(t+dt) = v(t) + 0.5×(a(t) + a(t+dt))×dt

**Euler metod** (alternativa, jednostavniji ali manje tačan):

Koraci po iteraciji:
1. Izračunati ubrzanja a(t)
2. Ažurirati brzine: v(t+dt) = v(t) + a(t)×dt
3. Ažurirati pozicije: x(t+dt) = x(t) + v(t)×dt

#### 6.2. Paralelizacija

**Python - multiprocessing:**
- Koristiti `multiprocessing.Pool` sa odgovarajućim brojem procesa
- Paralelizovati računanje ubrzanja (spoljašnja petlja po telima)
- Paziti na pickle overhead (može biti značajan za velike N)
- Opciona optimizacija: koristiti shared memory (`multiprocessing.Array`)

**Rust - Rayon:**
- Koristiti `par_iter()` na kolekciji tela
- Rayon automatski upravlja thread pool-om
- Broj niti kontrolisati preko `RAYON_NUM_THREADS` environment varijable
- Paziti na granularnost (chunk size) - premalе chunks = overhead

#### 6.3. Merenje vremena

**Šta meriti:**
- SAMO vreme simulacionih koraka (petlja kroz iteracije)
- NE uključivati: parsiranje argumenata, učitavanje podataka, inicijalizaciju, pisanje u CSV

**Python:**
```python
import time
start = time.perf_counter()
# simulacija ovde
elapsed = time.perf_counter() - start
```

**Rust:**
```rust
use std::time::Instant;
let start = Instant::now();
// simulacija ovde
let elapsed = start.elapsed().as_secs_f64();
```

**Ispisati na stdout:** `ElapsedSeconds=<vrednost>` (benchmark skript će parsirati ovaj format)

#### 6.4. Reproduktivnost

- Uvek koristiti isti seed za generator slučajnih brojeva
- Dokumentovati sve parametre simulacije
- CSV izlaz mora biti identičan za iste ulazne parametre (seq vs parallel)

#### 6.5. I/O optimizacija tokom benchmark-a

- Omogućiti opciju da se CSV pisanje potpuno isključi tokom merenja (`--write-every 0`)
- Meriti samo računski deo, ne I/O

### 7. Primeri komandne linije

#### Python - sekvencijalno
```powershell
python python/main.py --random 200 --steps 100 --dt 0.002 --G 1.0 --softening 1e-9 --mode seq --output output/py_seq.csv
```

#### Python - paralelno
```powershell
python python/main.py --random 200 --steps 100 --dt 0.002 --mode mp --workers 4 --output output/py_mp.csv
```

#### Rust - sekvencijalno
```powershell
cargo run --release -- --random 200 --steps 100 --dt 0.002 --mode seq --output output/rs_seq.csv
```

#### Rust - paralelno
```powershell
$env:RAYON_NUM_THREADS=4
cargo run --release -- --random 200 --steps 100 --dt 0.002 --mode threads --output output/rs_threads.csv
```

#### Vizualizacija
```powershell
cargo run --release -- --visualize output/rs_seq.csv --vis-size 3.0 --vis-trails 5 --gif-ms 60
```

#### Benchmarking
```powershell
python benchmarks/bench.py --repeats 30 --workers 1 2 4 8 --problem-n 1200 --base-n 300 --steps 120 --dt 0.002
```

### 8. Kriterijumi ocenjivanja

| Ocena | Zahtev | Kriterijum |
|-------|--------|-----------|
| **6** | Python seq + parallel | Funkcionalne implementacije, ispravan CSV izlaz |
| **7** | Rust seq + parallel | Funkcionalne implementacije, kompatibilan CSV izlaz |
| **8** | Python scaling | Kompletni eksperimenti (30+ ponavljanja), statistika, grafici, fitovanje |
| **9** | Rust scaling | Kompletni eksperimenti (30+ ponavljanja), statistika, grafici, fitovanje |
| **10** | Vizualizacija | Funkcionalna vizualizacija sa frameovima i GIF animacijom |

**Izveštaj** (obavezan za sve ocene ≥ 8):
- Tehnički detalji sistema
- Analiza paralelizacije
- Potporne tabele
- Grafici sa teorijskim modelima
- Diskusija rezultata

### 9. Reference

**Teorijska osnova:**
- Amdalov zakon: https://en.wikipedia.org/wiki/Amdahl%27s_law
- Gustafsonov zakon: https://en.wikipedia.org/wiki/Gustafson%27s_law
- Skalabilnost: https://www.kth.se/blogs/pdc/2018/11/scalability-strong-and-weak-scaling/

**Numerički metodi:**
- Velocity Verlet integrator
- Simplektički integratori

**N-body simulacije:**
- Barnes-Hut algoritam (opciono, za optimizaciju na O(N log N))
- Fast Multipole Method (opciono)
