# Estado: construção, orçamento e obra pública

Estado: primeira fatia da fase 3. O código está em `crates/bf-sim/src/econ/construction.rs` (fila de
obras), `crates/bf-sim/src/econ/public.rs` (orçamento e obra pública) e
`crates/bf-sim/src/econ/migration.rs` (migração interna); o estado salvo fica em `Market::queue`,
`Market::budget` e `Market::laws` (`econ/state.rs`). Moeda, dívida e banco central ficam para a
fatia seguinte.

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

O saldo é `Market::treasury` e pode ficar negativo: o déficit só acumula ali. `Budget::debt` existe e
fica em zero até a fatia de moeda, que transforma o saldo negativo em títulos com juro.

- **Receita:** imposto de renda por classe. Baixa: camponeses, trabalhadores rurais, operários,
  clero e soldados. Média: qualificados, escriturários, engenheiros, profissionais liberais e
  servidores. Alta: gerentes, capitalistas e proprietários de terra.
- **Regra histórica** (enquanto o jogador não fixa as alíquotas): 5 / 9 / 15 % em 1925, subindo em
  linha reta até 12 / 22 / 35 % em 1975 e parados depois. Uma correção fiscal multiplica a regra:
  sobe 5 % ao ano depois de um ano com o tesouro abaixo de −1/4 da receita anual, cai 5 % com o
  tesouro acima de 1/2 da receita anual (limites 0,6 e 1,6). Teto de 60 % por classe.
- **Despesa:**
  - **Funcionalismo:** servidores, clero e soldados recebem o salário médio do país vezes o peso da
    profissão vezes 0,7. É obrigação: é paga mesmo com o tesouro no vermelho.
  - **Compras** (armas, papel, transporte, remédios): 60 % da folha, cortadas na proporção do
    déficit (até 1/4) quando o tesouro está negativo, mais 10 % por semana do que o tesouro tiver
    acima de duas vezes a reserva.
  - **Serviços públicos:** o Estado compra tudo o que escolas e hospitais produzem; o valor
    adicionado deles vai inteiro para salário (não têm dono nem lucro, e não fecham por prejuízo).
  - **Obra pública:** paga semana a semana, conforme os pontos chegam.
- **Reserva:** 13 semanas de receita. Só o que passa da reserva (descontado o que falta pagar das
  obras públicas já na fila) vira obra nova.
- **Substituto do banco central:** a regra da fase 1 continua emitindo dinheiro no tesouro quando
  há fábrica ociosa e recolhendo quando os preços esquentam, mas agora só recolhe de tesouro em
  superávit. É ele que financia a diferença entre receita e despesa; some na fatia de moeda.

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
- **Infraestrutura:** ferrovia, transporte rodoviário e usina elétrica, só quando o produto
  (transporte, eletricidade) está mais de 10 % acima do preço-base; vai para as regiões com mais
  gente por nível existente. Pronta, vira um nível comum (a estatal com lucro para o tesouro fica
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

## Comandos do jogador

- `SetTaxRates { tag, rates }`: alíquotas das três classes em milésimos (teto 600); `None` devolve o
  país à regra histórica.
- `SetPublicPriority { tag, weights }`: pesos de infraestrutura, educação, saúde e moradia, de 0 a
  10. Tudo zero suspende obra pública nova.
- `SetMigrationLaw { tag, law }`: `Open` ou `Restricted`.

Saves: os campos novos têm `#[serde(default)]` e entram no hash. Como nas fases anteriores, o
snapshot é postcard, que não é autodescritivo: save de uma versão anterior não carrega.

## Calibração

Benchmark de 100 anos (`cargo run --release -p bf-bench -- --game --check-determinism`), mesma
semente. "Orçamento" é só a fila e o orçamento (sem migração); "fase 3" é esta fatia inteira.

| | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|
| População (bilhões), main | 2,16 | 3,25 | 4,83 | 6,76 |
| População, orçamento | 2,18 | 3,50 | 5,35 | 7,59 |
| População, fase 3 | 2,18 | 3,48 | 5,12 | 6,78 |
| PIB (US$ tri, preços de 1925), main | 6,0 | 14,8 | 32,9 | 49,7 |
| PIB, orçamento | 6,3 | 19,9 | 58,2 | 85,2 |
| PIB, fase 3 | 6,5 | 20,8 | 61,1 | 87,2 |
| Urbanização, main | 0,44 | 0,42 | 0,35 | 0,29 |
| Urbanização, orçamento | 0,43 | 0,44 | 0,38 | 0,32 |
| Urbanização, fase 3 | 0,45 | 0,52 | 0,56 | 0,60 |
| Necessidades básicas, fase 3 (main) | 0,65 (0,63) | 0,68 (0,63) | 0,72 (0,65) | 0,75 (0,63) |
| Impostos (% do PIB), fase 3 | 7,1 | 12,2 | 13,2 | 12,7 |
| Despesa do Estado (% do PIB), fase 3 | 15,5 | 23,9 | 30,5 | 32,0 |
| Obras na fila (mundo), fase 3 | 17 mil | 9 mil | 6 mil | 5 mil |

Fase 3: determinístico entre contagens de threads e depois de salvar e recarregar em 1975; tick mais
lento 35,9 ms (main 45,8 ms); 112 s de simulação (main 106 s).

- A urbanização sobe sem parar de 0,45 (1935) a 0,60 (2025; o real é cerca de 0,57). Cidade baixa a
  natalidade, por isso a população fica na da main (6,78 bilhões contra 8,1 reais), abaixo da
  rodada só com o orçamento.
- A despesa passa da receita em cerca de 19 pontos do PIB: é o dinheiro que o substituto do banco
  central emite no tesouro (já era assim na fase 1, sem aparecer como conta). Nenhum país fica no
  vermelho porque a emissão cobre a folha. A fatia de moeda e banco central tem de trocar isso por
  título público e juro.
- A fila começa longa (a construção civil de 1925 foi dimensionada só para a manutenção) e o preço
  da construção fica 25 % acima da base nos primeiros anos; cai para 4 a 5 % acima depois de 1945.
- PIB maior que na main vem do gasto público (mais demanda), dos hospitais e de mais gente empregada
  na cidade.

## Aberto

- Moeda por país, banco central com juro, títulos e dívida (`Budget::debt`).
- Estatal com lucro para o tesouro; indústria estatal como categoria de obra.
- A IA de país escolhe alíquota e prioridade pela regra; o jogador só por comando, sem tela ainda.
- Lei de migração histórica no cenário (propiska de 1932, hukou de 1958): hoje todos começam abertos.
- Moradia quase nunca limita: o investidor privado constrói o que o aluguel paga. O freio real é a
  renda (Harris–Todaro). Falta o custo de vida mais alto na cidade (comida comprada).
