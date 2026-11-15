# Cenário de 1925: jazidas minerais e de energia

Os dados de `deposits.ron` estão sob CC BY-SA 4.0. Este arquivo registra de onde vem cada número e
como o arquivo é montado. Formato: `bf_sim::resources::DepositsFile`.

## Como o arquivo é gerado

`deposits.ron` não se edita à mão. A fonte é a tabela curada
`tools/deposits/deposits_1925.csv`, uma linha por jazida, distrito ou agregado nacional:

```
resource,deposit,lon,lat,state,size,discovered,output_1925,potential,source
```

```bash
python3 -I tools/deposits/generate.py            # regrava data/scenario/1925/deposits.ron
python3 -I tools/deposits/generate.py --report   # onde cada linha caiu e os totais de 1925
python3 -I tools/deposits/generate.py --check    # falha se o RON estiver desatualizado
python3 -I tools/deposits/generate.py --where LON LAT
```

O script (precisa do Pillow) projeta `lon`/`lat` com a projeção de Miller do mapa, a mesma fórmula
de `bf-client::world::project`, lê o id da província em `assets/map/provinces.png` (cinza de 16
bits) e chega ao estado por `assets/map/provinces.ron`. Ponto que cai no mar (campo offshore, mina
na costa) fica com a província mais próxima num raio de 60 pixels. A coluna `state`, quando
preenchida, decide sozinha; o relatório mostra o estado que a busca daria, para o desvio ficar à
vista. Foi usada em:

| Linha | Estado | Por quê |
|---|---|---|
| Alta Silésia alemã (carvão, chumbo-zinco) | `GER-OBERSCHLESIEN` | No mapa, Gleiwitz, Hindenburg e Beuthen caem em `POL-SLASK` (limite moderno); em 1925 eram alemãs |
| Bacia de Dąbrowa, bacia de Cracóvia | `POL-KIELCE`, `POL-KRAKOW` | As três bacias ficam a menos de 30 km; o ponto cairia em `POL-SLASK` |
| Petsamo (níquel) | `FIN-OULU` | Kolosjoki era finlandesa de 1920 a 1944; o mapa põe o ponto em `RUS-MUR` |
| Uixan (ferro do Rif) | `ESP-MARRUECOS` | O contorno do Marrocos espanhol no mapa não chega ao ponto |
| Los Pelambres | `CHL-NORTE` | O ponto cai do lado argentino na resolução do mapa |
| Ocean Island (Banaba) | `GBR-GILBERT-ELLICE` | A ilha não tem pixel; a mais próxima é Nauru |
| Ekofisk, Statfjord, Dan, Hibernia, Bombay High, Bohai, Campos, pré-sal, Bach Ho, Kashagan, Sacalina, Zafiro e outros offshore | o país dono da plataforma | A costa mais próxima é de outro país ou fica além do raio |

Linhas que caem no mesmo estado com o mesmo recurso se fundem quando as duas já eram conhecidas em
1925 (produção e potencial somados, `size` o maior, notas juntas). Descoberta posterior a 1925
fica numa entrada própria, para continuar oculta até o ano dela.

Toquepala (cobre) cai em `CHL-NORTE` sem correção: Tacna estava sob administração chilena até 1929.

## Campos

- **`size`** (1 a 5): tamanho da jazida diante das jazidas do mesmo recurso no mundo ao longo do
  século. 5 é classe mundial (Witwatersrand, Ghawar, Chuquicamata, Escondida, Bayan Obo, Pilbara,
  Kalahari); 1 é produtor pequeno ou local.
- **`discovered`**: ano em que a jazida ficou conhecida o bastante para lavra. Até 1925, conhecida
  no início. Mina antiga leva o ano aproximado de início (Rammelsberg, 968). Jazida conhecida
  antes de 1925 mas só lavrada depois (Mount Isa, Trepča, Noranda, Karaganda, Bakony) fica com
  `discovered ≤ 1925` e `output_1925 = 0`: o jogador a vê e pode desenvolvê-la.
- **`output_1925`**: produção de 1925 na unidade do recurso. Zero quando ainda não se lavrava.
- **`potential`**: produção anual sustentável quando plenamente desenvolvida. Na falta de dado
  melhor, usa-se o pico anual que a jazida ou o distrito alcançou entre 1925 e 2025 (pela
  estatística do país ou da empresa) ou, para jazida ainda não lavrada, a capacidade de projeto.
  Isso mede o tamanho com a técnica do fim do século, não a de 1925: o motor deve escalar pelo
  nível de técnica (ver Dúvidas).
- **`note`**: nome da jazida ou do distrito e a etiqueta da fonte entre parênteses.

## Unidades

| Recurso | Unidade |
|---|---|
| coal | Mt de carvão bruto (hulha e linhito) |
| iron_ore | Mt de minério bruto |
| copper | kt de Cu contido |
| bauxite | kt de minério bruto |
| oil | Mt de petróleo bruto |
| natural_gas | bcm de gás comercializado |
| gold | t de Au |
| lead_zinc | kt de Pb + Zn contido |
| tin | kt de Sn contido |
| nickel | kt de Ni contido |
| manganese | kt de minério bruto |
| chromium | kt de cromita bruta |
| tungsten | t de WO3 contido |
| sulfur | kt de S contido (nativo, pirita e recuperado de gás) |
| phosphate | kt de fosfato natural bruto |
| potash | kt de K2O |
| uranium | t de U contido |
| rare_earths | kt de REO (óxidos de terras raras) |

## Fontes

Só fatos (produção, ano de descoberta, localização) entram, nunca tabelas copiadas de obra
protegida. Etiquetas usadas nas notas:

| Etiqueta | Fonte | Licença |
|---|---|---|
| `MRUS1925` | U.S. Bureau of Mines / USGS, *Mineral Resources of the United States, 1925* (e o *Minerals Yearbook* que o sucede) | Domínio público (governo federal dos EUA) |
| `DS140` | USGS Data Series 140, *Historical Statistics for Mineral and Material Commodities* (bauxita mundial 1925 ≈ 1,38 Mt) | Domínio público |
| `MCS` | USGS *Mineral Commodity Summaries* (produção recente, usada para o `potential` e o `size`) | Domínio público |
| `MRDS` | USGS Mineral Resources Data System (localização) | Domínio público |
| `IMRB1925` | Imperial Mineral Resources Bureau, *The Mineral Industry of the British Empire and Foreign Countries, Statistical Summary 1921–1925* | Fatos citados; a tabela não é copiada |
| `hist.` | Ano de descoberta e de início de lavra em histórias de empresas, serviços geológicos e verbetes da Wikidata | Fatos; Wikidata CC0 |

Coordenadas: centro aproximado do distrito, conferido em mapas de domínio público e no MRDS. Nada
vem de fonte NC ou GPL.

## Totais de 1925

Soma de `output_1925` contra a produção mundial de 1925 (o teste `deposits_1925.rs` cobra ±15 %
para carvão, ferro, cobre, bauxita, petróleo e ouro).

| Recurso | Entradas | Ocultas em 1925 | Soma 1925 | Mundo 1925 |
|---|---:|---:|---:|---:|
| coal | 175 | 7 | 1.384 Mt | ≈1.370 Mt (hulha ≈1.180 + linhito ≈190) |
| iron_ore | 91 | 23 | 155 Mt | ≈165 Mt |
| copper | 85 | 31 | 1.356 kt | ≈1.420 kt |
| bauxite | 38 | 18 | 1.238 kt | ≈1.380 kt (DS140) |
| oil | 137 | 87 | 145,4 Mt | ≈146 Mt (1.069 milhões de barris) |
| natural_gas | 63 | 47 | 33,2 bcm | ≈35 bcm (EUA ≈ 95 %) |
| gold | 72 | 15 | 565 t | ≈590 t |
| lead_zinc | 67 | 13 | 2.662 kt | ≈2.800 kt (Pb ≈1.600 + Zn ≈1.200) |
| tin | 25 | 7 | 145,6 kt | ≈145 kt |
| nickel | 22 | 18 | 36,8 kt | ≈37 kt |
| manganese | 22 | 7 | 2.800 kt | ≈2.800 kt |
| chromium | 21 | 8 | 310 kt | ≈300 kt |
| tungsten | 23 | 7 | 6.520 t WO3 | ≈6.500 t (≈11 kt de concentrado a 60 %) |
| sulfur | 24 | 8 | 4.370 kt S | ≈4.400 kt |
| phosphate | 33 | 17 | 9.060 kt | ≈9.500 kt |
| potash | 21 | 11 | 1.595 kt K2O | ≈1.600 kt |
| uranium | 29 | 24 | 66 t | sem mercado; urânio do minério de rádio |
| rare_earths | 12 | 12 | 0 | sem mercado |

## Decisões

- **Bauxita.** O mundo produziu cerca de 1,38 Mt em 1925 (USGS DS140), não 0,5–0,6 Mt. Os produtores
  do primeiro dia são Arkansas, Var e Hérault, Ístria (italiana em 1925), Dalmácia e Herzegovina,
  Guiana Inglesa e Suriname. A Hungria (Gánt, achada em 1920) só começou a lavrar em 1926, e a
  Grécia (Parnassos) também: as duas entram conhecidas, com produção zero.
- **Terras raras.** Toda entrada tem `discovered > 1925`. A monazita de Travancore e da Bahia já
  era lavrada em 1925, mas como minério de tório (camisas de lampião); as terras raras dela só
  viram negócio com a Indian Rare Earths (1950) e a Orquima (1948), e é esse o ano usado.
  Bayan Obo entra duas vezes: ferro em 1927, terras raras em 1936 (identificação).
- **Urânio.** Em 1925 não havia mercado de urânio; `output_1925` é o urânio contido no minério de
  rádio tratado (Shinkolobwe ≈60 t, Joachimsthal ≈5 t, Urgeiriça ≈1 t). O carnotito do Colorado
  e de Utah parou em 1923, diante do rádio do Congo: conhecido, produção zero.
- **Enxofre.** O domo de Sulphur (Louisiana) fechou em dezembro de 1924: conhecido, produção zero.
  O enxofre recuperado de gás ácido (Lacq, Alberta, Astrakhan, Tengiz) entra como jazida com o ano
  da descoberta do gás.
- **Potássio.** Solikamsk (outubro de 1925) e Carlsbad (perfuração de 1925) ficam com 1926: não
  eram conhecidos em 1º de janeiro de 1925.
- **Gás.** Só entra gás comercializado. O gás associado queimado (Oriente Médio antes de 1960) não
  conta.
- **Agregação.** Produtores pequenos de um estado viram uma linha (Appalachia, Sichuan, lavras de
  ouro do México por estado). Bacias que atravessam estados entram uma vez por estado (Donbass em
  três, Shenfu-Dongsheng em dois).
- **Svalbard** não tem estado próprio no mapa; o carvão de Spitsbergen cai em `NOR-NORD-NORGE`.

## Dúvidas

1. `potential` usa o pico do século, não a capacidade com a técnica de 1925 que o contrato
   descreve. O Pilbara (850 Mt/ano), Boké (100 Mt/ano de bauxita) e o Kalahari (15 Mt/ano de
   manganês) só chegam a isso com técnica de 1990+. Ou o motor escala por técnica, ou o contrato
   passa a dizer "pico do século".
2. Os limites modernos do mapa tiram de alguns estados de 1925 a jazida que era deles (Alta
   Silésia, Petsamo, Rif). As linhas foram corrigidas com `state`; o desenho dos estados em
   `provinces.png` merece revisão nesses pontos.
3. Produção de 1925 por país fora dos EUA e do Império Britânico (URSS, China, Japão) tem
   incerteza de 10 a 30 %; a da China mistura minas modernas e lavra nativa estimada.
