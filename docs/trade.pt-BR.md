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
- **O que fecha o par hoje.** Toda semana o comércio junta, num só conjunto de pares fechados
  que vale para a partilha da semana e para o equilíbrio do mês (`Geography::closed_with`):
  - os fechamentos datados da geografia;
  - os **embargos** que um governo ordenou (CT6, `GameCommand::Embargo`, `TariffState::embargoes`,
    save 31): unilaterais, fecham os dois sentidos; levantados, a preferência sobe um λ por mês;
  - com a chave `war`, os **pares em guerra** (CT7, `war::fight::war_pairs`: cada membro de um lado
    contra cada membro do outro, o aliado chamado só depois de entrar) e os pares que um
    **bloqueio** corta (`war::fight::blockaded_pairs`, do thread da guerra: os pares por mar
    cujas rotas passam todas por uma zona que o bloqueador controla; Alemanha e Suécia ficam
    abertas pelo Báltico com o mar do Norte britânico; vazia até a guerra no mar, fatia C2). A
    paz tira o par da lista na semana em que é assinada, e ele reabre devagar como o embargo
    levantado.
- **Ainda dormente:** a cláusula de embargo coletivo ou sanção de um acordo (thread dos tratados,
  em pausa). O encaixe é o mesmo: os pares que a cláusula fecha entram em `extra` de
  `closed_with`, como os embargos por comando. O não reconhecimento e o estreito fechado também
  entram por ali quando existirem.

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

## 13. A tarifa que vale (passo 1 das decisões de comércio)

Até aqui a tarifa só pesava na escolha entre vendedores estrangeiros (seção 3): não arrecadava
nada e não protegia nada. Com a chave `customs_duties` (desligada, `data/rules.ron`; precisa de
`trade_by_pair`), ela passa a fazer as duas coisas da vida real.

**A taxa de cada país.** É a da lei de comércio (`free_trade` 2 %, `revenue_tariffs` 10 %,
`protective_tariffs` 25 %, `autarky` 60 %), deslocada pela diferença entre a taxa de 1925 do país
(`diplomacy.ron`, `tariffs`: direitos sobre o valor de todas as importações, a medida de Clemens e
Williamson) e o degrau da lei que ele tinha em 1925. Em 1925 cada país cobra a sua taxa histórica;
mudar a lei move a taxa a partir dali pelo tamanho do degrau, nunca abaixo de zero
(`customs::Duties`).

**O que a tarifa faz.**
- **Receita.** Toda semana o país recolhe a taxa sobre o valor do que importou de cada parceiro,
  a troca de variedades do comércio bruto incluída, a preço de base (o preço mundial na fronteira;
  `customs::imports_by_buyer`). Sem o comércio por par, sobre a importação líquida ao preço que os
  compradores pagaram. O dinheiro sai do fundo do mercado, para onde foi o pagamento das
  importações, e entra no Tesouro, contado na receita da semana (`customs::pay`). Nada se cria:
  a soma mundial de fundos e tesouros não muda (teste `customs_duties.rs`).
- **Preço de chegada × (1 + t).** O importador paga o bem e a tarifa: no dinheiro, pela receita
  que sai do fundo; na quantidade, pela preferência entre origens do comércio bruto, que lê a
  mesma tarifa que o Tesouro cobra (a da lei deslocada pela diferença para 1925) e reparte a
  compra entre o produtor de casa e os de fora com σ = 5. O preço de mercado do bem não é tocado
  diretamente, porque cada mercado tem um preço por bem, que mistura o nacional e o importado.
- **Corte da quantidade, medido e desligado.** O primeiro desenho cortava a procura externa da
  falta em `1 / (1 + ε t)` (`customs::import_factor`, ε = 1). Medido (`BF_DIAG_MET`), o que deixava
  de vir de fora não ia ao produtor de casa: a procura sem vendedor subia de 0,7 a 1,5 ponto onde o
  país importa, a produção de comerciáveis caía cerca de 1 % e o PIB mundial perdia 4 % até 2022.
  O dado `elasticity` fica, em zero.
  Ter o preço separado (fase de equilíbrio) custa um segundo preço por bem e mercado, guardado no
  save, e a divisão da procura entre nacional e importado (Armington) em cada mercado toda semana.

**Os dados, um por decisão do dono** (`data/economy/purchases.ron`, `trade.customs`):
- `revenue` (CT3): a receita vai ao Tesouro;
- `elasticity` (CT4): a elasticidade-preço da importação. As de longo prazo do G-7 vão de cerca de
  0,3 a 1,6 (Hooper, Johnson e Marquez, 2000); 1 é valor de projeto. Zero desliga o efeito;
- `rates_1925` (CT5): a taxa de 1925 de cada país em vez do degrau da lei.

**A tabela por bem (passo 2, CT1 e CT2).** Em Governo → Comércio, cada um dos 76 bens comerciáveis,
agrupados pelas 9 categorias de `goods.ron`, fica na "Regra histórica" (a taxa da lei, deslocada
pela de 1925) ou "Fixado" pelo governo, de 0 a 200 %; "Aplicar ao grupo" fixa a categoria inteira e
"Voltar à regra" a devolve. Por parceiro, o governo soma pontos à tarifa de todo bem (sobretaxa) ou
os tira (preferência). O mundo guarda só as exceções (`GameWorld::tariffs`, `customs::TariffTable`,
save 30); sem exceção nenhuma, o hash é o de antes. A IA fica na regra.
- **Cobrança.** A tarifa de um fluxo do bem `g` do vendedor `s` ao comprador `b` é a do bem (fixada
  ou da regra) mais os pontos de `s`, sob os acordos em vigor, nunca abaixo de zero
  (`customs::Charge`, `trade::PairRules`): a união aduaneira não paga, a preferência tira a sua
  margem, o teto limita. Passar do teto de propósito (quebra de cláusula, com ponto de
  restauração) é o passo seguinte; por ora o acordo corta o excesso e a tela avisa.
- **Receita no Orçamento.** A linha "Receita aduaneira" lê a cobrança da semana
  (`TariffState::revenue`, um registro que nenhuma regra lê).
- **A preferência pela média do grupo (aproximação).** A preferência entre origens é por grupo de
  edifícios: com bens fixados, o grupo usa a média simples das tarifas dos seus bens. Um bem
  fixado longe do resto do grupo move a fatia de fora do grupo inteiro, não só a dele. Medido com
  `BF_TARIFF` (têxteis britânicos a 100 %, o resto na regra de 9 %): o comércio do Reino Unido cai
  de 15,7 % para 15,4 % do PIB em 1927, espalhado pelos bens do grupo.

**Ainda não.** A quebra da cláusula pela tarifa, com aviso, perda da reputação geral e ponto de
restauração. O embargo e a guerra fecham o par pelo encaixe que já existe (`Geography::closures`,
seção 11): o embargo por comando (CT6) e a guerra por `war::fight::war_pairs` (CT7). É o passo 3.

**A tarifa protege?** Medida com `BF_DIAG_PROTECT=TAG:10` e `BF_LAWS`, com a chave ligada: o
mesmo país sob `free_trade` e sob `protective_tariffs`, por 10 anos. Desligada, mudar a lei não
muda nada.

| | Reino Unido (9 % → 32 %) | Argentina (7 % → 30 %) |
|---|---|---|
| importação total, 1925 | −15 % | −17 % |
| importação total, 1930 | −12 % | −16 % |
| importação total, 1934 | −3 % | −22 % |
| preço do bem mais importado, 1925 | madeira +4 % | carvão +18 % |
| produção protegida, 1930 | têxteis +7 % (a importação vai a zero) | aço −14 % |
| produção protegida, 1934 | têxteis +5 % | aço −3 %, roupas +3 %, pescado +3 % |
| produção total, 1934 | −1,6 % | −1,7 % |

A importação cai logo e o preço de casa sobe pela falta. A produção reage devagar e só onde o país
já tinha plantas: o têxtil britânico cresce, porque já existia. Na madeira britânica e no carvão
argentino não há oferta em casa (o carvão argentino é zero), então a falta só encarece. O aço
argentino queima esse carvão: produz menos, e a importação de aço até sobe em 1930. A produção
total cai um pouco, porque o insumo importado ficou mais caro para todos. Há três razões
para a reação ser pequena:
- a planta nova sai do investimento, que leva anos (`construction`);
- a tarifa é uma só para todos os bens até o passo 2 (a tabela por grupo), e protege o insumo
  junto com o produto;
- a tarifa age sobre a falta (o saldo); a troca de variedades do comércio bruto já responde à tarifa
  da lei pela preferência entre origens (σ = 5), com a chave desligada ou ligada. No Reino Unido,
  `free_trade` → `protective_tariffs` leva o comércio de 15,0 % a 12,6 % do PIB em 1926 com a chave
  desligada e a 11,6 % ligada: os dois canais se somam pouco, sem contar a proteção duas vezes.

**A receita, com o volume de comércio novo** (`BF_DIAG_DUTIES`, 1926, chave ligada):

| país | tarifa | receita / PIB | importação / PIB |
|---|---|---|---|
| Brasil | 26 % | 4,33 % | 16,6 % |
| Argentina | 15 % | 1,99 % | 13,3 % |
| Austrália | 16 % | 1,36 % | 8,5 % |
| Índia | 15 % | 0,91 % | 6,1 % |
| Itália | 8 % | 0,72 % | 9,0 % |
| Reino Unido | 9 % | 0,71 % | 7,9 % |
| Japão | 7 % | 0,45 % | 6,5 % |
| França | 7 % | 0,43 % | 6,2 % |
| Alemanha | 6 % | 0,40 % | 6,6 % |
| Estados Unidos | 14 % | 0,12 % | 0,9 % |
| mundo (`BF_DIAG_CONSERVE`) | 10,8 % | 0,83 % | — |

A referência é de 1 a 3 % do PIB nos anos 1920 nos países com dado, e cerca de 0,6 % nos Estados
Unidos. A taxa é a de 1925 de cada país; o que falta é importação: os Estados Unidos importam 0,9 %
do PIB contra perto de 4,5 % reais, e o Reino Unido 7,9 % contra perto de 25 %. É a distribuição do
comércio bruto entre países (seção 14), não a tarifa.

**Ligada por 101 anos, contra a faixa de 9b54acd** (oito rodadas com empurrão e a base):

| | receita e corte da quantidade | só o corte | só a receita | preço de chegada (o conserto) | faixa |
|---|---|---|---|---|---|
| PIB mundial 2022 | 112,9 tri | 115,2 | 118,6 | 117,5 | 119,4 a 121,7 |
| erro log. do PIB por hab. 2025 | 1,05 | 0,97 | 0,91 | 0,93 | 0,84 a 0,94 |
| população 2025 | 8,07 bi | 8,17 | 8,02 | 8,03 | 8,13 a 8,19 |
| correlação de ranking 2022 / 1985 / 1955 | 0,57 / 0,64 / 0,62 | | | 0,57 / 0,65 / 0,65 | 0,56–0,58 / 0,62–0,64 / 0,62–0,63 |

A receita volta à economia pelo gasto: com a receita ligada, os impostos sobem 0,8 ponto do PIB em
1935, o gasto 0,7 e o caixa do Tesouro 0,3; em 2015 os três estão no nível da base. Ela custa cerca
de 1 % do PIB. O que fica fora da faixa é a população: nos países de tarifa alta e renda baixa, a
receita paga escola e saúde cedo demais. No Equador de 1985, com a receita, a alfabetização é 85 %,
a esperança de vida 75,6 anos e a fecundidade 1,87 com 2,2 mil dólares por habitante; na base, 67 %,
62 anos e 4,95. A Colômbia, a Venezuela, a Guatemala e o Equador param de crescer depois de 1985
(Colômbia 24 contra 40 milhões em 2025). A resposta da alfabetização e da saúde ao gasto é do
thread dos serviços públicos; por isso a chave fica desligada (`customs_duties: false`) até lá.

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

**Passo 2, o comércio bruto** (`trade_gross`, ligado). Um mercado só comerciava o saldo: quem
produz e consome o mesmo bem não importava nada dele, e o Reino Unido não comprava tecido francês
porque fazia tecido. No real, a maior parte do comércio é de variedades do mesmo bem.

- **A regra (Armington).** Do que um mercado ao mesmo tempo produz e consome de um bem
  (`min(oferta, procura)`), ele compra fora a fatia que as suas preferências dão aos fornecedores de
  fora contra o próprio produto. Cada fornecedor pesa o que produz vezes a preferência do comprador
  por ele; o próprio produto pesa o que o mercado produz vezes o viés nacional, no índice de preço
  do grupo, sem frete nem tarifa (`trade.home_bias`, na preferência de equilíbrio do mês). A compra
  de fora se divide entre os fornecedores pelos mesmos pesos.
- **Só o que vai nos dois sentidos.** De cada bem, cada par troca o menor dos dois sentidos: o
  comércio dentro da mesma indústria é a parte que se sobrepõe (Grubel e Lloyd, 1975); o que um
  lado quer além disso é saldo, e o comércio de saldos já o troca. Cada par não move bem nem
  dinheiro em termos líquidos (teste `swap_tests`). O primeiro desenho guardava a compra de um lado
  só, e o balanço de pagamentos medido (#247) mostrou o efeito: um país pequeno contra os parceiros
  comprava muito mais variedade do que vendia, e um grande o contrário. Em 1926, saldo de bens em %
  do PIB:

| | com a compra de um lado só | com os dois sentidos | real (conta corrente) |
|---|---|---|---|
| Argentina | −7,3 | −0,3 | superávit (sem série na base Jones-Obstfeld) |
| Estados Unidos | +5,4 | +2,2 | +0,5 |
| Alemanha | +3,9 | −0,3 | +1,6 |
| Reino Unido | −0,9 | −1,4 | −0,4 |

  O "não liquidado" do balanço cai de 3 a 7 % do PIB para menos de 0,6 % nesses países. A
  Argentina ainda importa mais do que vende (carvão demais e pouca exportação de grão, milho e
  carne): é dado, pré-requisito declarado de ligar a liquidação (docs/state.pt-BR.md).
- **O comércio não cria nem destrói dinheiro.** A compensação de cada mercado faz os compradores
  pagarem o que veio de fora, que nenhuma fábrica dali vendeu, e paga às fábricas o que saiu, que
  nenhum comprador dali comprou: sozinha, ela tira do dinheiro do mercado o seu saldo. O saldo volta
  ao mesmo fundo (`week::margin_transfers`). Antes voltava o saldo menos uma fatia do saldo do mundo,
  e o saldo do mundo (a diferença entre o preço de quem importa e o de quem exporta) saía do dinheiro
  do mundo: de 0,35 a 0,5 % do PIB mundial por ano, achado pelo balanço de pagamentos (#254). O teste
  `trade_money.rs` exige, mercado a mercado e semana a semana, que a devolução seja o que a
  compensação tirou; com a regra antiga ele falha na primeira semana. Pagar o que muda de país é da
  liquidação do padrão-ouro (`payments::settle`). Sem o vazamento, o PIB mundial de 2022 vai de
  119,3 para 126,8 trilhões (133,4 reais), o erro do PIB por habitante de 0,92 para 0,84 e a
  correlação de ranking de 0,55 para 0,59; a deflação de fundo de 1935 some (−0,32 → −0,08 %); o
  meio do século passa mais do real (17,1 contra 16,2 trilhões em 1955, 10,9 reais).
- **Não move bem em termos líquidos.** Só o saldo se fecha fisicamente
  (`trade::clear_good_flat`). O comércio bruto conta como comércio: o fluxo do par, a tarifa e a
  influência o leem, e par fechado não tem nenhum.
- **Calibração.** Só o viés nacional, para o comércio mundial de 1925, com os dois sentidos:

| `home_bias` | 1 | 3 | 6 | 9 | 12 | 36 |
|---|---|---|---|---|---|---|
| comércio mundial / PIB, 1925 | 33,9 % | 23,0 % | 17,4 % | 14,8 % | 13,2 % | 9,3 % |

  Fica 9 (era 36 com a compra de um lado só). Real: 14,8 % (Klasing e Milionis, 2014).
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
- **Ligado, contra a faixa da `main` 5574382** (oito rodadas com empurrão mínimo e a base, 101
  anos): depois de 1945, de 5 a 10 % dos territórios ficam fora de 2σ em cada série, perto dos 5 %
  do acaso; o PIB mundial de 2022 fica em 119,9 trilhões (faixa: 119,1 a 121,7) e a correlação de
  ranking em 0,56 (faixa: 0,56 a 0,59). O erro logarítmico médio do PIB por habitante em 2025 sobe
  para 0,94, acima da faixa (0,86 a 0,91): efeito da regra, pequeno, sem região que mude (as fatias
  regionais do PIB ficam iguais). O caso mais longe é a Etiópia, que cai para 683 dólares por
  habitante em 2025 contra 6 008 na faixa: instabilidade de trajetória de um país interior, anotada
  para a fila de equilíbrio. Custo nos 101 anos, tempo de usuário com `--check-determinism`: +7,1 %.

## Fontes

- Clemens, M. A.; Williamson, J. G. "Why did the tariff–growth correlation change after 1950?".
  *Journal of Economic Growth* 9 (1), 2004, pp. 5–46.
- Gallaway, M. P.; McDaniel, C. A.; Rivera, S. A. "Short-run and long-run industry-level estimates
  of U.S. Armington elasticities". *North American Journal of Economics and Finance* 14 (1), 2003,
  pp. 49–68.
- Head, K.; Mayer, T. "Gravity equations: workhorse, toolkit, and cookbook". *Handbook of
  International Economics*, vol. 4, 2014.
- Hirschman, A. O. *National Power and the Structure of Foreign Trade*. University of California
  Press, 1945.
- Hooper, P.; Johnson, K.; Marquez, J. "Trade elasticities for the G-7 countries". *Princeton
  Studies in International Economics* 87, 2000.
- Klasing, M. J.; Milionis, P. "Quantifying the evolution of world trade, 1870–1949". *Journal
  of International Economics* 92 (1), 2014, pp. 185–197 (via Our World in Data, CC BY 4.0).
