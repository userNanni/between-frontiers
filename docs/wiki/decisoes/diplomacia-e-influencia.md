# Diplomacia e influência

Como um país pesa nas decisões de outro, de onde vem esse peso e por que não dá para agradar todo
mundo ao mesmo tempo. O desenho completo está em [`influence.pt-BR.md`](../../influence.pt-BR.md);
aqui ficam as decisões e o efeito para quem joga.

A ideia geral: **influência de A sobre B é quanto B perde se A virar as costas, mais quanto B deve
a A pelo que A fez por ela.** Não é uma moeda: nasce de atos que custam (emprestar, investir,
comerciar, ajudar, proteger) e compra coisas medidas (o sim numa proposta, o voto, o lado numa
crise).

## Decisões

### Influência em duas partes: dependência e boa vontade

- **Decisão:** a influência tem duas partes. A **dependência** é o que B perde hoje se os laços com
  A forem cortados, recalculada todo mês, sem memória. A **boa vontade** é o que A fez por B (ou
  contra B), e decai devagar, com meia-vida. A afinidade de regime e de gente multiplica as duas.
- **O que o autor pensou:** o autor pediu que a coordenação conferisse se a diplomacia funciona
  mesmo assim e o que dizem os teóricos, já que os jogos do gênero somam uma "opinião" a um "favor
  devido". Ele aceitou as duas partes depois da conferência (seção 11 do documento).
- **Como o jogo funciona por causa disso:** se A corta o crédito a B, a dependência de B some no
  mês seguinte; a ajuda que A deu continua pesando, cada vez menos. A tela mostra os canais de cada
  par ("crédito 3,1 % do PIB, ajuda há 4 anos, regime próximo" é o exemplo do documento).
- **Como é na vida real:** quando a Iugoslávia rompeu com Moscou em 1948, o comércio e o crédito
  soviéticos cessaram, e o que a prendia acabou junto. A gratidão é pequena e passa: a ajuda
  americana aos membros temporários do Conselho de Segurança sobe no mandato e cai logo depois, e o
  voto acompanha (Kuziemko e Werker 2006). Na teoria, a boa vontade é a reciprocidade difusa de
  Keohane (1986).
- **Estado:** no jogo como contrato, fora do tick (#105); no tick (#110): o mapa por par é
  recalculado todo mês, a boa vontade decai em janeiro, e as duas entram no save (formato 7).
- **Onde está o detalhe:** [seção 2](../../influence.pt-BR.md#2-como-é-na-vida-real-e-se-a-diferença-importa),
  [seção 3](../../influence.pt-BR.md#3-os-canais-o-que-a-faz-e-quanto-vale-em-b) e
  [seção 11](../../influence.pt-BR.md#11-conferência-com-a-teoria-pedido-do-dono-na-decisão-1);
  pesos em `data/diplomacy/influence.ron`.

### Dependência é vulnerabilidade, não tamanho do fluxo

- **Decisão:** cada canal pesa o fluxo vezes a **concentração**, a parte de A no total daquele canal
  em B. Quem está fora do mercado de crédito (calote recente) depende ainda mais.
- **O que o autor pensou:** "Há países mais isolados que outros, e alguns fluxos são maiores que
  outros." A concentração responde a isso: o país isolado, ou de fluxos pequenos, depende pouco de
  qualquer um.
- **Como o jogo funciona por causa disso:** o mesmo empréstimo pesa mais em quem deve só a A do que
  em quem deve a dez credores. Um país grande com dívidas espalhadas não fica refém de ninguém.
- **Como é na vida real:** Keohane e Nye separam a sensibilidade (o custo imediato) da
  vulnerabilidade (o custo que sobra depois de procurar outro parceiro). Hirschman (1945) mostrou
  como a Alemanha dos anos 1930 prendia os Bálcãs comprando com marcos que só valiam na Alemanha.
- **Estado:** decidido; desenhado (seção 13, decisão 1).
- **Onde está o detalhe:** [seção 11](../../influence.pt-BR.md#11-conferência-com-a-teoria-pedido-do-dono-na-decisão-1),
  [seção 13](../../influence.pt-BR.md#13-decisões-que-sobraram-dos-dois-estudos-decididas-em-2026-11-17).

### Nenhum botão de gastar influência

- **Decisão:** não existe ação "melhorar relações" nem moeda abstrata de diplomacia. A influência só
  nasce de atos que custam no Tesouro, no livro de dívidas ou nas cláusulas de um tratado.
- **O que o autor pensou:** recomendação da coordenação que o autor aceitou (seção 9, decisão 3).
- **Como o jogo funciona por causa disso:** para ganhar peso sobre um país, o jogador empresta (e
  pode perder o dinheiro), doa (e não volta), promete defender (e vai ter de ir). Ajuda e crédito
  saem da mesa de negociação dos tratados. Sem nenhum ato, a influência de todos é só a estrutural.
- **Como é na vida real:** a influência que se vê nos casos do documento sempre veio de laços com
  custo: crédito, comércio, bases, ajuda.
- **Estado:** decidido; vale desde o contrato (#105).
- **Onde está o detalhe:** [seção 9](../../influence.pt-BR.md#9-decisões-do-dono-decididas-em-2026-11-17).

### Canais que ainda não existem ficam dormentes

- **Decisão:** o comércio entre parceiros e o investimento estrangeiro só contam quando existirem no
  jogo. Até lá aparecem como dormentes, sem estimativa.
- **O que o autor pensou:** recomendação aceita (seção 9, decisão 2). A alternativa, estimar o
  comércio por gravidade, daria um número com cara de medida que não é medida.
- **Como o jogo funciona por causa disso:** a lente de influência mostra "comércio: sem efeito até o
  comércio ter parceiro". Quando o comércio com parceiro entrar, o canal liga sozinho.
- **Estado:** decidido. Comércio com parceiro em rascunho (#113); capital entre países em rascunho
  (#109).
- **Onde está o detalhe:** [seção 3](../../influence.pt-BR.md#3-os-canais-o-que-a-faz-e-quanto-vale-em-b).

### O primeiro retorno é o sim numa proposta

- **Decisão:** a influência entra como uma linha própria ("influência de A: +X") na conta que o
  outro país faz para aceitar ou recusar uma proposta de tratado, com teto.
- **O que o autor pensou:** recomendação aceita (seção 9, decisão 4). Pelo princípio do documento,
  o que não tem retorno medido não entra.
- **Como o jogo funciona por causa disso:** a mesma proposta de A a B pode passar de recusa a aceite
  quando A é credor de B, e a tela mostra a linha. O teto impede que a influência compre qualquer
  perda: ela faz um país aceitar uma perda pequena por um patrono, nunca qualquer perda. Voto em
  organização, acesso a recurso e lado numa crise entram com as frentes de que dependem.
- **Estado:** o mapa por par entra no tick em #110, com o teto de 250‰ do tamanho da proposta; a
  avaliação de proposta no tick é a etapa 4 dos tratados.
- **Onde está o detalhe:** [seção 4](../../influence.pt-BR.md#4-o-retorno-o-que-a-influência-compra).

### Influência vale por assunto, e a afinidade pesa pouco

- **Decisão:** o peso de A sobre B é medido por assunto (dinheiro, comércio, território, segurança,
  soberania). A afinidade de regime e de gente tem peso pequeno. A ameaça é um canal negativo,
  dormente até a guerra e a IA estratégica.
- **O que o autor pensou:** saiu da conferência com a teoria que ele pediu; ele aceitou as três
  mudanças (seção 13, decisão 3).
- **Como o jogo funciona por causa disso:** emprestar a um país dá peso numa renegociação de dívida,
  não compra a aliança dele. Regimes parecidos facilitam, mas não decidem.
- **Como é na vida real:** Baldwin (1979) critica tratar o poder como moeda que serve em qualquer
  assunto. Walt (1987): os países se alinham contra quem ameaça, e a ideologia pesa menos do que se
  esperava.
- **Estado:** decidido; desenhado.
- **Onde está o detalhe:** [seção 11](../../influence.pt-BR.md#11-conferência-com-a-teoria-pedido-do-dono-na-decisão-1).

### Por que não se agrada todo mundo ao mesmo tempo

- **Decisão:** a diplomacia tem cinco limites medidos, no lugar de uma contagem de diplomatas:
  1. **serviço exterior:** cada par tem nenhuma missão, legação ou embaixada; a missão custa no
     orçamento e ocupa gente qualificada; sem missão, o ato rende metade e a negociação demora o
     dobro;
  2. **negociações simultâneas:** poucas, pelo tamanho do serviço; a eficiência da administração
     (informática) multiplica;
  3. **rivalidade e exclusividade:** ajudar B custa boa vontade em C quando B e C são hostis;
     reconhecer uma China exclui a outra;
  4. **promessa a muitos vale menos:** cada defesa prometida dilui as outras
     ([Alianças](aliancas.md#promessa-a-muitos-vale-menos));
  5. **custo interno:** aproximar-se de um regime que os grupos de interesse rejeitam custa
     legitimidade em casa.
- **O que o autor pensou:** "Não posso melhorar a minha relação com todo mundo ao mesmo tempo.
  Quais são esses limites?" Ele pediu o limite real por trás dos diplomatas dos jogos do gênero e
  aceitou os cinco.
- **Como o jogo funciona por causa disso:** um país pequeno abre poucas negociações de cada vez e
  escolhe onde ter embaixada; uma potência pode mais, mas paga a folha. A terceira negociação de um
  serviço pequeno espera vaga. A mesma aproximação custa legitimidade numa república com grupos
  contra e não custa numa autocracia alinhada.
- **Como é na vida real:** em 1925 muitos países tinham legação em poucas capitais, e só as
  potências trocavam embaixadas; a rede cresceu com a descolonização e com o orçamento. O custo
  interno é o jogo de dois níveis de Putnam (1988); a diluição, o dilema da aliança de Snyder (1984).
- **Estado:** decidido; serviço exterior, negociações, rendimento do ato e custo interno como
  contrato, fora do tick (#112); rivalidade e diluição desenhadas.
- **Onde está o detalhe:** [seção 12](../../influence.pt-BR.md#12-por-que-não-se-melhora-a-relação-com-todo-mundo-ao-mesmo-tempo-pedido-do-dono-na-decisão-3);
  bloco `service` de `data/diplomacy/influence.ron` no #112.

### Missões de 1925 por regra, sem o dado do Correlates of War

- **Decisão:** as missões diplomáticas de 1925 saem de uma regra: as maiores economias trocam
  embaixadas entre si e têm legação em toda capital soberana; os outros têm legação com os vizinhos
  e com o suserano.
- **O que o autor pensou:** recomendação aceita (seção 13, decisão 4), pela regra de dados do
  projeto: o dado de troca diplomática do Correlates of War cobre 1925, mas os termos de uso proíbem
  redistribuí-lo sem permissão escrita, e o repositório é público.
- **Como o jogo funciona por causa disso:** em 1925, Reino Unido e França têm embaixada um no outro;
  Brasil e Argentina, legação; Nova Zelândia e Paraguai, nada (os testes do #112).
- **Estado:** como contrato, fora do tick (#112).
- **Onde está o detalhe:** [seção 12](../../influence.pt-BR.md#12-por-que-não-se-melhora-a-relação-com-todo-mundo-ao-mesmo-tempo-pedido-do-dono-na-decisão-3).

### Favor devido é uma promessa num tratado: ato único ou entrega acumulada

- **Decisão:** o favor devido não é um número por par: é uma cláusula de promessa, com dois modos.
  **Ato único:** um apoio numa ocasião definida (um voto, um consentimento). **Entrega acumulada:**
  uma quantidade de um bem, de dinheiro ou de homens até uma data, de uma vez ou em parcelas, com o
  saldo à vista.
- **O que o autor pensou:** "Tem coisa que se paga aos poucos ou que tem de acumular, como 'prometo
  te vender tanto de tal produto até tal data'." Foi o acréscimo dele à recomendação de tratar o
  favor como promessa.
- **Como o jogo funciona por causa disso:** a promessa "vendo-te 2 milhões de toneladas de trigo até
  1930" (o exemplo do documento) mostra uma barra com o entregue e o prazo. No vencimento sem
  cumprir, a quebra vale só a parte que faltou, e cada parcela vencida já conta. A quebra entra na
  reputação de quem prometeu.
- **Como é na vida real:** Keohane (1986) separa a reciprocidade específica (troca definida, num
  prazo) da difusa. Exemplos do documento: o petróleo romeno prometido à Alemanha, os contingentes
  prometidos ao aliado.
- **Estado:** desenhado. O ato único depende das votações de organização e da proposta no tick; a
  entrega de bem depende do comércio com parceiro (#113); a de dinheiro pode vir antes; a de homens,
  com a guerra.
- **Onde está o detalhe:** [seção 13a](../../influence.pt-BR.md#13a-a-cláusula-de-promessa-ato-único-e-entrega-acumulada).

### Bloco é rótulo sobre tratados, e o país pequeno não tem regra especial

- **Decisão:** um bloco é um conjunto de tratados com o mesmo feixe de cláusulas e um líder, não
  uma estrutura nova. O país pequeno usa as mesmas cláusulas e a mesma conta que os outros.
- **O que o autor pensou:** recomendações aceitas (seção 9, decisões 5 e 6).
- **Como o jogo funciona por causa disso:** entrar e sair de bloco são decisões raras em Agendas,
  com o que se ganha e o que se cede, confirmação e ponto de restauração. Neutralidade garantida,
  proteção, renda de posição (estreitos, canais, bases) e o leilão entre dois patronos saem da conta
  do próprio país; a tela mostra "o que o país quer".
- **Como é na vida real:** o pequeno escolhe o patrono que ameaça menos e cobra caro quando dois o
  disputam: a Iugoslávia, o Egito dos anos 1950, a Índia não alinhada (Walt 1987; Fox 1959).
- **Estado:** blocos de 1925 no contrato (#105). O bloco é lido só das cláusulas em vigor, com o
  rótulo do tipo de aliança (#132, rascunho): um par sozinho é aliança, o bloco sem líder precisa de três
  países ligados entre si (a Pequena Entente, sim; a rede de pactos da França, não). O autor
  aceitou os limiares recomendados: sem líder, a partir de 3; garantia, a partir de 2 protegidos;
  aduaneiro e monetário, a partir de 1. Entrar e sair em Agendas esperam a proposta no tick dos tratados;
  telas desenhadas.
- **Onde está o detalhe:** [seção 5](../../influence.pt-BR.md#5-blocos) ("Como o jogo reconhece um bloco nas cláusulas") e
  [seção 6](../../influence.pt-BR.md#6-o-país-pequeno).

### Quebrar um tratado baixa a boa vontade do traído

- **Decisão:** quando A quebra um tratado com B, a boa vontade de B com A cai uma vez, pelo valor da
  quebra, e decai com a mesma meia-vida das outras. A memória longa fica na reputação de A, que é
  dos tratados. Os outros membros do bloco que atenderam o mesmo chamado perdem metade disso; quem
  também não atendeu não perde nada.
- **O que o autor pensou:** "Quebrar o tratado é possível, mas tem um preço muito alto." Ele
  decidiu que a reputação é por país e lembra por muito tempo, que os parceiros de quem rompeu
  podem denunciar sem custo e que a boa vontade com o traído cai. Sobre os parceiros: "se eu
  atendi o chamado e o outro país não", perco boa vontade com ele; e pediu que a regra fosse
  conferida com a história.
- **Como o jogo funciona por causa disso:** o aliado que não veio perde peso sobre o traído em
  todos os assuntos, inclusive numa renegociação de dívida, e a lente de influência mostra "quebra
  em 1938" entre os atos do par. Quem denuncia o feixe de um bloco sai do bloco no mês seguinte,
  sem regra própria.
- **Como é na vida real:** depois de Munique (1938), Praga e Moscou deixaram de contar com Paris
  ([Alianças](aliancas.md#trair-é-possível-e-custa-muito-caro)). A reputação de cumprir é o que
  pesa na confiança entre Estados (Tomz 2007).
- **Estado:** decidido; desenhado. Lê o registro de quebras dos tratados; entra quando a quebra
  existir no tick. Os parceiros de bloco que atenderam o chamado perdem metade do que perde o
  traído; quem também não atendeu, nada (decidido pelo autor, conferido com Munique, a OTAN de
  2003 e a Liga Árabe de 1979; seção 13b). O custo de associação quando um membro agride por
  conta própria (o BRICS) não mexe na boa vontade entre parceiros: mora no canal de ameaça, que
  terceiros leem pela profundidade das obrigações do bloco (decidido pelo autor, T1).
- **Onde está o detalhe:** [seção 13b](../../influence.pt-BR.md#13b-quando-um-tratado-é-quebrado-o-que-a-influência-lê-e-escreve).

## Fontes

As referências completas estão na [seção 14 do documento](../../influence.pt-BR.md#14-fontes).
