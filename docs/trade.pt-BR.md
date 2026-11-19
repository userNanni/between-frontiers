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
que ligou estão no PR.

O fechamento é por bem, por decisão do dono (T1, 2026-11-18,
[wiki](wiki/decisoes/comercio.md#o-fechamento-é-por-bem-não-por-grupo-de-bens-e-custa-9--da-simulação)).
Foi medida a alternativa de fechar por grupo de bens, em valor, com a fração do grupo aplicada a
cada bem. O código do comércio caía de 9,1–9,4 % para 5,2–5,3 % da simulação (relógio no trecho,
10 anos, três medidas por forma), mas o PIB mundial ficava de 5 a 8 % abaixo da faixa de 1945 em
diante (de −4 a −9 σ), e o par por bem, que influência, embargo e tarifa leem, se perdia. O código
dessa alternativa não ficou no jogo. O frete que baixa o comércio antes de 1960 e o contêiner que o sobe depois
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

## 12. Serviço e bem local dentro da união aduaneira (microestados)

**O problema.** San Marino, Liechtenstein, Mônaco e Andorra são menores que o grão do modelo. Um
nível de planta emprega de 870 a 990 pessoas; San Marino tem 5 160 trabalhadores. O comércio, o
banco e a ferrovia deles rodam a 5 % de uso e pagam quase nada, porque servem só à própria gente
(`docs/economy.pt-BR.md`, "Microestados"). Serviço, finança e construção não são comerciáveis
(`tradable: false`), e os bens locais (transporte, energia) só transbordam para estados
vizinhos do mesmo mercado (`econ::local`). Na vida real era o contrário: Mônaco vivia de turismo
e cassino para estrangeiros, e Liechtenstein, do mercado suíço. O serviço vendido ao vizinho é a
economia do microestado. As uniões aduaneiras de 1925 existem nos dados (Liechtenstein–Suíça,
1924; Mônaco–França, 1865; também Luxemburgo–Bélgica, Danzig–Polônia e Sarre–França), e com o
comércio por par já zeram a tarifa entre os parceiros nos bens comerciáveis. Isso não alcança o
que pesa.

**A regra (opção A, atrás de chave).** Entre dois países ligados por uma cláusula de união
aduaneira de um acordo em vigor (`TreatyState::customs_unions`, a mesma leitura da tarifa, de modo
que uma união negociada na mesa vale igual), duas coisas atravessam a fronteira:
- **Serviço e finança entram no comércio por par desse par**, com o frete do grupo de serviços. O
  excedente de serviço de um parceiro atende a procura do outro, e o pool mundial não vê esses
  bens: só o par em união os troca.
- **O bem local transborda para o estado vizinho do parceiro**, como já transborda para o vizinho
  do mesmo mercado. A ferrovia de Liechtenstein serve o vale do Reno suíço, e a usina francesa
  serve Mônaco.

O preço de cada bem continua sendo do mercado de cada país. A conta de cada país (PIB, impostos,
gente) segue própria.

**Dados.** San Marino entra com a sua união com a Itália. O tratado de amizade e cooperação de
1862 criou uma união aduaneira de fato com o Reino da Itália, renovada em 1872 e em 28 de junho de
1897, e a de 1897 é a que vale em 1925 (*The Statesman's Year-Book*, verbete San Marino; a
renovação seguinte é de 31 de março de 1939). Andorra não tinha união: havia comércio livre de
fato com a França e a Espanha, e contrabando. Fica fora da opção A e é caso da B.

**Ordem, para mexer no hash uma vez por passo.** (1) O leitor das uniões dos acordos em vigor
(`TreatyState::customs_unions`, #205). (2) A tarifa do comércio lendo os acordos em vigor (#207),
com efeito nulo nos dados de 1925. (3) Os acordos de 1925 em vigor ligados (`treaties_in_force`,
#208). (4) Esta regra, desligada (`union_services`). (5) Ligá-la junto com a entrada de San
Marino, que já muda a tarifa entre San Marino e a Itália, julgadas pela faixa. O custo se mede
pelo relógio no trecho; os pares em união são poucos.

**Resultado medido: não ligar como está.** Ligada, com a união de San Marino nos dados (101 anos,
determinismo e save conferidos), o mundo fica dentro da faixa de reembaralhamento do main 1097315
(PIB mundial de −1,7 a +1,0 %, população até +0,6 %, de 5 a 17 territórios de 166 fora de 2 σ).
Mas os países que a regra devia ajudar pioram:

| PIB contra a faixa | 1935 | 1965 | 2025 |
|---|---|---|---|
| Mônaco | −81 % | −78 % | ~0 |
| Luxemburgo | 0 % | −43 % (−9 σ) | −75 % (−9 σ) |
| San Marino | +16 % | −21 % | +20 % |
| Liechtenstein | ~0 | ~0 | ~0 |

| Salário de mercado, por semana | 1925 | 1927 | 1930 |
|---|---|---|---|
| San Marino (sem a regra: ~1,3) | 23,5 | 5,2 | 4,4 |
| Liechtenstein (sem a regra: ~0,3) | 16,8 | 5,2 | 1,1 |
| Mônaco (sem a regra: ~52) | 119 | 22,8 | 8,0 |

O porquê: o fechamento leva excedente para onde falta. O sócio grande tem sobra de serviço e o
pequeno tem falta (a sua gente compra mais serviço do que as suas plantas, no piso de uso, vendem),
então o serviço francês entra em Mônaco, e não o contrário. As plantas de serviço do sócio pequeno
perdem a venda que tinham, e a economia encolhe. O salário sobe no começo, enquanto a folha se paga,
e cai depois. Na vida real, o microestado vendia ao vizinho o que o vizinho queria *dele* (cassino,
sigilo bancário, turismo, selo), e o fechamento por excedente não tem como dizer isso. A chave
`union_services` fica no código, desligada, e não deve ser ligada como está.

**O que falta, para o dono (depois da fase 6, salvo pedido dele).** O microestado precisa de
demanda estrangeira pelo que é dele, não de um caminho para o excedente do vizinho:

| Opção | O que é | Prós | Contras |
|---|---|---|---|
| 1. Serviço de exportação de quem tem a planta | cassino, banco, turismo como plantas cujo serviço é comprado por estrangeiros, pela renda deles | é o que Mônaco e Liechtenstein foram; vale para Suíça, Luxemburgo, ilhas | plantas e demanda novas no catálogo |
| 2. Demanda do vizinho proporcional à renda dele | o sócio de união compra do pequeno uma parte fixa do seu gasto em serviço | simples, sem planta nova | uma parte fixa é dado arbitrário por país |
| 3. Trabalhador transfronteiriço | o ocioso do mercado pequeno trabalha no vizinho e manda o salário para casa | é Andorra e Liechtenstein | mexe no trabalho e na migração |

Recomendação: 1, com 3 para Andorra. É o processo real e serve além dos microestados.

**Alternativas anotadas, e por que não agora.**
- **B. Trabalho que atravessa.** O trabalhador ocioso de um mercado pequeno ocupa vaga num estado
  vizinho do parceiro, e o salário volta para casa. É o caso de Andorra e o complemento de A se os
  serviços não bastarem. Mexe no trabalho e na migração, que têm donos próprios, e só se justifica
  depois de medir A.
- **C. O microestado no mercado do vizinho, com a conta própria.** É o mais fiel: San Marino
  compraria e venderia ao preço italiano. Mas hoje mercado e país são o mesmo índice em todo o
  código (o PIB, o caixa e o fundo de cada país são do mercado). Separar os dois é uma refatoração
  grande, para quatro países com menos de 60 mil pessoas.

## 14. O volume do comércio

**O problema.** Com o comércio por par (#203), o comércio mundial (exportação mais importação) era
de 2,9 % do PIB em 1925, contra 14,8 % reais (Klasing e Milionis, 2014), e a margem ficava com 55 a
61 % do valor comerciado (`BF_DIAG_VOLUME=2`). Duas causas, uma por passo (decisão V1 do dono,
2026-11-18, opção A).

**Passo 1, o alcance a 100 %.** Só 85 % do saldo de cada mercado chegava ao mundo por semana
(`trade.reach`). O valor vinha do fundo comum da fase 1 (#29), "para que o preço local ainda leve
parte do sinal (distância, portos, hábitos)", antes de existirem o frete e a preferência pelo
parceiro. Eles levam esse sinal agora, e os 15 % restantes eram falta e sobra que voltavam toda
semana: a diferença de preço virava margem, e o comércio caía à metade. Com o alcance a 100 %:

| 1925 | alcance 85 % | alcance 100 % |
|---|---|---|
| comércio mundial / PIB | 2,9 % | 6,4 % |
| margem / comércio | 55 % | 11 % |
| variação semanal do preço, mediana (GBR, ARG, ITA, 2.º ano) | 0,10 a 0,15 % | 0,10 % |

O preço não oscila mais por isso (`BF_DIAG_WEEKLY`): a maior variação semanal média de um bem cai
na Argentina e na Itália e fica parecida no Reino Unido. No século, o PIB mundial cresce mais,
porque o que cada mercado produz chega a quem compra: 120 contra 92 trilhões em 2022 (133 reais),
erro logarítmico médio do PIB por habitante de 1,07 para 0,86 e correlação de ranking de 0,54 para
0,59 em 2022. O meio do século passa do real: 16,3 trilhões em 1955 contra 14,0 antes e 10,9 reais.

**Passo 2, o comércio bruto** (`trade_gross`, desligado). Um mercado só comerciava o saldo: quem
produz e consome o mesmo bem não importava nada dele, e o Reino Unido não comprava tecido francês
porque fazia tecido. No real, a maior parte do comércio é de variedades do mesmo bem.

- **A regra (Armington).** Do que um mercado ao mesmo tempo produz e consome de um bem
  (`min(oferta, procura)`), ele compra fora a fatia que as suas preferências dão aos fornecedores de
  fora contra o próprio produto. Cada fornecedor pesa o que produz vezes a preferência do comprador
  por ele; o próprio produto pesa o que o mercado produz vezes o viés nacional, no índice de preço
  do grupo, sem frete nem tarifa (`trade.home_bias`, na preferência de equilíbrio do mês). A compra
  de fora se divide entre os fornecedores pelos mesmos pesos.
- **Não move bem em termos líquidos.** O que um mercado manda de um bem é compensado pelo que ele
  guarda do próprio; só o saldo se fecha fisicamente (`trade::clear_good_flat`). O comércio bruto
  conta como comércio: o fluxo do par, a tarifa e a influência o leem, e par fechado não tem nenhum.
- **Calibração.** Só o viés nacional, para o comércio mundial de 1925:

| `home_bias` | 1 | 3 | 10 | 30 | 36 |
|---|---|---|---|---|---|
| comércio mundial / PIB, 1925 | 43,1 % | 32,9 % | 22,8 % | 15,9 % | 15,0 % |

  Fica 36. Real: 14,8 % (Klasing e Milionis, 2014).
- **Dependência anotada: a distribuição entre países.** Com o mesmo viés para todos, o tamanho do
  país quase não pesa: em 1925 os EUA ficam com 6,7 % do PIB em comércio e o Reino Unido com 14 %,
  contra cerca de 10 % e 40 % reais (ordem de grandeza). O Reino Unido de 1925 importava comida e
  matéria-prima de um império com tarifa de preferência e frota própria; o que falta é estudar o
  processo (custo do frete por rota, preferência imperial), não ajustar por país.
- **O bloqueio da Alemanha** (`tests/blockade.rs`, a pergunta da opção A): fechar todo par cujo
  caminho mais barato até a Alemanha é por mar, de 1925 a 1926 (o jogo começa em 1925; é o análogo
  de 1914–18). Fatia da procura atendida, aberto → fechado, com o comércio bruto:

| bem | atendido | preço / base |
|---|---|---|
| frutas | 0,70 → 0,11 | 1,32 → 4,00 |
| açúcar | 0,94 → 0,56 | 1,05 → 1,58 |
| comida processada | 0,93 → 0,49 | 1,05 → 1,79 |
| fosfato | 0,99 → 0,00 | 1,01 → 4,00 |
| carne | 0,89 → 0,84 | 1,09 → 1,14 |
| grãos, laticínios, potássio, fertilizante | 1,00 → 1,00 | — |

  O bloqueio deixa falta de comida; só com os saldos o resultado é quase o mesmo. Os grãos não
  faltam porque a Alemanha do jogo de 1925 exporta grão líquido, quando a real importava cerca de um
  quinto das calorias antes de 1914: é dado de produção agrícola, anotado para a calibração.
- **Custo** (2 anos, `--threads 1`, tempo de usuário, mediana de 5 rodadas intercaladas no mesmo
  slot; o ruído entre repetições vai na coluna):

| | tempo | contra a `main` | repetições |
|---|---|---|---|
| `main` 8d558c5 | 9,31 s | — | 9,23 a 9,43 s |
| alcance a 100 % | 9,38 s | 0 % | 9,02 a 10,27 s (outra bateria) |
| comércio bruto sem otimizar | 10,15 s | +9,0 % | 10,03 a 10,27 s |
| + uma divisão por comprador e bem (`trade::Divisor`) | 9,96 s | +7,0 % | 9,84 a 10,12 s |

  Os bens se fecham em paralelo entre os núcleos, cada um com o seu rascunho, e os fluxos se somam
  na ordem dos bens: com o comércio bruto desligado, o hash é o mesmo do alcance. Com 2 threads, o
  passo semanal cai de 5,2 s para 3,1 s. As demais candidatas já estavam no código ou não passaram
  do ruído: a poda dos pares (a linha de preferências guarda só fornecedores acima de 0,1 % do
  favorito, e o viés nacional deixa a maioria dos estrangeiros abaixo), o custo de chegada
  recalculado só uma vez por mês, a divisão em forma fechada por comprador e bem, o pulo de bem sem
  comércio ou sem consumo, e reservar de uma vez a lista de fluxos (mesmo hash, ganho não medível).

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
