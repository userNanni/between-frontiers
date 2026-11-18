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

- **Receita:** imposto de renda por classe, mais o imposto sobre o consumo quando a lei de
  tributação segura o imposto de renda (`Budget::indirect_rate`, `revenue_indirect`) e o lucro das
  estatais ao Tesouro (`revenue_firms`). Baixa: camponeses, trabalhadores rurais, operários,
  clero e soldados. Média: qualificados, escriturários, engenheiros, profissionais liberais e
  servidores. Alta: gerentes, capitalistas e proprietários de terra.
- **Regra histórica** (enquanto o jogador não fixa as alíquotas): 5 / 9 / 15 % em 1925, subindo em
  linha reta até 12 / 22 / 35 % em 1975 e parados depois. Uma correção fiscal multiplica a regra e
  responde ao custo da dívida: sobe 5 % ao ano com juros acima de 12 % da receita, 10 % com juros
  acima de 1/4 da receita ou caixa negativo (o Tesouro não conseguiu vender títulos), cai 5 % com
  juros abaixo de 4 % da receita, dívida abaixo de 40 % do PIB e caixa acima de 1,5 reserva
  (limites 0,6 e 2,2). Teto de 60 % por classe, ou o da lei de tributação em vigor, se for menor
  (`politics.pt-BR.md`).
- **Despesa:**
  - **Funcionalismo:** servidores, clero e soldados recebem o salário médio do país vezes o peso da
    profissão vezes 0,7. Com o caixa abaixo de 8 semanas de receita, o Tesouro paga com atraso: um
    quarto a menos por semana de receita que falta, até 10 % da folha (`public::pay_share`; o piso é
    paliativo, ver `politics.pt-BR.md`, "Crise fiscal"). A
    mesma parcela vale para a parte do Estado em escolas e hospitais; o resto o usuário paga. É o
    que impede o caixa negativo de crescer sem fim quando a lei de tributação segura o imposto.
  - **Compras** (armas, papel, transporte, remédios; com as forças carregadas, as armas são o
    pedido das formações, até a parte militar da cesta: `war.pt-BR.md`, "Forças em paz"): a regra de orçamento gasta a receita mais o
    déficit que a regra da dívida permite, menos folha, serviços públicos, obras e juros; no mínimo
    60 % da folha. Tudo isso é cortado quando o caixa fica negativo (na proporção do buraco) e
    quando os juros passam de 10 % da receita (ponto a ponto), até 1/4. Mais 10 % por semana do
    que o caixa tiver acima de duas vezes a reserva.
  - **Juros** da dívida em mãos privadas e no exterior (os do banco central voltam ao Tesouro).
  - **Serviços públicos:** escolas e hospitais vendem tudo o que produzem; o Estado paga a parte
    que as leis de saúde e educação dizem (`health_state_share`, `education_state_share`; sem lei,
    tudo) e o usuário paga o resto da poupança. O valor adicionado deles vai inteiro para salário
    (não têm dono nem lucro, e não fecham por prejuízo).
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

- **Alfabetização:** a velocidade anual da fase 1 é multiplicada por 0,3 + 0,9 × cobertura escolar
  (sem escola, 30 % da velocidade; com todas as crianças na escola, 120 %). Sem escola pouca gente
  aprende a ler: a Índia, com um décimo das crianças na escola, foi de 10 % para 17 % de
  alfabetizados entre 1925 e 1950; o Brasil, com um terço, de 37 % para 48 % (UNESCO 1957). Antes
  desta regra eram 0,6 + 0,8 × cobertura, e a Índia chegava a 39 % em 1955.
- **Escola de missão:** nas colônias, a matrícula conta também a escola religiosa, que não passa
  pelo Tesouro (igrejas, subvenção e taxa escolar): 0,4 × `church_privilege` da lei de religião da
  metrópole (a missão segue a bandeira; o privilégio negativo conta zero). Igreja oficial (Reino
  Unido, 0,5) dá 0,20; separação (França, 0,1) dá 0,04. Frankema (2012): matrícula bruta de 19 %
  na África britânica em 1938, 95 % dela em missões, contra 6 % na francesa. O país soberano tem a
  escola religiosa dentro da própria cobertura.
- **Mortalidade:** a cobertura hospitalar sobe o teto de saúde e o alcance do remédio
  (`population.pt-BR.md`, "Demografia").
- **1925:** as escolas começam com cobertura igual à alfabetização do país (5 % a 95 %); os
  hospitais, com renda per capita / US$ 15 mil (2 % a 50 %).

## Orçamento colonial

Metrópole e colônias dividem o mercado e o Tesouro, mas não a obra de serviço público. A doutrina
da colônia que se paga sozinha (Reino Unido; na França, a lei de 13 de abril de 1900) punha escola
e saúde da colônia na conta do imposto colonial: em 1925 a França gastava 524 francos por
habitante, a África Ocidental e Equatorial 25; a educação levava 20 % do gasto francês e 3,6 % do
colonial (Cogneau, Dupraz e Mesplé-Somps 2018). Antes desta regra, a fila preenchia primeiro a
maior lacuna do país, que estava sempre na colônia: em 1975 as colônias britânicas tinham 98 %
de cobertura hospitalar pagos pela metrópole, e a Grã-Bretanha perdia hospital para elas.

- **A bolsa colonial** (`public::colonial_purse`): a parte das colônias na base do imposto de
  renda (renda dos pops × alíquota da classe × arrecadação da lei, por região) é a parte do
  dinheiro de obra de cada categoria que vai para elas; o resto fica na metrópole.
- **O que chega ao nativo:** da parte colonial, sai o que a lei colonial manda para a metrópole
  (`colonial_tribute`) e o que serve ao colonizador (`discrimination` da lei colonial):
  `(1 − tributo) × (1 − discriminação)`. Governo indireto (0,2 e 0,3) deixa 56 %; tutela
  (0,1 e 0,15), 77 %; governo de companhia (0,4 e 0,5), 30 %. O que sobra fica no Tesouro.
- **Que categorias:** toda categoria cuja obra é serviço público, isto é, cujo prédio faz um bem
  público (`public::public_service`: o primeiro produto está em `Catalogue::public_goods`, o bem
  que o Estado compra e paga pelas leis). Hoje são educação e saúde. **Categoria nova entra sem
  código:** se o prédio dela produz um bem da categoria `public`, a obra na colônia sai da bolsa
  colonial; se não (infraestrutura, moradia), sai do dinheiro do país inteiro.
- **Cada bolsa anda sozinha:** quando a colonial acaba, as regiões coloniais esperam o mês
  seguinte e a metrópole continua com a sua, e vice-versa.
- **Fica de fora:** o custeio (o salário de professores e médicos) ainda sai do Tesouro único; o
  fundo de desenvolvimento do pós-guerra (Colonial Development and Welfare Act de 1945, £120
  milhões; FIDES de 1946, 18 % para programas sociais) só existiria como mudança de lei colonial
  (a tutela baixa o tributo e a discriminação). Quando a colônia vira país (`borders.pt-BR.md`),
  ela leva os prédios que tem e passa a ter o Tesouro inteiro: o ponto de leitura é a bolsa, que
  deixa de existir.

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

## Alavancas do orçamento e da dívida (desenho)

Estado: **desenho**, sem código. Decisões do autor de 2026-11-17 (D1 a D9 e D13,
`docs/wiki/o-que-o-jogador-decide.md`). Em cada alavanca, a regra de hoje continua como padrão
("Regra histórica") e o governo pode fixar o valor ("Fixado pelo governo"), como já faz com os
impostos.

### Forma comum

- **Onde se guarda.** Em `GameWorld::levers` (`econ::levers::LeversState`, uma `Levers` por
  país), último campo do mundo. Cada alavanca é um `Option`: `None` é a regra, `Some(x)` é o valor
  fixado, cortado ao piso e ao teto. Cada mercado leva uma cópia não salva (`Market::levers`),
  refeita na carga, para que as funções que leem um mercado só (`goods_budget`, `deficit_room`)
  vejam as alavancas. Guardar no `Market` obrigaria a congelar o layout antigo do mercado em todas
  as versões de economia já gravadas.
- **Hash e save.** Com tudo em `None`, o hash é o do `main`: o estado só entra no hash quando
  alguma alavanca está fixada. O save sobe para a versão 12 (a migração deixa todos na regra).
- **Sem chave de regra para a alavanca.** Com tudo em `None` o hash é o do `main`, então a escolha
  do jogador não entra desligada: o seletor aparece sempre. Entram atrás de chave em
  `data/rules.ron` só as mudanças de regra que valem para todos com a alavanca em `None`: o clero
  pago conforme a lei de religião (`clergy_by_law`) e a capacidade de cobrar pelos servidores
  (`fiscal_capacity`, decisão O1).
- **Um comando por alavanca**, no padrão de `SetTaxRates` e `SetPolicyRate`: `Option`, com `None`
  de volta à regra histórica. Valores em milésimos. Cada um ganha uma linha no
  `player_commands.rs`.

  | | Comando | Valor | Piso e teto (dados) |
  |---|---|---|---|
  | D1 | `SetPayroll { tag, per_mille: Option<[u16; 2]> }` | servidores e clero por mil habitantes; o clero só conta quando a lei de religião o põe na folha | servidores 2–80, clero 0–15 |
  | D2 | `SetPublicPay { tag, per_mille: Option<u16> }` | salário público sobre o privado da profissão (regra 700) | 400–1500 |
  | D3 | `SetStatePurchases { tag, goods: Option<u16>, military: Option<u16> }` | `goods`: fator sobre o orçamento de bens da regra (1000 = a regra); `military`: parcela militar do orçamento de bens (regra: a soma dos pesos `military`, hoje 400) | 250–2000; 0–800 |
  | D4 | `SetTreasury { tag, reserve_weeks: Option<u16>, surplus: Option<SurplusUse> }` | semanas de receita em reserva (regra 13); destino do excesso: `Spend`, `Repay` ou `Keep` | 4–52 |
  | D5 | `SetDeficitCap { tag, per_mille_gdp: Option<u16> }` | teto anual do déficit em milésimos do PIB | 0–100 |
  | D8 | `SetWorksOrder { tag, order: Option<Vec<u32>> }`, `AddWork { tag, state, kind }`, `CancelWork { tag, work: u32 }` | ver D8 | até 200 obras |
  | D9 | `SetIssuance { tag, shares: Option<[u16; 3]> }` | parcela da emissão nova em casa, fora em moeda própria e fora em dólar, somando 1000 | cada uma 0–1000 |
  | D6 | `DeclareDefault { tag, target: DebtTarget, haircut: u16 }` e `SetHonourDebt { tag, on: bool }` | nas Agendas, decisão rara com ponto de restauração | perdão 250, 500 ou 750 |
- **O que a tela lê:** `econ::levers::view(m: &Market) -> LeversView` (o mercado do país), uma linha por
  alavanca com o valor que a regra dá hoje (para "Regra histórica: 1,2 %" mostrar o número), o
  valor fixado quando houver, o piso e o teto que a simulação aceita, e o efeito estimado de uma
  semana do valor fixado sobre o caixa (diferença entre a despesa ou a receita da semana com o
  valor fixado e com a regra, pela mesma conta do tick). O cliente lê uma vez por tick publicado; a
  conta é escalar por país.
  No mesmo módulo, mais três leituras, todas sem estado novo (só contas sobre o mundo):
  - `works(world, cat, country) -> Vec<WorkLine>` (D8): cada obra pública da fila por id, com
    tipo, estado, categoria, progresso, semanas estimadas no ritmo das últimas quatro semanas, o
    gargalo da semana (caixa do Tesouro, falta de cimento ou de aço pela regra de insumos do setor
    de construção, ou preço da construção acima do de referência) e se a posição é fixada ou de
    chegada.
  - `buildable(world, cat, country) -> Vec<(state, Vec<(kind, cost)>)>` (`AddWork`): por estado,
    os tipos públicos que a pesquisa liberou e que a região pode receber (os mesmos filtros de
    `public_works`), com o custo pela regra da obra (dois anos do valor adicionado do nível a
    preço-base). O seletor não oferece obra impossível, e o tick recusa a mesma lista.
  - `default_terms(world, country) -> [DefaultTerms; 4]` (D6): para o perdão de 25, 50 e 75 % da
    dívida externa e para a reestruturação interna, os anos fora do crédito, o prêmio de risco
    depois e o quanto a dívida cai. Os anos e o prêmio vêm da tabela de Cruces e Trebesch (2013)
    em dados, com a fonte; a conta é a mesma que `DeclareDefault` aplica.
- **A IA fica na regra.** Os países sem jogador continuam em `None`. Ensinar a IA a fixar alavancas
  é trabalho à parte, depois do equilíbrio.
- **Gradual como na vida real.** O decreto vale no mês seguinte. O efeito anda na velocidade do
  processo real: contratar e demitir leva meses, um título novo só muda o custo do estoque quando
  vence, e a obra muda de lugar na fila mas não se constrói mais rápido.
- **Tela.** O Orçamento ganha uma linha por alavanca, com "Regra histórica" e "Fixado", o valor da
  regra ao lado do fixado e o efeito estimado. Tudo pelo teclado, como os impostos. D6 fica nas
  Agendas, e D9 em Moeda e dívida.
- **CPU.** Uma conta escalar por país por semana em D1 a D5 e D9. D8 ordena até 200 obras por país
  por mês. Nada mensurável no tick. O PR mede 10 anos contra o `main`.

### D1. Tamanho da folha

- **Regra:** servidores 1,2 % da população e clero 0,5 % (`monthly::STATE_SHARE`). O Estado
  contrata 1/10 da diferença por mês e nunca demite. Os soldados saem daqui: o efetivo de paz segue
  a lei de conscrição (#115, D13).
- **Fixado** (`SetPayroll`): servidores por mil habitantes, de 2 a 80, e o clero pago pelo Tesouro
  quando a lei de religião o põe na folha. O piso é o Estado mínimo de 1925. O teto é
  o emprego público dos nórdicos e do bloco soviético, por volta de 1980, sobre a população (fonte
  a fixar no PR: ILO, *Public sector employment*, e Maddison para a população).
- **Demissão gradual (nova).** Acima da meta, a folha encolhe pela aposentadoria e pelo
  congelamento de vagas, mais a demissão até um teto por mês. O ritmo é calibrado em cortes reais,
  como a Grécia de 2010 a 2015 e o Reino Unido de 2010 a 2015 (números com fonte no PR). Os
  demitidos voltam a trabalhadores urbanos, sem emprego, na região.
- **Clero e lei de religião** (nota do autor). A parte do clero paga pelo Tesouro vem da lei de
  religião em vigor, e não de um número fixo:
  - Teocracia e Igreja oficial: tudo pelo Tesouro.
  - Separação, Estado anticlerical e ateísmo de Estado: nada pelo Tesouro.
  - O resto do clero é pago pelos fiéis, como um gasto das famílias, igual à parte do usuário na
    escola.
  - A curva entre um caso e outro é `church_privilege`, de 100 a 500.
  - Casos para o teste: França depois de 1905 (separação, nada pelo Tesouro), Bélgica (o Estado
    paga), Alemanha (imposto da Igreja cobrado pelo Estado e repassado: conta como dos fiéis).
- **O que o servidor faz: capacidade de cobrar** (decisão do autor de 2026-11-17, O1). Sem efeito,
  cortar a folha seria dinheiro de graça. Abaixo dos servidores que a regra pediria para a renda do
  país, a arrecadação cai na proporção que falta; acima deles, não sobe. É a ideia de capacidade
  fiscal de Besley e Persson (2009, AER 99(4)). Entra como `fiscal_capacity` em `data/rules.ron`,
  desligada, e o PR traz a elasticidade e a conta.

### D2. Salário público

- **Regra:** o salário privado médio vezes o peso da profissão vezes 0,7 (`public::PUBLIC_PAY`).
- **Fixado:** o fator, de 0,4 a 1,5. O piso é o salário público achatado das hiperinflações. O teto
  é o prêmio do setor público em países pobres (fonte no PR: Banco Mundial, *Worldwide Bureaucracy
  Indicators*).
- **Efeito real:**
  - Com o salário abaixo do que a profissão ganharia no privado, a contratação anda mais devagar e
    parte dos servidores sai todo mês para o privado.
  - Acima, a vaga enche mais rápido.
  - O salário entra na folha no mês seguinte.

### D3. Compras do Estado e a parte militar

Forma combinada com o thread da guerra (#115):

- **`public::military_room(m, cat) -> Fixed`:** o teto militar por semana, a única leitura de
  `war`. Na regra, é `goods_budget(m)` × a soma dos pesos `military` da cesta / 1000 (hoje 400), e
  o hash fica o do `main`.
- **`Budget::military_share: Option<Fixed>`:** a parcela militar fixada, de 0 a 0,8. A cesta civil
  fica com `goods_budget − military_room`, repartida pelos pesos civis, que não mudam.
- **`Budget::purchases: Option<Fixed>`:** o fator sobre `goods_budget`, de 0,25 a 2. Abaixo de 1 é
  austeridade escolhida, igual à automática. Acima de 1, o déficit é pago por D5 e D9.
- **A ordem dentro do teto** é da guerra: necessidade das forças, encomenda (D13), geração nova e
  depósito.

- **No código (fatia D3):** `SetStatePurchases { tag, goods, military }`, os dois em milésimos e
  `None` de volta à regra. `goods` multiplica o orçamento de bens da regra (`public::goods_budget`,
  de 250 a 2000); `military` é a parcela militar dele (de 0 a 800). Com a parcela fixada, os bens
  civis da cesta repartem `1000 − parcela` pelos seus pesos e os militares a parcela
  (`public::fixed_basket`); na regra, a cesta segue com os seus pesos e o hash é o do `main`.
  `public::military_room` é o teto militar da semana, e a guerra lê a mesma parcela. A tela lê
  `levers::purchases_view` (o fator e a parcela, com a regra, o fixado, piso, teto e o efeito de uma
  semana; o orçamento de bens e o teto militar da semana). O `save` não muda: os campos já
  estavam em `Levers` desde a fatia 1.

### D4. Excedente e reserva do Tesouro

- **Regra:**
  - Reserva de 13 semanas de receita (`RESERVE_WEEKS`).
  - Acima do dobro da reserva, gasta 1/10 do excesso por semana (`SURPLUS_SPENDING`).
  - Com a dívida acima de 60 % do PIB, paga dívida com esse excesso (`money::REPAY_SHARE`,
    `REPAY_ABOVE`).
- **Fixado:**
  - Reserva, de 4 a 52 semanas.
  - Destino do excesso: gastar, pagar dívida ou guardar. Guardar é o fundo soberano: o caixa
    cresce sem gastar.
  - O ritmo fica em 1/10 por semana, porque é a velocidade da regra e não uma escolha.
- **Efeito real:** a reserva maior adia a obra pública, que só começa acima dela. A reserva menor
  aproxima o atraso de folha, que começa abaixo de 8 semanas (`PAY_RESERVE_WEEKS`, fixo).

### D5. Teto de déficit

- **Regra:** `money::deficit_room` para as compras, e `borrowing_room` para as obras. As duas caem
  a nada com a dívida no limite ou com o juro de 15 %.
- **Fixado:** um teto anual em % do PIB, de 0 a 10 %, no lugar do alvo da regra. Zero é orçamento
  equilibrado. O freio da dívida e do juro continua: o mercado não compra mais porque o governo
  quer, e o prêmio sobe pelas regras de hoje.
- **Efeito real:** o teto alto não vira dinheiro se ninguém compra o título. O caixa fica negativo,
  entram a austeridade e a correção fiscal, e com dívida fora vem o calote. É a lição do caso
  argentino que as regras já têm.

### D6. Calote e reestruturação

Decisão rara, nas Agendas, com ponto de restauração antes (motivo "dívida").

- **Regra:** o calote automático de hoje, quando o serviço em moeda estrangeira passa de 30 % da
  receita ou o caixa não fecha, e a reestruturação com juros acima de metade da receita. Continua
  igual para quem não decide.
- **Declarar calote** (`DeclareDefault { tag, target, haircut }`): a qualquer momento, com o perdão de 25,
  50 ou 75 % da dívida externa. Quanto maior o perdão, mais anos fora do crédito e mais prêmio
  depois. Cruces e Trebesch (2013, AEJ: Macro 5(3)) medem essa relação; o PR traz a tabela.
- **Reestruturar a dívida interna:** mesmo comando, com o alvo "interna". Corta o estoque em casa,
  e a poupança das famílias que tinha título perde junto.
- **Honrar a todo custo** (postura, `SetHonourDebt`, guardada em `Money::honour: bool`): o gatilho automático vira austeridade
  forçada, até a folha cair ao piso. É a Romênia de 1981 a 1989, que pagou a dívida externa
  cortando o consumo. Existe (decisão do autor de 2026-11-17, O3): é uma postura nas Agendas.

### D7. Câmbio

Fica para a fase 4, com o padrão-ouro e o regime internacional (`money::exchange_target`).

### D8. Fila de obras públicas

Nota do autor: "uma lista de prioridades que o jogador define e em que pode ordenar tudo".

- **Hoje:** as obras públicas e as privadas dividem `Market::queue`. Os pontos vão às mais velhas
  primeiro, e a regra escolhe cada obra pública (`public_works`, até 200 por mês).
- **Id estável:** cada `Project` ganha `id: u32`, de um contador por mercado que só cresce (vai no
  save; a obra antiga recebe id na migração, pela ordem da fila). A tela e os comandos falam pelo
  id, nunca pelo índice.
- **Fixado** (`SetWorksOrder { tag, order: Some(ids) }`): a ordem das obras públicas da fila. A
  tela move uma obra e manda a ordem inteira, o que torna o comando idempotente e fácil de
  repetir num replay. Ids que já não estão na fila são ignorados; obras públicas que não estão na
  lista ficam depois dela, na ordem de chegada. `None` volta à ordem de chegada.
- **Obra nova e cancelamento:** `AddWork { tag, state, kind }` põe uma obra pública escolhida pelo
  jogador no fim da sua ordem (o custo é o da regra da obra; a categoria do tipo diz de que parte do
  orçamento ela sai). `CancelWork { tag, work }` tira a obra; os pontos pagos se perdem.
- **Como convive com `SetPublicPriority`:** a prioridade por categoria continua decidindo quanto
  do dinheiro acima da reserva vai a cada categoria e, portanto, que obras novas a regra propõe
  todo mês. A ordem fixada só decide quem recebe primeiro os pontos de construção da semana entre
  as obras públicas. A obra que a regra acrescenta entra no fim da ordem fixada.
- **"Só a minha lista":** é a prioridade toda em zero (`SetPublicPriority`), que já suspende obra
  nova da regra; o jogador continua acrescentando com `AddWork`.
- **Gradual:** reordenar muda quem recebe os pontos da semana. A obra continua com o teto de 1/26
  por semana, e os pontos do país são os que a construção civil produz.
- **A obra privada fica fora da lista** (decisão do autor de 2026-11-17, O2: "o governo não tem
  acesso a essas decisões"). Ela segue na ordem de chegada; o jogador ordena só as obras públicas.

- **No código (fatia D8):** sem mudar o layout do mercado, a obra é identificada pelo que é,
  `econ::levers::WorkKey` (estado, prédio, método e custo em pontos); duas obras com a mesma chave
  são a mesma obra duas vezes e qualquer uma vale pela outra. A ordem fixada e os pedidos ficam em
  `GameWorld::levers` (versão 19 do save). Toda semana, antes da entrega dos pontos, as obras
  públicas vão para a ordem fixada nas posições que as públicas já ocupam
  (`levers::order_queue`); as privadas não mudam de lugar. `AddWork` só registra o pedido: o
  sistema de investimento, no mês seguinte, confere com `public::buildable_in` (os prédios que o
  Estado constrói, liberados pela pesquisa e possíveis no estado) e enfileira no custo da regra; o
  pedido impossível é descartado. `CancelWork` tira a obra na hora. A tela lê
  `levers::works` (por obra: chave, categoria, progresso, semanas no ritmo da semana passada,
  gargalo e se a posição é fixada) e `levers::buildable` (por estado, os prédios com o custo de
  um nível).

### D9. Dívida interna ou externa

Nota do autor: "o mercado decide quem compra; o jogador escolhe onde emite".

- **Regra:** a ordem de hoje: a poupança de casa, o banco central subordinado, os estrangeiros e o
  banco central de novo.
- **Fixado** (`SetIssuance { tag, shares: Option<[u16; 3]> }`): a parcela de cada emissão nova em casa,
  fora em moeda própria e fora em dólar.
  - Cada comprador segue limitado pelo quanto absorve por semana, como hoje.
  - Fora em moeda própria, o comprador só leva `1 − fx_share`: o "pecado original" continua, e o
    país pobre que pede tudo fora em moeda própria não vende.
  - O que não vende numa praça não passa sozinho para outra: o caixa cai e a austeridade entra.
- **Gradual:** a escolha muda só a emissão nova. O estoque troca de mãos quando vence, no ritmo de
  cinco anos que o custo já usa.

### D13. Construir tropas de propósito

É do thread da guerra (#115, A2). Daqui sai só `military_room` (D3). A encomenda come o espaço que
hoje vai ao depósito, e o efetivo de paz continua pela lei.

### Testes, por alavanca

- **Regra:** com todas as alavancas em `None`, 10 anos dão o mesmo hash do `main`.
- **Efeito:** fixar o valor move a quantidade na direção e no ritmo do desenho:
  - D1 sobe a folha 1/10 da diferença por mês e desce no teto de demissão.
  - D3 muda o teto militar e deixa a cesta civil nos seus pesos.
  - D5 em zero zera o déficit das compras.
  - D9 com tudo fora num país pobre não vende.
- **Limites:** valores fora do piso e do teto são cortados.
- **Save:** um save gravado pelo `main` carrega com o seu hash. Um save com alavanca fixada
  recarrega com o mesmo hash.
- **Comandos:** o `player_commands.rs` passa com as linhas novas.
- **Contrafactual na rodada de 100 anos:** um país grande com cada alavanca fixada ao contrário da
  regra (folha mínima, compras no teto, déficit zero, tudo emitido fora). O número que mexe tem de
  ser o esperado (receita, dívida, prêmio, calote), sem quebrar o resto do mundo.

### Fatias propostas

Estado: a fatia 1 (D1, D2, D4, D5) está no código, com os comandos `SetPayroll`, `SetPublicPay`,
`SetTreasury` e `SetDeficitCap`, sem tela (a tela é do thread de interface). Pisos e tetos estão
em `econ::levers` como valores de desenho, e a saída gradual da folha é um quadragésimo do excesso
por mês, a calibrar. A `fiscal_capacity` (O1) está no código, desligada em `data/rules.ron`: ligada,
cada região cobra a parcela dos servidores que a regra pede (`econ::levers::collection`), no imposto
de renda e no de consumo, e o `view` diz a fração perdida (`uncollected`) e o efeito de uma semana
no caixa (`week_effect`).

1. D1, D2, D4 e D5: escalares do orçamento, comando e tela no Orçamento.
2. D3 com `military_room`, junto com a guerra, e D9 em Moeda e dívida.
3. D8, a fila com a lista do jogador.
4. D6 nas Agendas, com ponto de restauração.

Cada fatia é um PR com determinismo, CPU, save e o contrafactual.

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
