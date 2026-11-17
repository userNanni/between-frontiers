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

- **Esforço:** os pesquisadores que os laboratórios empregam, vezes a alfabetização, elevados a
  3/4, vezes o efeito `research` (computador, internet). 70 % vai para a ciência em foco (ou a
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
gasto, com causa à vista.

### A regra

- **Laboratório:** prédio público (`laboratory`, grupo de serviços), com engenheiros e
  profissionais na bancada. Faz o bem público `research`. O esforço do país é o número de
  pesquisadores empregados nos laboratórios da metrópole, vezes a alfabetização.
- **Duplicação:** pontos ∝ esforço^(3/4). Dobrar os pesquisadores não dobra as ideias, porque
  eles pisam nos pés uns dos outros. O expoente é o caso de robustez de Bloom, Jones, Van Reenen
  e Webb (2020, *AER* 110(4), pp. 1115–1116 e tabela A1 do apêndice): λ = 3/4, com a ressalva dos autores de que não há
  consenso sobre o valor.
- **Quanto se quer** (`public::research_target`, valor anual a preço-base):
  - **Empresas:** o valor adicionado de cada grupo de edifícios × a intensidade do grupo
    (`purchases.ron`, campo `research`, com a razão em cada linha) × o quanto o país está perto
    da fronteira (`public::frontier`: das tecnologias que alguém faz em escala, a parcela que o
    país também faz). Empresa na fronteira tem de inventar; empresa longe dela copia.
  - **Estado:** a receita × `STATE_RESEARCH_SHARE` (1 %) × (1 + `research_priority`, efeito
    das leis `state_capitalism` e `planned_economy`) × o peso que o jogador dá à categoria
    "Pesquisa" na aba Orçamento, contra o peso padrão.
- **Quem paga:** o Estado compra tudo o que o laboratório faz, como na escola e no hospital. O
  Tesouro paga a parte da lei (`research_state_share`, grupo `ownership`: de 25 % no
  `laissez_faire` a 95 % no `planned_economy`) e o fundo das empresas paga o resto
  (`public::research_spending`, que a tela de Pesquisa pode mostrar).
- **Onde se constrói:** quinta categoria de obra pública, "Pesquisa", com peso do jogador como as
  outras. O laboratório vai aonde há engenheiros e profissionais que ainda não estão numa
  bancada. Sem engenheiro, o laboratório fica vazio e não rende.
- **Colônia não pesquisa.** Não tem laboratório (nem no setup de 1925), nem entra na meta, nem
  soma pontos. A técnica chega à colônia pela imitação.
- **Ideias mais difíceis de achar:** cada tecnologia custa `(1 + g)^(ano histórico − 1925)`
  vezes o custo do arquivo (`technologies.ron`, `harder_per_year_bp`; `Tech::research_cost`).
  O custo do arquivo continua sendo o da árvore e dos testes de requisito.

### A taxa `g`: 350 pontos-base ao ano, calibrado (fonte: 35)

- **O que a fonte dá.** A produtividade da pesquisa nos EUA caiu 5,1 % ao ano desde os anos
  1930, e o esforço cresceu 23 vezes, 4,3 % ao ano (Bloom et al. 2020, p. 1110). Os custos do
  arquivo já crescem 4,75 % ao ano com o ano histórico (ajuste sobre as 143 tecnologias depois
  de 1925). A diferença dá 35 pontos-base.
- **O que se mediu com a fonte.** Com 35, a fronteira de 51 anos sai 1,9 ano adiantada em média
  contra a contagem de cabeças. O adiantamento é de 4 a 5 anos nas tecnologias dos anos 1950 e
  1960.
- **O que se usa: 350, calibrado.** É o valor que leva as datas de volta a ±1 ano da `main`; o
  da fonte fica ao lado. Com ele, o custo da árvore cresce cerca de 8 % ao ano. Isso **não** é
  o que Bloom mede: compensa o defeito abaixo.

### Por que o calibrado passa da fonte: o custo do pesquisador

Medido nos EUA do modelo, em dólares de 2011 a preço-base:

| | 1926 | 1955 | 1975 |
|---|---|---|---|
| Cientistas e engenheiros nos laboratórios | 56 mil | 380 mil | 1,03 milhão |
| Produto do laboratório por pesquisador | US$ 59 mil | US$ 59 mil | US$ 59 mil |
| Insumos do laboratório por pesquisador | US$ 0,3 mil | US$ 0,3 mil | US$ 0,3 mil |
| Salário do engenheiro | US$ 78 mil | US$ 169 mil | US$ 268 mil |

Real (NSF, *National Patterns of R&D Resources 1953–1977*, NSF 77-310, tabelas B-5 e B-10):
gasto total de P&D ÷ cientistas e engenheiros em P&D, em tempo integral equivalente.

| | 1954 | 1975 |
|---|---|---|
| Gasto de P&D, US$ milhões de 1972 | 9 456 | 27 158 |
| Cientistas e engenheiros em P&D | 237,1 mil | 533,7 mil |
| Custo por pesquisador, dólares de 1972 | 39,9 mil | 50,9 mil |
| O mesmo em dólares de 2011 (deflator implícito do PIB, FRED `GDPDEF`: 22,126 em 1972, 91,478 em 2011) | 165 mil | 210 mil |

- **O salário não é o defeito.** O engenheiro do modelo ganha mais à medida que a
  produtividade sobe.
- **O defeito é o preço do pesquisador.** A meta é em valor a preço-base, e o laboratório
  entrega o mesmo valor por pesquisador o século todo. Um dólar de pesquisa compra sempre o mesmo
  número de pesquisadores, enquanto no real o custo por pesquisador sobe 28 % de 1954 a 1975 e
  já é quase três vezes o do modelo em 1955.
- **O tamanho do defeito:**
  - O modelo emprega cerca de 1,5 vez o pessoal real em 1955 e 2,9 vezes em 1975, com gasto das
    empresas perto do real (0,92 % do PIB em 1955 contra 0,58 % em 1953; 1,00 % em 1975 contra
    1,08 % em 1980, NSF 25-326).
  - De 1955 a 1975, o pessoal do modelo cresce 4,1 vezes contra 2,25 no real: cerca de 3 % ao
    ano a mais, que é o tamanho de `g` acima da fonte.
- **Conserto provável (não feito aqui):** o laboratório consumir instrumentos, material e
  pessoal de apoio por pesquisador, crescendo com a era. A meta passaria a ser em dinheiro ao
  custo de hoje (salário mais instrumentos). "Ideias mais difíceis de achar" ficaria visível
  como custo, e `g` poderia voltar para perto da fonte.

### Dependências anotadas

- **A parte do Estado fica em 0,1–0,2 % do PIB.** O P&D federal americano de 0,71 % do PIB em
  1953 e 1,86 % em 1964 (NSF 25-326) foi defesa e espaço: depende da guerra fria (fase 5), não é
  meta desta regra.
- **Tecnologias de programa estatal e militar.** Centrífuga de gás, satélite de comunicação e
  robô industrial chegam 5 a 8 anos antes da `main` com `g` calibrado. Pedem prioridade de Estado
  com casus belli ou corrida (fase 5), não custo ajustado à mão.
- **1925:** a pesquisa inicial sai de 0,3 % do produto fora do campo, vezes a proximidade da
  fronteira medida pela renda contra a americana (o estado da pesquisa ainda não existe no
  setup). Bernal (1939) dá cerca de 0,6 % da renda nacional nos EUA do fim dos anos 1930 e um
  décimo de por cento na Grã-Bretanha. O modelo emprega 56 mil pesquisadores nos EUA de 1926;
  os laboratórios industriais tinham 6 320 cientistas e engenheiros em 1927 (Mowery 1995,
  tabela 5.1), e a pesquisa universitária e do governo não tem série aberta. O nível de partida
  é alto, e o mesmo conserto do custo do pesquisador o corrige.
- **Série de referência:** P&D sobre o PIB por país desde 1996, `data/history/rd.csv` (Banco
  Mundial, `GB.XPD.RSDV.GD.ZS`, CC BY 4.0). A série da OECD (MSTI, desde 1981) não entrou
  porque a licença não foi confirmada.

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
