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

## Ordem de trabalho do mercado

1. Preço por estado e por mercado nacional: oferta e demanda fecham semanalmente, com preço sensível à escassez.
2. Estoques, para que a falta e o excesso não se resolvam no mesmo tick.
3. Comércio entre mercados nacionais, com custo de transporte.
4. Moeda, câmbio e inflação, por último (feito na fase 3, fatia 3c: `state.pt-BR.md`).

Cada passo entra com o benchmark de 100 anos verde e com a calibração de 1925 conferida: produção mundial de cada
bem, PIB e consumo por habitante dentro das faixas dos dados históricos. Para conferir 1925:
`bf-bench --diag-1925` (PIB alvo e medido por país, onde o valor adicionado se perde, demanda de
cada bem por origem, produção contra `industry.ron`; `BF_DIAG_YEARS=N` acompanha N anos).

## Aberto

- **Eletricidade de 1925 (0,48 da referência).** As plantas nascem no primeiro método de cada
  edifício; a eletrificação de 1925 (motores, linhas de montagem, bondes) teria de entrar no setup
  pela eletricidade de cada país em `industry.ron`.
- **Combustível e petróleo (0,3).** Navios, ferrovias e indústria queimam só carvão; querosene de
  iluminação não existe. O petróleo fica alto demais depois de 1985 (1,7× em 2025).
- **Investimento alto.** O fundo recebe cerca de 21 % do PIB, e a construção fica em 13 % do PIB
  (contra 6 a 8 % na história); por isso as intensidades de aço e cimento da obra calibradas em
  1925 são baixas por unidade, e o aço do século ainda sai 1,7 a 2 vezes o real.
- **Estrutura setorial dos pobres.** Ver `history.pt-BR.md`, "Depois da correção de 1925".
