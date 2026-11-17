# Capital entre países

Como o dinheiro de um país constrói fábricas em outro, o que volta para casa, quem aprende com isso
e o que acontece com as fábricas da metrópole quando a colônia fica independente. O detalhe está em
[`foreign-capital.pt-BR.md`](../../foreign-capital.pt-BR.md); o estudo de por que isso importa está
em [`late-industrialization.pt-BR.md`](../../late-industrialization.pt-BR.md).

Por que existe: a industrialização tardia de vários países (a China depois de 1978, os tigres
asiáticos) dependeu de capital de fora. Sem ele, o jogo não pode reproduzir esses casos sem forçar
o resultado ([Princípios](../principios.md#não-forçar-o-resultado-histórico-que-dependeu-de-política)).

## Decisões

### O estoque estrangeiro guarda camadas, cada uma com o seu ano e contrato

- **Decisão:** a fábrica continua sendo estado × tipo de prédio; por cima dela, camadas dizem
  quantos níveis são de quem, desde quando e com que contrato (parcela do sócio local, prazo, modo
  de passagem ao sócio).
- **O que o autor pensou:** recomendação aceita (C1).
- **Como o jogo funciona por causa disso:** uma empresa mista de 1985 segue o contrato de 1985 mesmo
  que a lei mude em 1990; no fim do prazo, só aquela camada passa ao sócio. Uma estatização mede
  quanto de cada dono foi tomado.
- **Como é na vida real:** o investimento direto chega como fábricas, cada uma com data, dono e
  contrato. Na China, o recebido no ano subiu ano a ano depois que a lei e as zonas abriram (a série
  em `data/history/fdi.csv`).
- **Estado:** camadas no save (#109); fluxo mensal e remessa semanal em rascunho (#117).
- **Onde está o detalhe:** [seção 2](../../foreign-capital.pt-BR.md#2-o-que-entra-no-estado).

### Ninguém começa com capital estrangeiro em 1925; semeia-se depois

- **Decisão:** o estoque de 1925 começa em zero. Um PR de dados semeia depois, quando houver fonte
  com licença clara.
- **O que o autor pensou:** recomendação aceita (C2), pela regra de nenhum número sem fonte.
- **Como o jogo funciona por causa disso:** até alguém investir, não há fábrica estrangeira no
  mundo. O candidato ao semeio é o estoque americano por país de destino (Lewis 1938); os donos
  britânico, francês, holandês e alemão ficam como dependência anotada.
- **Estado:** decidido; o semeio espera a fonte.
- **Onde está o detalhe:** [seção 5, decisão 2](../../foreign-capital.pt-BR.md#5-decisões-do-dono).

### O jogador controla a porta: a lei e uma lista de vetos

- **Decisão:** o jogador comanda a lei de capital estrangeiro do próprio país e uma lista curta de
  países vetados. O veto vale nos dois sentidos: não investir lá e não aceitar de lá.
- **O que o autor pensou:** recomendação aceita (C3).
- **Como o jogo funciona por causa disso:** fechar a porta a um país corta as camadas novas entre os
  dois. A lei decide quanto o estrangeiro pode ter e em que termos.
- **Estado:** decidido; o comando de veto entra num próximo passo.
- **Onde está o detalhe:** [seção 5, decisão 3](../../foreign-capital.pt-BR.md#5-decisões-do-dono).

### Remessa sem efeito no câmbio, por enquanto

- **Decisão:** o lucro remetido viaja em unidades reais, sem mexer no câmbio, até haver balanço de
  pagamentos.
- **O que o autor pensou:** recomendação aceita (C4).
- **Como o jogo funciona por causa disso:** um país que recebe muito capital não valoriza a moeda
  por isso, por enquanto; fica anotado.
- **Estado:** decidido.

### A fábrica estrangeira ensina: técnica como efeito de primeira ordem

- **Decisão:** a fábrica estrangeira espalha técnica no país que a recebe, mais ainda quando a lei
  exige entrega ao sócio local. A tela mostra quanto da difusão vem dela.
- **O que o autor pensou:** "Isso é muito importante para o processo de globalização."
- **Como o jogo funciona por causa disso:** a aba de pesquisa mostra, por grupo de técnica, a
  difusão do mês em três partes: a imitação sem fábrica estrangeira, a parte das fábricas
  estrangeiras e a da entrega ao sócio. Os tigres e a China depois de 1978 devem aprender mais
  depressa conforme o investimento recebido sobe; os mesmos países com a porta fechada servem de
  contraprova.
- **Estado:** decidido (C5); entra no passo 4.
- **Onde está o detalhe:** [seção 5, decisão 5](../../foreign-capital.pt-BR.md#5-decisões-do-dono).

### As fábricas da metrópole na independência: em decisão

- **Decisão:** **em decisão com o autor (C6).** Hoje, na independência, as fábricas da colônia passam
  inteiras ao país novo, sem indenização e sem nenhum resto de propriedade da metrópole.
- **O que o autor pensou:** ele não quis decidir a regra antes do estudo: "Veja como isso aconteceu
  na história; ela vai nos revelar quais fluxos são possíveis."
- **O que o estudo achou** (PR #118, seção 7):
  - na independência negociada, o dono estrangeiro ficou de 5 a 30 anos, e a tomada veio depois de
    uma ruptura política, quase nunca na data da independência (Índia, Indonésia, Congo, Birmânia,
    Malásia);
  - na guerra com muitos colonos, a terra caiu em menos de um ano e a mina e o petróleo demoraram
    mais (Argélia);
  - a garantia do tratado tinha prazo, e o conflito veio quando ela venceu (Zimbábue, Filipinas);
  - quase toda estatização terminou num acordo de soma fixa, abaixo do pedido e pago em parcelas;
  - a metrópole conseguiu o acordo com pressão (tarifas, ajuda, dinheiro bloqueado, boicote).
- **O que está proposto ao autor:**
  1. acordo sem cláusula: a ex-metrópole fica com os níveis privados não agrícolas, menos a parte
     dos donos locais; o que vem depois sai das leis do país novo;
  2. acordo com a cláusula de fábricas coloniais: a parte que fica, o prazo da garantia e o prazo de
     tratamento igual ao nacional; com parte zero e dívida no mesmo acordo, o país novo compra as
     fábricas;
  3. revolta sem acordo: a camada nasce sem garantia, e o país novo estatiza pelas próprias leis; a
     alternativa é a revolta não deixar camada;
  4. terra agrícola fica fora das camadas, com quem mora na região.
- **Como é na vida real:** os casos do estudo, com fonte, estão no PR #118.
- **Estado:** em decisão.
- **Onde está o detalhe:** [seção 2, "As fábricas das colônias"](../../foreign-capital.pt-BR.md#as-fábricas-das-colônias)
  e a seção 7 do mesmo documento, que entra com o #118.
