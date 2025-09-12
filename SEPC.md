# N-Body Problem - Projekat

Potrebno je uraditi sledeće:

1. (ocena 6) Rešiti problem upotrebom programskog jezika Python:

   - Implementirati sekvencijalnu verziju rešenja upotrebom programskog jezika Python. Rezultat mora biti bar jedna datoteka koja reprezentuje promene stanja modelovanog sistema (po iteracijama, ukoliko je problem rešavan iterativnim metodom).

   - Implementirati paralelizovanu verziju rešenja upotrebom `multiprocessing` biblioteke programskog jezika Python. Rezultat mora biti bar jedna datoteka koja reprezentuje promene stanja modelovanog sistema (po iteracijama, ukoliko je problem rešavan iterativnim metodom).

2. (ocena 7) Rešiti problem upotrebom programskog jezika Rust:

   - Implementirati sekvencijalnu verziju rešenja upotrebom programskog jezika Rust. Rezultat mora biti bar jedna datoteka koja reprezentuje promene stanja modelovanog sistema (po iteracijama, ukoliko je problem rešavan iterativnim metodom).

   - Implementirati paralelizovanu verziju rešenja uz oslonac na niti (engl. threads). Rezultat mora biti bar jedna datoteka koja reprezentuje promene stanja modelovanog sistema (po iteracijama, ukoliko je problem rešavan iterativnim metodom).

3. (ocena 8) Uraditi eksperimente jakog i slabog skaliranja, koji će uporediti dobijeno ubrzanje paralelizovane Python implementacije rešenja u odnosu na sekvencijalnu implementaciju upotrebom istog jezika.

4. (ocena 9) Uraditi eksperimente jakog i slabog skaliranja koji će uporediti dobijeno ubrzanje paralelizovane Rust implementacije rešenja u odnosu na sekvencijalnu implementaciju upotrebom istog jezika.

5. (ocena 10) Vizualizacija rešenja (po iteracijama, ukoliko je korišćen iterativni model za rešavanje problema) na osnovu prethodno generisanih datoteka, a uz oslonac na Rust okruženje. Dozvoljena je upotreba grafičkih biblioteka poput Plotters ili slično.

Eksperimente jakog i slabog skaliranja opisati u formi izveštaja. Tom prilikom, potrebno je ispoštovati sledeće stavke:

- Navesti tehničke detalje koji se tiču hardverske i softverske arhitekture sistema na kom su rađeni eksperimenti:

  - model procesora, radni takt, organizacija cache memorije, broj fizičkih/logičkih jezgara, broj NUMA node-ova, itd.
  - tip i količina RAM memorije
  - Operativni sistem
  - Dodatne biblioteke koju su korišćene, kao i njihove verzije
  - ostale informacije koje mogu uticati na rezultate eksperimenata.

- Odrediti procenat sekvencijalnog dela koda koji se po prirodi problema ne može paralelizovati.

- Odrediti procenat paralelnog dela koda koji se može paralelizovati.

- Odrediti teorijske maksimume ubrzanja u skladu sa Amdalovim, odnosno Gustafsonovim zakonom. Kao veoma koristan izvor informacija može poslužiti sledeći članak.

- Neophodno je generisati bar 4 grafika:

  - jako skaliranje Python paralelne implementacije u skladu sa Amdalovim zakonom
  - jako skaliranje Rust paralelne implementacije u skladu sa Amdalovim zakonom
  - slabo skaliranje Python paralelne implementacije u skladu sa Gustafsonovim zakonom
  - slabo skaliranje Rust paralelne implementacije u skladu sa Gustafsonovim zakonom

- Na svakom od prethodno pomenutih grafika, x-osa predstavlja broj procesorskih jezgara, dok y-osa predstavlja ostvareno ubrzanje. Takođe, na svakom grafiku nacrtati liniju teorijskog maksimuma (idealnog skaliranja) i uporediti je sa dobijenim rezultatima.

- Kod eksperimenta slabog skaliranja, objasniti na koji način se manipuliše poslom, tj. kako se modifikacijom parametara postiže konstantan posao po procesorskom jezgru.

- Svaki grafik treba da sledi potporna tabela sa informacijama o srednjem vremenu izvršavanja, standardnoj devijaciji, kao i o eventualnim outlier-ima. Kako bi rezultati bili relevantni, za svaku kombinaciju parametara jakog, odnosno slabog skaliranja (broj procesorskih jezgra, veličina problema iskazana odgovarajućom kombinacijom ulaznih argumenata programa, itd.), ekskluzivno izvršiti programsko rešenje 30-ak puta.

Dodatne informacije koje se tiču jakog i slabog skaliranja možete pronaći u https://www.kth.se/blogs/pdc/2018/11/scalability-strong-and-weak-scaling/

