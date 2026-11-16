# Estado: construção, orçamento, obra pública, moeda e dívida

Estado: fase 3, fatias 3a a 3c. O código está em `crates/bf-sim/src/econ/construction.rs` (fila de
obras), `crates/bf-sim/src/econ/public.rs` (orçamento e obra pública),
`crates/bf-sim/src/econ/migration.rs` (migração interna) e `crates/bf-sim/src/econ/money.rs` (moeda,
banco central e dívida); o estado salvo fica em `Market::queue`, `Market::budget`, `Market::laws` e
`Market::money` (`econ/state.rs`).

## Setor de construção e fila de obras

- **Construção civil** (`construction_sector`, grupo infraestrutura) é um edifício como os outros:
  operários, pedreiros, engenheiros e capatazes transformam madeira, cimento, aço e ferramentas em
  pontos de construção (bem `construction`, não comerciável). Dois métodos: manual (barato onde o
  trabalho é barato) e mecanizado (mais cimento, aço, máquinas e combustível, menos gente).
- **Todo nível novo vira obra.** O investimento privado e o Estado põem a obra na fila do país; o
  nível só entra em funcionamento quando recebe todos os pontos.
- **Custo da obra:** dois anos do valor adicionado do nível a preço-base (a regra da fase 1). A
  parte que é o prédio (serviços, cimento, madeira e aço da cesta de cada grupo) vira pontos ao
  preço-base da construção; máquinas e ferramentas continuam compradas direto.
- **Ritmo:** uma obra recebe no máximo 1/26 dos seus pontos por semana (meio ano no mínimo). Os
  pontos da semana são comprados no mercado nacional e distribuídos na ordem da fila.
- **Falta de material atrasa.** Sem cimento ou aço, a construtora produz menos pontos (a regra de
  insumos da fase 1: a perda é a parcela do insumo no valor do produto), o preço da construção sobe
  e a fila anda mais devagar. O preço alto chama investimento na própria construção civil.
- **Manutenção:** a parcela semanal do fundo de investimento que antes comprava "serviços" agora
  compra 78 % em pontos de construção.
- O investidor conta os níveis em obra como já construídos (ociosos), para não repetir a mesma obra
  todo mês, e soma o que falta pagar da fila ao que já comprometeu.

## Orçamento do Estado

O caixa é `Market::treasury`. Toda semana o Tesouro vende títulos para voltar o caixa à reserva
(ver "Moeda, banco central e dívida"); só fica negativo quando ninguém compra. `Budget::debt` é a
dívida total; `Budget::interest`, o juro da semana.

- **Receita:** imposto de renda por classe. Baixa: camponeses, trabalhadores rurais, operários,
  clero e soldados. Média: qualificados, escriturários, engenheiros, profissionais liberais e
  servidores. Alta: gerentes, capitalistas e proprietários de terra.
- **Regra histórica** (enquanto o jogador não fixa as alíquotas): 5 / 9 / 15 % em 1925, subindo em
  linha reta até 12 / 22 / 35 % em 1975 e parados depois. Uma correção fiscal multiplica a regra e
  responde ao custo da dívida: sobe 5 % ao ano com juros acima de 12 % da receita, 10 % com juros
  acima de 1/4 da receita ou caixa negativo (o Tesouro não conseguiu vender títulos), cai 5 % com
  juros abaixo de 4 % da receita, dívida abaixo de 40 % do PIB e caixa acima de 1,5 reserva
  (limites 0,6 e 2,2). Teto de 60 % por classe.
- **Despesa:**
  - **Funcionalismo:** servidores, clero e soldados recebem o salário médio do país vezes o peso da
    profissão vezes 0,7. É obrigação: é paga mesmo com o tesouro no vermelho.
  - **Compras** (armas, papel, transporte, remédios): a regra de orçamento gasta a receita mais o
    déficit que a regra da dívida permite, menos folha, serviços públicos, obras e juros; no mínimo
    60 % da folha. Tudo isso é cortado quando o caixa fica negativo (na proporção do buraco) e
    quando os juros passam de 10 % da receita (ponto a ponto), até 1/4. Mais 10 % por semana do
    que o caixa tiver acima de duas vezes a reserva.
  - **Juros** da dívida em mãos privadas e no exterior (os do banco central voltam ao Tesouro).
  - **Serviços públicos:** o Estado compra tudo o que escolas e hospitais produzem; o valor
    adicionado deles vai inteiro para salário (não têm dono nem lucro, e não fecham por prejuízo).
  - **Obra pública:** paga semana a semana, conforme os pontos chegam.
- **Reserva:** 13 semanas de receita. O que passa da reserva (descontado o que falta pagar das
  obras públicas já na fila), mais o que a regra da dívida deixa tomar para obra (2 % do PIB ao
  ano com dívida zero, caindo a nada com dívida igual ao PIB ou juro de 15 %), vira obra nova.
- **Sem emissão invisível:** o substituto do banco central da fase 1 (dinheiro criado no Tesouro
  quando havia fábrica ociosa) acabou. A diferença entre despesa e receita vira título.

## Obra pública

Todo mês, o dinheiro acima da reserva é dividido entre quatro categorias pelos pesos do país
(padrão 3 / 2 / 2 / 3): infraestrutura, educação, saúde e moradia urbana (esta entra com a
migração interna). Categoria sem obra a fazer passa a sua parte às outras. Até 200 obras por país
por mês.

- **Educação:** escola (`school`), um nível com 800 professores para 24 mil alunos a preço-base. A
  demanda é 20 % da população (idade escolar); o Estado preenche primeiro as regiões com o maior
  número de alunos sem vaga.
- **Saúde:** hospital (`hospital`), 300 médicos e enfermeiros e 650 outros funcionários. A
  necessidade é US$ 200 de atendimento por pessoa por ano; mesma regra de lacuna.
- **Infraestrutura:** ferrovia, transporte rodoviário e usina elétrica. Transporte e eletricidade
  são bens do estado (`infrastructure.pt-BR.md`): a obra vai para os estados com mais demanda sem
  atender. O que já corre pela rede nacional segue a regra antiga: só com o produto mais de 10 %
  acima do preço-base, nas regiões com mais gente por nível existente. Pronta, vira um nível comum (a estatal com lucro para o tesouro fica
  para a fase de política).
- **Escola e hospital não existiam no catálogo**: são os edifícios mínimos desta fatia, com a
  categoria de bem `public` (ensino, atendimento hospitalar).

### Efeitos

- **Alfabetização:** a velocidade anual da fase 1 é multiplicada por 0,6 + 0,8 × cobertura escolar
  (metade das crianças na escola mantém a velocidade antiga).
- **Mortalidade:** multiplicada por 1,05 − 0,25 × cobertura hospitalar (um quinto da necessidade
  atendida mantém a taxa antiga).
- **1925:** as escolas começam com cobertura igual à alfabetização do país (5 % a 95 %); os
  hospitais, com renda per capita / US$ 15 mil (2 % a 50 %).

## Migração interna e moradia urbana

Urbano, como na fase 1, é quem tem profissão de cidade (todas menos camponês, trabalhador rural e
proprietário de terra). A migração muda a profissão: o camponês ocioso vira operário, empregado ou
não, e passa a morar na cidade.

- **Moradia urbana** (`urban_housing`, grupo serviços): um nível abriga 5 mil pessoas a preço-base
  (vezes a produtividade da região, como todo edifício) com 30 empregados. As profissões urbanas têm
  a necessidade `urban_housing` (aluguel, 1 lugar por pessoa por ano, nível 1); as rurais ficam com
  `shelter`, que antes valia para todos. O campo `urban` das necessidades diz quem as tem.
- **Quem constrói:** o investidor privado, quando o aluguel dá retorno (é um edifício comum), e o
  Estado, pela categoria moradia da obra pública, onde há mais gente na cidade do que lugares.
- **Limite:** a cidade aceita até 1,25 pessoa por lugar (cortiço, favela). Passado isso, ninguém
  chega e os operários sem emprego voltam para a terra (um décimo por mês).
- **Quando se muda:** todo mês, comparando a renda por pessoa do operário da região (salário dividido
  por todos os operários e dependentes, então o desemprego já desconta: regra de Harris–Todaro) com a
  do camponês e do trabalhador rural, as duas somadas à comida que os sem emprego plantam, ao
  preço-base. Abaixo de 1,5 vez nada acontece; o fluxo cresce até dobrar esse prêmio.
- **Quanto:** 0,03 % por mês dos camponeses ociosos para as cidades do próprio estado, e 0,01 % para
  as três cidades do país que pagam mais e têm lugar. O operário sem emprego fica na cidade enquanto
  ela paga melhor que o campo e tem lugar; senão volta à terra, como na fase 1.
- **Lei de migração** por país (`Market::laws.migration`): aberta (padrão) ou restrita
  (passaporte interno: propiska soviética, hukou chinês, leis de passe). Restrita deixa passar 1/4
  do fluxo dentro do estado e 1/10 entre estados.
- O dinheiro acompanha quem se muda na proporção do tamanho do grupo (regra da fase 1 para troca de
  profissão), o que cria ou destrói um pouco de poupança entre regiões.

## Moeda, banco central e dívida

Fatia 3c. Os livros da simulação continuam em unidades reais (dólar a preço-base de 1925), que
também são a moeda de referência mundial; o comércio mundial em pool já é feito nela. Cada país
(cada mercado; colônias usam a moeda da metrópole) põe por cima um nível de preços, uma inflação e
um câmbio. Esses números não mudam os preços do mercado, que seguem ancorados no preço-base; mudam
o valor real da dívida, os juros e o crédito.

- **Inflação** (% ao ano, `Money::inflation`) fecha 1/13 da distância ao alvo por semana. O alvo é
  a expectativa (ancorada na meta de 2 % com banco central autônomo; no passado, no mundo e só um
  pouco na meta com banco subordinado), mais 0,3 ponto por ponto de utilização das fábricas acima
  da tendência de dois anos, mais 0,2 por ponto do índice de preços do mercado acima da tendência,
  mais 1,5 ponto por ponto do PIB de moeda criada fora da regra (o banco subordinado comprando o que
  ninguém compra, ou a taxa fixada abaixo da regra). Piso de −10 %. O nível de preços (1925 = 100)
  corta três zeros acima de 100 milhões (reforma monetária).
- **Câmbio** (moeda local por unidade de referência, 1925 = 100) anda 1/26 por semana para a
  paridade de poder de compra, enfraquecida 3 % por ponto de prêmio de risco. O câmbio real fica
  entre 1/4 e 4 vezes a paridade. Gancho da fase 4: `money::exchange_target` recebe o regime
  internacional (padrão-ouro, Bretton Woods, flutuação, euro, moeda de reserva).
- **Banco central:** taxa de Taylor, 2 % + inflação + 0,5 × (inflação − 2 %) + 0,5 × hiato de
  utilização, com piso zero; o jogador pode fixar a taxa. Cria moeda no ritmo de 1,5 % do PIB ao
  ano (moeda ampla de uma economia que cresce: sem multiplicador bancário, o crédito dos bancos
  está dentro do banco central), mais 1 % do PIB ao ano por ponto da taxa abaixo da neutra, até 10
  (abaixo de zero continua: afrouxamento quantitativo); acima da neutra, retira. Essa moeda vai primeiro para os títulos novos do
  Tesouro (só banco subordinado) e o resto vira crédito aos bancos (o fundo de investimento). Para
  retirar moeda, chama o crédito e depois vende títulos.
- **Autonomia:** regra histórica (`CentralBank::Historical`) subordinada até 1990, ou até um ano de
  inflação acima de 200 % (plano de estabilização), autônoma depois. O banco autônomo não compra
  título novo do Tesouro (a proibição de financiamento direto). O subordinado nunca retira moeda e
  compra o que sobrar.
- **Quem compra os títulos**, na ordem: (1) poupança doméstica, 0,5 % por semana da poupança das
  famílias acima de 26 semanas de renda e 0,5 % do fundo de investimento, vezes a profundidade do
  mercado (quadrado da renda per capita contra o dobro da mundial, mínimo 5 %); (2) banco central
  subordinado, dentro da regra; (3) estrangeiros, 0,5 % por semana da poupança ociosa dos outros
  países, repartida entre os que pedem; (4) banco subordinado, o resto. Sem comprador o caixa
  fica negativo e entram a austeridade e a correção fiscal.
- **Moeda estrangeira:** a parte dos estrangeiros em dólar sobe com a pobreza, a inflação e o
  histórico de calote ("pecado original"), de 10 % (país rico) a 95 %.
- **Juro:** taxa do título local = metade da taxa do banco central + metade de (taxa mundial +
  inflação local − inflação mundial) + prêmio, piso 0,1 %. Em moeda estrangeira, taxa mundial +
  prêmio. A taxa mundial é 2 % + a inflação mundial (média pelo PIB, cada país até 50 %). Prêmio, em
  pontos (teto 25): 4 por unidade de dívida/PIB acima de 0,6 (pesando mais a parte no exterior),
  0,2 por ponto de inflação acima de 3 %, 6 por unidade de dívida em moeda estrangeira/PIB, 4 por
  calote no histórico; menos 1 ponto por PIB de poupança ociosa em casa, até 3. O custo do estoque
  segue a taxa de mercado em cinco anos.
- **Inflação corrói a dívida em moeda própria;** a em moeda estrangeira só cai com a inflação mundial
  e cresce com o câmbio real.
- **Calote:** quando o serviço em moeda estrangeira (juro + 10 % do principal) passa de 30 % da
  receita, ou o Tesouro deve 1/4 da receita anual que não conseguiu tomar enquanto deve ao exterior,
  metade da dívida externa é perdoada, o país fica três anos sem crédito estrangeiro e o histórico
  sobe 1 (perde 10 % ao ano: o prêmio fica alto por anos). Quando os juros passam de metade da
  receita, o Estado reestrutura 40 % de toda a dívida fora do banco central. No máximo um calote
  por ano.
- **O caso japonês** sai das regras: dívida alta em moeda própria, comprada em casa, com poupança
  sobrando, tem prêmio negativo e juro no piso, e não há calote porque não há serviço em moeda
  estrangeira; dois quintos do PIB em dólar com pouca poupança passam do limite e dão calote (teste
  `the_japanese_case_and_the_dollar_debt`).
- **Investimento privado:** o retorno mínimo de uma obra é o maior entre 5 % e o juro real do
  título + 2 pontos (até 25 %).
- **Poupança ociosa não fica parada:** além do que compra títulos, os bancos emprestam 2 % por
  semana dela ao fundo de investimento. Sem isso a poupança das famílias crescia sem fim (874 % do
  PIB em 2025 na main) e a demanda vazava.
- **Dívida de 1925:** todos começam com 25 % do PIB em casa e 5 % no banco central (as dívidas de
  guerra por país ficam para os dados do cenário).

## Comandos do jogador

- `SetTaxRates { tag, rates }`: alíquotas das três classes em milésimos (teto 600); `None` devolve o
  país à regra histórica.
- `SetPublicPriority { tag, weights }`: pesos de infraestrutura, educação, saúde e moradia, de 0 a
  10. Tudo zero suspende obra pública nova.
- `SetMigrationLaw { tag, law }`: `Open` ou `Restricted`.
- `SetPolicyRate { tag, rate }`: taxa do banco central em milésimos ao ano (50 é 5 %, teto 100 %);
  `None` devolve à regra de Taylor.
- `SetCentralBank { tag, bank }`: `Historical`, `Autonomous` ou `Subordinate`.

Saves: os campos novos têm `#[serde(default)]` e entram no hash. Como nas fases anteriores, o
snapshot é postcard, que não é autodescritivo: save de uma versão anterior não carrega.

## Calibração

Benchmark de 100 anos (`cargo run --release -p bf-bench -- --game --years 100 --check-determinism`),
mesma semente. "Main" é a main com transporte e eletricidade regionais (#36), ainda com emissão no
Tesouro; "3c" é com moeda, banco central e dívida.

| | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|
| População (bilhões), main | 2,17 | 3,46 | 5,09 | 6,74 |
| População, 3c | 2,16 | 3,29 | 4,94 | 6,71 |
| PIB (US$ tri, preços de 1925), main | 6,5 | 21,2 | 64,0 | 92,8 |
| PIB, 3c | 5,9 | 18,4 | 57,0 | 97,6 |
| Urbanização, main | 0,45 | 0,52 | 0,57 | 0,61 |
| Urbanização, 3c | 0,45 | 0,52 | 0,56 | 0,60 |
| Necessidades básicas, main | 0,65 | 0,67 | 0,71 | 0,75 |
| Necessidades básicas, 3c | 0,66 | 0,71 | 0,75 | 0,77 |
| Impostos (% do PIB), main | 6,9 | 11,9 | 12,9 | 12,2 |
| Impostos, 3c | 11,9 | 20,5 | 22,7 | 21,9 |
| Despesa do Estado (% do PIB; na 3c com juros), main | 16,0 | 24,4 | 30,3 | 31,0 |
| Despesa, 3c | 14,3 | 22,9 | 24,7 | 23,8 |
| Inflação média da década (média pelo PIB, igual à mediana), 3c | 0,1 | 1,0 | 1,8 | 2,2 |
| Dívida pública (% do PIB), 3c | 47 | 48 | 48 | 52 |
| Parcela do banco central na dívida, 3c | 23 % | 23 % | 16 % | 4 % |
| Juros (% do PIB), 3c | 0,9 | 0,9 | 1,5 | 2,1 |
| Crédito dos bancos centrais aos bancos (% do PIB), 3c | 20 | 31 | 31 | 34 |

Na main a diferença entre despesa e receita (9 a 19 % do PIB ao ano) é dinheiro emitido no Tesouro,
que não aparece como dívida.

Por país, 3c (inflação média da década; dívida/PIB; parte do banco central; parte em dólar):

| | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|
| USA | 0,4 %; 45 %; 10 %; 0 | 0,7 %; 50 %; 2 %; 0 | 1,9 %; 59 %; 0; 0 | 2,1 %; 60 %; 0; 0 |
| GBR | 0,4 %; 46 %; 12 %; 0 | 1,4 %; 44 %; 3 %; 0 | 1,7 %; 50 %; 1 %; 0 | 2,2 %; 53 %; 0; 0 |
| JAP | −0,3 %; 47 %; 23 %; 0 | 0,9 %; 55 %; 5 %; 0 | 1,6 %; 54 %; 1 %; 0 | 2,2 %; 56 %; 0; 0 |
| BRA | 0,6 %; 41 %; 42 %; 6 % | 0,7 %; 47 %; 64 %; 1 % | 1,6 %; 45 %; 34 %; 5 % | 2,1 %; 56 %; 8 %; 2 % |
| ARG | −0,2 %; 58 %; 12 %; 0 | 1,3 %; 54 %; 4 %; 0 | 1,8 %; 36 %; 2 %; 0 | 2,0 %; 42 %; 1 %; 0 |
| GER | 0,1 %; 50 %; 10 %; 0 | 0,9 %; 50 %; 2 %; 0 | 1,6 %; 57 %; 0; 0 | 2,2 %; 59 %; 0; 0 |

3c: determinístico entre contagens de threads e depois de salvar e recarregar em 1975. Tick mais
lento 34,1 ms medido com a máquina livre, antes do rebase sobre o #36 (main 30,1 ms na mesma
máquina); a rodada sobre o #36 foi medida com a máquina carregada por outros processos (carga 17 a
89) e o tempo dela não vale.

- **O buraco virou dívida.** Sem emissão no Tesouro, a correção fiscal sobe a regra de impostos (de
  12 % para 22 % do PIB) e o Estado gasta a receita mais o déficit da regra; a dívida fica em torno
  de metade do PIB, com juros de 1 a 2 % do PIB. A despesa cai de 31 % para 24 % do PIB.
- **PIB:** abaixo da main até os anos 1990 (menos gasto público) e 5 % acima em 2025 (o crédito dos
  bancos centrais e a poupança emprestada ao investimento). Nenhum colapso.
- **Inflação** de 1,5 % em média no século, um dígito o tempo todo; nenhum país passou de 25 % ao
  ano. Os episódios altos existem nas regras (banco subordinado sem comprador, taxa fixada abaixo
  da regra), mas a IA não os produz: os déficits cabem na poupança doméstica. Faltam os choques
  (guerra, petróleo) e o populismo fiscal das fases seguintes.
- **Calotes:** nenhum entre os países grandes. Os microestados (San Marino, Andorra, Liechtenstein,
  Mônaco; PIB abaixo de US$ 1 bilhão e fábricas quase paradas) reestruturam quase todo ano; o bench
  conta à parte (112 até 2025). Os 5 que aparecem fora deles em 2025 são de um país pequeno que
  passou de US$ 1 bilhão de PIB e saiu da conta dos microestados.
- **Brasil** se financia no banco central antes de 1990 (mercado raso) e no exterior em parte em
  dólar; o Japão e os ricos, em casa. O caso japonês extremo e o calote em dólar aparecem nas
  regras (teste), não na rodada.
- A fila começa longa (a construção civil de 1925 foi dimensionada só para a manutenção) e o preço
  da construção fica acima da base nos primeiros anos.

## Aberto

- Dívidas de 1925 por país (a britânica e a francesa da guerra) no cenário; hoje todos começam com
  30 % do PIB.
- Choques que gerem inflação alta e calote na rodada da IA: guerra (fase 5), petróleo e regime
  monetário internacional com FMI (fase 4), populismo fiscal (política, fase 3).
- O nível de preços não volta ao mercado: os preços seguem ancorados no preço-base.
- Microestados com economia quase parada reestruturam a dívida todo ano.
- Estatal com lucro para o tesouro; indústria estatal como categoria de obra.
- A IA de país escolhe alíquota e prioridade pela regra; o jogador só por comando, sem tela ainda.
- Lei de migração histórica no cenário (propiska de 1932, hukou de 1958): hoje todos começam abertos.
- Moradia quase nunca limita: o investidor privado constrói o que o aluguel paga. O freio real é a
  renda (Harris–Todaro). Falta o custo de vida mais alto na cidade (comida comprada).
