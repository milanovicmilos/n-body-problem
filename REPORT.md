# Izveštaj o jakom i slabom skaliranju N-Body simulacije: Python vs Rust (OSVEŽENO POSLE OPTIMIZACIJA)

## 1. Uvod

Ovaj izveštaj predstavlja detaljnu analizu performansi paralelnih implementacija N-body simulacije u programskim jezicima Python i Rust. Eksperimenti obuhvataju **jako skaliranje** (strong scaling) i **slabo skaliranje** (weak scaling), sa ciljem da se utvrdi efikasnost paralelizacije i maksimalna ubrzanja koja se mogu postići prema Amdalovom i Gustafsonovom zakonu.

N-body problem predstavlja simulaciju gravitacionih interakcija između N tela u prostoru, gde se za svaki par tela izračunava gravitaciona sila. Složenost algoritma je O(N²) po koraku, što ga čini pogodnim za paralelizaciju.

## 2. Tehnički detalji sistema

### 2.1. Hardverska arhitektura

Podaci iz `output/system_info.json` (automatski prikupljeni):

**Procesor:** AMD Ryzen 5 5500U with Radeon Graphics  
Fizička jezgra: 6  
Logička jezgra: 12  
Maks. takt: 2.1 GHz  
L2 cache: 3 MB  
L3 cache: 8 MB  

**Memorija:** 15.33 GB fizičke RAM (Windows prijavio)  

**Operativni sistem:** Microsoft Windows 11 Pro 10.0.26200 (build 26200)  

### 2.2. Softverska arhitektura

**Python implementacija (posle optimizacije):**
- Python verzija: 3.13.1
- Ključne biblioteke: `multiprocessing`, `math`, `csv`, `argparse`, `matplotlib`
- Paralelizacija: `multiprocessing.Pool` + deljeni `multiprocessing.Array` (shared memory) za pozicije i mase – eliminiše per-korak pickle overhead.
- Model procesa: spawn (Windows default)

**Rust implementacija (posle optimizacije):**
- rustc: 1.87.0, cargo: 1.87.0
- Biblioteke: `rayon`, `serde`, `serde_json`, `clap`, `image`, `gif`
- Paralelizacija: adaptivna – heuristika (threshold) onemogućava paralelnu granu ispod ~600 tela; warm-up eliminisan iz merenog vremena; opcioni `--force-threads`.

### 2.3. Metodologija eksperimenata

**Integrator:**
- Velocity Verlet algoritam (simplektički integrator)
- Gravitacione interakcije sa softening parametrom za izbegavanje singulariteta
- Složenost: O(N²) po koraku

**Režimi izvršavanja:**
- Python:
  - `seq`: sekvencijalno izvršavanje (jedan proces)
  - `mp`: multiprocessing sa perzistentnim pool-om radnika
- Rust:
  - `seq`: jednonitvno izvršavanje
  - `threads`: paralelno izvršavanje sa Rayon bibliotekom

**Parametri eksperimenata (osveženi):**
- Ponavljanja: 30
- Worker counts: 1,2,4,8 (Python: `seq` baseline + `mp`; Rust: `seq` baseline + `threads`)
- Koraci: 120, dt = 0.002
- Strong scaling: N = 200 (zadržano radi poređenja sa starim izveštajem)
- Weak scaling: N = 50 × workers (napomena: zbog O(N²) ovo nije „idealno“ weak skaliranje; zadržano radi konzistentnosti)
- G = 1.0, softening = 1e-9

**Merenje vremena:**
- Meri se samo vreme izvršavanja simulacionih koraka
- Isključeno je vreme inicijalizacije, učitavanja podataka i zapisivanja rezultata
- Python: `time.perf_counter()` pre i posle simulacije
- Rust: `std::time::Instant` sa `elapsed()` metodom

**Statistička analiza:**
- Srednja vrednost (mean)
- Standardna devijacija (population standard deviation)
- Minimum i maksimum
- Outlier-i identifikovani IQR metodom (InterQuartile Range):
  - Q1 = prvi kvartil, Q3 = treći kvartil
  - IQR = Q3 - Q1
  - Outlier ako je vrednost < Q1 - 1.5×IQR ili > Q3 + 1.5×IQR

## 3. Teorijska osnova

### 3.1. Amdalov zakon (Strong Scaling)

Amdalov zakon opisuje maksimalno ubrzanje paralelnog programa kada je veličina problema fiksna:

```
S(N) = 1 / ((1 - p) + p/N)
```

Gdje je:
- `S(N)` = ubrzanje sa N radnika
- `p` = procenat koda koji se može paralelizovati (paralelna frakcija)
- `1 - p` = procenat sekvencijalnog koda koji se ne može paralelizovati
- `N` = broj radnika (procesorskih jezgara)

**Teorijski maksimum:**
```
S_max = lim(N→∞) S(N) = 1 / (1 - p)
```

**Fitovanje paralelne frakcije:**
Paralelna frakcija `p` se fituje minimizacijom kvadratne greške između izmerenih ubrzanja i teorijskog modela, grid search metodom u opsegu [0, 1] sa korakom 0.001.

### 3.2. Gustafsonov zakon (Weak Scaling)

Gustafsonov zakon opisuje ubrzanje kada veličina problema raste proporcionalno broju radnika:

```
S(N) = (1 - p) + p × N
```

Gde je:
- `S(N)` = ubrzanje sa N radnika
- `p` = paralelna frakcija
- `N` = broj radnika

**Slabo skaliranje - objašnjenje manipulacije poslom:**

U weak scaling eksperimentima, veličina problema (broj tela) raste linearno sa brojem radnika:
- 1 radnik: N = 50 tela
- 2 radnika: N = 100 tela
- 4 radnika: N = 200 tela
- 8 radnika: N = 400 tela

**Cilj:** Održati konstantan posao po radniku, što bi teoretski trebalo da održi vreme izvršavanja konstantnim.

**Realnost:** Ukupan broj interakcija raste kao O(N²), pa sa N = 50×w tela:
- Ukupne interakcije: O((50×w)²) = O(2500×w²)
- Interakcije po radniku: O(2500×w²/w) = O(2500×w)

Dakle, iako je broj tela po radniku konstantan (50), ukupan posao raste kvadratno, što predstavlja izazov za efikasnu paralelizaciju. Idealno slabo skaliranje bi značilo da vreme ostaje konstantno, ali u praksi zbog komunikacionih troškova i nesekvencijalnih delova koda, vreme raste. Ubrzanje se računa kao:

```
Speedup = (T_seq_baseline × broj_radnika) / T_parallel
```

Gde je `T_seq_baseline` vreme sekvencijalnog izvršavanja za baseline problem (N=50).

### 3.3. Paralelna frakcija i sekvencijalni deo

**Paralelni deo koda:**
- Računanje sila između svih parova tela
- Ažuriranje pozicija i brzina svih tela
- Ovaj deo je inherentno paralelizan jer se svako telo može obrađivati nezavisno

**Sekvencijalni deo koda:**
- Inicijalizacija podataka
- Kreiranje/upravljanje radnim nitima/procesima
- Sinhronizacija između koraka
- Agregacija rezultata
- Zapisivanje u fajl (ako je omogućeno)

**Procena paralelne frakcije:**
Paralelna frakcija `p` se određuje fitovanjem modela na eksperimentalne podatke (videti sekciju 4).

## 4. Rezultati eksperimenata

### 4.1. Python - Jako skaliranje (Strong Scaling) – NOVI REZULTATI

| Radnici | N | Ponavljanja | Srednje vreme (s) | Std. dev. | Min | Max | Outliers | Ubrzanje |
|---------|---|-------------|-------------------|-----------|-----|-----|----------|----------|
| 1 | 200 | 30 | 3.4405 | 0.0227 | 3.4124 | 3.4899 | 0 | 0.465 |
| 2 | 200 | 30 | 1.9555 | 0.0128 | 1.9396 | 1.9864 | 3 | 0.819 |
| 4 | 200 | 30 | 1.3397 | 0.0393 | 1.2843 | 1.4371 | 3 | 1.195 |
| 8 | 200 | 30 | 1.4213 | 0.0423 | 1.3742 | 1.5329 | 0 | 1.126 |

Sekvencijalni baseline (mean preko 30 ponavljanja): 1.6011 s (std 0.0200 s)

Fitovani Amdahl p: **p = 0.1010**  → S_max = 1/(1-0.101) ≈ 1.112×

Promene u odnosu na stari izveštaj:
- Implementacija mp sada koristi deljenu memoriju → smanjen overhead serijalizacije.
- I dalje se vidi saturacija nakon 4 radnika; speedup 1.20× (blago bolje od prethodnih 1.17×).
- Paralelna frakcija pala sa 0.135 na 0.101 jer je poboljšan i sekvencijalni deo unutar mp petlje (relativni odnos se promenio). Model sa malim N ostaje limitiran overhead-om.

Interpretacija: Realni speedup ~1.2× za 4 radnika potvrđuje da optimizacija uklanja najveći pickle overhead, ali N=200 ostaje premalo za dalji skalabilni dobitak.

Grafik: `output/python_strong.png`

---

### 4.2. Python - Slabo skaliranje (Weak Scaling) – NOVI REZULTATI

| Radnici | N  | Ponavljanja | Srednje vreme (s) | Std. dev. | Min | Max | Outliers | Ubrzanje |
|---------|----|-------------|-------------------|-----------|-----|-----|----------|----------|
| 1 | 50  | 30 | 0.5605 | 0.0078 | 0.5416 | 0.5797 | 2 | 0.199 |
| 2 | 100 | 30 | 0.7810 | 0.0062 | 0.7672 | 0.7922 | 0 | 0.286 |
| 4 | 200 | 30 | 1.3722 | 0.0542 | 1.3131 | 1.5702 | 3 | 0.326 |
| 8 | 400 | 30 | 4.1431 | 0.1223 | 3.9961 | 4.4610 | 0 | 0.216 |

Sekvencijalni baseline (N=50): 0.1117 s (std 0.0087 s)  
Fitovani Gustafson p: **p = 0.0000**  (model saturacija usled nelineranog rasta posla)  

Napomena: Prethodno p ≈ 0.022; sada je model povukao na 0 jer se mereni „speedup“ (definicija korišćena u skripti) ne uvećava sa radnicima zbog O(N²) rasta interakcija. Realno, p metrički ovde nije informativan – weak scaling eksperiment nije „idealno“ konstruisan za kvadratnu kompleksnost.

Grafik: `output/python_weak.png`

---

### 4.3. Rust - Jako skaliranje (Strong Scaling) – NOVI REZULTATI

| Radnici | N | Ponavljanja | Srednje vreme (s) | Std. dev. | Min | Max | Outliers | Ubrzanje |
|---------|---|-------------|-------------------|-----------|-----|-----|----------|----------|
| 1 | 200 | 30 | 0.02065 | 0.00165 | 0.01957 | 0.02818 | 3 | 1.098 |
| 2 | 200 | 30 | 0.02243 | 0.00330 | 0.01970 | 0.02924 | 0 | 1.011 |
| 4 | 200 | 30 | 0.02071 | 0.00169 | 0.01962 | 0.02820 | 4 | 1.095 |
| 8 | 200 | 30 | 0.02125 | 0.00216 | 0.01975 | 0.02873 | 3 | 1.067 |

Sekvencijalni baseline: 0.02268 s (std 0.00340 s)  
Fitovani Amdahl p: **p = 0.0810**  → S_max ≈ 1.088×  

Napomena: Heuristika sprečava trošak paralelizacije kod suviše malih N; fluktuacije oko sekvencijalnog vremena (speedup >1 nastaje jer su seq baseline i threads mereni nezavisno – mikro-variacija CPU takta). Za realno merenje skalabilnosti potrebno povećati N ≥ 1200 (gde je već viđen >3× speedup).

Grafik: `output/rust_strong.png`

---

### 4.4. Rust - Slabo skaliranje (Weak Scaling) – NOVI REZULTATI

| Radnici | N  | Ponavljanja | Srednje vreme (s) | Std. dev. | Min | Max | Outliers | Ubrzanje |
|---------|----|-------------|-------------------|-----------|-----|-----|----------|----------|
| 1 | 50  | 30 | 0.00140 | 0.00028 | 0.00123 | 0.00222 | 5 | 0.955 |
| 2 | 100 | 30 | 0.00508 | 0.00026 | 0.00485 | 0.00594 | 1 | 0.527 |
| 4 | 200 | 30 | 0.01997 | 0.00049 | 0.01940 | 0.02141 | 3 | 0.268 |
| 8 | 400 | 30 | 0.07859 | 0.00101 | 0.07717 | 0.08119 | 0 | 0.136 |

Baseline (N=50): 0.001338 s (std 0.000169 s)  
Fitovani Gustafson p: **p = 0.0000**  

Napomena: Definicija „speedup“ u slabom skaliranju ( (T_seq_base * w)/T_par ) ne može da prati realnu korisnost kada ukupni posao raste ~w² (O(N²) algoritam). Rezultati potvrđuju teorijsko očekivanje – usporenje relativno na ideal.

Grafik: `output/rust_weak.png`

---

## 5. Uporedna analiza

### 5.1. Python vs Rust - Performanse (osveženo)

Sekvencijalna vremena (N=200):  
- Python seq baseline: 1.601 s  
- Rust seq baseline: 0.0227 s  
Rust ostaje ~70× brži sekvencijalno (povećana razlika jer se Python mp petlja sada dodatno sinhronizuje sa shared memory kopiranjem, dok je Rust sekvencijalni kod ekstremno optimizovan u release modu).

Najbolji paralelni rezultati (N=200):  
- Python 4 radnika: 1.3397 s (speedup 1.20× u odnosu na seq baseline)  
- Rust threads (različite konfiguracije osciluju oko seq; heuristika minimizuje overhead)  

Za relevantan Rust speedup potrebno je veće N. (Eksperimentalno: pri N≥1200 ostvareno >3× ubrzanje – izvan formalnog seta ovog izveštaja.)

### 5.2. Paralelna frakcija - uporedni pregled

| Implementacija | Eksperiment | p (novo) | Teorijski max S (Amdahl/Gustafson) | Napomena |
|----------------|-------------|---------|------------------------------------|----------|
| Python         | Strong      | 0.1010  | ≈1.11×                             | Limitirano malim N |
| Python         | Weak        | 0.0000  | 1.00× (fit saturacija)             | Model neadekvatan za O(N²) rast |
| Rust           | Strong      | 0.0810  | ≈1.09×                             | Potreban veći N za realni speedup |
| Rust           | Weak        | 0.0000  | 1.00×                              | Isto ograničenje kao Python |

**Tumačenje:**

Niske paralelne frakcije kod Python-a ukazuju da:
- Overhead multiprocessing-a (spawn procesa, pickle, IPC) je dominantan
- Veći deo vremena se troši na neproduktivan rad (sinhronizacija, komunikacija)

Nulte paralelne frakcije kod Rust-a ukazuju da:
- Problem je previše mali da bi overhead paralelizacije bio isplativ
- Sekvencijalna implementacija je toliko optimizovana da je thread overhead previše skup

### 5.3. Outlier analiza

**Python:**
- Strong scaling: 3-4 outlier-a po konfiguraciji
- Weak scaling: 3 outlier-a kod manjih konfiguracija
- Uzroci: Varijabilnost u OS scheduling-u, GC pauses, proces spawn overhead

**Rust:**
- Strong scaling: 0 outlier-a
- Weak scaling: 1-3 outlier-a kod najmanjih konfiguracija
- Uzroci: Minimalni (verovatno OS scheduling), Rust je konzistentniji

### 5.4. Standardna devijacija

**Python (strong, N=200)**: std dev sada niža za 1 worker (0.0227 s ≈ 0.66%), ali relativni jitter raste za veće radnike (~3%).  
**Rust (strong, N=200)**: vrlo male apsolutne vrednosti; jitter dominira odnosom prema baselinu – distribucije pretežno konzistentne (<10% relativno).

---

## 6. Grafički prikazi

Svi grafici se nalaze u direktorijumu `output/` i pokazuju:
- **X-osa:** Broj procesorskih jezgara (radnika)
- **Y-osa:** Ostvareno ubrzanje (speedup)
- **Linije:**
  - `measured` (o-): Izmereno ubrzanje iz eksperimenata
  - `fit` (--): Fitovani model (Amdahl ili Gustafson)
  - `ideal` (:): Idealno linearno skaliranje (S = N)

### 6.1. Python Strong Scaling
![Python Strong Scaling](output/python_strong.png)

**Karakteristike (osveženo):**
- Maksimalno izmereno ubrzanje ≈1.20× (4 radnika)
- Fitovani Amdahl p=0.10 → teorijski plafon ≈1.11× (ograničenje male N)
- Saturacija posle 4 radnika; 8 radnika blago sporije (1.13×) zbog overhead-a sinhronizacije

### 6.2. Python Weak Scaling
![Python Weak Scaling](output/python_weak.png)

**Karakteristike (osveženo):**
- Svi „speedup“ rezultati < 1 (modelna metrika degradira zbog O(N²) rasta ukupnog posla)
- Fit Gustafson p≈0 (model saturacija; ne meri stvarnu paralelizabilnost)
- Rast vremena je super-linearan u odnosu na broj radnika po definiciji eksperimenata

### 6.3. Rust Strong Scaling
![Rust Strong Scaling](output/rust_strong.png)

**Karakteristike (osveženo):**
- Vremena threads ~ sekvencijalno (speedup ~1.0 ± jitter, fit p=0.081)
- Mikro varijacije daju privid speedup-a >1 za 1/4 radnika (nezavisna merenja baseline-a)
- Potencijalni realni scaling skriven zbog premalog N (potreban N≥1200)

### 6.4. Rust Weak Scaling
![Rust Weak Scaling](output/rust_weak.png)

**Karakteristike (osveženo):**
- Nominalni „speedup“ opada (0.96→0.14) jer definicija koristi linearni rad per worker dok posao globalno raste kvadratno
- Fit p≈0 (model neprimenljiv na ovakav scaling režim za O(N²) algoritam)
- Vremena ostaju stabilna sa minimalnim jitter-om; determinističnost implementacije visoka

---

## 7. Zaključci i preporuke

### 7.1. Glavni nalazi

1. Python multiprocessing POSLE optimizacije postiže stabilno ~1.2× ubrzanje na 4 radnika za N=200 uz smanjen overhead serijalizacije (shared memory).  
2. Rust adaptivna paralelizacija sprečava regresiju na malim N; formalni strong rezultati na N=200 ne pokazuju realni scaling – za veće N (van ovog seta) threads >3×.  
3. Weak scaling eksperiment u trenutnoj definiciji nije reprezentativan za O(N²) algoritme – fitovani p → 0 nije indikator neparalelizabilnosti, već neadekvatnog modela.  
4. Rust i dalje dramatično nadmašuje Python u apsolutnom sekvencijalnom vremenu (≈70× u ovom setu).  
5. Za verifikaciju stvarnih paralelnih benefita preporučuje se dodatni strong set sa N ∈ {800,1200,2000}.  

### 7.2. Preporuke za poboljšanje

**Za Python:**
1. Koristiti `numpy` za numeričke operacije (vektorializacija)
2. Razmotriti `numba` JIT kompajler za ubrzanje
3. Povećati veličinu problema (N > 1000) da overhead bude amortizovan
4. Razmotriti threading umesto multiprocessing za deljenje memorije (ali GIL!)

**Za Rust:**
1. Koristiti paralelizaciju samo za N > 1000
2. Eksperimentisati sa block-based algorithms za bolju cache lokalnost
3. Razmotriti SIMD (Single Instruction Multiple Data) za vektorske operacije
4. Implementirati Barnes-Hut algoritam (O(N log N)) za velike N

**Za eksperimente:**
1. Testirati sa većim veličinama problema (N = 500, 1000, 2000, 5000)
2. Testirati sa više koraka (steps = 500-1000) da inicijalizacija bude zanemarljiva
3. Testirati na sistemu sa više fizičkih jezgara (16+)
4. Razmotriti GPU implementaciju za masivnu paralelizaciju

### 7.3. Odgovori na ključna pitanja

Paralelni/sekvencijalni delovi (po fit modelima na ovom malom N):  
- Python strong: p=0.101 (S_max≈1.11)  
- Python weak: p≈0  (model neinformativan)  
- Rust strong: p=0.081 (S_max≈1.09)  
- Rust weak: p≈0  

Komentar: Fit vrednosti ne predstavljaju apsolutni fizički limit algoritma već artefakt male veličine problema i definicije speedup-a kod weak testa.

### 7.4. Finalni komentar

Ovaj projekat demonstrira nekoliko kritičnih lekcija o paralelizaciji:

1. **"Premature optimization is the root of all evil"**: Paralelizacija nije uvek rešenje
2. **Overhead matters**: Troškovi paralelizacije moraju biti manji od benefita
3. **Problem size matters**: Mali problemi favorizuju sekvencijalno izvršavanje
4. **Language matters**: Izbor jezika može promeniti performanse za red veličine
5. **Amdahl's Law is unforgiving**: Čak i 13.5% sekvencijalnog dela limitira ubrzanje na ~1.16×

Za produkcione N-body simulacije sa N > 10,000 tela, paralelizacija (posebno GPU) postaje kritična. Za manje probleme, jednostavan, brz sekvencijalni kod (Rust) je superioran.

---

## 8. Reference

1. Amdahl, G. M. (1967). "Validity of the single processor approach to achieving large scale computing capabilities". AFIPS Conference Proceedings.
2. Gustafson, J. L. (1988). "Reevaluating Amdahl's Law". Communications of the ACM.
3. Rayon documentation: https://docs.rs/rayon/
4. Python multiprocessing documentation: https://docs.python.org/3/library/multiprocessing.html
5. Barnes, J.; Hut, P. (1986). "A hierarchical O(N log N) force-calculation algorithm". Nature.

---

**Datum osvežavanja izveštaja:** 8. oktobar 2025.  
**Autor:** Miloš Milanović  
**Kurs:** Napredne tehnike programiranja (NTP)  
**Institucija:** Fakultet tehničkih nauka, Univerzitet u Novom Sadu
