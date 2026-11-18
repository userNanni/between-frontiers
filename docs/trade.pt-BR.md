# Comércio com parceiro (fase 4)

Estado: desenho decidido pelo dono em 2026-11-17 (seção 9); o PR 1 é o #113. Fica entre a economia (`economy.pt-BR.md`: mercados nacionais
que fecham toda semana), os tratados (`treaties.pt-BR.md`: `tariff_cap`, `tariff_preference`,
`customs_union`, promessa de entrega) e a influência (`influence.pt-BR.md`: o canal de comércio,
dormente até o comércio ter parceiro).

## 1. O que existe hoje

O comércio mundial é um *pool* por bem (`econ::week`): toda semana cada mercado oferece 85 % do
que sobra (`TRADE_REACH`) e procura 85 % do que falta, e o que se troca, o menor dos dois lados, se
reparte pro rata. Não há par de países, frete, tarifa por parceiro nem preferência: a Argentina
vende trigo ao mundo, não à Grã-Bretanha. O canal de comércio da influência fica dormente por isso,
e a decisão 2 da influência proíbe preenchê-lo com comércio bilateral estimado por gravidade depois
do *pool*: quem vende para quem tem de sair do próprio fechamento.

## 2. Como é na vida real

- **O comércio é por par, e o que decide é o custo de chegada:** o preço de quem vende, o frete da
  rota e a tarifa de quem compra.
- **Política muda o custo de chegada por parceiro.** Preferência imperial de Ottawa (1932), área da
  libra e bloco do ouro, porta aberta dos mandatos, união aduaneira (Zollverein, CEE em 1957).
- **Estados contratam entregas.** *Clearing* bilateral da Alemanha com os Bálcãs nos anos 1930
  (Hirschman, 1945); trigo, petróleo e metais soviéticos para a Alemanha em 1939–41; acordos de
  fornecimento da Guerra Fria.
- **O frete muda com a técnica e com a rota.** O contêiner barateia o frete depois de 1960; Suez
  fechado (novembro de 1956 a abril de 1957; 1967–75) manda o navio pelo Cabo.
- **Ninguém compra só do mais barato.** Cada importador reparte entre fornecedores, e a parte de
  cada um cai com o custo de chegada (elasticidade de substituição entre origens, a de Armington:
  perto de 5 no longo prazo; Head e Mayer, 2014).
- **O parceiro tem inércia.** Contrato, rede comercial e costume seguram o fornecedor: a
  elasticidade de longo prazo é, em média, o dobro da de curto prazo (Gallaway, McDaniel e Rivera,
  2003, 309 indústrias dos EUA). Em 1956–57 o comércio entre a Europa e a Ásia continuou entre os
  mesmos parceiros, pelo Cabo e mais caro; não trocou de fornecedor em cinco meses.

## 3. O fechamento por par (PR 1)

Por bem comerciável, toda semana, depois que cada mercado soma o que oferece e o que procura:

1. **Promessas de entrega primeiro** (seção 5).
2. **Custo de chegada** de *i* para *j*: preço de *i* na semana anterior × (1 + frete(*i*, *j*) +
   tarifa de *j* sobre *i*).
   - **Frete:** a taxa de frete do grupo do bem que o catálogo já tem (`purchases.ron`, por mil do
     valor) vezes a distância pela rota marítima entre os portos principais dos dois, sobre uma
     distância de referência; o contêiner a reduz (efeito `trade` da pesquisa). Calculado uma vez
     por mês numa matriz de países (estreito e canal mudam a rota: `sea::route` com os estreitos do
     mês). País sem mar: o trecho por terra até o porto do vizinho.
   - **Tarifa:** a da lei de comércio de *j* (`LawEffects::tariff`), menos a margem de
     `tariff_preference`, limitada por `tariff_cap`, zero dentro de `customs_union`.
   - **Embargo:** custo infinito para o par; a cláusula ainda não existe no catálogo dos tratados
     e fica dormente até as fases 5 e 6.
3. **Repartição:** cada importador *j* reparte o que procura entre os vendedores *i* por
   oferta_*i* × preferência_*ij*. A preferência de equilíbrio é custo_*ij*^(−σ), σ = 5, normalizada
   entre os vendedores; a oferta entra semana a semana.
4. **Inércia** (seção 4): a preferência que vale é a do mês, que anda só uma parte do caminho até a
   de equilíbrio; a oferta não tem inércia (quem não tem o que vender não vende).
5. **Capacidade:** se um vendedor recebe pedidos acima do que oferece, todos os pedidos a ele são
   cortados pro rata; o que sobrou de procura passa uma segunda vez pelos vendedores com sobra, na
   ordem fixa dos países.
6. **O total de cada mercado é a soma dos pares.** O *pool* deixa de existir; o preço interno
   continua fechando como hoje (o preço de chegada entra no PR 2).

## 4. Inércia do parceiro

Repartir toda semana pelo custo com o preço da semana anterior faria a parcela de cada fornecedor
saltar de uma semana para outra e trocar de parceiro no mês em que um estreito fecha. Na vida real
isso leva meses ou anos.

- **Regra.** Por par (vendedor, comprador) e grupo de bem, a preferência anda todo mês uma fração λ
  do caminho até a de equilíbrio do mês: `preferência += λ × (equilíbrio − preferência)`. Na semana,
  a repartição pesa oferta × preferência entre os vendedores que têm o que oferecer. Com custos
  iguais as preferências são iguais e a repartição é o pro rata do *pool*.
- **Velocidade.** Com a elasticidade de longo prazo o dobro da de curto (Gallaway, McDaniel e
  Rivera, 2003), metade do caminho anda num período dos dados; num trimestre, λ ≈ 0,2 ao mês
  (meia-vida de cerca de três meses). Valor de desenho a calibrar: λ = 1/6 ao mês (meia-vida de
  quatro meses; 90 % do caminho em um ano).
- **A diferença importa.** Suez fechado por cinco meses: sem inércia, o comprador europeu de
  borracha e chá troca de fornecedor na semana do fechamento e volta na da reabertura; com λ = 1/6,
  anda pouco mais da metade do caminho, paga o frete mais caro pelo Cabo e quase volta ao que era
  quando o canal reabre. Também segura o choque de um embargo curto e torna caro (e lento) o desvio
  de comércio por preferência, como foi o de Ottawa.
- **No save:** as preferências em matriz esparsa por (comprador, grupo de bem): só os vendedores com
  parcela acima de 0,1 %, em u16 por cem mil. Com cerca de 83 compradores × 12 grupos × 15 vendedores
  relevantes, perto de 15 mil entradas, cerca de 60 KB (o save tem 1,5 MB). Por bem seriam 83 × 60 ×
  15 = 75 mil entradas, 300 KB: o grupo basta, porque a rede comercial e o contrato são do ramo, não
  do produto.

## 5. Promessa de entrega acumulada

A cláusula dos tratados "vendo X de Y até tal data" vira uma camada de prioridade antes da
repartição livre:

- Por semana, entrega-se o que falta dividido pelas semanas que restam, até o que o vendedor tem,
  ao preço do contrato.
- **Dura** (recomendação): sai da oferta do vendedor antes do mercado interno dele, como o trigo
  soviético em 1940; falta em casa, e só quebra se a produção não der.
- O entregue soma no livro do tratado; o teste mensal de quebra, que já existe, compara o
  acumulado com o devido até a data.
- O resto do bem vai para a repartição livre.

## 6. O que vai para o save, e determinismo

- Grava-se: o valor comerciado por par no mês corrente e no último ano (matrizes de países em
  `Fixed`, cerca de 7 mil cada), as promessas em aberto com o acumulado e as parcelas esparsas da
  seção 4. A matriz por bem da semana é transitória.
- Tudo no fim do `Economy` (`econ::state`), com um espelho do layout de hoje e mais um degrau na
  cadeia de `econ::compat` / `game::decode_world`: save antigo carrega com parcelas vazias, que a
  primeira semana preenche com a repartição de equilíbrio.
- Determinismo: cada bem é independente e pode ir em paralelo; dentro do bem, a segunda passada
  corre os países em ordem fixa de índice, e as somas por mercado se fazem em ordem de índice.

## 7. Conservação e neutralidade

- **Conservação, por bem e por semana:** a soma da linha *i* (o que *i* exporta) é no máximo a
  oferta de *i*; a da coluna *j* (o que *j* importa), no máximo a procura de *j*; e a soma de todas
  as exportações é a soma de todas as importações, exata em `Fixed`, porque é a mesma matriz somada
  nas duas ordens. Logo Σ `net_import` = 0 em todo bem. Teste de propriedade com mercados sorteados,
  custos iguais e diferentes, e promessas.
- **Neutralidade:** com todos os custos de chegada iguais e sem inércia, a repartição é oferta ×
  procura / (oferta total × procura total), o pro rata do *pool*; o teste exige o total de cada
  mercado igual ao do *pool* a 1 ulp. O hash muda (o arredondamento muda).

## 8. Como julgar a rodada de 101 anos

Declarado antes de rodar:

- **Comércio mundial sobre o PIB**, exportações mais importações sobre o PIB mundial
  (`data/history/trade_openness.csv`: Klasing e Milionis até 1949, a preços constantes, 14,8 % em
  1925 e 8 % em 1932; Penn World Table de 1950 a 1969 e Banco Mundial depois, a preços correntes,
  19,9 % em 1950, 25,8 % em 1970, 50,1 % em 2000; a série quebra em 1950). A rodada mede o mesmo na
  `main` e na branch; a branch não pode ficar mais longe da referência que a `main` em mais de 20 %
  em nenhuma década, e o frete deve baixá-lo um pouco antes de 1960 e o contêiner subi-lo depois.
- **PIB mundial e população** dentro da faixa de reembaralhamento da `main` (2 desvios) em todas as
  décadas; ranking do PIB por habitante idem.
- **Grandezas diretas:** a parte da Alemanha no comércio dos Bálcãs em 1938 (perto de 40 a 50 %)
  depende de política (*clearing*) e fica anotada até a cláusula existir. O comércio de cada império
  com as próprias colônias não se mede: metrópole e colônias são um mercado só, e o que passa entre
  elas não é comércio exterior; fica anotado como dependência da descolonização (`borders.pt-BR.md`,
  parte B).
- **CPU** de 10 anos contra a `main`: até 2 % a mais no passo semanal.

## 8a. A chave

O fechamento por par fica atrás da chave `game::Switches::trade_by_pair` em `data/rules.ron`.
Desligada, a semana roda o *pool* mundial de antes (`week::world_pool`, o mesmo código e o mesmo
alcance de 85 %), o estado por par fica vazio e fora do hash, e nada do frete é calculado, nem as
distâncias entre países na carga (`GameData::trade_geography`, feita na primeira vez que a chave
liga). O teste `tests/trade_by_pair.rs` roda os dois caminhos.

**Ligada desde o PR que a ligou**, julgada pela faixa de oito amostras da `main` com as forças
ligadas (6bdba90). A primeira rodada ligada, de 2026-11-17, deu PIB mundial 3 a 6 % acima da faixa
com o comércio sobre o PIB igual. Era a interação com o vazamento de dinheiro nas margens (#193):
com o vazamento consertado, o PIB fica dentro da faixa, e com σ = 0 (custos iguais) o fechamento já
reproduzia o pool. A CPU do caminho ligado caiu no #201, de +25 % para cerca de +8 % do semanal: o
custo estava nas varreduras da matriz vendedor × comprador, não nos pares. Os números da rodada
que ligou estão no PR. O frete que baixa o comércio antes de 1960 e o contêiner que o sobe depois
não aparecem; ficam anotados como dependência, não como meta.

## 9. Decisões do dono (2026-11-17)

1. **T1. Repartição suave** (σ = 5): decidido.
2. **T2. Rota marítima e terrestre**: decidido, e o frete cai com a técnica e a infraestrutura de
   transporte e logística (seção 10).
3. **T3. Ordem dos PRs**: tarifa e preferência no custo de chegada do PR 1; receita e preço de chegada
   no PR 2. Decidido.
4. **T4. Promessa de entrega dura** (antes do mercado interno): decidido, "prometeu tem que arcar com
   aquilo". A cláusula leva um campo de modo (dura ou branda) para escolher depois, sem tela agora;
   o catálogo de cláusulas é do thread dos tratados.
5. **T5. Embargo e sanção entram já, dormentes**: decidido (seção 11).
6. **T6. Tela**: principais parceiros por país e por bem no último ano, a parte do comércio por bloco
   e a conta do custo de chegada do par (preço, frete e o que o reduz, tarifa, preferência).
   Decidido; grava-se o valor por par.
7. **T7. Inércia do parceiro** (λ = 1/6 ao mês): decidido, "mas lembre de casos impeditivos como
   embargos". A inércia nunca segura comércio impedido (seção 11).

## 10. O que barateia o frete

O frete não é só distância × taxa: cai com a técnica de transporte e logística e com a
infraestrutura do trecho.

- **Trecho marítimo: a fronteira mundial difundida.** O frete de navio é um mercado mundial: o
  exportador pobre aluga o mesmo graneleiro, petroleiro ou porta-contêiner que o rico, e a bandeira
  do navio não é a de quem vende. Conta o efeito `trade` da pesquisa (contêiner, superpetroleiro,
  jato e carga aérea, cadeia global de suprimento, comércio eletrônico) médio dos países ponderado
  pelo PIB: o frete marítimo é multiplicado por 1 − 0,7 × esse efeito (com a árvore inteira, 30 % do
  de 1925; valor de desenho).
- **Trecho por terra e porto: o de cada lado.** Ferrovia, rodovia e porto são do país: conta o
  efeito `trade` do país (a autoestrada, por exemplo) e o preço do transporte interno sobre o
  preço-base, que já mede a falta ou a sobra de ferrovia, caminhão e porto (`econ::local`). O
  frete por terra e o custo do porto usam a média dos dois lados.
- **O que falta na árvore:** navio a diesel, graneleiro e ferrovia não têm efeito `trade` hoje; entram
  quando a árvore os ganhar (cada tecnologia com a sua parte, sem refazer a regra).
- **Na tela**, a conta do par mostra cada termo: preço de quem vende, frete por distância, o fator da
  fronteira mundial e o do transporte de cada lado, tarifa e preferência.

## 11. Par fechado

Embargo, sanção, guerra, bloqueio, não reconhecimento e estreito fechado sem rota alternativa fecham
o par.

- **Fechar corta na semana.** O peso do par na repartição é zero a partir da semana em que fecha; a
  inércia não segura comércio impedido.
- **Enquanto fechado, a preferência guardada decai** um λ por mês em direção a zero: a rede comercial
  se desfaz.
- **Reabrir não volta de uma vez.** Na reabertura a preferência sobe de onde decaiu, um λ por mês,
  como o comércio depois de uma guerra.
- **Hoje dormente:** as cláusulas de embargo e sanção ainda não existem no catálogo (do thread dos
  tratados), a guerra não tem estado no mundo e a rota é fixa na carga; o fecho entra pela mesma
  porta (`Geography::closed`) quando cada um existir.

## Fontes

- Gallaway, M. P.; McDaniel, C. A.; Rivera, S. A. "Short-run and long-run industry-level estimates
  of U.S. Armington elasticities". *North American Journal of Economics and Finance* 14 (1), 2003,
  pp. 49–68.
- Head, K.; Mayer, T. "Gravity equations: workhorse, toolkit, and cookbook". *Handbook of
  International Economics*, vol. 4, 2014.
- Hirschman, A. O. *National Power and the Structure of Foreign Trade*. University of California
  Press, 1945.
- Klasing, M. J.; Milionis, P. "Quantifying the evolution of world trade, 1870–1949". *Journal
  of International Economics* 92 (1), 2014, pp. 185–197 (via Our World in Data, CC BY 4.0).
