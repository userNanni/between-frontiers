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
- **Estado:** camadas no save (#109); fluxo mensal e remessa semanal no #117.
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

### O dono estrangeiro reinveste 40 % do lucro onde a fábrica está

- **Decisão:** da parte do lucro positivo que é do dono estrangeiro, 60 % vai para o fundo dele em
  casa. Os outros 40 % ficam no fundo do país de destino como crédito do dono e pagam primeiro as
  fábricas novas dele naquele país. O que não acha obra volta para casa aos poucos, em cerca de um
  ano.
- **O que o autor pensou:** o orquestrador pediu para conferir se remeter todo o lucro toda semana
  era o real ("lucro reinvestido no destino é parte grande do investimento direto"). Com a fonte, o
  valor de desenho de 0,40 foi aceito e relatado ao dono.
- **Como o jogo funciona por causa disso:** a filial cresce com o próprio lucro. O país que recebe
  não vê sair todo mês um lucro que na vida real em boa parte ficava lá.
- **Como é na vida real:**
  - cerca de um terço do lucro do investimento direto no mundo foi reinvestido em 2005–2011, e
    49 % nas economias em desenvolvimento em 2011 (UNCTAD, *World Investment Report 2013*);
  - nas filiais de empresas americanas foi de 55 % a 66 % em 2000–2003 (BEA);
  - a parcela cai numa crise e chegou a ser negativa com a anistia fiscal americana à repatriação
    de 2018. Esses choques ficam para o equilíbrio.
- **Estado:** no #117 (`foreign_capital::REINVESTED`).
- **Onde está o detalhe:** [seção 8](../../foreign-capital.pt-BR.md#8-o-fluxo-no-tick-passo-3).

### As fábricas da metrópole na independência ficam como capital estrangeiro

- **Decisão (C6, 2026-11-17):** na independência, a fábrica da metrópole não passa ao país novo no
  dia; vira capital estrangeiro, e o que acontece depois sai das leis e dos acordos do país novo.
  - **Acordo sem cláusula:** a ex-metrópole fica com as fábricas privadas não agrícolas, menos a
    parte dos donos locais, como camada estrangeira, sem garantia.
  - **Acordo com a cláusula de fábricas coloniais** (`ColonialPlants`): o acordo fixa a parte que
    fica com a ex-metrópole, o prazo da garantia contra estatização e o prazo em que o capital dela é
    tratado como nacional. Com parte zero e uma dívida no mesmo acordo, o país novo compra as
    fábricas ou indeniza.
  - **Revolta sem acordo:** a fábrica e a mina também viram camada, sem garantia, e o país novo
    decide pelas suas leis. Com o capital fechado ou a economia planificada, estatiza na hora, sem
    pagamento, com memória de expropriação e prêmio de risco.
  - **Terra agrícola:** sempre fora das camadas; fica com quem mora na região.
  - **Indenização posterior:** um tratado com dívida entre o país que estatizou e o antigo dono
    reduz o prêmio de risco na proporção do que foi **de fato pago**. A dívida pode ser renegociada
    pela revisão de dívida e pode não ser cumprida.
- **O que o autor pensou:** ele não quis decidir antes do estudo: "Veja como isso aconteceu na
  história; ela vai nos revelar quais fluxos são possíveis." Com o estudo na mão, aceitou as cinco
  partes. Lembrou ainda que a indenização precisa poder ser renegociada e que um tratado pode não
  ser cumprido.
- **Como o jogo funciona por causa disso:** a Índia que sai por acordo continua com as fábricas
  britânicas, que remetem lucro para Londres até uma lei indiana mudar isso. Uma colônia que sai por
  revolta e nasce com a economia planificada toma as fábricas no primeiro dia e paga com a memória
  de expropriação: o capital de fora cobra prêmio de risco por anos. Uma indenização acertada e
  paga em parcelas baixa esse prêmio aos poucos; uma moratória congela a queda; um perdão reduz a
  dívida, mas não conta como pago.
- **O preço de não cumprir:** a quebra custa pela regra geral dos tratados (registro, suspensão
  recíproca, causa, reputação; ver [Tratados e acordos](tratados-e-acordos.md#quebrar-é-possível-e-paga-na-próxima-mesa)).
  No capital, custa também o veto: a ex-metrópole pode fechar o capital entre os dois sem quebrar
  nada, por suspensão recíproca. E a memória de expropriação volta a pesar na parte não paga.
  Estatizar dentro do prazo da garantia é expropriação sem pagamento mais quebra da garantia.
- **Como é na vida real:** em 16 independências estudadas (Índia, Indonésia, Argélia, Congo,
  Quênia, Zimbábue e outras), o dono estrangeiro ficou de 5 a 30 anos. A estatização veio com uma
  ruptura política posterior, não na data da independência. A terra caiu rápido; a mina, devagar (o
  petróleo argelino ficou francês por 8,6 anos). Quase toda estatização terminou num acordo de soma
  fixa, abaixo do pedido e pago em parcelas: a Indonésia acertou em 1966 uma soma paga em 30
  parcelas a 1 %.
- **Estado:** decidido; acordo sem cláusula (C6a), revolta sem garantia (C6c) e estatização pela
  lei no passo 4a, atrás da chave `foreign_contract`, desligada.
- **Onde está o detalhe:** [`foreign-capital.pt-BR.md`, seção 5](../../foreign-capital.pt-BR.md#5-decisões-do-dono)
  e seção 7, "As fábricas da metrópole na independência" (com o PR #118);
  [seção 2, "As fábricas das colônias"](../../foreign-capital.pt-BR.md#as-fábricas-das-colônias).

### A parte dos donos locais segue a densidade de capitalistas da colônia

- **Decisão (C7, 2026-11-17):** na independência, a parte das fábricas privadas que fica com os
  donos da própria colônia é a densidade de capitalistas da colônia sobre a da metrópole, até 1. O
  resto vira camada da ex-metrópole.
- **O que mudou:** a regra da C6 dizia "menos a parte dos donos locais", e o desenho levado dizia
  que essa parte era "a fração do dividendo que, no último ano, ficou com os capitalistas da
  própria região". No código isso virava tudo ou nada: a divisão semanal do lucro dá o dividendo
  inteiro de um estado aos capitalistas dele se houver um só. Quase toda colônia tem algum
  capitalista, e nenhuma camada nascia.
- **O que o autor pensou:** aceitou a proposta de ler quem pode ser dono em vez de para onde vai o
  dividendo: uma colônia com tantos capitalistas por habitante quanto a metrópole é dona das suas
  fábricas; uma com um décimo deles, de um décimo.
- **Como o jogo funciona por causa disso:** a Nigéria que sai por acordo em 1925 deixa 2.137
  níveis britânicos em 108 fábricas; uma colônia com tantos capitalistas por habitante quanto a
  metrópole sai sem nenhuma camada.
- **Estado:** decidido; entra com o passo 4a (`foreign_stakes::local_owners`).
- **Onde está o detalhe:** [`foreign-capital.pt-BR.md`, seções 5 e 6](../../foreign-capital.pt-BR.md#5-decisões-do-dono).
