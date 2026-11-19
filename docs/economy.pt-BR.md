# Economia: catálogo e regras

Estado: fase 1 concluída (#29). Mercados nacionais fecham toda semana, com comércio mundial em pool; o
investimento privado, o trabalho e a demografia rodam todo mês. O que ficou simplificado está no plano de
melhoria em `architecture.pt-BR.md`, distribuído pelas fases 2 a 4.

Os dados ficam em `data/economy/` (`goods.ron`, `buildings.ron`, `needs.ron` e `purchases.ron`),
com o contrato em `crates/bf-sim/src/economy.rs` e os nomes em `assets/i18n/*/economy.ftl`.
`purchases.ron` traz as cestas de compra do Estado, da manutenção e da construção de cada grupo
de edifício, e o frete de cada grupo; `needs.ron` traz o peso de cada necessidade no gasto além
das faixas (`spare`). O catálogo resolve tudo na carga e recusa chave desconhecida. O teste
`crates/bf-sim/tests/economy_catalogue.rs` recusa referência quebrada, método que usa um bem antes de ele existir,
bem que ninguém produz, bem civil que ninguém consome, grupo de edifício sem compras e cesta que não
soma 1000 por mil.

## Bens por mercado

São 85 bens. Os preços-base estão em dólares de 2011 e ainda serão calibrados.

- **Alimentos básicos:** Cereais, Milho, Arroz.
- **Alimentos:** Carne, Laticínios, Pescado, Frutas e hortaliças, Açúcar, Café, Chá, Cacau, Tabaco, Bebidas, Alimentos industrializados.
- **Produtos agrícolas:** Algodão, Lã, Borracha natural, Fibras vegetais, Madeira.
- **Minérios:** Minério de ferro, Minério de cobre, Bauxita, Ouro, Minério de chumbo e zinco, Cassiterita, Minério de níquel, Minério de manganês, Cromita, Minério de tungstênio, Enxofre, Rocha fosfática, Potássio, Minério de urânio, Minério de terras raras (a partir de 1950).
- **Energia:** Carvão, Petróleo bruto, Gás natural, Combustíveis, Eletricidade.
- **Insumos industriais:** Aço, Alumínio, Cobre, Metais não ferrosos, Ferroligas, Produtos químicos, Fertilizantes, Explosivos, Cimento, Vidro, Papel, Tecidos, Borracha sintética (a partir de 1931), Plásticos, Componentes eletrônicos (a partir de 1950), Combustível nuclear (a partir de 1945).
- **Bens de consumo:** Roupas, Móveis, Utensílios domésticos, Rádios e eletrônicos, Eletrodomésticos, Automóveis, Medicamentos, Artigos de luxo.
- **Bens de capital:** Ferramentas, Máquinas, Motores, Equipamento elétrico, Material ferroviário, Navios, Aviões, Computadores (a partir de 1951).
- **Material militar:** Armas leves, Munição, Artilharia, Blindados, Navios de guerra, Aviões militares.
- **Serviços:** Transporte, Comércio e serviços, Serviços financeiros, Telecomunicações.
- **Construção e serviços públicos (fase 3):** Construção (pontos da fila de obras), Moradia urbana, Ensino, Atendimento hospitalar.

## Recursos minerais

Dezoito recursos existem desde o primeiro dia, cada jazida com tamanho de 1 a 5, produção de 1925, potencial e ano
de descoberta (`data/scenario/1925/deposits.ron`, contrato em `crates/bf-sim/src/resources.rs`).

- **Críticos desde 1925:** cobre e bauxita, com o alumínio feito por eletrólise (Hall-Héroult): a fundição consome
  muita eletricidade, então alumínio barato depende de hidrelétrica.
- **Ligas que decidem guerras:** manganês, cromo, níquel e tungstênio entram como ferroligas no aço especial, na
  artilharia e nos blindados.
- **Descobertos depois:** jazidas cujo ano de descoberta é posterior a 1925 existem no subsolo, mas ficam ocultas
  até serem achadas. É o caso das terras raras (Bayan Obo identificadas em 1936, mineradas a partir dos anos 1950;
  Mountain Pass, 1949) e de boa parte do petróleo do Oriente Médio, do Mar do Norte e da Sibéria.
- **Urânio:** minerado em 1925 como minério de rádio; vira combustível nuclear a partir de 1945.

## Agropecuária por clima

Cada estado tem a parcela de cada clima de Köppen-Geiger, a área de lavoura, pasto e floresta de 1925, se tem costa,
e a aptidão de 0 a 1 para cada tipo de fazenda (`data/scenario/1925/land.ron`; regras em `LAND.md`). Os tipos:

| Tipo | Produz | Onde |
|---|---|---|
| Lavoura de cereais | Trigo, centeio, cevada, aveia | Temperado oceânico e continental, estepe fria, mediterrâneo |
| Lavoura de milho | Milho | Subtropical úmido, continental quente, savana, planaltos tropicais |
| Arrozal | Arroz | Monção e tropical úmido; subtropical com água; irrigado |
| Pecuária extensiva | Carne, lã | Estepes, savanas, pastagens |
| Leiteria | Laticínios | Oceânico e continental, perto das cidades |
| Plantação de algodão | Algodão | Subtropical, savana, deserto irrigado (Egito, Punjab, Turquestão) |
| Canavial / beterraba | Açúcar | Cana nos trópicos; beterraba no clima temperado |
| Cafezal, chazal, cacaual | Café, chá, cacau | Planaltos tropicais; monção de altitude; floresta equatorial |
| Seringal | Borracha natural | Floresta equatorial |
| Tabaco, pomar, fibras, vinhedo | Tabaco, frutas, juta e sisal, bebidas | Conforme o clima de cada um |
| Exploração florestal, pesca | Madeira, pescado | Floresta; costa |

## Construções

São 86 tipos, cada um com métodos de produção que entram ao longo do século (o ano é um piso; a pesquisa da
fase 2 vai liberá-los por tecnologia).

- **Agropecuária:** Lavoura de cereais, Lavoura de milho, Arrozal, Plantação de algodão, Canavial, Lavoura de beterraba, Cafezal, Chazal, Cacaual, Seringal, Plantação de tabaco, Pomar e horta, Plantação de fibras, Vinhedo, Pecuária extensiva, Leiteria, Exploração florestal, Pesca.
- **Extração:** Mina de carvão, Mina de ferro, Mina de cobre, Mina de bauxita, Mina de ouro, Mina de chumbo e zinco, Mina de estanho, Mina de níquel, Mina de manganês, Mina de cromo, Mina de tungstênio, Mina de enxofre, Mina de fosfato, Mina de potássio, Mina de urânio, Mina de terras raras, Campo de petróleo, Campo de gás.
- **Indústria pesada:** Siderúrgica, Usina de alumínio, Fundição de cobre, Fundição de metais, Usina de ferroligas, Refinaria, Indústria química, Fábrica de fertilizantes, Fábrica de explosivos, Cimenteira, Refinaria de terras raras, Usina de combustível nuclear, Fábrica de borracha sintética, Fábrica de plásticos, Fábrica de máquinas, Fábrica de motores, Indústria elétrica, Fábrica de automóveis, Fábrica de material ferroviário, Estaleiro, Fábrica de aviões, Indústria eletrônica.
- **Indústria leve:** Vidraria, Fábrica de papel, Fábrica de tecidos, Confecção, Indústria de alimentos, Fábrica de móveis, Fábrica de utensílios, Fábrica de ferramentas, Indústria farmacêutica, Ateliê de luxo, Fábrica de eletrodomésticos.
- **Energia:** Usina elétrica.
- **Infraestrutura:** Ferrovia, Transporte rodoviário, Porto, Construção civil.
- **Serviços:** Comércio, Banco, Central de telecomunicações, Moradia urbana, Escola e Hospital (públicos; ver `state.pt-BR.md`).
- **Indústria bélica:** Fábrica de armas, Fábrica de munição, Fundição de artilharia, Fábrica de blindados, Arsenal naval, Fábrica de aviões militares.

## Profissões e necessidades

Treze profissões formam a chave da tabela de população (estado × comunidade × profissão). As necessidades têm quatro
níveis, do básico ao afluente; dentro de cada uma, a população troca de bem quando o preço muda (cereais por arroz
ou milho; carvão por lenha, querosene ou gás). Os números por habitante são de partida. Desde a fase 3,
uma necessidade pode ser só das profissões urbanas (aluguel de moradia urbana) ou só das rurais (abrigo).

## 1925: o tamanho e o fluxo do dinheiro

O setup (`econ/setup.rs`) põe as plantas, as pessoas e o dinheiro de 1925 de modo que a primeira
semana compre o que as plantas fazem. Até a fase 3 ele dimensionava as plantas por gastos
supostos (consumo 70 % do PIB, investimento 10 %, compras do Estado 2,6 %), enquanto as regras
semanais pagavam outra coisa: a renda inicial dos pops era 54 % do PIB, o fundo recebia o dobro
do que gastava e a folha do Estado não entrava no PIB. A demanda ficava abaixo da oferta na
primeira semana, as plantas cortavam a utilização e o nível baixo se sustentava: o primeiro ano
media 57 % do PIB do cenário.

Agora, em seis rodadas, o setup alterna entre plantas para um gasto e o gasto que essas plantas
pagam pelas regras da simulação: salários pela `labor_share` e pelo `WAGE_WEIGHT`, 75 % do lucro
aos donos (`PAYOUT`) e o resto ao fundo, a folha do Estado, impostos por classe, poupança por
profissão e o que o plano de consumo não consegue gastar. Cada bloco é escalado para que o valor
adicionado das plantas, a folha do Estado e a comida dos camponeses (comida e vendida) deem o PIB
do cenário, e a produtividade do bloco é a que lota essas plantas com os trabalhadores que ele
tem (antes ela supunha US$ 6 mil por trabalhador e cortava indústria e serviços em até 40 %).
Os pops começam com a renda dessas contas; o fundo, no nível em que a manutenção gasta o que
entra; o Tesouro, com a receita e a folha previstas.

Outras peças da mesma correção:

- **Folha do Estado no PIB.** O PIB medido soma o que o Estado paga a servidores, soldados e
  clero (produção pública pelo custo, como nas contas nacionais).
- **O que os ricos não gastam é investido.** O plano de consumo tem teto (`SPARE_CEILING`, 120
  vezes o custo ponderado das necessidades de cima); o orçamento que ele não consegue colocar vai
  ao fundo (`PopPlan::unplaced`), em vez de ficar parado na poupança. Os donos de 1925 são 0,6 % da
  população com 14 a 20 % da renda.
- **Plantas nos estados com gente.** Indústria e serviços vão às cidades do bloco até 88 % da
  população de cada estado (`TOWN_ROOM`); fazendas e minas, à terra que tem gente para
  trabalhá-la. Antes, o corte por estado jogava fora 6 % do valor adicionado.
- **Comida da própria terra primeiro.** Bens de fazenda atendem a demanda do próprio bloco antes
  de ir ao mundo; o comércio semanal só alcança 85 % da sobra, e os exportadores ficavam com
  fazenda parada. Minas mantêm as parcelas de 1925.

Resultado em 1925: PIB medido 96 % do cenário no mundo (era 57 %); os 20 maiores entre 0,85 e
0,98 do PIB por habitante do Maddison (eram 0,46 a 0,68). O placar está em `history.pt-BR.md`.

### Produção de 1925 contra `industry.ron`

| Bem | Simulado | Referência | Causa da diferença fechada |
|---|---|---|---|
| Aço | 90,9 Mt | 90,5 Mt | A obra usava 0,089 t de aço por US$ 1 000; agora 0,022 (método manual) e 0,031 (mecanizado) |
| Cimento | 58,4 Mt | 62,0 Mt | A obra usava 0,89 t de cimento por US$ 1 000, intensidade do concreto moderno, e a moradia rural comprava cimento; agora 0,087 t e a moradia rural é obra e madeira |
| Veículos | 5,7 M | 4,9 M | Carro era "mobilidade" (nível 3) de todo trabalhador; agora é a necessidade "automóvel" (nível 4), com a gasolina junto |
| Combustível | 39 Mt | 128 Mt | Subiu de 13 Mt com a gasolina dos carros; óleo de navio, ferrovia e indústria e querosene não estão modelados |
| Petróleo bruto | 45 Mt | ≈ 141 Mt | Segue o combustível |
| Carvão | 1 321 Mt | ≈ 1 205 Mt | — |
| Eletricidade | 93 TWh | 195 TWh | Não fechada: as plantas de 1925 nascem no primeiro método, sem eletricidade |

Faixa declarada: ±15 % para aço, cimento, carvão e veículos; eletricidade, combustível e
petróleo ficam fora dela e estão em "Aberto". O valor de um veículo é a média de fábrica dos EUA
em 1925 (US$ 684, *Historical Statistics of the United States*, séries Q 148–151, domínio
público) a preços de 2011 (CPI-U, BLS, fator 12,85): US$ 8 787, ou 8,79 lotes de `automobiles`.
O gasto de gasolina por carro na necessidade "automóvel" segue o mesmo volume: 456 galões por
veículo registrado (séries Q 152 e Q 156), cerca de 27 % do que se gasta comprando carros.

## Investimento e crescimento

Com 1925 no tamanho certo, as plantas rodam cheias e o investimento constrói de verdade (só se
constrói o tipo que roda acima de 90 %; na `main` as plantas rodavam a 80 % e isso freava o
investimento por acaso). Três regras faltavam para o crescimento não disparar:

- **O capital acompanha a produtividade.** Um nível custa `CAPITAL_YEARS` (3) anos do valor que
  adiciona **na produtividade em que vai rodar** (`monthly::level_capital`). Antes o custo era a
  produtividade 1: nos EUA (produtividade 4,8) um nível saía por menos de meio ano de produção.
  Três anos é a ordem da razão capital-produto.
- **Trocar de método é investir.** A troca anual paga o equipamento do método novo (a parte da
  cesta de construção do grupo que não é obra) com o fundo do país, as de maior ganho por dinheiro
  primeiro, até onde o fundo der (`yearly::paid_switches`). Antes era de graça, e em 1926 a
  capacidade subia 28 % num ano.
- **O capital se gasta.** Cada planta privada perde 1,5 % dos níveis por ano
  (`yearly::DEPRECIATION`, fração acumulada em `Plant::wear`); prédios públicos ficam com o
  Estado. A manutenção que o fundo compra (`purchases.ron`, `upkeep`) agora é três quintos obra
  e dois quintos equipamento, como a formação bruta de capital de antes da guerra.

## Convergência

- **Condicional.** A produtividade de base de cada estado converge para uma parte da do líder:
  a posição de 1925 (instituições) mais `CLOSE_WITH_SCHOOLING` (metade) do resto da distância,
  vezes a alfabetização ao quadrado (`yearly.rs`). Antes convergia para o líder inteiro.
- **Terra do camponês.** Quem vive da própria terra só planta a lavoura que as fazendas
  comerciais deixam livre (`week::subsistence_food`, `Catalogue::staple_farms`), e a parte que
  vende cresce com a técnica agrícola do país, não com a convergência de base
  (`week::peasant_surplus`). Antes, os ociosos plantavam sem limite de terra e vendiam até metade
  do que comiam, tirando o mercado das fazendas.

## Saída do campo

- **Migração pelo salário esperado** (`econ/migration.rs`). O camponês ocioso vai para a cidade
  quando a renda por cabeça dos operários (o salário urbano vezes a chance de achar trabalho,
  com os desempregados na conta: Harris–Todaro) passa da do campo com a comida da própria terra.
  Ritmo: até 1,7 % dos camponeses ociosos por ano, cerca de 0,8 % da população rural, a ordem da
  migração líquida campo–cidade dos países em desenvolvimento depois de 1960 (1,5 % da população
  urbana por ano; Jedwab, Christiaensen e Gindelsky, 2017, *Journal of Urban Economics* 98). Antes
  era 0,5 % dos ociosos por ano.
- **Trabalho informal urbano** (`informal_trade`, `BuildingDef::informal`). Comércio de rua,
  serviço doméstico, oficina: ninguém constrói; todo mês o mercado de trabalho o dimensiona para os
  operários que nenhuma planta emprega, nas cidades que pagam mais que o campo e têm lugar. Sem
  capital nem insumo, um trabalhador faz um quarto do que faz um balconista, e todo o valor
  adicionado é renda dele. Vende serviços a outras pops: nenhum dinheiro é criado. Encolhe quando
  as plantas contratam: o investimento conta como mão de obra disponível os camponeses ociosos, os
  operários sem emprego e os informais (antes, só os camponeses ociosos, e uma cidade sem
  camponeses não recebia planta). Emprego informal por país: `monthly::informal_jobs`; no placar,
  contra a OIT (`data/history/informal.csv`), com os camponeses que vivem da própria terra, como a
  OIT os conta.
- **Sem substituição de importações por regra.** Construir o que o país importa, mesmo com as
  plantas paradas, foi testado: industrializou colônias sem capital nem política por trás (Congo
  Belga de 3,5 para 17 vezes o PIB por habitante real em 2022) e piorou o ranking. Industrializar
  pede capital, técnica ou um Estado que planeje; isso fica para o capital estrangeiro e o regime
  (fases 4 e 7). A tarifa da lei de comércio em vigor já chega à economia (`LawEffects::tariff`).

### Informal dos países ricos: serviços sem teto, testado e retirado

Com a demografia (#54), o informal dos ricos caiu sozinho (Alemanha 8 % contra 4 % da OIT, França
9 % contra 3 %), mas Itália e Espanha seguem em 26 % (8 e 4 % na OIT). Foi testado deixar os
serviços sem o teto do plano de consumo (`SPARE_CEILING`): o que o teto deixa de fora iria para a
necessidade de serviços, em vez de ir ao fundo de investimento. Rodada de 100 anos contra a `main`
em fc887b3: informal dos EUA 18 → 28 %, Alemanha 8 → 12 %, França 9 → 16 %, Itália 26 → 31 %; PIB
mundial de 2025 115,5 → 103,7 tri; crescimento por habitante de 1991–2008 +1,4 → +1,3 %; ranking
do PIB por habitante de 2022 0,50 → 0,48. Duas causas da piora: o informal vende o mesmo bem
(`services`) que o comércio formal, então a demanda nova também o enche e ele não encolhe; e o
dinheiro que ia ao fundo e construía planta formal passou a ir para consumo. Desenho candidato,
para depois: o informal produz um bem próprio, mais barato, que os pobres compram e que os ricos
trocam pelo formal quando a renda sobe (substituição pelo preço, não pelo teto).

## 1925: eletricidade e petróleo

- **Eletricidade.** Antes de pôr as usinas, o setup passa as plantas de cada país para o método
  elétrico de 1925 do edifício, os estados mais urbanos primeiro, mantendo a produção, até usarem a
  eletricidade que o país gerava (`industry.ron`) menos a das famílias; as usinas vêm depois, para
  a demanda posta. 1925: 228 TWh contra 195 (era 93); o excesso é de colônias e países pequenos
  cujas plantas já usam eletricidade no primeiro método.
- **Petróleo.** Navios e ferrovias a vapor queimam óleo na parcela de 1925: 30,3 % da tonelagem
  mercante mundial (Lloyd's Register, via *Transportation and Communication in the United States
  1925*) e 11 % do combustível das ferrovias dos EUA em energia (HSUS Q 388–389), trocando carvão
  pela mesma energia. 1925: combustível 0,65 do real (era 0,30), petróleo bruto 92 contra 146 Mt
  (*Mineral Resources of the United States 1925*), carvão 1 269 contra 1 205 Mt.

## Fronteira agrícola

O teto de cada lavoura num estado era a lavoura cultivada em 1925 (HYDE 3.3): o Amazonas tinha 47 km²
de lavoura e 1,7 milhão de km² de floresta, e não podia abrir nenhum. Agora:

- **Terra aberta e potencial** (`Region::land` e `Region::frontier`). A terra aberta é a de 1925,
  como antes. A potencial é o pasto e a floresta do estado vezes a aptidão que o cenário já dá a
  cada lavoura (`land.ron`), dividida pela terra de um nível; nada por país.
- **Abrir terra é obra** (`monthly::Investment`). Uma fazenda que já ocupa toda a terra aberta pode
  abrir um nível na fronteira: custa o nível e outro tanto em obra (desmate e preparo), em pontos
  da fila de construção. O pasto e a floresta que restam no estado (`Region::wild_km2`) diminuem na
  proporção de cada um, a pecuária e a exploração florestal do estado ficam com a parte da terra
  delas que sobrou, e o convertido fica em `Region::cleared_km2`.
- **Só abre onde vende:** a decisão é a do investimento, com o tipo cheio e retorno acima da taxa
  mínima. O camponês sem emprego continua plantando só na terra aberta e livre.

Primeira rodada: 125 mil km² abertos até 1955 (Índia, China, Japão e colônias, Brasil 11 mil),
quase nada depois; cereais 2 157 contra 2 835 Mt em 2015. A fronteira para quando a comida para
de faltar: a produtividade agrícola cresce e a população simulada fica abaixo da real. Abertos: o
pioneiro que desmata para si (o camponês sem terra que abre a própria roça), que foi a maior
parte da fronteira histórica no Brasil e na África.

## Pioneiros

- **Quem abre e quanto** (`monthly::pioneers`). O camponês ocioso de um estado em que a terra
  aberta livre não alimenta os sem emprego abre a fronteira do próprio estado com o trabalho
  dele: 1,5 ha por trabalhador por ano (colonos da Transamazônica, cerca de 10 ha a cada dois anos
  por família nos primeiros oito; Colúmbia Britânica, 2 a 3,7 acres por fazenda por ano em
  1943–45), vezes a parte do transporte que chega ao estado e o `frontier_access` da lei de
  terras. A área acumula em `Region::pioneer_km2`; cada nível completo sai da fronteira, do pasto
  e da floresta e vira terra aberta, onde ele planta a própria comida.
- **Assentados** (`migration::plan`). O camponês ocioso de um estado sem terra muda para o vizinho
  por terra, do mesmo país, com mais fronteira aberta à lei, como camponês, ao ritmo da migração
  local (10 por 10 mil ociosos por mês, vezes o alcance da lei de migração e o acesso à
  fronteira). A migração para a cidade não mudou.
- **Lei de terras** (`frontier_access`, `data/politics/laws.ron`): posse livre e reforma agrária,
  acesso inteiro; latifúndio, 0,3 (a terra pública vendida e tomada pelos grandes, Lei de Terras de
  1850); servidão, 0,1; fazenda coletiva, 0: a terra nova nela é aberta pelo plano do Estado (as
  Terras Virgens), que a simulação ainda não tem.

Rodada de 100 anos: 292 mil km² abertos até 2015 (125 mil sem os pioneiros), contra 5,5 milhões de
lavoura nova no mundo (HYDE 3.5, `data/history/cropland.csv`). Índia 71 contra 658, China 113
contra 479, Brasil 24 contra 523, Argentina, Canadá e Austrália 0 contra 285, 183 e 267. A
fronteira que falta foi comercial e de exportação (trigo da Argentina, do Canadá e da Austrália;
soja e gado no Brasil) e de Estado (Terras Virgens, colonização dirigida); o pioneiro quase não
acha quem abrir, porque o ocioso já vai para a cidade (Brasil 12 % no campo), e a população
simulada fica em 6,0 contra 8,2 bilhões, com menos comida para vender.

## Colônias: salário e técnica (estudo, regra estacionada)

O placar dava às colônias PIB por habitante muito acima do real (2022, `main` antes do #82:
britânicas 2,75 vezes, francesas 3,56, americanas 5,19, Congo Belga 7,56). Estudo medido com
`bf-bench --diag-1925` e `BF_DIAG_COLONIES=1` (na branch estacionada `fix/colonial-productivity`),
que decompõe o valor adicionado das plantas por habitante em emprego de mercado, valor adicionado
por emprego à produtividade 1 (prédios e métodos), produtividade de base e técnica.

**O que foi medido**

- **Metade do excesso era população.** Em 2025 a população colonial ficava em 0,27 a 0,49 da
  real, com o PIB total perto do real (colônias britânicas 0,98, portuguesas 0,87). O #82
  (bolsa colonial) levou a população colonial para perto do real e baixou sozinho o PIB por
  habitante colonial (britânicas 2,75 → 2,16, francesas 3,56 → 2,12).
- **A produtividade de 1925 não separa as colônias.** O setup dá às colônias 1,3 a 1,7 porque
  poucos trabalhadores de mercado fazem o PIB de 1925; tirar as minas de 1925 do cálculo muda
  pouco (Congo 1,44 → 1,39, as demais até 0,01). Turquia (1,67), Sião (1,76) e Pérsia (1,82),
  soberanos com a mesma produtividade de partida, terminam em 0,27 a 0,38 do real.
- **O que separa é a técnica da metrópole.** Contra a Turquia em 2025, em log do valor adicionado
  por habitante, as colônias britânicas ficam +1,23: +0,80 de prédios e métodos (a colônia
  constrói os métodos da metrópole: um centro comercial colonial faz 17,6 mil dólares por emprego
  à produtividade 1, contra 12,3 mil na Turquia), +0,33 de produtividade técnica, +0,07 de base.

**Salário por bloco (testado e retirado).** Um salário só por mercado (`monthly::market_wages`)
mistura metrópole e colônia; separado por bloco (`[metrópole, colônias]`), o colonial fica em
cerca de um terço do metropolitano (britânico: 5,1 contra 16 mil dólares por ano em 1925). Na
escolha de método e no investimento, isso piorou o placar: com mão de obra barata o fundo do
país constrói mais na colônia (emprego de mercado das colônias americanas em 1975: 28 → 34 %), e
as colônias ficam mais ricas (1955: americanas 1,63 → 3,19 vezes o real, Congo 2,20 → 3,04). Na
história o capital não correu para as colônias (risco, falta de mercado e de gente treinada,
política de manter a colônia fornecedora): é assunto da alocação de capital (fase 4,
`foreign_capital.rs`).

**Estágio colonial (testado e estacionado).** Cada mercado guarda o estágio das suas colônias em
cada tecnologia; a colônia não pesquisa, chega ao estágio industrial pela mesma imitação de
qualquer país (alfabetização da colônia, parcela do mundo que faz, canal do país), só depois da
metrópole, e difunde pela regra de difusão; o estágio colonial decide os métodos da colônia e a
produtividade técnica dela. Rodada de 101 anos contra a `main` com o #82 (e0d6a7e):

| PIB por habitante, razão simulado/real | `main` 1975 | estágio colonial 1975 | `main` 2025 | estágio colonial 2025 |
|---|---|---|---|---|
| Colônias britânicas | 2,28 | 1,83 | 2,16 | 1,90 |
| Colônias francesas | 2,53 | 2,20 | 2,12 | 1,93 |
| Colônias americanas (Filipinas) | 1,80 | 1,13 | 3,45 | 3,10 |
| Congo Belga | 3,39 | 2,63 | 8,73 | 4,18 |
| Indonésia | 1,70 | 1,37 | 0,61 | 0,23 |
| Metrópole holandesa | 1,21 | 1,09 | 1,54 | 0,55 |
| Metrópole espanhola | 0,97 | 0,84 | 1,21 | 0,58 |
| Ranking (Spearman) | 0,612 | 0,632 | 0,528 | 0,512 |

As colônias chegam perto do real em 1955 e 1975, mas três metrópoles caem. O que foi medido nelas:

- **Espanha e Bélgica: pesquisa.** O esforço de pesquisa do país conta os engenheiros e
  profissionais empregados no mercado inteiro, colônias incluídas; com métodos atrasados a colônia
  emprega menos deles e a pesquisa da metrópole cai. Contar só os pesquisadores da metrópole
  (a colônia não pesquisa) deixa as duas como na `main` (2025: 1,22 e 1,76). Medir também a
  imitação e a difusão pela alfabetização da metrópole derruba a Espanha (1,21 → 0,63) mesmo sem
  o estágio colonial: a Espanha já oscila de década em década na `main`.
- **Holanda: insumo e construção.** A metrópole tem as mesmas tecnologias com e sem a regra (149
  em escala, 146 difundidas em 2025), uso das plantas de 0,85 a 1,0 e o mesmo emprego de mercado;
  o que cai é a construção do mercado inteiro (5 111 → 1 065 níveis em 2025; na metrópole, 1 453 →
  230) e, com ela, as plantas modernas de máquinas e remédios. Já em 1985 os sítios
  metropolitanos de construção, máquinas e remédios ficam sem método possível: algum insumo dos
  métodos deles é atendido abaixo de 50 % no mercado (`monthly::best_method`; o único comum aos
  três é ferramentas, o que não foi isolado). Sem a regra, as máquinas do mercado holandês são
  feitas sobretudo na Indonésia, que tem nove vezes a população da metrópole (2025: 986 níveis na
  colônia contra 271 na metrópole), e os mesmos sítios rendem 0,58 a 1,30 contra o mínimo de 0,05.
  Medir o gatilho de 90 % de uso por bloco, em vez de no mercado inteiro, não muda a Holanda (2025:
  0,57).
- **Refutado:** migração da Indonésia para a Holanda (a chegada é menor com a regra; a população
  da metrópole cresce por natalidade depois que a renda trava) e demanda baixa da metrópole.

**Por que estacionada.** Na história a Indonésia é independente desde 1949 e a Holanda importou e
investiu fora; no modelo a colônia fica para sempre no mercado da metrópole e o fundo do país não
tem para onde ir. A regra volta quando existirem a descolonização (`borders.pt-BR.md`, parte B: a
colônia que vira país herda `ResearchState::stages(dono, true)`, o estágio colonial, com o teto
da decisão 6) e a alocação de capital entre países (fase 4); e o insumo que falta tem de poder ser
importado. A branch `fix/colonial-productivity` tem a regra, os testes e os diagnósticos.

## Aço

Medido com `bf-bench --diag-1925` e `BF_DIAG_ENERGY=<ano>` mais `BF_DIAG_ENERGY_TAGS=USA,FRA,...`
(produção, uso por mil dólares de PIB e quem usa cada bem, por país). Na `main` em d5570d0 o aço
mundial de 2020 sai perto do real (1 957 contra 1 880 Mt), mas por razões erradas: os ricos fazem
muito mais que o real (EUA 407 Mt contra cerca de 73; França 119 contra 12; Grã-Bretanha 102 contra
7) e a China, menos (62 contra cerca de 1 065). Quem usa o aço em 2020: construção 38 %,
**estaleiro naval militar 24 %**, máquinas 17 %.

**O navio de guerra.** O estaleiro naval gastava 4 000 t de aço por nível de US$ 10,2 a 25,5
milhões de navio, cerca de 157 t por milhão: a proporção de um graneleiro. A estrutura e a
blindagem de um cruzador do tratado (USS Pensacola, encomendado em 1924: limite de US$ 11,1
milhões, 9 100 t padrão) dão cerca de 35 t por milhão de dólares de 2011, e as de um destróier de
guerra (Fletcher, 1942: US$ 6 milhões, 2 050 t), cerca de 11 (`buildings.ron`, com a conta). O
método de 1925 passa a 360 t por nível, e o de 1935, a 280. Um destróier moderno (Arleigh Burke
IIA) fica perto de 1,6 t por milhão; o século guarda a proporção de 1942, que depois de 1960 é um
teto.

| Aço, Mt (simulado / real) | 1955 | 1985 | 2020 |
|---|---|---|---|
| Mundo, antes | 353 / 270 | 1 023 / 719 | 1 957 / 1 880 |
| Mundo, depois | 281 / 270 | 821 / 719 | 1 603 / 1 880 |
| EUA, antes / depois | 83 / 68 (real 106) | 223 / 177 (80) | 407 / 338 (73) |
| França, antes / depois | 20 / 19 (13) | 60 / 47 (19) | 119 / 90 (12) |
| Grã-Bretanha, antes / depois | 27 / 19 (20) | 74 / 56 (16) | 102 / 81 (7) |

O estaleiro passa de 24 % para menos de 4 % do aço mundial em 2020; em 1925 o aço mundial fica
em 86 contra 90,5 Mt (era 93). O mundo de 2020 fica abaixo do real porque o aço chinês, que
dependeu de política e capital externo, não acontece (sem a China, o real é cerca de 815 Mt). O
carvão cai junto (2025: 0,44 → 0,40 do real), porque a usina nuclear já toma a eletricidade
(`Aberto`).

**O que fica.** Os ricos continuam com 4 a 7 vezes o aço real em 2020: é o volume de construção
(o fundo recebe 28 % do PIB nos EUA; a intensidade de aço da obra, 31 t por milhão de dólares,
está perto do real), em "Aberto". Os 10 % das compras do Estado em navio de guerra (`purchases.ron`),
iguais para todo país e em tempo de paz, são provisórios até as forças de
`data/scenario/1925/forces.ron` e a guerra ditarem a frota (fase 5).

## Gás

Na `main` em d5570d0 o gás mundial fica em 0,36 do real em 2025 (0,46 em 1985). Medido com
`BF_DIAG_ENERGY=2020`: o gás sobra (oferta 1,5 contra demanda 0,9 bilhão de mil m³ por ano, preço a
0,63 da base), porque sai junto do petróleo e quase ninguém o queima fora o aquecimento das casas
(80 a 87 % nos países ricos). Duas causas, uma por PR:

- **Calor de indústria (este PR).** Forno de cimento, caldeira de papel e alto-forno só tinham
  método a carvão; a indústria só usava gás como matéria-prima (química, plásticos, fertilizante) e
  no vidro float. Três métodos da história, ao mesmo calor do carvão que trocam (carvão 7,0 MWh/t,
  gás 10 MWh por mil m³, os poderes caloríficos do placar), para o mercado escolher pelo preço:
  forno seco a gás (com o pré-aquecedor de 1960), vapor a gás no papel (com os gasodutos) e
  redução direta do minério pelo gás, fundido no forno elétrico (tecnologia nova
  `direct_reduction`, Midrex 1969/1971, sem ganho de produtividade próprio para não mexer nas
  somas calibradas da árvore). Em 2020 a indústria passa de 17 % para 34 % do gás usado (papel 13 %,
  cimento 6 %; a redução direta quase não pega) e o consumo mundial sobe cerca de 20 %. A produção
  quase não muda (2025: 0,36 → 0,38 do real; carvão 0,44 → 0,41), porque a oferta ainda passa da
  demanda.
- **Geração elétrica (capital por método, PR próprio).** Em 2020 toda a eletricidade do mundo é
  nuclear (5 924 níveis, nenhum a carvão, óleo ou gás): o método se escolhia por lucro por nível, e o
  nível nuclear faz três vezes a eletricidade de um a gás com o mesmo pessoal, sem pagar o capital a
  mais que custa.
## Capital por método

Um nível custa `CAPITAL_YEARS` (3) anos do valor que adiciona (`monthly::level_capital`), o que
iguala o capital por dólar de todos os métodos: uma usina nuclear, que faz três vezes a eletricidade
de uma a gás com o mesmo pessoal, ganhava a escolha por lucro por nível sem pagar o capital a mais
que custa, e em 2020 toda a eletricidade do mundo era nuclear.

**A regra.** Um prédio pode declarar, em todos os métodos (`capital: Some(x)` em `buildings.ron`), o
capital de um nível relativo a um nível do seu primeiro método (que vale 1); a carga recusa a
declaração parcial. Onde há declaração, a escolha de método pesa o lucro do nível sobre esse capital
(`monthly::method_score`) e a obra nova e a troca de método o pagam (`monthly::method_capital`). Onde
não há, nada muda: com nenhum prédio declarando, a rodada de 101 anos dá o hash final da `main`.

**As usinas (testado e guardado).** Os valores pelo custo de capital por kW (EIA, *Capital Cost and
Performance Characteristic Estimates for Utility Scale Electric Power Generating Technologies*,
fevereiro de 2020, Tabela 2, dólares de 2019 por kW: carvão ultrassupercrítico 3 676, ciclo
combinado 958, nuclear AP1000 6 041, hidrelétrica 5 316) sobre um fator de capacidade típico (0,65;
0,55; 0,90; 0,40), vezes a eletricidade de um nível: carvão 1, óleo 1,08, gás 0,385,
hidrelétrica 2,94, nuclear 3,96. Rodada de 101 anos sobre a `main` em 40ca61c:

| Razão simulado/real | 1955 | 1985 | 2025 |
|---|---|---|---|
| Gás, sem → com | 0,59 → 0,89 | 0,46 → 1,62 | 0,38 → 1,59 |
| Carvão, sem → com | 0,85 → 1,04 | 0,52 → 0,57 | 0,40 → 0,42 |

Parte da capacidade de geração por método (`BF_DIAG_POWER=1`), carvão / gás / hidrelétrica /
nuclear (o óleo fica em zero nas duas):

| Ano | `main` | Com os valores |
|---|---|---|
| 1935 | 73 / 0 / 27 / 0 % | 99 / 0 / 1 / 0 % |
| 1955 | 12 / 0 / 88 / 0 % | 51 / 49 / 0 / 0 % |
| 1965 | 7 / 3 / 64 / 26 % | 38 / 62 / 0 / 0 % |
| 1975 | 2 / 3 / 26 / 70 % | 5 / 95 / 0 / 0 % |
| 1985 | 0 / 1 / 6 / 93 % | 0 / 100 / 0 / 0 % |
| 2005–2025 | 0 / 0 / 0 / 100 % | 0 / 100 / 0 / 0 % |

Sem os valores, a hidrelétrica, que não paga a barragem, toma a geração em 1955 e a nuclear a toma
depois; com eles, carvão e gás convivem de 1955 a 1965 e o gás fica com tudo depois de 1975. A
escolha de método é de canto: cada usina toma o melhor método, o preço do gás não sobe porque os
campos acompanham a demanda, e o modelo não tem o que mantém a mistura real: preço regional do
combustível (o carvão é barato perto da mina e caro longe dela), a idade da usina que já existe e o
despacho (a térmica a gás cobre o pico, o carvão e a nuclear a base). Os valores ficam na branch `feat/method-capital-power` até uma dessas peças
existir; a parcela nuclear que a política travou depois de 1979 não é meta.

## Ordem de trabalho do mercado

1. Preço por estado e por mercado nacional: oferta e demanda fecham semanalmente, com preço sensível à escassez.
2. Estoques, para que a falta e o excesso não se resolvam no mesmo tick.
3. Comércio entre mercados nacionais, com custo de transporte.
4. Moeda, câmbio e inflação, por último (feito na fase 3, fatia 3c: `state.pt-BR.md`).

Cada passo entra com o benchmark de 100 anos verde e com a calibração de 1925 conferida: produção mundial de cada
bem, PIB e consumo por habitante dentro das faixas dos dados históricos. Para conferir 1925:
`bf-bench --diag-1925` (PIB alvo e medido por país, onde o valor adicionado se perde, demanda de
cada bem por origem, produção contra `industry.ron`; `BF_DIAG_YEARS=N` acompanha N anos).

## Microestados: o salário de quem é menor que o grão

**Diagnóstico (main 7d851c1).** Um nível de fábrica emprega de 870 a 990 pessoas. San Marino
tem 5 160 trabalhadores, e o setup pede para ele, em níveis fracionários, comércio 1,22, moradia
0,97, banco 0,88, construção 0,38, ferrovia 0,37 e fazendas de 0,1 cada. Arredondado, sobram um
comércio e uma ferrovia, porque os serviços locais arredondam para cima estado por estado. Em uma
semana as duas rodam no piso de uso (5 %) e seguem com a tripulação inteira: o emprego segue os
níveis, não o uso. Daí o salário de mercado em 1927 de 1,3 por semana em San Marino, 0,3 em
Liechtenstein e 0 em Andorra, contra 127 na Colômbia, lido como trabalho de graça pela folha do
Estado, pela escolha de método e pelo retorno das obras. Nos países grandes o mesmo acontece nos
estados pequenos, mas a média do mercado esconde. O dado do cenário não está errado.

**O piso de salário, desligado** (`game::Switches::small_market_wage_floor`,
`monthly::market_wages`). Um mercado com menos de `MIN_HANDS_FOR_A_WAGE` (10 mil) trabalhadores
em fábricas, uns dez níveis, lê o menor salário entre os mercados com leitura quando o seu é
menor: a gente de lá trabalha do outro lado da fronteira. É inteiro abaixo de 5 mil trabalhadores
e some linearmente até 10 mil, para que o mercado que cruza o limite não veja salto de salário.
Ligado contra a base da main 36b2ef5 em 101 anos (determinismo e save de 1975 conferidos), o PIB
mundial fica no ruído (−0,05 % a +1,4 % até 1985), mas a mudança não fica nos microestados: em
1930 só San Marino muda mais de 5 % no PIB (+9 %), em 1945 já são 12 países (Itália +8 %,
Tchecoslováquia +8 %, Polônia −8 %, Nova Zelândia −9 %), 33 em 1965 e 21 de 81 em 2025 (Polônia
+140 %, Hungria +95 %, Países Baixos −52 %). É o modelo amplificando um empurrão pequeno, a mesma
sensibilidade da faixa de reembaralhamento, mas espalha a mudança além dos mercados pequenos e
dos vizinhos; por isso entra desligado e vai para a fila de equilíbrio com a regra abaixo.
Mônaco (uns 7 mil trabalhadores em fábricas) fica na faixa parcial; com salário acima do piso,
o piso não o alcança.

**Achado principal: instabilidade de trajetória.** Um empurrão que em 1930 só move San Marino
desloca 21 de 81 países em mais de 5 % no PIB de 2025 (Polônia +140 %, Países Baixos −52 %), com
o PIB mundial no ruído. O modelo amplifica diferenças pequenas país a país sem mudar o agregado:
é o primeiro item da fila de equilíbrio, antes de ligar qualquer das duas regras. Hipótese, não
investigada: a realimentação vem de decisões discretas em cascata — nível de fábrica inteiro e
troca de método quando o lucro cruza um limiar (`best_method`) —, reforçadas pelo investimento,
que vai para o maior retorno e faz o vencedor ganhar mais, e pela migração, que segue o salário.

Segundo caso, com dado só (#222, 2026-11-18): a união aduaneira de San Marino com a Itália
(1897) só tira a tarifa entre os dois. Contra a `main` 153632f, a mesma rodada de 101 anos:
- San Marino não se mexe (PIB a 0,3 % da `main` em toda década), e a Itália fica entre −3,4 % e
  +6,1 %;
- o PIB mundial fica na faixa (−1,4σ a +0,5σ) e 0,3 a 2,2 % abaixo da `main`;
- os países longe se separam depois de 1955: a Turquia chega a 1,14× a `main` em 1965, 2,9× em
  2005 e 4,4× em 2025 (PIB por habitante de 2.588 contra 9.305), e as colônias espanholas a
  0,78× em 1965 e 0,30× em 2025;
- ficam fora da faixa a 2σ 4 a 15 de 166 territórios por década no PIB, contra 0 a 4 da própria
  `main`.
Não se conserta agora.

**Regra desligada: o emprego acompanha o uso** (`game::Switches::jobs_follow_use`,
`monthly::LAYOFF_PER_MONTH`). A fábrica contrata de imediato o que o uso pede e demite 15 % do
excesso por mês (meia-vida de uns quatro meses): retenção de mão de obra, com ajuste do emprego
ao produto em alguns meses (Hamermesh, *Labor Demand*, 1993; Fay e Medoff, AER 75(4), 1985). Ligada
contra a main, 16 de 82 países mudam mais de 5 % no PIB em 10 anos (PIB mundial +1,8 %) e 53 de 82
em 51 anos (+4 %), nos dois sentidos: Estônia +74 %, Irlanda +38 %, Finlândia +37 %, Hungria +30 %,
Portugal −36 %, Egito −14 %. O efeito é grande demais para entrar sem estudo do processo: quem
recebe os demitidos (campo, informal, migração), e por que uns ganham e outros perdem. Fica na
fila de equilíbrio. Sozinha ela não conserta os microestados: San Marino sobe para 18 por semana,
mas Andorra e Liechtenstein seguem com o PIB desabando.

**O que fica de fora.** A economia real deles vivia dentro do mercado do vizinho: união aduaneira
de San Marino com a Itália, de Liechtenstein com a Suíça (1924) e de Mônaco com a França, além de
turismo, selos, cassino e remessas. É assunto de blocos e comércio; até lá o PIB deles fica
pequeno.

## Aberto

- **PIB mundial do meio do século acima do real (fila de equilíbrio).** 17,1 trilhões em 1955 contra
  10,9 reais (Maddison) na `main` com o comércio que não cria nem destrói dinheiro (#256); era 16,2
  antes. O fim do século se aproximou (126,8 contra 133,4 em 2022). O crescimento de 1925 a 1955 é
  rápido demais: a guerra e a crise de 1929 ainda não estão no tick (fases 5 e 7), mas o excesso já
  aparecia antes delas.
- **Ensino e saúde de país pobre respondem cedo demais ao gasto (fila de equilíbrio; trava a
  tarifa que conta).** Com `customs_duties` ligada, a receita aduaneira de um país pobre de tarifa
  alta paga escola e saúde décadas antes: no Equador de 1985, alfabetização 85 %, esperança de vida
  75,6 anos e fecundidade 1,87 com 2,2 mil dólares por habitante, contra 67 %, 62 anos e 4,95 na
  base; Colômbia, Venezuela, Guatemala e Equador param de crescer depois de 1985 (Colômbia 24
  contra 40 milhões em 2025), e a população mundial sai da faixa (8,03 contra 8,13 a 8,19 bilhões).
  Para ligar a tarifa (docs/trade.pt-BR.md, seção 13), falta a resposta da alfabetização e da saúde
  ao gasto em país pobre (serviços públicos, `econ/public.rs`); depois, refazer os 101 anos contra a
  faixa com a chave ligada.
- **PIB mundial abaixo do real depois de 2000 (fila de equilíbrio, não meta agora).** O comércio
  criava dinheiro nas margens, de 0,65 a 0,87 % do PIB mundial por ano, porque a partilha
  saturava o `Fixed` (#193, `week::margin_transfers`). O mundo tinha sido ajustado com esse
  dinheiro. Sem ele, o PIB mundial fica 10 % menor em 1965 e 17 % menor em 2025. Contra o real,
  ele melhora até 1985 (1,25 → 1,12 em 1965, 1,19 → 1,01 em 1985) e piora depois (1,04 → 0,87 em
  2005, 0,83 → 0,69 em 2025). A dívida pública sobe de 7 a 10 pontos do PIB, e a população quase
  não muda. Por decisão do dono (2026-11-18, M1), o conserto entrou assim; o PIB baixo do fim do
  século vai para o equilíbrio, depois das fases.
- **Comércio mundial: o volume foi tratado (decisão V1 do dono).** O alcance a 100 % (#228) e o
  comércio bruto (#231, #234) levam o comércio mundial de 2,3 % para 15,0 % do PIB em 1925 (real
  14,8 %; docs/trade.pt-BR.md, seção 14). Fica na fila a distribuição entre países: Estados Unidos
  6,7 % e Reino Unido 14 % contra cerca de 10 % e 40 % reais, e por isso a receita aduaneira dos
  dois fica abaixo da real (seção 13).
- **Eletricidade de 1925 (0,48 da referência).** As plantas nascem no primeiro método de cada
  edifício; a eletrificação de 1925 (motores, linhas de montagem, bondes) teria de entrar no setup
  pela eletricidade de cada país em `industry.ron`.
- **Combustível e petróleo (0,3).** Navios, ferrovias e indústria queimam só carvão; querosene de
  iluminação não existe. O petróleo fica alto demais depois de 1985 (1,7× em 2025).
- **Investimento alto.** O fundo recebe cerca de 21 % do PIB, e a construção fica em 13 % do PIB
  (contra 6 a 8 % na história); por isso as intensidades de aço e cimento da obra calibradas em
  1925 são baixas por unidade, e o aço do século ainda sai 1,7 a 2 vezes o real.
- **Estrutura setorial dos pobres.** Ver `history.pt-BR.md`, "Depois da correção de 1925".
- **Colônias ricas demais.** O estágio colonial aproxima as colônias do real até 1975 e espera a
  descolonização e a alocação de capital: "Colônias: salário e técnica".
