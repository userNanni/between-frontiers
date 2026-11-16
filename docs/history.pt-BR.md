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
