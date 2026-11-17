# Placar histórico: a simulação contra 1925–2025

O placar responde, depois de cada rodada de 100 anos, se o mundo simulado se comporta como o mundo
real se comportou de 1925 a 2025, país por país. Ele cruza a série anual de cada país da
simulação com séries históricas públicas e mostra onde o modelo acerta, onde erra e em que
período o erro aparece.

## Como rodar

    tools/history/run.sh                  # compila, roda 1925 → 2026-01-01 e pontua
    tools/history/run.sh --score-only     # pontua de novo target/history/history.json
    BF_LOCK=/tmp/bf-bench.lock tools/history/run.sh   # espera outro bench longo terminar

O resultado fica em `target/history/`:

| Arquivo | O que é |
|---|---|
| `scorecard.md` | O placar para ler: tabelas por década, por região, por período e os maiores desvios |
| `scorecard.json` | Os mesmos números, para comparar rodadas |
| `world.png` | Mundo: população, PIB, PIB por habitante, fora da agropecuária, alfabetização |
| `regions_gdp_per_capita.png`, `regions_population.png` | As 11 regiões |
| `countries_gdp_per_capita.png`, `countries_population.png` | As 20 maiores unidades de 1925 |
| `production.png` | Produção mundial de carvão, petróleo, gás, eletricidade, aço e cereais |
| `history.json` | A série anual da simulação (entrada do placar) |

A rodada é de 101 anos para fechar o ano de 2025 inteiro. O placar sozinho roda em segundos; os
gráficos pedem o matplotlib, e o `run.sh` cria um ambiente virtual em `target/history/.venv` se ele
não estiver instalado. Os dados de referência já estão no repositório; para baixá-los de novo,
`python3 tools/history/fetch.py`.

### O que o bench grava

`bf-bench --game --history-out PATH` grava, além do relatório de sempre, a série anual de cada
país, com metrópole e colônias separadas, e de cada um dos 870 estados. O módulo
(`crates/bf-bench/src/history.rs`) só lê o mundo depois de cada tick; o hash final é o mesmo com e
sem ele.

| Série | Como sai da simulação |
|---|---|
| População | Média de 1º de janeiro do ano e do seguinte (≈ meio do ano, como nas séries históricas) |
| PIB | `gdp_last_year` do mercado, a preços-base de 1925 (dólares de 2011). A divisão entre metrópole, colônias e estados segue o valor adicionado que o módulo estima semana a semana (produção vendida menos insumos comprados, a preço-base, mais a comida de subsistência) |
| Fora da agropecuária | `Region::urban_share()`: parcela da população fora das profissões rurais (camponês, assalariado rural, proprietário de terra). É a única "urbanização" que o modelo tem (ver "Urbanização" abaixo) |
| Alfabetização | `literacy` dos estados, ponderada pela população |
| Emprego por setor | Vagas preenchidas por grupo de edifício (agropecuária; extração, energia e indústrias; o resto em serviços), mais o funcionalismo, o clero e os soldados em serviços e os camponeses sem emprego, que vivem do próprio lote, na agropecuária |
| Produção | Soma semanal da produção das plantas (níveis × ocupação × produtividade × utilização × insumos). Cereais somam trigo, milho e arroz, com a subsistência dos camponeses |
| Impostos, gasto, dívida, inflação | Do país inteiro (o modelo não separa as contas das colônias): receita e gasto do ano sobre o PIB, dívida no fim do ano sobre o PIB, variação do nível de preços de janeiro a janeiro |

## Como ler

### A unidade de comparação

A simulação mantém as fronteiras de 1925 durante os 100 anos (ainda não há descolonização nem
guerra). Por isso a comparação é feita por **unidade de 1925**: cada tag, com a metrópole e as
colônias separadas (`GBR/metropole`, `GBR/colonies`). O mapa `tools/history/mapping.csv` soma
cada país moderno à unidade que o governava em 1925:

- SOV = as 15 repúblicas soviéticas (menos os Bálticos, que têm tag própria, e a Moldávia, que era
  romena); RAJ = Índia, Paquistão, Bangladesh e Birmânia; as colônias vão para a metrópole que as
  detinha (a Nigéria em `GBR/colonies`, a Indonésia em `NLD/colonies`, a Coreia e Taiwan em
  `JAP/colonies`).
- Um país moderno dividido entre unidades leva uma linha a mais por pedaço, com a população de
  1925 do pedaço tirada das tabelas de componentes de `data/scenario/1925/SOURCES.md` (o Tibete e
  as concessões dentro da China; Hejaz na Arábia Saudita; Danzig na Polônia; o Sarre na Alemanha;
  Terra Nova no Canadá; Tuva e Karafuto na Rússia). A participação do pedaço fica fixa nos 100
  anos.
- Os casos que o mapa não divide estão na seção "Limites".
- `crates/bf-bench/tests/history_mapping.rs` garante que toda tag (e todo império colonial) recebe
  algum país e que todo país das séries de referência vai para alguma unidade.

O PIB de uma unidade só tem referência quando os países com dado cobrem pelo menos 80 % da
população dela; abaixo disso a célula fica `–`. As lacunas do Maddison (anos de guerra,
colônias antes de 1950) são preenchidas por interpolação geométrica entre observações a até
25 anos de distância. A antiga URSS, a Tchecoslováquia, a Iugoslávia e o Sudão de antes de 2011
entram pelo agregado do Maddison: o que falta aos membros sem série é o agregado menos os membros
que têm série.

### As métricas

- **Razão simulado/real** de população, PIB e PIB por habitante, por unidade e década. 1,00 é
  acerto. Em 1925 a razão já pode fugir de 1 porque as fronteiras de 1925 e as modernas não
  batem (a Polônia de 1925 tinha parte da Ucrânia e da Bielorrússia; a Alemanha, a Silésia e a
  Prússia Oriental): o que interessa é como a razão anda depois.
- **Erro logarítmico médio**: média de |ln(simulado/real)| sobre as unidades com referência.
  0,10 é cerca de 10 % de erro típico; 0,69 é um fator 2.
- **Correlação de ranking** (Spearman) do PIB por habitante entre as unidades: se os ricos e os
  pobres estão na ordem certa, mesmo com nível errado.
- **Participação das regiões no PIB mundial.** As regiões são de países modernos; do lado da
  simulação, cada estado vai para o país moderno que tem a maior parte da sua área (pelas
  províncias do Natural Earth em `assets/map/provinces.ron`).
- **Crescimento por período histórico**, em % ao ano, por região: 1925–29, 1929–33 (Depressão),
  1933–39, 1939–45 (guerra), 1945–73 (anos dourados), 1973–91 (choques do petróleo ao colapso
  soviético), 1991–2008, 2008–25. É onde aparece o que o modelo ainda não tem.
- **Maiores desvios, por causa**: candidatos (mundo, as 20 maiores unidades, região × período)
  medidos por |ln(simulado/real)| × √(peso no mundo) e agrupados pela falta do modelo que os explica
  (`CAUSES` em `tools/history/scorecard.py`: começo pequeno, sem ciclo nem guerra, sem efeito de
  regime, demografia, emprego fora da agropecuária, convergência mecânica). A causa pesa a raiz da
  soma dos quadrados dos seus desvios, e a lista mostra uma linha por causa; a tabela "um a um"
  logo abaixo mantém a ordem bruta. O PIB por habitante das unidades entra como nível no fim
  (por habitante, a diferença de fronteira de 1925 pesa pouco); a população, como crescimento
  desde 1925, para que essa diferença de partida não conte.

Como o PIB do Maddison termina em 2022, as linhas de PIB de 2025 e o último período (2008–25)
comparam 2022 dos dois lados.

### O que não é a mesma coisa dos dois lados

- **Urbanização: o modelo não tem.** `Region::urban_share()`, que o placar grava e o HUD mostra
  (`bf-client/src/live.rs`), é a parcela da população fora das três profissões rurais. A
  urbanização da HYDE de `countries.ron` só é usada no início: divide a demanda de 1925 entre
  cidade e campo e posiciona as plantas (`econ/setup.rs`). Depois disso, migração, moradia
  urbana e natalidade leem a profissão. Por isso o placar compara "fora da agropecuária" com o
  **emprego não agrícola** da OIT (100 − `SL.AGR.EMPL.ZS`, desde 1991) e mostra a urbanização da
  HYDE só como contexto, sem par do lado simulado. Em 1925 as duas medidas dão 37 % (fora da
  agropecuária) e 21 % (urbana): mineiros, artesãos de aldeia, padres e soldados do interior estão
  fora da agropecuária sem morar em cidade. Falta ao modelo uma população urbana própria por
  estado (quem mora em cidade, além de em que trabalha), movida pela migração e pela moradia, com
  empregos não agrícolas no campo e, ao contrário, agricultores que moram na cidade.
- **Energia.** As séries de carvão, petróleo e gás estão em TWh de energia primária; a simulação
  produz toneladas e m³, convertidas por poder calorífico médio (carvão 7,0 MWh/t, petróleo
  11,63 MWh/t, gás 10 MWh por mil m³). Erros de 20 a 30 % no carvão podem ser só a mistura de
  hulha e linhito.
- **Cereais.** A FAO conta o arroz em casca; a simulação, beneficiado (cerca de dois terços).
- **Contas públicas.** O WDI dá receita sem doações e despesa do governo central, desde 1972 e com
  falhas; a simulação dá o Estado inteiro.

## Primeiro placar: `main` em 662bb7a

Rodada de 1925 a 2026-01-01, semente 1925, 4 threads (729 s de CPU, 266 s de parede), hash final
`d165c5e4…`. O hash de 10 anos com 1 thread é o mesmo com e sem `--history-out` e igual ao da
`main` (`831ab118…`).

| Ano | População | PIB (tri US$ 2011) | PIB por hab. | Fora da agropecuária¹ | Urbana (HYDE) | Erro log. médio pop / PIB pc | Ranking PIB pc |
|---|---|---|---|---|---|---|---|
| 1925 | 1,96 / 1,97 bi | 3,0 / 5,6 | 1 525 / 2 859 | 37 % / – | 21 % | 0,04 / 0,64 | 0,94 |
| 1935 | 2,17 / 2,17 bi | 6,3 / 6,4 | 2 888 / 2 969 | 43 % / – | 24 % | 0,07 / 0,37 | 0,81 |
| 1945 | 2,44 / 2,38 bi | 9,4 / 8,0 | 3 857 / 3 378 | 47 % / – | 27 % | 0,12 / 0,52 | 0,64 |
| 1955 | 2,81 / 2,74 bi | 13,0 / 10,9 | 4 620 / 3 986 | 49 % / – | 31 % | 0,20 / 0,57 | 0,63 |
| 1965 | 3,30 / 3,33 bi | 20,2 / 17,2 | 6 128 / 5 168 | 51 % / – | 35 % | 0,26 / 0,67 | 0,59 |
| 1975 | 3,83 / 4,07 bi | 32,6 / 26,8 | 8 503 / 6 577 | 52 % / – | 37 % | 0,32 / 0,76 | 0,54 |
| 1985 | 4,38 / 4,87 bi | 48,4 / 37,1 | 11 052 / 7 622 | 53 % / – | 41 % | 0,39 / 0,79 | 0,57 |
| 1995 | 4,96 / 5,76 bi | 63,6 / 50,0 | 12 825 / 8 686 | 55 / 59 % | 45 % | 0,46 / 0,89 | 0,58 |
| 2005 | 5,56 / 6,58 bi | 77,4 / 74,7 | 13 928 / 11 348 | 56 / 63 % | 49 % | 0,52 / 0,85 | 0,53 |
| 2015 | 6,17 / 7,47 bi | 91,8 / 109,3 | 14 881 / 14 637 | 57 / 70 % | 53 % | 0,59 / 0,80 | 0,48 |
| 2025² | 6,78 / 8,23 bi | 102,7 / 133,4 | 15 578 / 16 633 | 59 / 73 % | 57 % | 0,64 / 0,84 | 0,48 |

Simulado / real. ¹ Simulação: população fora das profissões rurais; referência: emprego não
agrícola da OIT, só desde 1991. A coluna HYDE é contexto: o modelo não tem urbanização própria.
² PIB de 2022 dos dois lados.

Leitura curta: a média mundial acerta a ordem de grandeza e termina perto (PIB por habitante 6 %
abaixo em 2022), mas por um caminho errado. A economia começa com pouco mais da metade do PIB, se
recupera em dez anos, cresce demais entre 1930 e 1990 e de menos depois. O mundo fica mais
igual do que foi: a correlação de ranking cai de 0,94 para 0,48, e o erro típico do PIB por
habitante passa de um fator 2 (0,84). A população anda em linha reta e termina 18 % abaixo. E o
mundo simulado sai da agropecuária devagar demais depois de 1990 (59 % contra 73 % em 2025).

### Os maiores desvios, por causa

Seis causas explicam os 15 maiores desvios; a ordem é a do placar (peso da causa). "Confirmado"
quer dizer que o mecanismo foi visto no código, não que o tamanho do efeito foi medido.

1. **Convergência mecânica** (peso 0,73). China em 2022: 8 500 contra 19 200 $ por habitante,
   depois de estar 2,6 vezes à frente em 1975; Leste da Ásia +0,4 contra +4,6 % ao ano em 2008–25
   e +0,9 contra +3,7 % em 1991–2008; México em 2022: 3 100 contra 16 200 $; Sul da Ásia +3,2 contra
   +0,8 % ao ano em 1945–73. Confirmado: fora das tecnologias, a produtividade de cada estado
   converge para a do estado líder na proporção da distância e da alfabetização
   (`econ/yearly.rs`, `catch_up`), sem regime, guerra, reforma ou abertura. Quem é pobre e
   alfabetizado cresce cedo demais; quando a distância fecha, para. **Hipóteses**: falta um motor
   para a decolagem chinesa pós-1978 (abertura e investimento estrangeiro), e no México a
   alfabetização sobe devagar demais no campo para puxar a convergência.
2. **Começo pequeno** (0,68). O PIB de 1925 é 3,0 contra 5,6 tri, e de 1925 a 1929 o mundo cresce
   8,4 % ao ano por habitante, contra 2,4 % (+11,4 % no Ocidente de colonização europeia). O
   mercado nasce com `gdp_last_year` igual ao PIB-alvo da calibração (`econ/setup.rs`), mas o PIB
   contado no primeiro ano (produção vendida a preço-base menos insumos, mais a subsistência,
   `econ/week.rs::settle_region`) dá 57 % dele, em todas as grandes unidades (razão PIB pc de
   0,46 a 0,68 em 1925). Confirmado que as duas contas são diferentes. **Hipótese**: a diferença
   está em produção não vendida, plantas que reduzem a utilização na primeira semana e serviços
   que a calibração conta e o mercado não fecha. O antigo "tombo de 1926–27" era isto: o bench
   lia `gdp_last_year` antes da virada de 1º de janeiro, que ainda guardava o alvo calibrado. O
   placar lê `gdp_running`; o relatório decenal do `--game` continua com o rótulo um ano atrasado.
3. **Sem ciclo nem guerra** (0,59). Na Depressão (1929–33), o Ocidente de colonização europeia
   cresce +5,5 % ao ano contra −9,1 %, e a Europa Ocidental +6,9 % contra −2,2 %. Na guerra
   (1939–45), a Europa Ocidental cresce +2,4 % contra −3,3 %, e os EUA e afins +1,8 % contra +6,5 %
   (o boom da produção de guerra). Na reconstrução (1945–73), a Europa Ocidental cresce +2,3 %
   contra +4,0 %, e os EUA e afins +3,0 % contra +1,8 %. Confirmado: não há padrão-ouro, crise
   bancária, colapso do comércio nem guerra (fase 4, `architecture.pt-BR.md`). Sem destruição,
   não há o que reconstruir, e a Europa chega a 1945 já rica.
4. **Demografia** (0,46). A população das colônias britânicas cresce ×4,4 contra ×10, e a das
   francesas ×4,5 contra ×8,1; a dos EUA ×1,6 contra ×3,1, e a do Brasil ×3,2 contra ×7,0. O mundo
   tem 6,78 contra 8,23 bilhões em 2025, numa curva quase linear (≈ 57 milhões por ano) sem a
   explosão de 1950–90. Confirmado: a natalidade em `econ/monthly.rs::vital_rates` cai com
   `urban_share` (fora da agropecuária, 37 % do mundo em 1925 contra 21 % urbanos) e com a
   alfabetização ao quadrado; a migração é só interna (`econ/migration.rs`; o contrato de política
   prevê efeitos de imigração e emigração, ainda sem sistema), o que pesa nos EUA, no Brasil e na
   Argentina; natalidade e
   mortalidade têm pisos de 7 e 6 por mil. **Hipótese**: a transição demográfica chega décadas
   cedo onde foi tardia (África, Sul da Ásia), e isso responde pela maior parte do 1,45 bilhão
   que falta.
5. **Sem efeito de regime** (0,26). A URSS cresce +3,7 % ao ano em 1973–91 contra +0,4 %, e o
   Leste Europeu +3,2 % contra −0,1 %; depois, em 1991–2008, o Leste Europeu cresce +1,3 % contra
   +4,6 % (sem colapso, não há recuperação). Confirmado pela ausência: o regime político não
   pesa na economia, e não há planejamento central nem transição.
6. **Emprego fora da agropecuária** (0,22). O mundo tem 59 % da população fora das profissões
   rurais em 2025, contra 73 % do emprego fora da agropecuária. Em 2025, a agropecuária ocupa 71 %
   contra 22 % na China e 75 % contra 12 % no México. Confirmado: os camponeses sem emprego
   ficam na agropecuária (subsistência), e a migração campo-cidade (`econ/migration.rs`) só move
   quem acha emprego e moradia urbana. **Hipótese**: a indústria e os serviços abrem vagas
   devagar demais nos países pobres; a indústria fica baixa em todo lugar (5 a 21 % contra 19 a
   35 %).

### Outros sinais do primeiro placar

- **Energia**: o carvão simulado passa da referência a partir de 1935 (o dobro em 1945) e o gás
  fica em um terço da referência. O petróleo começa baixo (15 % em 1925) e chega em 1985.
  A conversão por poder calorífico médio pode responder por 20 a 30 % do carvão, não pelo
  dobro.
- **Contas públicas e inflação**: impostos e gasto em 22–26 % do PIB, dívida em 41–59 % e inflação de
  2 % em quase todos os países em 2015–24. A referência varia muito mais (impostos de 12 % na Índia a
  44 % na França; dívida de 48 % no México a 133 % no Reino Unido).

## Depois da correção de 1925 (`fix/economy-1925`)

Mesma rodada (1925 → 2026-01-01, semente 1925, 2 threads), hash final `2206408a…`. O que mudou
está em `economy.pt-BR.md` ("1925: o tamanho e o fluxo do dinheiro" e "Investimento e
crescimento"). Antes = primeiro placar acima; depois = esta branch; real = referência.

### Mundo por década

| Ano | PIB (tri): antes / depois / real | PIB por hab.: antes / depois / real | Erro log. PIB pc: antes / depois | Ranking PIB pc: antes / depois |
|---|---|---|---|---|
| 1925 | 3,0 / 5,0 / 5,6 | 1 525 / 2 570 / 2 859 | 0,64 / 0,12 | 0,94 / 0,97 |
| 1935 | 6,3 / 7,3 / 6,4 | 2 888 / 3 373 / 2 969 | 0,37 / 0,27 | 0,81 / 0,88 |
| 1945 | 9,4 / 11,6 / 8,0 | 3 857 / 4 745 / 3 378 | 0,52 / 0,53 | 0,64 / 0,74 |
| 1955 | 13,0 / 17,0 / 10,9 | 4 620 / 6 065 / 3 986 | 0,57 / 0,62 | 0,63 / 0,70 |
| 1965 | 20,2 / 25,3 / 17,2 | 6 128 / 7 722 / 5 168 | 0,67 / 0,67 | 0,59 / 0,67 |
| 1975 | 32,6 / 41,3 / 26,8 | 8 503 / 10 874 / 6 577 | 0,76 / 0,76 | 0,54 / 0,60 |
| 1985 | 48,4 / 63,0 / 37,1 | 11 052 / 14 597 / 7 622 | 0,79 / 0,92 | 0,57 / 0,60 |
| 1995 | 63,6 / 85,5 / 50,0 | 12 825 / 17 589 / 8 686 | 0,89 / 1,11 | 0,58 / 0,60 |
| 2005 | 77,4 / 103,1 / 74,7 | 13 928 / 19 015 / 11 348 | 0,85 / 1,06 | 0,53 / 0,55 |
| 2015 | 91,8 / 119,8 / 109,3 | 14 881 / 19 971 / 14 637 | 0,80 / 1,01 | 0,48 / 0,49 |
| 2025 (PIB 2022) | 102,7 / 132,4 / 133,4 | 15 578 / 20 650 / 16 633 | 0,84 / 1,02 | 0,48 / 0,46 |

Os 20 maiores começam entre 0,85 e 0,98 do PIB por habitante real (eram 0,46 a 0,68). O PIB
mundial de 2025 bate; o PIB por habitante fica 24 % acima porque a população fica em 6,4 contra
8,2 bilhões (calibração da demografia, outro trabalho). O erro típico por país piora depois de
1985: os ricos crescem demais e os pobres de menos (abaixo).

### Crescimento do PIB por habitante no mundo, % ao ano

| | 1925–29 | 1929–33 | 1933–39 | 1939–45 | 1945–73 | 1973–91 | 1991–2008 | 2008–25 |
|---|---|---|---|---|---|---|---|---|
| Antes | +8,4 | +6,2 | +3,7 | +2,5 | +2,6 | +2,5 | +0,9 | +0,6 |
| Depois | +2,3 | +2,6 | +3,6 | +3,5 | +2,6 | +3,0 | +0,8 | +0,5 |
| Real | +2,4 | −3,6 | +3,6 | +0,1 | +2,4 | +1,2 | +2,6 | +2,0 |

O salto de 1925–35 era a recuperação do começo pequeno e, por baixo dela, um crescimento de
8 a 10 % ao ano que o modelo tinha sempre que as plantas rodavam cheias (capital barato nos
países ricos, troca de método de graça, capital que não se gasta). A crise de 1929, a guerra e a
estagnação depois de 1991 continuam fora do modelo.

### Emprego por setor em 2025 (agro / indústria / serviços, %)

| Unidade | Antes | Depois | Real |
|---|---|---|---|
| CHN | 71 / 5 / 25 | 82 / 2 / 17 | 22 / 32 / 46 |
| RAJ | 65 / 6 / 29 | 76 / 2 / 21 | 42 / 25 / 34 |
| SOV | 52 / 7 / 42 | 40 / 10 / 50 | 13 / 25 / 62 |
| USA | 13 / 21 / 67 | 9 / 22 / 69 | 2 / 19 / 79 |
| GER | 14 / 19 / 67 | 16 / 17 / 68 | 1 / 26 / 73 |
| JAP | 19 / 15 / 66 | 23 / 13 / 64 | 3 / 23 / 74 |
| GBR | 4 / 18 / 77 | 5 / 17 / 78 | 1 / 16 / 83 |
| FRA | 6 / 19 / 76 | 5 / 16 / 79 | 2 / 19 / 78 |
| ITA | 27 / 15 / 58 | 21 / 14 / 65 | 3 / 27 / 70 |
| BRA | 42 / 9 / 49 | 45 / 8 / 47 | 8 / 20 / 72 |
| POL | 29 / 14 / 57 | 26 / 12 / 62 | 6 / 30 / 64 |
| ESP | 46 / 13 / 41 | 19 / 15 / 66 | 3 / 20 / 77 |
| ROM | 54 / 7 / 39 | 58 / 5 / 37 | 17 / 30 / 54 |
| MEX | 75 / 6 / 19 | 38 / 9 / 52 | 12 / 25 / 64 |
| CZE | 21 / 17 / 62 | 25 / 14 / 61 | 3 / 35 / 62 |
| EGY | 53 / 8 / 39 | 58 / 6 / 37 | 18 / 29 / 53 |

A agropecuária não esvazia porque conta os camponeses sem emprego, e é para lá que voltam os
trabalhadores que perdem o posto. Nos pobres a indústria encolhe ao longo do século (China: 8 %
em 1925, 2 % em 2025): a produtividade industrial sobe com a pesquisa e a convergência, a
demanda dos pobres é quase toda comida, e a planta que se gasta só é refeita onde o tipo roda
acima de 90 %, o que não acontece quando a importação (o mercado mundial pega 85 % da sobra de
cada país toda semana) cobre a demanda antes. A migração para a cidade é lenta demais para
compensar: 3 por 10 mil camponeses ociosos por mês para as cidades do próprio estado e 1 por 10
mil para as de fora (`econ/migration.rs`), contra 1 a 2 % da população rural por ano na história
do pós-guerra; não foi mexida aqui.

### Produção mundial (simulado antes / depois / real)

| Ano | Carvão (TWh) | Petróleo (TWh) | Gás (TWh) | Eletricidade (TWh) | Aço (Mt) |
|---|---|---|---|---|---|
| 1925 | 6 440 / 8 447 / 8 436 | 246 / 448 / 1 644 | 219 / 219 / 402 | – | 79 / 91 / 90,5¹ |
| 1955 | 17 275 / 10 508 / 10 237 | 5 448 / 8 239 / 8 128 | 1 270 / 1 788 / 3 202 | – | – |
| 1975 | 18 258 / 10 062 / 14 395 | 20 584 / 26 086 / 26 063 | 4 045 / 5 464 / 9 059 | – | 1 327 / 918 / 442 |
| 1995 | 36 203 / 17 889 / 25 968 | 43 430 / 61 893 / 38 220 | 7 880 / 10 829 / 20 908 | 11 035 / 14 244 / 12 986 | 3 028 / 1 750 / 751 |
| 2025 | 58 858 / 23 310 / 49 401 | 67 409 / 91 330 / 53 992 | 12 932 / 15 874 / 41 274 | 17 271 / 21 648 / 29 606 | 3 980 / 2 761 / 1 593 |

¹ `industry.ron`. As colunas de aço somam o mundo simulado inteiro; o real de 1975 em diante é a
soma das unidades com série no placar do orquestrador. O carvão agora acerta até 1955 e fica
baixo depois (o petróleo toma o lugar dele mais cedo); o gás fica em um terço a dois quintos o
século todo.

## Convergência e campo (`fix/growth-structure`)

Mesma rodada de 100 anos (2 threads), hash `74771363…`. Antes = `main` com o #55.

| | Antes | Depois | Real |
|---|---|---|---|
| PIB por hab. mundial, 1985 | 14 597 | 11 475 | 7 622 |
| PIB mundial 2025 (PIB 2022), tri | 132,4 | 101,1 | 133,4 |
| Ranking do PIB pc, 1985 / 2022 | 0,60 / 0,46 | 0,65 / 0,45 | — |
| Erro log. PIB pc, 1985 / 2022 | 0,92 / 1,02 | 0,91 / 1,14 | — |
| Crescimento pc 1945–73 / 1973–91 / 1991–2008 / 2008–25 (% a.a.) | +2,6 / +3,0 / +0,8 / +0,5 | +2,2 / +2,8 / +0,6 / +0,5 | +2,4 / +1,2 / +2,6 / +2,0 |
| Emprego agro / indústria 2025: CHN | 82 / 2 | 85 / 2 | 22 / 32 |
| RAJ | 76 / 2 | 76 / 3 | 42 / 25 |
| MEX | 38 / 9 | 38 / 8 | 12 / 25 |
| BRA | 45 / 8 | 39 / 10 | 8 / 20 |
| ESP | 19 / 15 | 26 / 12 | 3 / 20 |

O que mudou (`economy.pt-BR.md`, "Convergência"): a convergência da produtividade de base é
condicional, e o camponês sem emprego só planta na terra que as fazendas deixam livre e só vende
mais à medida que a técnica agrícola melhora.

### Por que todos convergem cedo e depois param

Medido com `bf-bench --diag-1925` (`BF_DIAG_TFP=1` e `BF_DIAG_COUNTRY=<tag>`):

- **A produtividade por trabalhador converge mecanicamente e cedo.** Pela tecnologia, a imitação
  copia em poucos anos o que é industrial lá fora: a China tem em 1985 a produtividade industrial
  dos EUA (3,47 contra 3,28) e em 2025 mais. Pela base, o `catch_up` fechava a distância até o
  estado líder sem limite; China 1,35 → 4,98, México 2,78 → 5,40, EUA parados em 5,89 (o líder
  não cresce por esse canal). Agora a base só fecha uma parte da distância de 1925, maior com a
  escola (`CLOSE_WITH_SCHOOLING`).
- **A árvore de pesquisa concentra os ganhos entre 1930 e 1979.** Ganho de produtividade por
  década, em log, da indústria pesada: +20, +11, +25, +28, +42 % de 1930 a 1979; +6 % em 1980,
  +8 % em 1990, 0 em 2000. Serviços ficam abaixo de 10 % por década o século todo. Quando os
  seguidores alcançam a fronteira, o mundo cresce o que a fronteira cresce, e depois de 1980 ela
  quase não cresce.
- **O que fez o mundo crescer depois de 1991 não acontece:** a saída do campo na Ásia. O PIB por
  habitante passa a depender de quanta gente está em emprego de mercado, e na China ela diminui:
  fazendas comerciais a 37–54 % de uso e 21 → 7 milhões de empregos de 1930 a 1980; camponeses
  ociosos de 121 para 270 milhões; indústria a 46–80 % de uso, abaixo dos 90 % que destravam o
  investimento. É uma armadilha de demanda: quem perde o emprego vive da própria terra, sem renda
  em dinheiro, e não compra o que a indústria faria; sem planta cheia não há investimento, e sem
  investimento não há emprego. O ranking cai por isso (China 0,16 do real em 2025), e pelo outro
  lado colônias pequenas com minas sobem demais (Congo Belga 11×, Haiti 6×, Afeganistão 7×), com a
  produtividade alta que o setup de 1925 lhes dá para lotar as minas com poucos trabalhadores.

### Aço, gás, eletricidade e petróleo

- **Aço (2 761 contra 1 593 Mt em 2025):** o fundo de investimento recebe cerca de 21 % do PIB e a
  construção fica em 13 % do PIB; a obra e as máquinas puxam o aço.
- **Gás (um terço):** a demanda é quase só doméstica (aquecimento) e das usinas a gás depois de
  1950; não há gás na indústria nem na química. Não medido além disso.
- **Eletricidade de 1925 (0,48):** as plantas de 1925 nascem no primeiro método de cada edifício,
  sem eletricidade.
- **Petróleo de 1925 (0,3):** navios, ferrovias e indústria só queimam carvão e não há querosene;
  o petróleo é a gasolina dos carros e pouco mais.

Nenhum dos quatro foi corrigido aqui.

## Saída do campo (`fix/leave-the-land`)

Rodada de 100 anos (2 threads), hash `96f15a6d…`, contra a `main` em 16535f9 (com política,
manutenção corrigida e escala única), hash `0419184…`. O que mudou: `economy.pt-BR.md`, "Saída do
campo" e "1925: eletricidade e petróleo".

| | `main` 16535f9 | Esta branch | Real |
|---|---|---|---|
| Fora da agropecuária, mundo 2025 | 48 % | 70 % | 73 % |
| Agro 2025: Brasil / México / China / Índia | 41 / 38 / 91 / 79 | 12 / 10 / 53 / 37 | 8 / 12 / 22 / 42 |
| Informal 2025: Brasil / México / Índia / Alemanha / Itália | – | 36 / 48 / 81 / 31 / 27 | 36 / 57 / 87 / 4 / 8 |
| Ranking do PIB pc, 1985 / 2022 | 0,63 / 0,47 | 0,63 / 0,47 | — |
| Crescimento pc 1973–91 / 1991–2008 / 2008–25 (% a.a.) | +2,6 / +0,7 / +0,5 | +2,8 / +1,0 / +1,0 | +1,2 / +2,6 / +2,0 |
| PIB mundial 2025 (PIB 2022), tri | 93,3 | 114,2 | 133,4 |

- **A urbanização que a política tirou volta:** de 48 % para 70 % fora da agropecuária (73 % real).
  O que faltava era a saída: o camponês sem emprego não migrava (0,5 % dos ociosos por ano) e a
  cidade não tinha onde pô-lo.
- **A China fica pobre e agrária, como deve sem abertura nem capital estrangeiro:** indústria em
  1 %; quem sai do campo vai para o informal urbano, não para a fábrica.
- **Quanto do crescimento de 1991–2008 que falta é a Ásia:** Leste da Ásia +0,7 contra +3,7 % ao
  ano, Sul +0,9 contra +3,9, Sudeste +1,8 contra +3,5. Com as parcelas da Ásia no PIB (simuladas ou
  reais), é 0,6 a 0,9 dos 1,6 pontos que faltam ao mundo: o resto é o crescimento depois de 1991 dos
  demais, que ainda depende da fronteira tecnológica e da convergência.
- **Aberto:** o informal dos países ricos fica muito acima da OIT (Alemanha 31 % contra 4 %): a
  automação dispensa operários e a indústria e os serviços formais não crescem para recebê-los.
  Colônias continuam ricas demais (Congo Belga 9 vezes o real na `main` e 17 aqui).
- **Substituição de importações testada e retirada:** construir o que se importa, por regra,
  levou o Congo Belga de 3,5 a 17 vezes o real e o ranking de 2022 a 0,40.
## Placar com a demografia

Rodada de 101 anos da branch `feat/demography` contra a `main` do mesmo dia (9cc71e1, com a saída
do campo do #69 e a fronteira agrícola do #77), semente 1925, 2 threads. "`main`" é a base;
"branch", a demografia (pirâmide, transição, corrente, diáspora, ondas, decomposição:
`population.pt-BR.md`, "Demografia").

**A população mundial de 2025 fica em 5,76 bilhões contra 8,23; a transição chega cedo demais
depois de 1975 na Ásia do Sul e na África.** Até 1975 o mundo acompanha o real (4,26 contra 4,07);
depois ele quase para (5,46 em 2005). A causa está nas entradas da economia (abaixo, "Por que o
mundo para depois de 1975").

### Mundo

| Ano | População `main` | branch | real | erro log pop `main` → branch | erro log PIB pc `main` → branch |
|---|---|---|---|---|---|
| 1925 | 1.96 | 1.96 | 1.97 | 0.04 → 0.04 | 0.11 → 0.11 |
| 1935 | 2.18 | 2.28 | 2.17 | 0.07 → 0.07 | 0.26 → 0.26 |
| 1945 | 2.44 | 2.64 | 2.38 | 0.12 → 0.12 | 0.47 → 0.45 |
| 1955 | 2.80 | 3.11 | 2.74 | 0.20 → 0.18 | 0.61 → 0.63 |
| 1965 | 3.25 | 3.70 | 3.33 | 0.27 → 0.22 | 0.68 → 0.73 |
| 1975 | 3.74 | 4.26 | 4.07 | 0.35 → 0.27 | 0.80 → 0.84 |
| 1985 | 4.21 | 4.72 | 4.87 | 0.43 → 0.34 | 0.92 → 0.94 |
| 1995 | 4.68 | 5.13 | 5.76 | 0.51 → 0.42 | 1.13 → 1.11 |
| 2005 | 5.13 | 5.46 | 6.58 | 0.58 → 0.49 | 1.11 → 1.11 |
| 2015 | 5.56 | 5.67 | 7.47 | 0.66 → 0.59 | 1.11 → 1.12 |
| 2025 | 5.98 | 5.76 | 8.23 | 0.72 → 0.67 | 1.18 → 1.18 |

### Erro por grupo (média de |ln(simulado/real)| da população em 2025)

| Grupo | n | `main` | branch |
|---|---:|---:|---:|
| Total | 96 | 0,72 | 0,67 |
| Resto (Américas, África, Ásia colonial, domínios) | 49 | 0,86 | 0,69 |
| Europa ocidental | 16 | 0,24 | 0,38 |
| URSS e Leste Europeu | 13 | 0,45 | 0,45 |
| China e Tibete | 2 | 0,15 | 0,04 |
| Golfo e Arábia | 9 | 1,42 | 1,34 |
| Microestados | 7 | 0,69 | 0,87 |

A Europa ocidental piora (França a 1,37 por imigração de mais, sem a guerra) e os microestados
também; o resto melhora, e a China fica na medida (1,08) pelo freio da fome de terra.

### Os 20 maiores: população simulada / real

| Unidade | 1925 | 1945 | 1965 | 1985 | 2005 | 2025 |
|---|---|---|---|---|---|---|
| CHN/metropole | 1.02 → 1.02 | 1.16 → 1.26 | 1.17 → 1.29 | 1.10 → 1.16 | 1.15 → 1.11 | 1.31 → 1.08 |
| RAJ/metropole | 0.88 → 0.88 | 0.95 → 1.03 | 0.97 → 1.08 | 0.85 → 0.92 | 0.73 → 0.72 | 0.70 → 0.60 |
| SOV/metropole | 1.03 → 1.04 | 1.11 → 1.36 | 1.11 → 1.52 | 1.13 → 1.51 | 1.21 → 1.63 | 1.23 → 1.57 |
| USA/metropole | 1.04 → 1.04 | 0.98 → 0.95 | 0.80 → 0.76 | 0.69 → 0.66 | 0.59 → 0.57 | 0.53 → 0.50 |
| GBR/colonies | 1.00 → 1.00 | 0.94 → 1.00 | 0.81 → 0.98 | 0.63 → 0.72 | 0.45 → 0.50 | 0.32 → 0.34 |
| FRA/colonies | 1.01 → 1.02 | 1.00 → 1.05 | 0.87 → 1.05 | 0.71 → 0.86 | 0.54 → 0.65 | 0.40 → 0.48 |
| GER/metropole | 0.97 → 0.97 | 1.01 → 0.87 | 1.10 → 0.80 | 1.15 → 0.76 | 1.14 → 0.71 | 1.17 → 0.67 |
| JAP/metropole | 0.98 → 0.98 | 0.88 → 0.83 | 0.81 → 0.70 | 0.74 → 0.60 | 0.74 → 0.58 | 0.81 → 0.59 |
| NLD/colonies | 1.16 → 1.16 | 1.11 → 1.22 | 0.95 → 1.22 | 0.77 → 1.02 | 0.67 → 0.90 | 0.59 → 0.86 |
| GBR/metropole | 0.98 → 0.98 | 1.14 → 1.04 | 1.24 → 1.02 | 1.33 → 1.04 | 1.34 → 1.00 | 1.24 → 0.85 |
| FRA/metropole | 1.03 → 1.03 | 1.23 → 1.20 | 1.25 → 1.23 | 1.24 → 1.29 | 1.24 → 1.33 | 1.24 → 1.37 |
| ITA/metropole | 1.04 → 1.04 | 1.07 → 1.13 | 1.14 → 1.18 | 1.15 → 1.19 | 1.18 → 1.24 | 1.22 → 1.23 |
| BRA/metropole | 0.99 → 0.99 | 0.78 → 1.02 | 0.58 → 0.96 | 0.46 → 0.79 | 0.39 → 0.68 | 0.37 → 0.62 |
| POL/metropole | 1.15 → 1.16 | 1.33 → 1.54 | 1.41 → 1.65 | 1.34 → 1.56 | 1.39 → 1.66 | 1.46 → 1.70 |
| JAP/colonies | 0.98 → 0.99 | 0.87 → 0.99 | 0.72 → 0.95 | 0.63 → 0.78 | 0.61 → 0.74 | 0.62 → 0.71 |
| ESP/metropole | 1.00 → 1.00 | 1.02 → 1.19 | 1.07 → 1.26 | 1.02 → 1.14 | 0.96 → 1.02 | 0.92 → 0.89 |
| ROM/metropole | 1.11 → 1.12 | 1.14 → 1.45 | 1.27 → 1.75 | 1.31 → 1.63 | 1.60 → 1.82 | 2.00 → 2.00 |
| MEX/metropole | 1.02 → 1.03 | 0.87 → 1.10 | 0.64 → 0.95 | 0.47 → 0.69 | 0.38 → 0.56 | 0.32 → 0.47 |
| CZE/metropole | 1.04 → 1.04 | 1.14 → 1.05 | 1.31 → 1.09 | 1.29 → 1.00 | 1.34 → 1.01 | 1.36 → 0.97 |
| EGY/metropole | 1.04 → 1.04 | 0.93 → 1.02 | 0.75 → 0.89 | 0.62 → 0.77 | 0.51 → 0.57 | 0.41 → 0.43 |

### Por que o mundo para depois de 1975

Termos da fecundidade (`fertility_terms`, valores de 0 a 1) e o que sai deles, por década:

| Território | Ano | TFR | e0 | Sobrevivência | Escolaridade | Fora do campo | Conforto | Contracepção | Nota |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Colônias britânicas | 1955 | 6,2 | 48 | 0,50 | 0,33 | 0,54 | 0,58 | 0,00 | 0,41 |
| Colônias britânicas | 1975 | 2,3 | 70 | 0,89 | 0,56 | 0,66 | 0,68 | 0,79 | 0,81 |
| Colônias britânicas | 1995 | 1,8 | 82 | 1,00 | 0,74 | 0,75 | 0,76 | 1,00 | 0,99 |
| Colônias britânicas | 2015 | 1,8 | 82 | 1,00 | 0,82 | 0,82 | 0,83 | 1,00 | 1,04 |
| Brasil | 1955 | 5,8 | 49 | 0,53 | 0,52 | 0,64 | 0,67 | 0,00 | 0,54 |
| Brasil | 1975 | 2,6 | 63 | 0,78 | 0,64 | 0,75 | 0,76 | 0,54 | 0,79 |
| Brasil | 1995 | 1,8 | 78 | 0,98 | 0,75 | 0,82 | 0,84 | 0,86 | 0,98 |
| Brasil | 2015 | 1,8 | 82 | 1,00 | 0,82 | 0,87 | 0,88 | 1,00 | 1,05 |
| China | 1955 | 5,5 | 41 | 0,35 | 0,35 | 0,34 | 0,31 | 0,00 | 0,34 |
| China | 1975 | 4,2 | 51 | 0,55 | 0,47 | 0,33 | 0,32 | 0,48 | 0,56 |
| China | 1995 | 2,8 | 62 | 0,76 | 0,58 | 0,38 | 0,38 | 0,68 | 0,72 |
| China | 2015 | 2,4 | 66 | 0,81 | 0,66 | 0,43 | 0,44 | 0,74 | 0,80 |
| Índia (RAJ) | 1955 | 5,6 | 41 | 0,35 | 0,27 | 0,43 | 0,43 | 0,00 | 0,31 |
| Índia (RAJ) | 1975 | 4,1 | 53 | 0,60 | 0,45 | 0,47 | 0,45 | 0,47 | 0,58 |
| Índia (RAJ) | 1995 | 2,5 | 66 | 0,82 | 0,59 | 0,52 | 0,53 | 0,70 | 0,77 |
| Índia (RAJ) | 2015 | 1,9 | 72 | 0,90 | 0,70 | 0,62 | 0,63 | 0,78 | 0,88 |
| Egito | 1955 | 5,7 | 40 | 0,33 | 0,31 | 0,52 | 0,55 | 0,00 | 0,34 |
| Egito | 1975 | 4,5 | 55 | 0,64 | 0,47 | 0,65 | 0,67 | 0,34 | 0,60 |
| Egito | 1995 | 1,9 | 71 | 0,90 | 0,61 | 0,74 | 0,76 | 0,71 | 0,84 |
| Egito | 2015 | 1,8 | 78 | 0,99 | 0,75 | 0,81 | 0,82 | 0,85 | 0,97 |

As entradas, simulado contra real:

| Território, 1985 | Alfabetização | Expectativa de vida | Natalidade (por mil) |
|---|---|---|---|
| Índia (RAJ) | 0,65 / 0,42 | 61 / 56 | 22 / 37 |
| Egito | 0,65 / 0,44 | 62 / 59 | 22 / 38 |
| Colônias britânicas | 0,79 / 0,57 | 78 / 54 | 15 / 42 |
| Colônias francesas | 0,78 / 0,56 | 77 / 59 | 15 / 40 |
| China | 0,65 / 0,70 | 57 / 67 | 25 / 23 |

A queda vem da escolaridade e da sobrevivência infantil, os dois termos de maior peso, e da
contracepção. As três entram infladas na Ásia do Sul, no Egito e nas colônias:

- a economia alfabetiza as colônias no ritmo da metrópole (+0,22 nas colônias britânicas em 1985);
- os efeitos de pesquisa (remédio e pílula) são por mercado, e as colônias dividem o mercado com a
  metrópole: recebem a medicina britânica e francesa (expectativa de vida de 78 anos em 1985,
  contra 54) e a pílula com a força delas (contracepção 1,0 em 1985).

Entre 1965 e 1975 a nota das colônias britânicas sobe de 0,58 para 0,81: +0,10 vêm da
contracepção, +0,07 da escolaridade e +0,05 da sobrevivência. É entrada errada da economia e da
política (ensino, saúde e orçamento coloniais), não parâmetro da demografia: com as entradas reais
de 1985, a nota da Índia ficaria perto de 0,60 em vez de 0,70, e a fecundidade perto de 4 filhos
(real 4,3) em vez de 2,9. A China vai no sentido oposto (alfabetização e saúde abaixo do real) e
fica na medida pelo freio da fome de terra.

### O que guerra e fome tirariam

O modelo não tem guerra nem fome. A Segunda Guerra matou de 60 a 75 milhões de pessoas, das quais
19 a 28 milhões por fome e doença da guerra (Wikipedia, *World War II casualties*, CC BY-SA). As
fomes de 1920 a 2019 mataram 98 milhões, 61 milhões delas de 1930 a 1969 (China, URSS, Bengala;
World Peace Foundation, via Our World in Data, *Deaths from famines by decade*, CC BY). Juntas, e
descontada a sobreposição, são de 130 a 150 milhões de mortes antes de 1970, mais os nascimentos
que não houve e os descendentes de ambos. É o que explica o mundo simulado de 1945 a 1975 acima do
real; não é o que explica 2025, que fica abaixo.

**China e Índia, dependência.** A queda real da fecundidade chinesa (6 filhos em 1970, 2,7 em
1979) veio de saúde, escola e planejamento familiar de um regime ("mais tarde, mais espaçados,
menos" e o filho único), que o modelo ainda não tem (fase 7). Na rodada a China chega a 1,08 do
real pelo freio da fome de terra; a Índia, a 0,60, pela entrada inflada acima.

### Chegadas ao Brasil por origem (mil por ano, simulado / IBGE)

| Estrada | 1925s | 1935s | 1945s | 1955s | 1965s |
|---|---|---|---|---|---|
| Brasil ← Itália | 18.4 / 6.0 [7] | 25.2 / 1.3 [5] | 32.5 / 4.9 [3] | 38.5 / – | 41.2 / – |
| Brasil ← Portugal | 18.5 / 24.1 [7] | 23.0 / 10.1 [5] | 26.3 / 12.3 [3] | 31.6 / – | 37.0 / – |
| Brasil ← Espanha | 14.4 / 4.9 [7] | 21.5 / 0.5 [5] | 28.0 / 4.0 [3] | 37.9 / – | 40.1 / – |
| Brasil ← Alemanha | 9.1 / 4.8 [7] | 9.8 / 2.3 [5] | 13.7 / 1.7 [3] | 13.2 / – | 20.2 / – |
| Brasil ← Japão | 7.6 / 12.0 [7] | 3.6 / 2.6 [5] | 10.3 / 0.0 [3] | 14.9 / – | 21.8 / – |
| Brasil ← URSS | 0.1 / 0.9 [7] | 0.3 / 0.0 [5] | 0.4 / 0.0 [3] | 0.6 / – | 1.0 / – |
| Brasil ← todos | 90.3 / 73.1 [7] | 116.0 / 21.6 [5] | 150.2 / 29.9 [3] | 189.9 / – | 237.3 / – |
| EUA ← todos | 295.5 / 194.8 | 317.1 / 47.6 | 336.5 / 175.2 | 358.2 / 281.9 | 377.9 / 371.6 |

- **1925:** dentro do fator 3 Portugal (0,77), Espanha (2,9), Alemanha (1,9) e Japão (0,63); a
  Itália fica no limite (3,07). O total (90 contra 73) fica perto.
- **1935:** dentro, Portugal (2,3) e Japão (1,4). Fora, a Itália (19×), a Espanha (43×) e a
  Alemanha (4,3×). A referência despenca pela Depressão, pela cota brasileira de 1934 (a lei muda
  só por movimento político, e na rodada o `nativism` tira o subsídio em 1976 e põe cotas em 1981),
  pela restrição fascista de 1927 e pela Guerra Civil Espanhola (1936–39). Regime e guerra:
  dependência anotada.
- **Sírio-libaneses:** fluxo positivo, de 2,9 mil por ano em 1925–34 a 14 mil por ano nos anos
  1980. O IBGE de 1952 não os separa ("outros"), e a série anual não existe em fonte aberta. A
  conferência é o estoque: 19 290 no estado de São Paulo em 1920 (Truzzi, *Revista Brasileira de
  Estudos de População*, 2019, CC BY) e 45 786 de nacionalidade síria no Brasil em 1940
  (recenseamento de 1940, edição de 1947, nota 7). No modelo, 41 mil em 1925 (comunidades, com os
  filhos) mais cerca de 52 mil chegados até 1940: a mesma ordem, contando descendentes.

### Ondas de migração (as maiores)

| Origem → destino | Abriu | Fechou | Por quê abriu | Por quê fechou | Pessoas (mil) |
|---|---|---|---|---|---:|
| CHN/metropole → NLD/colonies | 2002-04 | 2013-01 | Land | Full | 6712 |
| ESP/metropole → ARG/metropole | 1925-01 | aberta | Poverty | – | 3789 |
| ROM/metropole → FRA/metropole | 1951-10 | aberta | Poverty | – | 3029 |
| CHN/metropole → NLD/colonies | 2013-02 | 2017-02 | Land | Full | 2728 |
| CHN/metropole → NLD/colonies | 2017-03 | 2020-02 | Land | Full | 2048 |
| CHN/metropole → NLD/colonies | 2021-10 | 2024-03 | Land | Full | 1751 |
| BRA/metropole → FRA/metropole | 2012-12 | aberta | Poverty | – | 1632 |
| CHN/metropole → NLD/colonies | 2024-04 | aberta | Land | – | 1286 |
| TUR/metropole → BUL/metropole | 1925-01 | 1983-01 | Land | DoorClosed | 1195 |
| CHN/metropole → NLD/colonies | 2020-03 | 2021-09 | Land | Full | 1072 |
| GER/metropole → CAN/metropole | 2004-06 | aberta | Poverty | – | 872 |
| EGY/metropole → FRA/colonies | 2008-03 | aberta | Poverty | – | 781 |

### Determinismo e desempenho

Salvar em 1975 e recarregar dá o mesmo hash final na rodada de 100 anos. Com 10 anos e 1 thread,
o hash é o mesmo com outro número de threads (`--check-determinism`). CPU de 10 anos, 1 thread,
sob a trava: `main` 48,4 s, branch 50,8 s (+4,8 %). O sistema `demography` custa 0,55 s, contra
0,06 s da demografia antiga; o tick mais lento é o primeiro mês (197 ms), que monta a pirâmide de
1925 de cada estado.

## Fontes

As fontes, licenças e atribuições estão em `data/history/SOURCES.md`. Em resumo: Maddison Project
2023 (PIB por habitante), Our World in Data (população com HYDE, Gapminder e ONU; urbanização da
HYDE 3.5; alfabetização da UNESCO; carvão, petróleo e gás do Energy Institute e do Shift Project;
eletricidade do Ember e do Energy Institute; cereais da FAO) e Banco Mundial (emprego por setor da
OIT, inflação, contas públicas).

Sem referência por licença: aço por país (os termos do British Geological Survey proíbem repassar
o dado; o placar usa a série só na máquina que a baixou), cimento por país e veículos (sem fonte
livre por país). Sem referência por falta de série: cereais antes de 1961, emprego por setor antes
de 1991, contas públicas antes de 1972.

### Lacunas e o que as cobriria

| Série | Referência hoje | O que cobriria antes | Por que não entrou |
|---|---|---|---|
| Alfabetização | Mundo só desde 1995 (a série da OWID tem um ponto por país em torno de 1950, da UNESCO, e quase nada antes; abaixo de 80 % da população coberta, o placar deixa `–`) | Clio Infra, "Literacy" (Buringh e van Zanden, por país e década desde 1500), e a série de anos de estudo de *How Was Life?* (OECD, CC BY 4.0), que é a origem da parte antiga da OWID | Clio Infra não declara licença na página e o download não foi testado; com a regra nova (dado público pode), é a primeira a entrar. A parte de *How Was Life?* já vem pela OWID, mas cobre poucos países antes de 1950 |
| Aço | Desde 1970 (OWID `minerals`, USGS e BGS), só local | USGS *Minerals Yearbook* (domínio público): tabelas de produção mundial por país desde 1932, em PDF | O site da USGS recusou o acesso automático (403) e as tabelas são PDF por ano; daria para extrair, não foi feito. Antes de 1932, Mitchell (*International Historical Statistics*) é livro com direito autoral |
| Cereais | Desde 1961 (FAOSTAT) | International Institute of Agriculture, anuários de 1910–1946 (domínio público por idade) | Escaneados, sem série digital por país encontrada |
| Emprego por setor | Desde 1991 (OIT modelada) | Herrendorf, Rogerson e Valentinyi (via OWID, 1801–2015, poucos países) | Sem licença no metadado; com a regra nova pode entrar, com a licença anotada como desconhecida |
| Contas públicas e inflação | Desde 1960–72 (WDI) | Jordà–Schularick–Taylor (CC BY-NC-SA, 18 países ricos, 1870–2020); IMF *Public Finances in Modern History* (termos do IMF, dívida de 151 países desde 1800) | NC e termos próprios eram proibidos quando a referência foi montada; com a regra nova, entram num arquivo separado com a licença original |


## Placar de eventos (formato de entrada)

Para perguntas de "aconteceu? quando?" (uma lei, uma crise, um regime), outro sistema pode gravar
um arquivo JSON que o placar vai ler ao lado das séries. Ainda sem uso: o formato fica fixado aqui
para quem for produzir os dados (o thread de política, primeiro).

Arquivo: `target/history/events.json`, gravado pela rodada (por exemplo, por uma opção do
`bf-bench` como `--events-out`), com a lista de perguntas e respostas:

```json
{
  "version": 1,
  "events": [
    {
      "id": "GBR-women-suffrage-equal",
      "country": "GBR",
      "question": "Voto feminino em igualdade com o masculino",
      "reference_date": "1928-07-02",
      "simulated_date": "1931-03-01",
      "source": "Representation of the People (Equal Franchise) Act 1928"
    }
  ]
}
```

| Campo | Tipo | Significado |
|---|---|---|
| `id` | texto | Chave estável, única no arquivo: `<tag>-<assunto>`, em inglês, minúsculas e hifens |
| `country` | texto | Tag de 1925 (`GBR`, `SOV`…), como em `countries.ron`; `WORLD` para eventos sem país |
| `question` | texto | A pergunta em uma linha, para a tabela do placar |
| `reference_date` | `AAAA-MM-DD` ou `null` | Quando aconteceu na história; `null` se não aconteceu até 2025 |
| `simulated_date` | `AAAA-MM-DD` ou `null` | Quando aconteceu na rodada; `null` se não aconteceu até o fim dela |
| `source` | texto, opcional | Onde está a data de referência (lei, livro, base de dados) |

Datas no lugar de anos porque a simulação anda por dia; quem só souber o ano grava `AAAA-01-01`.
O placar vai contar acertos (os dois `null` ou as duas datas dentro de uma janela, por exemplo 5
anos), eventos que aconteceram só de um lado, e o atraso médio em anos entre os que aconteceram dos
dois, por país e no total. As datas de referência seguem a regra do repositório: com fonte, nunca
de memória.
