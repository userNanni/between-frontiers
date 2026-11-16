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
