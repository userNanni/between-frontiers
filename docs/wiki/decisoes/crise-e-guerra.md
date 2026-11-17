# Crise e guerra

Como os países se armam em paz, como uma crise começa e escala, quem cede, quem entra e quanto a
guerra custa. O detalhe está em [`war.pt-BR.md`](../../war.pt-BR.md).

A ideia em uma frase: **a guerra é ganha ou perdida na economia.** Uma força é gente (tirada da
pirâmide e do trabalho), equipamento (que a economia fabrica) e treino; mantê-la custa bens toda
semana, e os bens têm de chegar à frente. A batalha não tem sorteio: o resultado sai com a conta
termo a termo.

## Em paz

### O exército tem o tamanho real de 1925 e acompanha a lei

- **Decisão:** os soldados são o efetivo real de 1925, tirados da força de trabalho. Em paz, o
  exército segue a parcela de 1925 sobre os homens em idade de serviço, escalada pela lei de
  conscrição.
- **O que o autor pensou:** recomendação do thread da guerra que a coordenação seguiu e relatou ao
  autor, que não objetou.
- **Como o jogo funciona por causa disso:** um soldado não trabalha; quem é desmobilizado volta à
  profissão. Se o jogador muda a lei de serviço militar, o exército muda no janeiro seguinte, e a
  população que cresce leva o exército junto.
- **Como é na vida real:** o tamanho do exército em paz muda devagar, por lei e por orçamento.
- **Estado:** em revisão (#115).
- **Onde está o detalhe:** "Forças em paz" em `war.pt-BR.md` no #115; regras trocáveis em
  `data/military/peace.ron`.

### O orçamento militar tem teto, e a sobra moderniza o equipamento

- **Decisão:** o teto do orçamento militar segue a lei. O efetivo paga primeiro o consumo e a
  reposição; a sobra moderniza o equipamento, uma parcela do efetivo por vez, para a geração
  seguinte ligada à árvore tecnológica.
- **O que o autor pensou:** decisão da coordenação, relatada ao autor sem objeção. Motivo: na vida
  real o gasto militar acompanha a renda, e o equipamento troca de geração.
- **Como o jogo funciona por causa disso:** sem modernização, o exército ficava com o equipamento
  de 1925 e o gasto militar caía a quase nada do PIB ao longo do século. Com ela, o caça de 1925 e
  o de 2025 são o mesmo tipo em gerações diferentes, e a geração nova chega aos poucos, comprada à
  indústria.
- **Como é na vida real:** cada geração de equipamento custa mais, e o gasto militar em paz fica de
  2 a 5 % do PIB nas potências (a faixa citada em `war.pt-BR.md`).
- **Estado:** em revisão (#115; o código está no ramo, o documento ainda descreve a fatia anterior).
- **Onde está o detalhe:** [`war.pt-BR.md`, "Tipos e gerações"](../../war.pt-BR.md#tipos-e-gerações).

### Mobilizar é gradual e decisão de Agendas

- **Decisão:** a reserva se apresenta aos poucos, semana a semana, e sai do trabalho enquanto
  durar. Mobilizar e desmobilizar são decisões em dois passos, com ponto de restauração.
- **O que o autor pensou:** os princípios de [gradual como na vida real](../principios.md#gradual-como-na-vida-real-mas-não-tanto-assim)
  e de [decisão rara fora do fluxo comum](../principios.md#decisão-rara-fica-fora-do-fluxo-comum).
- **Como o jogo funciona por causa disso:** o reservista chega com treino parcial e sem
  equipamento, que é comprado aos poucos. A Suíça, sem exército permanente, tem meio milhão de
  homens em um mês.
- **Estado:** em revisão (#115); a aba "Forças" do governo (tecla T) vem no mesmo PR.
- **Onde está o detalhe:** [`war.pt-BR.md`, "Gente: quem pode ser mobilizado"](../../war.pt-BR.md#gente-quem-pode-ser-mobilizado).

## A crise

### Não há declaração seca: a guerra nasce de uma exigência recusada

- **Decisão:** a crise anda em degraus: exigência, prazo, resposta (aceitar, recusar ou deixar
  vencer), e quem exigiu decide entre declarar ou recuar. Exigir e declarar são decisões raras em
  Agendas, com dois passos e ponto de restauração. A exigência é feita de cláusulas do catálogo dos
  tratados; aceita, ela executa como tratado.
- **O que o autor pensou:** o autor aceitou exigência mesmo sem motivo, registrada para a
  reputação, e o prazo padrão de 4 semanas (W1 e W2).
- **Como o jogo funciona por causa disso:** a crise vira uma entrada do diário com o prazo em
  barra, a exigência e a parcela do país que ela leva, a resposta esperada da IA com seus termos, e
  quem entraria e por qual cláusula.
- **Como é na vida real:** a escalada anda por gestos públicos (nota, ultimato, mobilização,
  declaração). O prazo do ultimato foi de 48 horas (Áustria à Sérvia, 1914) a meses (Sudetos,
  1938); no jogo vira semanas, porque a mobilização, que é o que pesa, já leva semanas.
- **Estado:** desenhado e decidido (fatia B da fase 5, ramo `feat/war-crisis`, ainda sem PR).
- **Onde está o detalhe:** "Crise e declaração" em `war.pt-BR.md`, que entra com a fatia B;
  [`war.pt-BR.md`, "Como começa e como acaba"](../../war.pt-BR.md#como-começa-e-como-acaba).

### A IA cede à força, mas nunca cede a existência

- **Decisão:** como alvo, a IA pesa a força contra o que se pede. Uma exigência periférica cede a
  partir de três vezes a força em terra de quem exige. A razão exigida sobe com a parcela do país
  que a exigência leva. Uma exigência existencial (anexação total, fim do Estado, a capital, a
  maior parte da gente ou do território) nunca é cedida, qualquer que seja a razão.
- **O que o autor pensou:** "Com três vezes a força, cede: é basicamente o que diz Sun Tzu. Mas,
  diante da possível perda da nação, não se cede; uma anexação total não é cedida. A Crimeia em 2014
  faz sentido nessa lógica, e a primeira guerra árabe-israelense também tem de fazer."
- **Como o jogo funciona por causa disso:** um vizinho muito mais forte que exige uma província
  periférica leva sem guerra; o mesmo vizinho que exige o país inteiro ouve não e tem de lutar. A
  tela mostra a razão de força, a parcela pedida e a curva.
- **Como é na vida real:** a Crimeia em 2014, parcela pequena e periférica diante de força muito
  maior, foi cedida sem guerra; Israel em 1948, diante de uma ameaça existencial, recusou contra
  força maior. Os dois são testes de jogo.
- **Estado:** decidido (W3); desenhado no ramo `feat/war-crisis`. A IA da fatia B só responde: não
  exige nem declara; isso é da fase 6.
- **Onde está o detalhe:** "Crise e declaração" em `war.pt-BR.md`, com a fatia B.

### Quem entra, e por qual cláusula

- **Decisão:** na declaração, entra quem as cláusulas obrigam: quem tem defesa com o atacado, ou
  ofensiva com quem declarou. Quem tem neutralidade com um lado não entra contra ele; a consulta dá
  aviso sem obrigar. Quem é chamado escolhe honrar ou trair, e trair custa caro.
- **O que o autor pensou:** "O aliado entra somente quando as cláusulas preverem isso." "Quebrar o
  tratado é possível, mas tem um preço muito alto."
- **Como o jogo funciona por causa disso:** o jogador chamado recebe uma entrada com prazo e duas
  saídas, honrar ou não honrar, com o preço à vista. Na fatia B a IA honra sempre; a escolha da IA é
  da fase 6. A página [Alianças](aliancas.md) traz os nomes e os casos.
- **Estado:** decidido (W4); desenhado. O valor de cada termo do preço ainda não está fixado.
- **Onde está o detalhe:** [Alianças](aliancas.md#o-aliado-só-entra-quando-a-cláusula-prevê).

### O empurrão histórico age só na decisão de atacar

- **Decisão:** quando o trilho histórico chega a um marco de guerra, ele abre um casus belli e põe
  pressão popular na IA. Nunca mexe no resultado militar.
- **O que o autor pensou:** decisão do autor de 2026-11-16 sobre o trilho histórico.
- **Como o jogo funciona por causa disso:** o jogador que controla o país recebe só o casus belli,
  sem multa por não usar. Frentes, logística e paz seguem as mesmas regras para todos.
- **Estado:** desenhado (fase 7).
- **Onde está o detalhe:** [`history-rails.pt-BR.md`, "Casus belli e pressão popular"](../../history-rails.pt-BR.md#casus-belli-e-pressão-popular-o-empurrão-na-guerra).

## A guerra

### Objetivo por frente, e a doutrina como estado do exército

- **Decisão:** o jogador dá objetivo à frente (toda a fronteira contínua com um inimigo); os
  segmentos só lutam. A geração da doutrina (2ª ou 3ª, na leitura de William Lind) é estado do
  exército, com causa, e muda como a frente se reparte e avança.
- **O que o autor pensou:** o autor pediu a leitura de Lind sobre as gerações da guerra: a 2ª
  marcada pela conquista de território e pelo avanço e recuo, a 3ª pela ação independente das
  frações, com foco nos centros de gravidade.
- **Como o jogo funciona por causa disso:** um exército de 2ª geração ganha por atrito, segmento a
  segmento; um de 3ª concentra no ponto fraco rumo ao centro de gravidade. Sem rádio ou sem
  combustível, a 3ª degrada para a 2ª.
- **Estado:** desenhado; contrato e funções fora do tick no jogo (#64). O combate é a fatia C.
- **Onde está o detalhe:** [`war.pt-BR.md`, "Doutrina"](../../war.pt-BR.md#doutrina-as-gerações-da-guerra)
  e ["Decisões do dono"](../../war.pt-BR.md#decisões-do-dono).

### Tipos com gerações, em poucos tipos

- **Decisão:** um tipo de unidade é um papel que atravessa o século; a tecnologia cria geração do
  mesmo tipo, não tipo novo. 33 tipos em quatro domínios (terra, mar, ar, estratégico).
- **O que o autor pensou:** recomendação aceita (decisão 5 da guerra).
- **Como o jogo funciona por causa disso:** em qualquer década o jogador lida com no máximo uns 20
  tipos, porque as gerações substituem umas às outras e os tipos tardios chegam quando os antigos
  saem de uso.
- **Estado:** catálogo no jogo, fora do tick (#64); `data/military/units.ron`.
- **Onde está o detalhe:** [`war.pt-BR.md`, "Tipos e gerações"](../../war.pt-BR.md#tipos-e-gerações).

### A conta da guerra sempre à vista

- **Decisão:** mortos e inválidos por idade na pirâmide, civis também, e uma conta da guerra com
  gente, bens, dinheiro e dívida, produção perdida e preços. A exaustão cresce com as semanas mesmo
  sem derrota.
- **O que o autor pensou:** o autor pediu a decisão com o capítulo II de *A Arte da Guerra*: o
  exército custa todo dia, e nenhum país ganhou com guerra prolongada.
- **Como o jogo funciona por causa disso:** a guerra longa dói também em quem está ganhando. A paz
  é negociada pelas exigências da crise, quando o valor de continuar fica abaixo do oferecido; não
  há pontuação de guerra.
- **Como é na vida real:** Sun Tzu, cap. II (trad. Giles, 1910, domínio público); a pirâmide
  francesa depois de 1914–18 é o teste contrafactual previsto.
- **Estado:** desenhado; funções da conta fora do tick (#64).
- **Onde está o detalhe:** [`war.pt-BR.md`, "A conta da guerra"](../../war.pt-BR.md#a-conta-da-guerra).

### Arma nuclear: estoque e dissuasão primeiro, uso por último

- **Decisão:** estoque e dissuasão entram na fase 5, medidos como segundo golpe; o uso, numa fatia
  própria no fim. A dissuasão age na escada de crise (que degrau cada lado pode subir), não no
  combate.
- **O que o autor pensou:** o autor concordou e pediu que a dissuasão se apoiasse nos trabalhos
  sobre a diplomacia dos mísseis (Wohlstetter, Brodie, Schelling, Kahn).
- **Estado:** desenhado.
- **Onde está o detalhe:** [`war.pt-BR.md`, "Arma nuclear"](../../war.pt-BR.md#arma-nuclear-dissuasão-medida).
