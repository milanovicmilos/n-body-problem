# Projekat: N-Body simulacija u Python-u i Rust-u (HPC) - Miloš Milanović SV32/2021

## 0) Ciljna ocena i opis problema

Ciljna ocena: 10 

Tema koju sam odabrao je N-body problem, koji predstavlja jedan od najpoznatijih i najvažnijih izazova u oblasti fizike i računarskih simulacija.

N-body problem opisuje situaciju u kojoj više tela u prostoru međusobno deluje gravitacionim silama. Svako telo svojim prisustvom utiče na svako drugo, a kako broj tela raste, tako raste i složenost problema. Za mali broj tela problem se može rešiti analitički, ali za veći broj (stotine, hiljade ili više) potrebno je koristiti numeričke metode i simulacije. Upravo zato je N-body problem odličan primer za ispitivanje efikasnosti algoritama i optimizacija u računarstvu.

U ovom radu koristiću različite metode za rešavanje problema. Najpre ću implementirati naivni pristup (brute-force), gde se računa interakcija svake čestice sa svakom drugom, što ima vremensku složenost O(n²). Nakon toga ću primeniti RSUT (Barnes–Hut) algoritam, koji koristi hijerarhijsku dekompoziciju prostora i aproksimacije da bi značajno smanjio broj potrebnih računanja, čime se složenost svodi na O(n log n). Na kraju ću razmotriti i mogućnosti paralelizacije i optimizacije korišćenjem modernih tehnologija poput programiranja na GPU-u (npr. CUDA) ili višeprocesorskog računanja.

Na ovaj način projekat ne samo da rešava jedan klasičan matematičko-fizički problem, već i pokazuje kako se kroz algoritamski dizajn i računarske metode mogu prevazići izazovi skalabilnosti i efikasnosti.

---
