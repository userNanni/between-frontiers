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

- **Esforço:** engenheiros e profissionais empregados, vezes a alfabetização, vezes o efeito
  `research` (computador, internet). 70 % vai para a ciência em foco (ou a mais barata aberta) e
  30 % para protótipos do que já é ciência.
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
