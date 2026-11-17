# Pesquisa: tecnologia com maturidade por país

Estado: fase 2 ligada à simulação. O contrato está em `crates/bf-sim/src/research.rs`, os dados em
`data/research/technologies.ron` e os nomes em `assets/i18n/*/research.ftl`.

## A ideia

Não basta pesquisar: a tecnologia tem de chegar ao mercado. Cada tecnologia passa, país por país,
por quatro estágios inspirados na escala TRL:

| Estágio | O que significa | Como se avança |
|---|---|---|
| Ciência | A ideia é entendida | Esforço de pesquisa (engenheiros e profissionais, vezes a alfabetização) |
| Protótipo | Funcionou uma vez | Mais esforço e engenheiros empregados no ramo |
| Industrial | É feita em escala; os métodos dela podem ser construídos | Precisa de comprador: demanda do mercado pelos bens do ramo, ou o Estado como comprador (armas, espaço, nuclear) |
| Difundida | Está em toda parte, com a produtividade inteira | Parcela dos níveis do país que já usa os métodos dela |

- **Operar e produzir são estados distintos.** Um bem existe no mundo quando algum país chega ao
  estágio industrial; daí em diante qualquer país compra e usa pelo comércio, mesmo sem saber fazer.
- **Vale da morte:** sem comprador, a tecnologia fica parada no protótipo.
- **Difusão:** pesquisar o que outros já sabem custa menos, tanto mais quanto maior a parcela do PIB
  mundial que já sabe e quanto mais o país comercia com quem sabe.
- **Curva de aprendizado:** o custo de chegar à indústria cai com o que o mundo já produziu.
- **Licença e engenharia reversa** pulam estágios com teto de eficiência; **embargo** corta o canal
  (fase 4).

## O que a pesquisa muda

- **Métodos:** um método listado por uma tecnologia só pode ser construído no estágio industrial.
  Método que nenhuma tecnologia lista fica sempre disponível. Substitui o `available_from` por ano.
- **Bens:** consumo de um bem novo começa quando algum país o industrializa.
- **Produtividade:** substitui o crescimento provisório da fase 1. Cada tecnologia dá um ganho por
  grupo de edifícios, metade no estágio industrial e inteiro quando difundida. Serviços continuam
  ganhando menos (efeito Baumol).
- **Demografia e sociedade:** vacinas e antibióticos baixam a mortalidade; a pílula, a fecundidade;
  escola de massa e rádio aceleram a alfabetização; computador e internet aumentam o rendimento da
  pesquisa; contêiner barateia o comércio.

## Escolha

O jogador escolhe a tecnologia em foco; o resto do esforço se espalha pelas candidatas. Os outros
países escolhem sozinhos, por regra determinística (barata, pedida pela demanda, do ramo forte do
país), até a IA da fase 6.

## Como a simulação aplica

- **Esforço:** os pesquisadores que os laboratórios empregam, vezes a qualidade (salário
  relativo), elevados a 3/4, vezes o efeito `research` (computador, internet). 70 % vai para a ciência em foco (ou a
  mais barata aberta) e 30 % para protótipos do que já é ciência. De onde sai o laboratório e
  quem o paga: "Pesquisa que custa", abaixo.
- **Custo:** cai até 75 % com a parcela do PIB mundial que já sabe; dobra a cada cinco anos de
  adiantamento sobre a história (`ahead`), o que segura o líder perto das datas reais.
- **Imitação:** o que já é feito em escala lá fora se copia, um estágio por vez, sem pesquisa
  própria, mais rápido onde se lê e quanto maior a parcela do mundo que já faz.
- **Uso sem fabricar:** os efeitos de mortalidade, fecundidade e alfabetização chegam pelo uso
  (vacina, antibiótico, pílula importados), em 15 anos a partir da primeira fábrica no mundo,
  mais depressa onde se lê. A pílula baixa o piso da fecundidade na proporção da escolaridade das
  mães; como os efeitos entram na demografia: `population.pt-BR.md`, "Demografia".
- **Escolha de método:** lucro = valor adicionado − salário de mercado × trabalhadores. Mecanizar
  compensa onde o trabalho é caro e não onde é barato; antes, com o lucro como fração fixa do valor
  adicionado, quase nenhuma fábrica trocava de método.
- **Produtividade:** a de cada grupo de edifícios, relativa ao estágio de 1925 do país, multiplica
  a da região; a convergência da produtividade de base entre regiões (instituições, escola)
  continua. A subsistência ganha metade do ganho agrícola (Revolução Verde).

## Calibração

O benchmark de 100 anos compara o ano em que cada tecnologia chega ao protótipo e à indústria com os
anos históricos (`demonstrated`, `industrialized`) e acusa desvio grande. A produtividade da
fronteira deve crescer perto de 1,5 a 2 % ao ano.

Primeira rodada com a árvore completa (1925–2025, sem população por censo):

| | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|
| População (bilhões) | 2,16 | 3,25 | 4,83 | 6,76 |
| PIB mundial (US$ tri, preços de 1925) | 6,0 | 14,8 | 32,9 | 49,7 |
| Tecnologias feitas em escala | 31 | 88 | 126 | 164 |

As 133 tecnologias posteriores a 1925 chegam ao protótipo, em média, 1,5 ano depois da história
(desvio absoluto médio de 2,8 anos); 129 chegam à indústria, 1,7 ano depois. Abertos: população
17 % abaixo dos 8,1 bilhões reais e urbanização caindo (0,44 → 0,29), porque a mecanização tira
emprego do campo sem a migração e a construção da fase 3.

Com a árvore em níveis (requisitos novos e custo crescente ao longo de cada requisito), o
protótipo chega em média 2,6 anos depois da história (desvio absoluto 3,6) e a indústria 2,1 anos
depois (125 de 133 até 2025); população e PIB de 2025 quase não mudam (6,75 bilhões, US$ 49,6 tri).

## A fronteira depois de 1980, conferida

Ganho de produtividade da árvore por década (soma de ln(1 + ganho), em %), contra a produtividade
total dos fatores (PTF) medida nos EUA, a fronteira. A PTF é a comparação certa: a troca de método
(máquina por trabalhador) já entra como método, e a tecnologia dá o resto.

| Grupo | Árvore 1980–89 / 1990–99 / 2000–09 / 2010–25 | PTF medida, % ao ano (anos 1980 / 1990 / 2000 / 2010) |
|---|---|---|
| Serviços | 6,5 / 10,1 / 2,8 / 12,1 | economia de mercado 0,88 (2000) e 0,35 (2010); setor empresarial privado 0,56 / 1,01 / 1,11 / 0,55 |
| Indústria pesada | 5,6 / 8,5 / 0 / 14,2 | manufatura 0,29 (1987–90) / 1,51 / 1,34 / −0,30 |
| Indústria leve | 14,1 / 21,1 / 0 / 17,9 | bens duráveis 0,79 / 2,33 / 1,97 / 0,23; eletrônicos 4,65 / 11,08 / 6,00 / 0,99 |
| Agropecuária | 0 / 20,7 / 8,3 / 8,3 | fazendas 3,26 / 1,47 / 1,17 / −0,36 |

Fontes: BLS (Major Sector Productivity e Multifactor Productivity: séries OPHPBS, MFPPBS,
OPHMFG, MPU9900013, MPU9920013, MPU5250013, lidas pelo espelho do FRED; domínio público); USDA ERS,
*Agricultural Productivity in the U.S.*, tabela 1 (domínio público); EU KLEMS & INTANProd, release
2024 (Luiss Lab), contas de crescimento dos EUA (VATFP_I).

Conclusão: a árvore não para depois de 1980. Ela dá à fronteira cerca de 0,7 % ao ano nos serviços
e 0,5 a 1,5 % na indústria, a mesma ordem da PTF medida (0,5 a 1,1 % no setor privado). Os ganhos
grandes de 1930–1979 (indústria pesada +20 a +42 % por década) também seguem a história: foi a
época de PTF alta. Eletrônica, informática, telecomunicações, contêiner (1966) e comércio
eletrônico já estão na árvore. Por isso ela não foi mexida: o crescimento mundial lento depois de
1991 não vem da fronteira, e sim de os países pobres não saírem do campo (`history.pt-BR.md`,
"Convergência e campo"), que é o que cresceu no mundo real depois de 1991.

## Pesquisa que custa

Até aqui o esforço era cabeça contada: engenheiros e profissionais empregados em qualquer lugar,
vezes a alfabetização. Uma China pobre e fechada, com a maior população, pesquisava mais que os
Estados Unidos (115 tecnologias no estágio de ciência em 1985, contra 108). Agora pesquisa é
gasto, com causa à vista: quem paga, quanto custa um pesquisador e onde está a bancada.

### O que não fechou (declarado)

Entrou abaixo do limiar combinado, por decisão do orquestrador (funcionalidade antes de
equilíbrio; equilíbrio fino depois da fase 6). Medido sobre a base `d5570d0` contra o branch antes
do rebase sobre `22d67de`; uma rodada no head rebaseado confere que a tabela não mudou de sinal.

**Datas da fronteira**, branch menos base, por década do ano histórico da tecnologia (101 anos):

| Década | Média (anos) | Fora de ±1 ano |
|---|---|---|
| 1930 | −0,08 | 0 de 37 |
| 1940 | −0,40 | 7 de 25 |
| 1950 | −0,51 | 4 de 47 |
| 1960 | +0,91 | 8 de 33 |
| 1970 | **+1,81** | 13 de 32 |
| 1980 | **+2,80** | 7 de 15 |
| 1990 | **+2,07** | 11 de 27 |
| 2000 | +1,25 | 8 de 20 |
| 2010 | +1,15 | 7 de 26 |
| 2020 | +0,80 | 2 de 5 |

- A **solar de usina** não chega à indústria até 2025 (protótipo em 2018, contra 1997 na base).
- Piores casos: ciclo combinado +10 anos, GPS +9 no protótipo e +7 na indústria, robô industrial
  +6, drone armado +6, bateria de rede +6, bomba atômica na indústria +5.
- **1985:** o **Japão empata** com a China e a Índia no estágio de ciência (82 tecnologias pós-1925
  cada) e passa por uma só no protótipo (67 contra 66). EUA, URSS, Alemanha, Reino Unido e França
  ficam à frente.
- **Seguidores** chegam a 1985 com 14 a 24 tecnologias a menos no estágio de ciência que na base:
  Reino Unido −17, URSS −20, Japão −18, China −24, Índia −20, Brasil −19.
- **Pessoal de P&D dos EUA** 31 % (1955) e 49 % (1975) abaixo do real; **custo por pesquisador**
  31 % e 37 % acima (NSF 77-310, tabelas B-5 e B-10; dólares de 2011 pelo deflator do PIB).
- **Eletricidade e gás piores** (`noise.py compare`, faixa de reembaralhamento): eletricidade de 1955
  de 0,97 para 1,20 do real (+6,8 desvios), de 1985 de 0,93 para 0,82 (−5,2), de 1995 de 0,88 para
  0,82 (−6,1); gás de 1985 de 0,46 para 0,42 (−4,1). Carvão de 1955 e 1995 piores, de 1965 melhor.
- Melhor: o PIB mundial por habitante mais perto do real em todas as décadas de 1945 a 2005 (1985:
  1,284 → 1,145 do real, −8,0 desvios); ranking de 1955, 1985 e 2025 igual ou melhor.

**Causa conhecida.** A árvore convertida muda as datas, e as datas mudam o esforço: menos
tecnologia difundida nos seguidores, menos PIB e menos efeito de computador. Por isso o fator
medido com a árvore antiga fica alto para a árvore nova nos anos 1975–1995. **Remédio:** rodar de
novo o script de conversão (abaixo) quando a fase 5 puser o P&D federal da guerra fria, ou antes, se
o dono pedir.

### A regra

- **Laboratório:** prédio público (`laboratory`, grupo de serviços) com engenheiros e
  profissionais na bancada, pessoal de apoio e instrumentos. Faz o bem público `research`.
- **Quanto se quer gastar** (`public::research_target`, valor anual):
  - **Empresas:** o valor adicionado de cada grupo de edifícios × a intensidade do grupo
    (`purchases.ron`, campo `research`, com a razão em cada linha) × o quanto o país está perto
    da fronteira (`public::frontier`: das tecnologias que alguém faz em escala, a parcela que o
    país também faz). Empresa na fronteira tem de inventar; empresa longe dela copia. A escala
    vem da NSF: P&D financiado pelas empresas americanas foi 0,58 % do PIB em 1953 e 1,08 % em
    1980 (NSF 25-326, tabela 1).
  - **Estado:** a receita × `STATE_RESEARCH_SHARE` (1 %) × (1 + `research_priority`, efeito
    das leis `state_capitalism` e `planned_economy`) × o peso que o jogador dá à categoria
    "Pesquisa" na aba Orçamento, contra o peso padrão.
- **Quanto custa um pesquisador** (`public::research_level_cost`): o salário de mercado do país
  naquele mês (`monthly::market_wages`, o mesmo que as plantas pagam) × o peso salarial dos
  empregos do laboratório, mais os instrumentos a preço de mercado. País rico paga caro pelo
  pesquisador; país pobre paga barato. Os laboratórios que o país quer = gasto ÷ custo de um
  nível.
- **Instrumentos:** equipamento elétrico, máquinas, ferramentas, químicos, papel e serviços,
  como insumo do laboratório, a 0,79 da folha na produtividade 1 (e, como em toda planta,
  crescendo com a produtividade do lugar). A parte do P&D que não é pagamento de pessoal ficou
  entre 33 % e 61 % em 2021: pessoal de 38,6 % a 66,5 % do gasto no Japão, Coreia, Polônia,
  Alemanha, Espanha e Itália, mediana de cerca de 56 % (Eurostat `rd_e_gerdcost`). A NSF 77-310
  não divide o gasto por tipo, e não há série aberta mais longa: a parcela é fixa.
- **Falta de instrumento:** o laboratório que não obtém os insumos perde esforço só na parte
  não salarial (`INSTRUMENT_SHARE`, 44 %), na proporção do que faltou (`Plant::input_ratio`);
  nunca zera e não trava.
- **Esforço:** pesquisadores empregados nos laboratórios da metrópole × qualidade, elevado a 3/4.
  - **Qualidade:** o salário do país ÷ o da economia mais bem paga entre as que têm pelo menos
    1 % do PIB mundial, no máximo 1. Gasto em P&D deflacionado por um salário comum é número de
    pesquisadores ajustado por qualidade (Bloom, Jones, Van Reenen e Webb 2020, *AER* 110(4),
    p. 1113). O pesquisador barato do país pobre conta pelo que custa.
  - **Duplicação:** dobrar os pesquisadores não dobra as ideias. O expoente 3/4 é o caso de
    robustez dos mesmos autores (pp. 1115–1116 e tabela A1 do apêndice), com a ressalva deles
    de que não há consenso sobre o valor.
- **Quem paga:** o Estado compra tudo o que o laboratório faz, como na escola e no hospital. O
  Tesouro paga a parte da lei (`research_state_share`, grupo `ownership`: de 25 % no
  `laissez_faire` a 95 % no `planned_economy`) e o fundo das empresas paga o resto
  (`public::research_spending`, que a tela de Pesquisa pode mostrar).
- **Onde se constrói:** quinta categoria de obra pública, "Pesquisa", com o peso do jogador como
  as outras. O laboratório vai aonde há engenheiros e profissionais que ainda não estão numa
  bancada. Sem engenheiro, o laboratório fica vazio e não rende.
- **Colônia não pesquisa.** Não tem laboratório (nem no setup de 1925), nem entra na meta, nem
  soma pontos. A técnica chega à colônia pela imitação.

### A árvore na unidade nova

Os custos da árvore estavam escritos em pontos de cabeça contada. Com a regra nova, o esforço do
líder cresce de outro jeito, e sem conversão toda data mudaria. A conversão é uma troca de
unidade, feita por script (`tools/research/convert_tree.py`), não um ajuste por tentativa:

- Duas rodadas de 100 anos com a árvore antiga: a base (cabeça contada) e a regra nova.
- Para cada tecnologia, fator = pontos do líder (o país com mais pontos no ano) na **janela da
  própria tecnologia**, regra nova ÷ base. A janela vai do ano histórico do requisito mais
  recente ao ano dela, limitada a 5–15 anos (média de 11,9). A tecnologia é paga com o fluxo de
  pesquisa dos anos antes dela, não com o estoque desde 1925. Uma primeira passada com o fator
  cumulativo errou nas duas pontas, pelo mesmo motivo: barateou os anos 1950 e encareceu os
  anos 2000.
- A árvore deste PR saiu deste comando, com as duas rodadas de 101 anos (semente 7, 2 threads,
  base `d5570d0` e branch com a árvore antiga):

  ```
  python3 tools/research/convert_tree.py c-base-history.json c-old-history.json --write
  ```

  Cada histórico é o `--history-out` de `bf-bench --game --years 101`; a reconversão é o mesmo
  comando com as duas rodadas novas.
- Custo novo = custo antigo × fator. Depois, a regra da árvore de que toda tecnologia custa pelo
  menos 10 % mais que cada requisito é restaurada subindo o custo onde a conversão a quebrou (20
  de 168 tecnologias).
- O fator da janela de 10 anos: 1,05 em 1935; 2,67 em 1955; 2,71 em 1965; 2,24 em 1975; 1,93 em
  1985; 1,37 em 1995; 1,05 em 2005; 0,94 em 2015; 0,87 em 2025.
- **A conversão embute na árvore a desaceleração do esforço do modelo depois de 1975.** O fator
  cai porque a intensidade das empresas é fixa (parcela do valor adicionado que não satura) e o
  P&D federal da guerra fria não existe; no real, o esforço seguiu subindo. Por isso a mediana do
  custo pode cair até 15 % de um nível da árvore para o seguinte (o teste aceita isso, com a
  razão). Quando a fase 5 puser o P&D federal, roda-se o script de novo.
- **Sem taxa por cima.** A árvore já cresce com o ano histórico. Os custos de cabeça contada
  cresciam 4,75 % ao ano (ajuste sobre as 143 tecnologias depois de 1925); Bloom et al. medem
  queda de 5,1 % ao ano da produtividade da pesquisa americana desde os anos 1930 (p. 1110). A
  comparação é conferência, não parâmetro.

### Dependências anotadas

- **Salário americano alto:** o custo por pesquisador nos EUA sai acima do real porque o PIB por
  habitante americano do modelo é alto; o salário de mercado acompanha.
- **Computação como método:** o efeito global de computador e internet sobre os pontos fica até a
  seção "Depois: computação como método".

- **Pessoal de P&D dos EUA abaixo do real.** Com a regra de custo, o modelo emprega cerca de 31 %
  menos cientistas e engenheiros que o real em 1955 e 42 % menos em 1975 (real: 249 mil em 1955,
  Historical Statistics W 178–180; 237,1 mil em 1954 e 533,7 mil em 1975, NSF 77-310, tabela
  B-10). Falta o P&D federal da guerra fria: 0,71 % do PIB em 1953 e 1,86 % em 1964 (NSF 25-326),
  contra 0,1–0,2 % da parte do Estado no modelo. É fase 5.
- **Intensidade fixa das empresas:** o esforço do modelo desacelera depois de 1975; a conversão
  da árvore carrega isso (acima).
- **Tecnologias de programa estatal e militar** (centrífuga de gás, satélite de comunicação, robô
  industrial, energia nuclear) saem antes da data da base. Pedem prioridade de Estado com corrida
  ou guerra (fase 5), não custo ajustado à mão.
- **1925:** a pesquisa inicial sai de 0,3 % do produto fora do campo, vezes a proximidade da
  fronteira medida pela renda contra a americana (o estado da pesquisa ainda não existe no
  setup). Bernal (1939) dá cerca de 0,6 % da renda nacional nos EUA do fim dos anos 1930 e um
  décimo de por cento na Grã-Bretanha.
- **Série de referência:** P&D sobre o PIB por país desde 1996, `data/history/rd.csv` (Banco
  Mundial, `GB.XPD.RSDV.GD.ZS`, CC BY 4.0). A série da OECD (MSTI, desde 1981) não entrou porque
  a licença não foi confirmada.

## Depois: computação como método

Desenho do dono (2026-11-17), para depois da fase 6. Não está no código.

- **A ideia:** o computador, e depois a IA, é ganho de **eficiência**, não de capacidade. Um país
  sem computador consegue pesquisar, mas com perda enorme de escala: a dinâmica dos fluidos foi
  feita à mão, com cálculos muito complexos e demorados. A IA, do mesmo jeito, diminui o tempo de
  fazer tudo.
- **A forma:** métodos nos prédios que pensam (laboratório, universidade e, na administração do
  Estado, a repartição), cada um consumindo os bens correspondentes:
  - cálculo manual;
  - computador central (`computers`);
  - microcomputador e rede (`computers`, `telecommunications`);
  - IA (computação em nuvem, eletricidade).

  O método novo rende mais por pesquisador e por servidor. Quem não tem o bem fica no método
  antigo, mais lento, sem ser travado.
- **O que substituiria no código de hoje:** o efeito global `research` das tecnologias
  (`EFFECTS[3]`: computador eletrônico 80, educação superior de massa 60, computador pessoal 60,
  comutação de pacotes 40, web 120, sequenciamento genético 20, nuvem 30, aprendizado profundo
  60, modelos de linguagem 80, ensino a distância 10, por mil no estágio difundido). Hoje ele
  multiplica os pontos de todo pesquisador do país por até cerca de 2,1
  (`research_system`, `boost = 1 + 2 × efeito`), sem comprar nenhum computador. Como método,
  o ganho passa a depender do bem comprado e do preço dele, e a administração do Estado ganha o
  mesmo tipo de eficiência (folha menor para o mesmo serviço).
- **O que precisaria medir antes:**
  - o ganho de produtividade de pesquisa e de administração por método, com fonte (horas de
    cálculo à mão contra máquina; produtividade do setor público com informática);
  - o consumo de bens por pesquisador em cada método (quanto computador e rede um laboratório
    compra), para a demanda nova não criar falta crônica;
  - a conversão da árvore de novo (`tools/research/convert_tree.py`), porque o esforço do líder
    muda de unidade outra vez;
  - o crescimento da alfabetização nos ajustes de custo: hoje a qualidade do pesquisador vem do
    salário relativo, e a alfabetização só entra pela produtividade de base. Se a escola de massa
    mudar o custo de formar um pesquisador, isso entra no custo do nível de laboratório, não num
    multiplicador de pontos.

## A árvore de 1925 a 2025

168 tecnologias em 12 campos e 6 eras (`data/research/technologies.ron`). Regras que os testes de
`crates/bf-sim/tests/research_tree.rs` cobram:

- Todo método e todo bem com `available_from` depois de 1925 tem exatamente uma tecnologia. Métodos de
  ponta em 1925 (linha de montagem, Haber-Bosch, hidrelétrica, ferrovia elétrica, aço elétrico,
  trator, perfuração rotativa, central automática) ficam com uma tecnologia da era 0 que já começa
  industrial ou difundida nos países que os usavam. O primeiro método de cada edifício de 1925 nunca
  tem tecnologia: é como o edifício funciona em 1925 (por isso o alumínio Hall-Héroult, único método
  da fundição, fica livre).
- Datas históricas dentro da era, com folga de 5 anos; custo cresce cerca de 2,5× por era.
- **É uma árvore de verdade:** toda tecnologia posterior a 1925 nasce de pelo menos uma outra, e
  custa pelo menos 10 % mais que cada um dos seus requisitos.
- **Uma árvore só, não doze:** as áreas se alimentam umas das outras, como na vida real (o
  militar passa o radar, o foguete e a navegação por satélite para o civil; a escola e a
  universidade, para a ciência; a mineração, para os materiais). Pelo menos 40 % dos requisitos
  cruzam de área, e toda área alimenta pelo menos duas tecnologias de outras. É a diferença para
  o Victoria 3, com três árvores separadas (produção, militar, sociedade).
- **Níveis:** duas colunas por era (`LEVELS_PER_ERA`). O nível de uma tecnologia é o primeiro da
  sua era ou um a mais que o maior nível dos seus requisitos, o que vier depois. São 13 níveis,
  do 1 (≈ 40 pontos) ao 13 (≈ 6000 pontos); a mediana do custo nunca cai de um nível para o
  seguinte.
- Produtividade: nas eras 1–5, Σ ln(1 + g) por grupo fica perto de 1,55 (≈ 4,7× em cem anos) nos
  grupos industriais e na agropecuária, e perto de 0,5 nos serviços. O cabeçalho do arquivo explica
  como os pesos foram escalados.
- Tecnologia de era 1 em diante começa desconhecida, ou no estágio de ciência onde a ideia já existia
  em 1925 (televisão, foguete a combustível líquido, radar, milho híbrido).

## A tela

A árvore é um grafo, como nos editores de nós (xyflow): colunas são os níveis, com a era e o
custo típico no cabeçalho; curvas ligam cada tecnologia aos requisitos. Não há faixas por área:
cada cartão fica perto dos cartões a que se liga, venham de que área vierem (desenho em camadas
pelo baricentro), e só de leve perto das outras da sua área. A área é o ponto colorido no cartão,
e o filtro por área acende as dela sem esconder o resto.
Arrastar move, a roda do mouse aproxima, o minimapa no canto mostra onde se está e leva a
qualquer ponto, e "Ajustar" mostra a árvore inteira. Passar o mouse por uma tecnologia acende o
que ela pede e o que ela libera; o caminho até a tecnologia em foco fica animado. As abas de era
levam à primeira coluna da era. Com o zoom afastado, o cartão vira só a faixa do estágio.
