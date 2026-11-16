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
| `world.png` | Mundo: população, PIB, PIB por habitante, urbanização, alfabetização |
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
| Urbanização | `urban_share`: parcela da população fora das profissões rurais (camponês, assalariado rural, proprietário de terra) |
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
- **Maiores desvios**: candidatos (mundo, as 20 maiores unidades, região × período) ordenados por
  |ln(simulado/real)| × √(peso no mundo). O crescimento das unidades é medido desde 1925, para que
  a diferença de fronteira de partida não conte.

Como o PIB do Maddison termina em 2022, as linhas de PIB de 2025 e o último período (2008–25)
comparam 2022 dos dois lados.

### O que não é a mesma coisa dos dois lados

- **Urbanização.** A simulação conta todo mundo fora da agropecuária; a HYDE conta quem mora em
  área urbana. Em 1925 isso dá 38 % contra 21 %: não é erro de dado de entrada (o cenário carrega a
  urbanização da HYDE, 21,5 % no mundo, em `countries.ron`), é definição (`Region::urban_share`).
  Cidades pequenas, aldeias com artesãos, mineiros e funcionários do interior são "urbanos" no
  modelo.
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

| Ano | População | PIB (tri US$ 2011) | PIB por hab. | Urbanização¹ | Erro log. médio pop / PIB pc | Ranking PIB pc |
|---|---|---|---|---|---|---|
| 1925 | 1,96 / 1,97 bi | 3,0 / 5,6 | 1 525 / 2 859 | 37 / 21 % | 0,04 / 0,64 | 0,94 |
| 1935 | 2,17 / 2,17 bi | 6,3 / 6,4 | 2 888 / 2 969 | 43 / 24 % | 0,07 / 0,37 | 0,81 |
| 1945 | 2,44 / 2,38 bi | 9,4 / 8,0 | 3 857 / 3 378 | 47 / 27 % | 0,12 / 0,52 | 0,64 |
| 1955 | 2,81 / 2,74 bi | 13,0 / 10,9 | 4 620 / 3 986 | 49 / 31 % | 0,20 / 0,57 | 0,63 |
| 1965 | 3,30 / 3,33 bi | 20,2 / 17,2 | 6 128 / 5 168 | 51 / 35 % | 0,26 / 0,67 | 0,59 |
| 1975 | 3,83 / 4,07 bi | 32,6 / 26,8 | 8 503 / 6 577 | 52 / 37 % | 0,32 / 0,76 | 0,54 |
| 1985 | 4,38 / 4,87 bi | 48,4 / 37,1 | 11 052 / 7 622 | 53 / 41 % | 0,39 / 0,79 | 0,57 |
| 1995 | 4,96 / 5,76 bi | 63,6 / 50,0 | 12 825 / 8 686 | 55 / 45 % | 0,46 / 0,89 | 0,58 |
| 2005 | 5,56 / 6,58 bi | 77,4 / 74,7 | 13 928 / 11 348 | 56 / 49 % | 0,52 / 0,85 | 0,53 |
| 2015 | 6,17 / 7,47 bi | 91,8 / 109,3 | 14 881 / 14 637 | 57 / 53 % | 0,59 / 0,80 | 0,48 |
| 2025² | 6,78 / 8,23 bi | 102,7 / 133,4 | 15 578 / 16 633 | 59 / 57 % | 0,64 / 0,84 | 0,48 |

Simulado / real. ¹ Simulação: fora da agropecuária; referência: HYDE, urbana. ² PIB de 2022 dos
dois lados.

Leitura curta: a média mundial acerta a ordem de grandeza e termina perto (PIB por habitante 6 %
abaixo em 2022), mas por um caminho errado. A economia começa com pouco mais da metade do PIB, se
recupera em dez anos, cresce demais entre 1930 e 1990 e de menos depois. O mundo fica mais
igual do que foi: a correlação de ranking cai de 0,94 para 0,48, e o erro típico do PIB por
habitante passa de um fator 2 (0,84). A população anda em linha reta e termina 18 % abaixo.

### Os 10 maiores desvios

Ordem do placar (|log| × √peso). "Confirmado" quer dizer que o mecanismo foi visto no código, não
que o tamanho do efeito foi medido.

1. **PIB do primeiro ano: 3,0 tri contra 5,6 tri.** O mercado nasce com `gdp_last_year` igual ao
   PIB-alvo da calibração (`econ/setup.rs`), mas o PIB contado no primeiro ano (produção vendida a
   preço-base menos insumos, mais a subsistência, `econ/week.rs::settle_region`) dá 57 % dele, em
   todas as grandes unidades (razão PIB pc de 0,46 a 0,68 em 1925). Confirmado que as duas contas
   são diferentes; **hipótese**: a diferença está em produção não vendida, plantas que reduzem a
   utilização na primeira semana e serviços que a calibração conta e o mercado não fecha. O
   antigo "tombo de 1926–27" era isto: o bench lia `gdp_last_year` antes da virada de 1º de
   janeiro, que ainda guardava o alvo calibrado (o placar lê `gdp_running`; o relatório decenal do
   `--game` continua com o rótulo um ano atrasado). É o desvio que mais pesa, porque todo o
   crescimento de 1925–35 (+8 %/ano no mundo) é a recuperação dele.
2. **China em 2022: 8 500 contra 19 200 $ por habitante**, depois de passar 2,6 vezes à frente em
   1975. Confirmado: fora das tecnologias, a produtividade converge para o estado líder na
   proporção da distância e da alfabetização (`econ/yearly.rs`, `catch_up`), sem regime, guerra
   ou reforma; a China cresce cedo demais (1945–73) e para quando a distância fecha. Não há
   mecanismo para a decolagem pós-1978 (**hipótese**: falta abertura comercial e investimento
   estrangeiro como motor de convergência).
3. **Depressão ausente nas economias ricas.** Ocidente de colonização europeia em 1929–33: +5,5 %
   contra −9,1 % ao ano. Confirmado: não há padrão-ouro, crise bancária nem colapso do comércio
   (planejados para a fase 4, `architecture.pt-BR.md`); o período ainda é a recuperação do item 1.
4. **Leste da Ásia em 2008–25: +0,4 contra +4,6 % ao ano** (e 1991–2008: +0,9 contra +3,7 %). Mesma
   causa do item 2: a convergência termina quando a distância ao líder fecha. Confirmado.
5. **População das colônias britânicas: ×4,4 contra ×10** (Nigéria, África Oriental, Ceilão,
   Malásia…). Confirmado: a natalidade cai com a "urbanização" (`econ/monthly.rs::vital_rates`
   recebe `urban_share`, que conta todo mundo fora da agropecuária, 37 % do mundo em 1925 contra
   21 % urbanos) e com a alfabetização ao quadrado, que o modelo faz crescer depressa. **Hipótese**:
   isso adianta a transição demográfica em décadas onde ela foi tardia (África), e é a maior parte
   dos 1,45 bilhão de pessoas que faltam em 2025.
6. **Europa Ocidental em 1945–73: +2,3 contra +4,0 % ao ano.** Confirmado: sem guerra, não há
   destruição para reconstruir; os anos dourados históricos foram em boa parte recuperação e
   difusão tecnológica americana. A Europa chega a 1945 já rica (o item 9).
7. **Leste da Ásia em 1991–2008:** ver item 4.
8. **México em 2022: 3 100 contra 16 200 $ por habitante** (razão cai de 0,46 em 1925 para 0,19), e
   população 0,50 da real. **Hipótese**: a convergência depende da alfabetização, que o modelo
   eleva devagar no México rural, e a urbanização de 1925 (29 % fora da agropecuária) prende a
   produtividade agrícola; não conferido no código além da fórmula do `catch_up`.
9. **População mundial em 2025: 6,78 contra 8,23 bilhões.** Ver item 5; a curva é quase linear
   (≈ 57 milhões por ano), sem a explosão de 1950–90. Confirmado também que a mortalidade
   não cai abaixo de 6 por mil nem a natalidade abaixo de 7 (`vital_rates`), o que segura os
   países ricos mas não explica o déficit dos pobres.
10. **Guerra ausente: Europa Ocidental em 1939–45, +2,4 contra −3,3 % ao ano**; Ocidente de
    colonização europeia, +1,8 contra +6,5 % (o boom americano de guerra). Confirmado: não há
    guerra no modelo ainda.

Também no placar, abaixo dos dez: a URSS cresce 3,7 % ao ano em 1973–91 contra 0,4 % (sem
estagnação nem colapso, porque o modelo não tem efeito de regime sobre a economia; confirmado
pela ausência) e o Sul da Ásia cresce 3,2 % ao ano em 1945–73 contra 0,8 % (a mesma convergência
do item 2).

### Outros sinais do primeiro placar

- **Urbanização de 1925**: 37 % contra 21 % é definição, não dado (ver "O que não é a mesma coisa").
  Em 2025 as duas medidas quase se encontram (59 % contra 57 %), porque a parcela fora da
  agropecuária quase não sobe depois de 1950.
- **Emprego por setor em 2025**: a agropecuária fica alta demais nos países pobres (China 71 %
  contra 22 %; México 75 % contra 12 %) e a indústria baixa em todo lugar (5 a 21 % contra 19 a
  35 %). Os camponeses sem emprego contam na agropecuária.
- **Energia**: o carvão simulado passa da referência a partir de 1935 (o dobro em 1945) e o gás
  fica em um terço da referência. O petróleo começa baixo (15 % em 1925) e chega em 1985.
  A conversão por poder calorífico médio pode responder por 20 a 30 % do carvão, não pelo
  dobro.
- **Contas públicas e inflação**: impostos e gasto em 22–26 % do PIB, dívida em 41–59 % e inflação de
  2 % em quase todos os países em 2015–24. A referência varia muito mais (impostos de 12 % na Índia a
  44 % na França; dívida de 48 % no México a 133 % no Reino Unido).

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
