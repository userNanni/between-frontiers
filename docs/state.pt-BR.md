# Estado: construção, orçamento e obra pública

Estado: primeira fatia da fase 3. O código está em `crates/bf-sim/src/econ/construction.rs` (fila de
obras) e `crates/bf-sim/src/econ/public.rs` (orçamento e obra pública); o estado salvo fica em
`Market::queue` e `Market::budget` (`econ/state.rs`). Moeda, dívida e banco central ficam para a
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

## Comandos do jogador

- `SetTaxRates { tag, rates }`: alíquotas das três classes em milésimos (teto 600); `None` devolve o
  país à regra histórica.
- `SetPublicPriority { tag, weights }`: pesos de infraestrutura, educação, saúde e moradia, de 0 a
  10. Tudo zero suspende obra pública nova.

Saves: os campos novos têm `#[serde(default)]` e entram no hash. Como nas fases anteriores, o
snapshot é postcard, que não é autodescritivo: save de uma versão anterior não carrega.

## Calibração

Benchmark de 100 anos (`cargo run --release -p bf-bench -- --game`), mesma semente:

| | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|
| População (bilhões), main | 2,16 | 3,25 | 4,83 | 6,76 |
| População, com esta fatia | 2,18 | 3,50 | 5,35 | 7,59 |
| PIB (US$ tri, preços de 1925), main | 6,0 | 14,8 | 32,9 | 49,7 |
| PIB, com esta fatia | 6,3 | 19,9 | 58,2 | 85,2 |
| Urbanização, main | 0,44 | 0,42 | 0,35 | 0,29 |
| Urbanização, com esta fatia | 0,43 | 0,44 | 0,38 | 0,32 |
| Impostos (% do PIB) | 7,1 | 12,0 | 13,3 | 12,7 |
| Despesa do Estado (% do PIB) | 15,6 | 24,1 | 30,7 | 31,8 |
| Obras na fila (mundo) | 15 mil | 8 mil | 4 mil | 3 mil |

- A despesa passa da receita em cerca de 19 pontos do PIB: é o dinheiro que o substituto do banco
  central emite no tesouro (já era assim na fase 1, sem aparecer como conta). Nenhum país fica no
  vermelho porque a emissão cobre a folha. A fatia de moeda e banco central tem de trocar isso por
  título público e juro.
- A fila começa longa (a construção civil de 1925 foi dimensionada só para a manutenção) e o preço
  da construção fica 25 % acima da base nos primeiros anos; cai para 3 a 5 % acima depois de 1945.
- PIB e população maiores que na main vêm do gasto público (mais demanda) e dos hospitais.

## Aberto

- Moeda por país, banco central com juro, títulos e dívida (`Budget::debt`).
- Estatal com lucro para o tesouro; indústria estatal como categoria de obra.
- A IA de país escolhe alíquota e prioridade pela regra; o jogador só por comando, sem tela ainda.
