# Tratados e acordos

Como os países fazem acordos, o que um acordo faz no mundo e o que acontece quando alguém não
cumpre. O detalhe está em [`treaties.pt-BR.md`](../../treaties.pt-BR.md).

A ideia em uma frase: **um acordo é um conjunto de cláusulas, cada uma com quem dá e quem recebe**,
assinado por partes que o aprovaram por dentro, com prazo e um teste medido de cumprimento. Paz,
independência, reconhecimento, sujeição e aliança não são tipos de tratado: são acordos com certas
cláusulas.

## Decisões

### Cláusula é o átomo, acordo é o envelope

- **Decisão:** não há tipos fixos de tratado. Há um catálogo de cláusulas (ceder estados, criar
  dívida, pagar, reconhecer, defender, base, tarifa, moeda, tributo, política externa e outras), e
  o acordo junta as que as partes quiserem, com prazo, renovação e aviso de denúncia.
- **O que o autor pensou:** o autor pediu a qualidade da lógica de acordos dos jogos do gênero como
  referência, sem copiar. O nome do dado (`clauses.ron`, `Clause`) foi escolhido pela coordenação
  por ser o nome jurídico, sem o vocabulário de nenhum jogo.
- **Como o jogo funciona por causa disso:** a mesa de negociação tem duas colunas, o que cada lado
  dá e o que pede. Uma paz como a de 1919 é ceder estados, criar país, criar dívida, limitar armas,
  desmilitarizar e ocupar até a dívida ser paga, tudo no mesmo acordo. Cláusula de uma frente que
  ainda não existe (defesa, base, tarifa) pode ser assinada e fica dormente, com "sem efeito até a
  fase 5" na tela.
- **Como é na vida real:** os acordos do século juntam muita coisa: o Iraque (1932) e o Egito
  (1922) viraram soberanos mantendo bases, trânsito e aliança; as Filipinas (1946) e a Argélia
  (1962) nasceram com comércio, moeda e bases presos ao antigo dono.
- **Estado:** contrato e os 58 tratados de 1925 convertidos em cláusulas no jogo (#92, #93); a mesa
  de negociação, desenhada.
- **Onde está o detalhe:** [seção 3](../../treaties.pt-BR.md#3-o-acordo),
  [seção 4](../../treaties.pt-BR.md#4-catálogo-de-cláusulas),
  [seção 13](../../treaties.pt-BR.md#13-interface-a-mesa-de-negociação); `data/diplomacy/clauses.ron`.

### Dívida entre Estados com nome e calendário

- **Decisão:** a dívida de um governo a outro fica num livro com devedor e credor nomeados, moeda,
  juro e calendário, revisável por outro acordo. É a fonte da verdade: o crédito de cada país é a
  soma do livro.
- **O que o autor pensou:** decisão técnica da coordenação (seção 15, decisão 1), que serve ao
  acréscimo do autor de que acordos podem criar dívida (paz com reparação, independência,
  reconhecimento).
- **Como o jogo funciona por causa disso:** toda semana o Tesouro do devedor paga juro e anuidade ao
  do credor; com moeda fraca, pesa mais. A dívida do livro conta no risco e no limite de calote. Uma
  moratória pode ser dada por um credor só, e um perdão vale só para quem o aceitou.
- **Como é na vida real:** a reparação alemã foi refeita várias vezes em treze anos (Londres 1921,
  Dawes 1924, Young 1929, Hoover 1931, Lausanne 1932): mudava o calendário, não o total. O gargalo
  era transferir divisas, e o Plano Dawes protegia a transferência.
- **Estado:** no jogo (#101; ponte com as fronteiras em #104).
- **Onde está o detalhe:** [seção 5](../../treaties.pt-BR.md#5-dívida-entre-estados).

### As dívidas de guerra de 1925 entram no cenário

- **Decisão:** a partida começa com a reparação alemã e as dívidas interaliadas no livro, além da
  dívida genérica de todos os países.
- **O que o autor pensou:** decisão do autor (seção 15, decisão 6), que aceitou a recomendação. O
  efeito no placar da Alemanha, do Reino Unido e da França fica anotado como dependência, não como
  meta ([Princípios](../principios.md#não-forçar-o-resultado-histórico-que-dependeu-de-política)).
- **Como o jogo funciona por causa disso:** em 1925 a Alemanha deve aos credores pela reparação, e o
  Reino Unido deve aos Estados Unidos; os pagamentos correm no passo semanal e pesam no orçamento.
- **Estado:** no jogo (#101), em `data/scenario/1925/war_debts.ron`, ligado pela chave
  `war_debts_1925`.
- **Onde está o detalhe:** [seção 15, decisão 6](../../treaties.pt-BR.md#15-decisões-decidido-em-2026-11-17).

### Reconhecimento é uma relação de cada país com cada país

- **Decisão:** o jogo guarda só os pares que **não** se reconhecem. O país novo nasce reconhecido só
  por quem assinou a criação dele.
- **O que o autor pensou:** decisão do autor (seção 15, decisão 4), que aceitou a recomendação.
- **Como o jogo funciona por causa disso:** com quem não o reconhece, um país só pode assinar o
  próprio reconhecimento e pagamentos: sem aliança, sem tarifa, sem base, sem empréstimo de
  governo. O mercado anônimo de títulos continua. A tela mostra quantos países reconhecem e quanto
  do PIB mundial eles somam.
- **Como é na vida real:** a URSS ficou sem crédito americano até o reconhecimento de 1933; quem
  reconhecia a República da China caiu em ondas depois da ONU (1971) e dos Estados Unidos (1979).
- **Estado:** a lista de quem assinou a criação no jogo (#95); o que o reconhecimento bloqueia, em
  andamento no thread dos tratados.
- **Onde está o detalhe:** [seção 6](../../treaties.pt-BR.md#6-reconhecimento).

### Sujeição é uma escada de cláusulas, e o súdito sai por condição medida

- **Decisão:** colônia da coroa, protetorado, mandato, domínio e satélite são nomes de feixes de
  cláusulas. O súdito sai negociando cláusula por cláusula, ou por denúncia, que só vale quando a
  pressão de saída passa o controle do suserano. Denúncia contra controle maior vira crise quando a
  guerra existir; até lá, é recusada com a conta na tela.
- **O que o autor pensou:** decisão do autor (seção 15, decisão 3). O caso que ele trouxe: jogar com
  a Polônia sob a Rússia e se separar.
- **Como o jogo funciona por causa disso:** o jogador que escolhe um súdito vê dois números, o
  controle do suserano (guarnição, dependência, menos a crise do suserano) e a pressão de saída
  (movimento de independência, grupos de interesse, patrono). Ele age nos termos que são dele:
  paga a dívida, muda o comércio, aprova leis que os grupos apoiam, procura um patrono. Quando a
  pressão passa o controle, denuncia.
- **Como é na vida real:** os domínios britânicos perderam o vínculo cláusula por cláusula
  (Westminster 1931). O súdito saiu quando o suserano estava em crise e não havia tropa dele em
  casa: a Iugoslávia, sem tropas soviéticas, saiu em 1948; a Hungria (1956) e a Tchecoslováquia
  (1968), com tropas, não.
- **Estado:** desenhado (etapa 5 dos tratados); o vínculo de súdito e o suserano já existem no jogo
  (#95).
- **Onde está o detalhe:** [seção 7](../../treaties.pt-BR.md#7-súditos).

### A IA aceita porque a conta fecha, e mostra a conta

- **Decisão:** cada cláusula vale, para cada lado, uma soma de termos com nome (dinheiro, comércio,
  território, segurança, soberania, política interna, confiança). O que não é dinheiro vira dinheiro
  por pesos declarados. A mesma função avalia para a IA e para o jogador, sem sorteio.
- **O que o autor pensou:** decisão técnica da coordenação (seção 15, decisão 2).
- **Como o jogo funciona por causa disso:** na mesa, o jogador vê a resposta provável do outro lado
  e por quê ("indústria pesada contra, pelo imposto"). A mesma proposta, no mesmo mundo, tem sempre
  a mesma resposta. A influência entra como linha própria, com teto
  ([Diplomacia e influência](diplomacia-e-influencia.md#o-primeiro-retorno-é-o-sim-numa-proposta)).
- **Estado:** desenhado (etapa 4 dos tratados).
- **Onde está o detalhe:** [seção 8](../../treaties.pt-BR.md#8-como-se-chega-a-um-acordo) e
  [seção 10](../../treaties.pt-BR.md#10-como-a-ia-avalia).

### Ratificar contra a vontade de dentro custa legitimidade

- **Decisão:** o acordo passa pela política interna. A IA não ratifica abaixo do limiar; o jogador
  pode, pagando legitimidade.
- **O que o autor pensou:** decisão do autor (seção 15, decisão 5), que aceitou a recomendação.
- **Como o jogo funciona por causa disso:** numa república com parlamento, os grupos de interesse
  pesam pela mesma conta das leis; num partido único, só os grupos com força fora do voto; a
  monarquia absoluta ratifica sozinha. Assinar contra a oposição baixa a legitimidade e dá causa ao
  movimento revisionista.
- **Como é na vida real:** o Senado americano recusou Versalhes.
- **Estado:** desenhado (etapa 4 dos tratados).
- **Onde está o detalhe:** [seção 8, "Quem aprova dentro do país"](../../treaties.pt-BR.md#quem-aprova-dentro-do-país).

### Quebrar é possível, e paga na próxima mesa

- **Decisão:** a quebra é o teste mensal da cláusula falhar. Ela fica registrada, permite ao outro
  lado suspender o que dava em troca, dá a ele uma causa contra quem quebrou e pesa na reputação,
  que desconta as promessas futuras.
- **O que o autor pensou:** "Quebrar o tratado é possível, mas tem um preço muito alto." O caso das
  alianças está em [Alianças](aliancas.md#trair-é-possível-e-custa-muito-caro).
- **Como o jogo funciona por causa disso:** a IA quebra quando o valor de quebrar passa o custo, e a
  tela mostra essa conta à vítima. Denunciar pelo prazo e aviso do próprio acordo não é quebra.
- **Como é na vida real:** promessa sem sanção quebra quando o equilíbrio muda (a não agressão
  polaco-soviética de 1932, rompida em 1939; Locarno, rompido com a Renânia em 1936).
- **Estado:** desenhado; o preço próprio da traição na guerra, em desenho.
- **Onde está o detalhe:** [seção 9](../../treaties.pt-BR.md#9-quando-se-quebra).

### Toda dívida de acordo se renegocia, e todo acordo pode não ser cumprido

- **Decisão:** duas regras gerais, que valem para qualquer acordo: a dívida criada por um acordo
  (reparação, indenização, compra) pode ser renegociada por outro acordo de revisão (prazo, juro,
  parte adiável, perdão parcial; só o credor concede); e qualquer parte pode deixar de cumprir, com
  o custo da quebra (registro, suspensão recíproca, causa, reputação).
- **O que o autor pensou:** o autor lembrou, ao decidir as fábricas da colônia na independência, que
  a dívida de indenização precisa poder ser renegociada e que um tratado pode não ser cumprido. O
  thread do capital tratou isso como regra geral dos tratados.
- **Como o jogo funciona por causa disso:** a reparação alemã pode ser revista como foi na vida real
  (Dawes, Young, Hoover, Lausanne); uma moratória suspende os pagamentos sem quebra; um país que para
  de pagar atrasa, dá calote parcial ou repudia, cada grau medido. Quem lê a dívida (o prêmio de
  risco do capital, por exemplo) olha o que foi pago, não o que foi prometido: o perdão reduz a
  dívida, mas não conta como pago.
- **Como é na vida real:** a reparação alemã refeita várias vezes em treze anos; a Indonésia acertou
  em 1966 a indenização das fábricas holandesas por uma soma abaixo do pedido, paga em 30 parcelas.
- **Estado:** a revisão de dívida por acordo (`revise_debt`, #104) e o calote ligado ao livro
  (#101) no jogo; o repúdio e o registro do que foi pago e perdoado por linha, desenhados (PR #118 e
  seção 5 dos tratados).
- **Onde está o detalhe:** [seção 5](../../treaties.pt-BR.md#5-dívida-entre-estados) e
  [seção 9](../../treaties.pt-BR.md#9-quando-se-quebra); o caso do capital em
  [Capital entre países](capital-entre-paises.md#as-fábricas-da-metrópole-na-independência-ficam-como-capital-estrangeiro).

### Promessa de ato único e de entrega acumulada

- **Decisão:** um tipo de cláusula de promessa com dois modos: um apoio numa ocasião definida, ou
  uma quantidade entregue até uma data, com saldo à vista.
- **O que o autor pensou:** "Tem coisa que se paga aos poucos ou que tem de acumular, como 'prometo
  te vender tanto de tal produto até tal data'."
- **Estado:** desenhado; detalhe em
  [Diplomacia e influência](diplomacia-e-influencia.md#favor-devido-é-uma-promessa-num-tratado-ato-único-ou-entrega-acumulada).
