<!-- Página gerada de `data/` por `crates/bf-sim/tests/wiki_reference.rs`. Não edite à mão: regere com `BF_BLESS=1 cargo test -p bf-sim --test wiki_reference`. -->

# Forças: tipos e gerações

Na ordem do catálogo. Um elemento é mil homens em terra, um navio no mar, uma aeronave no ar, uma arma ou lançador no estratégico. Equipamento por elemento; manutenção por elemento e por semana; papéis em milésimos da força do elemento.

## Milícia e guerrilha (`irregular`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 1000 homens | Armas leves 321,25 | Alimentos industrializados 5, Roupas 6,425, Munição 6,425 | 4 semanas | — |

## Guarnição (`garrison`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 1000 homens | Armas leves 771, Artilharia 128,5 | Alimentos industrializados 5, Roupas 12,85, Munição 6,425 | 8 semanas | — |

## Infantaria (`infantry`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Metralhadora | 1000 homens | Armas leves 1285, Artilharia 257 | Alimentos industrializados 5, Roupas 12,85, Munição 12,85 | 16 semanas | — |
| 2 | Metralhadora, Produção em massa de armamento | 1000 homens | Automóveis 128,5, Armas leves 1799, Artilharia 514 | Alimentos industrializados 5, Combustíveis 2, Roupas 12,85, Munição 25,7 | 16 semanas | — |
| 3 | Metralhadora, Produção em massa de armamento, Míssil guiado | 1000 homens | Componentes eletrônicos 257, Automóveis 385,5, Armas leves 2570, Artilharia 771 | Alimentos industrializados 5, Combustíveis 5, Roupas 12,85, Medicamentos 6,425, Munição 38,55 | 16 semanas | — |

## Infantaria leve (`light_infantry`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Metralhadora | 1000 homens | Armas leves 1156,5, Artilharia 128,5 | Alimentos industrializados 5, Roupas 12,85, Munição 12,85 | 24 semanas | — |
| 2 | Metralhadora, Produção em massa de armamento | 1000 homens | Armas leves 1799, Artilharia 257 | Alimentos industrializados 5, Roupas 12,85, Munição 12,85 | 24 semanas | — |
| 3 | Metralhadora, Produção em massa de armamento, Míssil guiado | 1000 homens | Componentes eletrônicos 257, Armas leves 2570, Artilharia 514 | Alimentos industrializados 5, Roupas 12,85, Medicamentos 6,425, Munição 25,7 | 24 semanas | — |

## Cavalaria (`cavalry`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 1000 homens | Armas leves 1028, Artilharia 64,25 | Cereais 30, Alimentos industrializados 5, Roupas 12,85, Munição 6,425 | 20 semanas | — |

## Artilharia (`artillery`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 1000 homens | Armas leves 257, Artilharia 3212,5 | Cereais 20, Alimentos industrializados 5, Roupas 12,85, Munição 38,55 | 20 semanas | — |
| 2 | Automóvel e caminhão | 1000 homens | Automóveis 514, Armas leves 257, Artilharia 3855 | Alimentos industrializados 5, Combustíveis 10, Roupas 12,85, Munição 38,55 | 20 semanas | — |
| 3 | Automóvel e caminhão, Míssil guiado | 1000 homens | Componentes eletrônicos 642,5, Automóveis 771, Armas leves 257, Artilharia 5782,5 | Alimentos industrializados 5, Combustíveis 10, Roupas 12,85, Munição 38,55 | 20 semanas | — |
| 4 | Automóvel e caminhão, Míssil guiado, Munição guiada a laser | 1000 homens | Componentes eletrônicos 1542, Automóveis 771, Armas leves 257, Artilharia 6425 | Alimentos industrializados 5, Combustíveis 10, Roupas 12,85, Munição 38,55 | 20 semanas | — |

## Engenharia (`engineers`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 1000 homens | Ferramentas 385,5, Máquinas 257, Armas leves 642,5 | Alimentos industrializados 5, Roupas 12,85, Munição 6,425 | 20 semanas | engenharia 1000 |
| 2 | Máquinas de terraplenagem | 1000 homens | Automóveis 385,5, Ferramentas 514, Máquinas 1028, Armas leves 771 | Alimentos industrializados 5, Combustíveis 8, Roupas 12,85, Munição 6,425 | 20 semanas | engenharia 1000 |

## Antiaérea (`anti_air`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Aviação | 1000 homens | Automóveis 257, Artilharia 1927,5 | Alimentos industrializados 5, Combustíveis 5, Roupas 12,85, Munição 25,7 | 20 semanas | antiaéreo 1000 |
| 2 | Aviação, Radar | 1000 homens | Automóveis 385,5, Equipamento elétrico 385,5, Artilharia 2570 | Alimentos industrializados 5, Combustíveis 5, Roupas 12,85, Munição 25,7 | 20 semanas | antiaéreo 1000 |
| 3 | Aviação, Radar, Míssil guiado | 1000 homens | Componentes eletrônicos 1028, Automóveis 514, Artilharia 3855 | Alimentos industrializados 5, Combustíveis 5, Roupas 12,85, Munição 25,7 | 20 semanas | antiaéreo 1000 |

## Blindados (`armor`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Carro de combate | 1000 homens | Armas leves 257, Blindados 3855 | Alimentos industrializados 5, Combustíveis 20, Roupas 12,85, Munição 25,7 | 26 semanas | — |
| 2 | Carro de combate, Guerra blindada | 1000 homens | Equipamento elétrico 128,5, Armas leves 385,5, Blindados 6425 | Alimentos industrializados 5, Combustíveis 30, Roupas 12,85, Munição 38,55 | 26 semanas | — |
| 3 | Carro de combate, Guerra blindada, Blindagem composta | 1000 homens | Componentes eletrônicos 642,5, Armas leves 385,5, Blindados 11565 | Alimentos industrializados 5, Combustíveis 40, Roupas 12,85, Munição 51,4 | 26 semanas | — |

## Motorizada e mecanizada (`mechanized`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Automóvel e caminhão | 1000 homens | Automóveis 771, Armas leves 1285, Artilharia 257 | Alimentos industrializados 5, Combustíveis 10, Roupas 12,85, Munição 12,85 | 20 semanas | — |
| 2 | Automóvel e caminhão, Guerra blindada | 1000 homens | Automóveis 385,5, Armas leves 1542, Artilharia 385,5, Blindados 1927,5 | Alimentos industrializados 5, Combustíveis 20, Roupas 12,85, Munição 25,7 | 20 semanas | — |
| 3 | Automóvel e caminhão, Guerra blindada, Blindagem composta | 1000 homens | Componentes eletrônicos 385,5, Armas leves 1927,5, Artilharia 642,5, Blindados 5140 | Alimentos industrializados 5, Combustíveis 30, Roupas 12,85, Munição 38,55 | 20 semanas | — |

## Paraquedistas (`airborne`)

Terra

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Avião todo de metal | 1000 homens | Armas leves 1413,5, Artilharia 128,5 | Alimentos industrializados 5, Roupas 12,85, Munição 12,85 | 40 semanas | — |
| 2 | Avião todo de metal, Helicóptero, Turbina a gás | 1000 homens | Armas leves 1927,5, Artilharia 257, Aviões militares 771 | Alimentos industrializados 5, Combustíveis 10, Roupas 12,85, Munição 25,7 | 40 semanas | — |

## Encouraçados (`battleship`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 1200 homens, 30000 t | Navios de guerra 385500 | Alimentos industrializados 6, Combustíveis 300, Munição 64,25 | 26 semanas | canhão 850, antiaéreo 100 |
| 2 | Radar | 1800 homens, 40000 t | Equipamento elétrico 6425, Navios de guerra 578250 | Alimentos industrializados 9, Combustíveis 400, Munição 77,1 | 26 semanas | canhão 700, antiaéreo 250 |

## Porta-aviões (`carrier`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Porta-aviões | 2000 homens, 25000 t, 60 aeronaves | Navios de guerra 321250 | Alimentos industrializados 10, Combustíveis 250, Munição 38,55 | 26 semanas | canhão 100, antiaéreo 150 |
| 2 | Porta-aviões, Motor a jato | 3000 homens, 45000 t, 70 aeronaves | Equipamento elétrico 12850, Navios de guerra 771000 | Alimentos industrializados 15, Combustíveis 450, Munição 51,4 | 26 semanas | canhão 100, antiaéreo 150 |
| 3 | Porta-aviões, Motor a jato, Energia nuclear | 5000 homens, 90000 t, 80 aeronaves | Componentes eletrônicos 64250, Navios de guerra 1927500 | Alimentos industrializados 25, Combustível nuclear 2, Munição 64,25 | 26 semanas | antiaéreo 250 |

## Cruzadores (`cruiser`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 600 homens, 8000 t | Navios de guerra 102800 | Alimentos industrializados 3, Combustíveis 80, Munição 25,7 | 26 semanas | canhão 650, torpedo 100, antissubmarino 50, antiaéreo 100 |
| 2 | Míssil guiado | 600 homens, 9000 t | Componentes eletrônicos 25700, Navios de guerra 192750 | Alimentos industrializados 3, Combustíveis 90, Munição 38,55 | 26 semanas | canhão 200, antissubmarino 100, antiaéreo 450, antinavio 200 |

## Contratorpedeiros (`destroyer`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 120 homens, 1200 t | Navios de guerra 19275 | Alimentos industrializados 0,6, Combustíveis 12, Munição 6,425 | 20 semanas | canhão 250, torpedo 450, antissubmarino 200, antiaéreo 50 |
| 2 | Radar | 300 homens, 2500 t | Equipamento elétrico 2570, Navios de guerra 44975 | Alimentos industrializados 1,5, Combustíveis 25, Munição 12,85 | 20 semanas | canhão 250, torpedo 250, antissubmarino 300, antiaéreo 200 |
| 3 | Radar, Míssil guiado | 300 homens, 7000 t | Componentes eletrônicos 25700, Navios de guerra 154200 | Alimentos industrializados 1,5, Combustíveis 70, Munição 25,7 | 20 semanas | canhão 100, torpedo 50, antissubmarino 300, antiaéreo 350, antinavio 200 |

## Submarinos (`submarine`)

Mar · submerso

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 40 homens, 900 t | Navios de guerra 19275 | Alimentos industrializados 0,2, Combustíveis 9, Munição 3,855 | 30 semanas | canhão 50, torpedo 900 |
| 2 | Submarino nuclear | 110 homens, 5000 t | Componentes eletrônicos 12850, Navios de guerra 192750 | Alimentos industrializados 0,6, Combustível nuclear 0,2, Munição 6,425 | 30 semanas | torpedo 700, antissubmarino 300 |
| 3 | Submarino nuclear, Míssil de cruzeiro | 110 homens, 5000 t | Componentes eletrônicos 38550, Navios de guerra 231300 | Alimentos industrializados 0,6, Combustível nuclear 0,2, Munição 6,425 | 30 semanas | torpedo 500, antissubmarino 300, antinavio 200 |

## Escoltas (`escort`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 100 homens, 1000 t | Navios de guerra 15420 | Alimentos industrializados 0,5, Combustíveis 10, Munição 3,855 | 16 semanas | canhão 300, antissubmarino 500, antiaéreo 100 |
| 2 | Radar | 100 homens, 1500 t | Equipamento elétrico 1927,5, Navios de guerra 25700 | Alimentos industrializados 0,5, Combustíveis 10, Munição 3,855 | 16 semanas | canhão 200, antissubmarino 600, antiaéreo 200 |
| 3 | Radar, Míssil guiado | 200 homens, 3500 t | Componentes eletrônicos 12850, Navios de guerra 77100 | Alimentos industrializados 1, Combustíveis 35, Munição 12,85 | 16 semanas | canhão 100, antissubmarino 550, antiaéreo 300 |

## Defesa costeira (`coastal`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 150 homens, 2000 t | Navios de guerra 25700 | Alimentos industrializados 0,8, Combustíveis 20, Munição 6,425 | 16 semanas | canhão 650, torpedo 250 |

## Guerra de minas (`mine_warfare`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 60 homens, 600 t | Navios de guerra 6425 | Alimentos industrializados 0,3, Combustíveis 6, Explosivos 1 | 16 semanas | antissubmarino 100, minas 800 |

## Transportes (`transport`)

Mar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | — | 80 homens, 8000 t, 6000 t de carga | Navios 25700 | Alimentos industrializados 0,4, Combustíveis 40 | 8 semanas | — |

## Caças (`fighter`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Aviação | 10 homens | Aviões militares 192,75 | Combustíveis 0,5, Munição 1,285 | 52 semanas | ar-ar 900, ataque ao solo 100 |
| 2 | Aviação, Avião todo de metal | 10 homens | Aviões militares 514 | Combustíveis 1, Munição 2,57 | 52 semanas | ar-ar 900, ataque ao solo 100 |
| 3 | Aviação, Avião todo de metal, Motor a jato | 10 homens | Aviões militares 2570 | Combustíveis 4, Munição 3,855 | 52 semanas | ar-ar 900, ataque ao solo 100 |
| 4 | Aviação, Avião todo de metal, Motor a jato, Míssil guiado | 10 homens | Componentes eletrônicos 1285, Aviões militares 7710 | Combustíveis 4, Munição 3,855 | 52 semanas | ar-ar 900, ataque ao solo 100 |
| 5 | Aviação, Avião todo de metal, Motor a jato, Míssil guiado, Avião furtivo | 10 homens | Componentes eletrônicos 6425, Aviões militares 25700 | Combustíveis 4, Munição 3,855 | 52 semanas | ar-ar 900, ataque ao solo 100 |

## Ataque e bombardeio tático (`attack`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Aviação | 12 homens | Aviões militares 231,3 | Combustíveis 1, Munição 3,855 | 52 semanas | ar-ar 150, ataque ao solo 700, antinavio 150 |
| 2 | Aviação, Avião todo de metal | 12 homens | Aviões militares 642,5 | Combustíveis 1,5, Munição 6,425 | 52 semanas | ar-ar 150, ataque ao solo 700, antinavio 150 |
| 3 | Aviação, Avião todo de metal, Motor a jato | 12 homens | Aviões militares 3212,5 | Combustíveis 4, Munição 12,85 | 52 semanas | ar-ar 150, ataque ao solo 700, antinavio 150 |
| 4 | Aviação, Avião todo de metal, Motor a jato, Munição guiada a laser | 12 homens | Componentes eletrônicos 1927,5, Aviões militares 8995 | Combustíveis 4, Munição 12,85 | 52 semanas | ar-ar 150, ataque ao solo 700, antinavio 150 |
| 5 | Aviação, Avião todo de metal, Motor a jato, Munição guiada a laser, Munição guiada por satélite | 12 homens | Componentes eletrônicos 3855, Aviões militares 15420 | Combustíveis 4, Munição 12,85 | 52 semanas | ar-ar 150, ataque ao solo 700, antinavio 150 |

## Bombardeiros estratégicos (`strategic_bomber`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Bombardeiro estratégico | 30 homens | Aviões militares 3212,5 | Combustíveis 8, Munição 25,7 | 60 semanas | ar-ar 50, estratégico 850, antinavio 100 |
| 2 | Bombardeiro estratégico, Motor a jato | 30 homens | Aviões militares 19275 | Combustíveis 25, Munição 38,55 | 60 semanas | ar-ar 50, estratégico 850, antinavio 100 |
| 3 | Bombardeiro estratégico, Motor a jato, Míssil de cruzeiro | 30 homens | Componentes eletrônicos 6425, Aviões militares 38550 | Combustíveis 25, Munição 38,55 | 60 semanas | ar-ar 50, estratégico 850, antinavio 100 |
| 4 | Bombardeiro estratégico, Motor a jato, Míssil de cruzeiro, Avião furtivo | 30 homens | Componentes eletrônicos 25700, Aviões militares 128500 | Combustíveis 25, Munição 38,55 | 60 semanas | ar-ar 50, estratégico 850, antinavio 100 |

## Reconhecimento e patrulha (`patrol`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Aviação | 12 homens | Aviões militares 257 | Combustíveis 1 | 40 semanas | antissubmarino 50, ar-ar 100, antinavio 250, reconhecimento 600 |
| 2 | Aviação, Radar | 12 homens | Equipamento elétrico 257, Aviões militares 1028 | Combustíveis 3 | 40 semanas | antissubmarino 250, antinavio 250, reconhecimento 500 |
| 3 | Aviação, Radar, Motor a jato | 12 homens | Equipamento elétrico 1285, Aviões militares 6425 | Combustíveis 6 | 40 semanas | antissubmarino 250, antinavio 250, reconhecimento 500 |

## Transporte aéreo (`air_transport`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Aviação | 8 homens, 1 t de carga | Aviões militares 257 | Combustíveis 1 | 30 semanas | — |
| 2 | Aviação, Avião todo de metal | 8 homens, 3 t de carga | Aviões militares 771 | Combustíveis 2 | 30 semanas | — |
| 3 | Aviação, Avião todo de metal, Motor a jato | 8 homens, 40 t de carga | Aviões militares 10280 | Combustíveis 15 | 30 semanas | — |

## Aviação naval (`naval_aviation`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Porta-aviões | 10 homens | Aviões militares 231,3 | Combustíveis 1, Munição 3,855 | 60 semanas | ar-ar 350, antinavio 550, reconhecimento 100 |
| 2 | Porta-aviões, Avião todo de metal | 10 homens | Aviões militares 642,5 | Combustíveis 1, Munição 3,855 | 60 semanas | ar-ar 350, antinavio 550, reconhecimento 100 |
| 3 | Porta-aviões, Avião todo de metal, Motor a jato | 10 homens | Aviões militares 3855 | Combustíveis 4, Munição 12,85 | 60 semanas | ar-ar 350, antinavio 550, reconhecimento 100 |

## Helicópteros (`helicopter`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Helicóptero | 8 homens, 2 t de carga | Aviões militares 1927,5 | Combustíveis 2, Munição 3,855 | 40 semanas | antissubmarino 200, ataque ao solo 400, reconhecimento 300 |
| 2 | Helicóptero, Munição guiada a laser | 8 homens, 2 t de carga | Componentes eletrônicos 1285, Aviões militares 6425 | Combustíveis 2, Munição 3,855 | 40 semanas | antissubmarino 200, ataque ao solo 600, reconhecimento 200 |

## Drones (`drone`)

Ar

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Drone armado | 4 homens | Componentes eletrônicos 642,5, Aviões militares 3855 | Combustíveis 0,5, Munição 2,57 | 20 semanas | ataque ao solo 400, reconhecimento 600 |
| 2 | Drone armado, Munição vagante | 4 homens | Componentes eletrônicos 257, Aviões militares 257 | Combustíveis 0,05 | 20 semanas | ataque ao solo 900, reconhecimento 100 |

## Mísseis balísticos (`ballistic_missile`)

Estratégico

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Míssil balístico intercontinental | 30 homens | Combustíveis 30, Componentes eletrônicos 12850, Máquinas 38550 | Combustíveis 1, Máquinas 25,7 | 30 semanas | estratégico 1000 |
| 2 | Míssil balístico intercontinental, Planador hipersônico | 30 homens | Combustíveis 40, Componentes eletrônicos 64250, Máquinas 102800 | Combustíveis 1, Máquinas 25,7 | 30 semanas | estratégico 1000 |

## Submarinos lança-mísseis (`missile_submarine`)

Estratégico · submerso

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Submarino nuclear, Míssil balístico intercontinental | 140 homens, 9000 t | Componentes eletrônicos 64250, Navios de guerra 514000 | Alimentos industrializados 0,8, Combustível nuclear 0,3 | 40 semanas | estratégico 900 |

## Ogivas nucleares (`nuclear_warhead`)

Estratégico

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Bomba atômica | 5 homens, 21 kt | Núcleos de arma nuclear 1, Conjuntos de ogiva 1 | Equipamento elétrico 1,285 | 10 semanas | estratégico 1000 |
| 2 | Bomba atômica, Bomba de hidrogênio | 5 homens, 1400 kt | Núcleos de arma nuclear 0,73, Conjuntos de ogiva 1 | Equipamento elétrico 1,285 | 10 semanas | estratégico 1000 |

## Satélites (`satellite`)

Estratégico

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Satélite artificial | 50 homens | Componentes eletrônicos 64250, Máquinas 64250 | Componentes eletrônicos 64,25 | 30 semanas | reconhecimento 1000 |
| 2 | Satélite artificial, GPS | 50 homens | Componentes eletrônicos 102800, Máquinas 51400 | Componentes eletrônicos 64,25 | 30 semanas | reconhecimento 1000 |

## Defesa antimíssil (`missile_defense`)

Estratégico

| Geração | Requer | Tamanho | Equipamento | Manutenção | Treino | Papéis |
|---|---|---|---|---|---|---|
| 1 | Defesa antimíssil | 500 homens | Componentes eletrônicos 257000, Artilharia 128500 | Combustíveis 5, Componentes eletrônicos 257 | 40 semanas | antiaéreo 1000 |
