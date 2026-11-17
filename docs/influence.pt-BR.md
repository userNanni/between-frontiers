# Influência, blocos e o país pequeno (fase 4)

Estado: desenho decidido em 2026-11-17 (seções 9 e 14); contrato fora do tick em andamento (#105). Fica entre os tratados (`treaties.pt-BR.md`:
cláusulas, livro de dívida, avaliação de proposta) e a IA estratégica da fase 6
(`history-rails.pt-BR.md`: interesses medidos). Princípios de sempre: causa visível, nada sorteado,
determinismo, e **sem retorno mensurável não entra**.

## 1. A ideia

Influência de A sobre B é **quanto B perde se A virar as costas**, mais **quanto B deve a A pelo
que A fez por ela**. Não é uma moeda que se gasta: não existe botão "comprar influência". Ela nasce
de atos medidos que já existem ou vão existir no jogo (emprestar, investir, comerciar, ajudar,
proteger, compartilhar regime e gente) e compra coisas medidas (o voto, o sim numa proposta, a
concessão, o lado numa crise).

## 2. Como é na vida real, e se a diferença importa

- **Dependência assimétrica é poder.** A Alemanha dos anos 1930 comprava a produção dos Bálcãs com
  marcos que só valiam na Alemanha: quanto mais o parceiro pequeno dependia daquele comércio, mais
  caro lhe custava contrariar Berlim (Hirschman, 1945). O que pesa é a **assimetria**: o que o
  pequeno perderia, sobre o que o grande perderia.
- **A dependência acaba com o fluxo.** Quando a Iugoslávia rompeu com Moscou em 1948, o comércio e o
  crédito soviéticos cessaram; o que segurava a Iugoslávia acabou, e ela se apoiou nos
  americanos. Essa parte não tem memória: ela é o fluxo de agora.
- **A gratidão decai devagar e é pequena.** A ajuda americana aos membros temporários do Conselho
  de Segurança sobe durante o mandato e cai logo depois, e o alinhamento de voto acompanha (Kuziemko
  e Werker, 2006). A ajuda compra voto, mas pouco e por pouco tempo. Há boa vontade que dura mais
  (o Plano Marshall nas décadas seguintes), mas sempre sustentada por laços que continuaram
  (bases, comércio, aliança).
- **Afinidade de regime e de gente é estrutural.** Regimes parecidos se alinham mais (votos na ONU
  agrupados por regime). Diáspora e cultura comum barateiam a influência: o patrono conhece o lugar,
  a língua e em quem confiar.
- **O pequeno escolhe o patrono que ameaça menos** e cobra caro quando dois o disputam: a Iugoslávia
  e o Egito dos anos 1950, a Índia não alinhada, a Turquia do pós-guerra (Walt, 1987; Fox, 1959).

**A diferença importa.** Um estoque único que sobe e desce devagar faria a Iugoslávia de 1948
continuar soviética por anos depois do rompimento e faria a ajuda comprar alinhamento para sempre.
Por isso a influência tem duas partes (decisão 1):

| Parte | O que mede | Memória |
|---|---|---|
| **Dependência** | o que B perde hoje se os laços com A forem cortados, sobre o tamanho de B | nenhuma: recalculada todo mês dos fluxos e estoques atuais |
| **Boa vontade** | o que A fez por B (ajuda, perdão de dívida, defesa cumprida) e o que fez contra (quebra, sanção) | estoque com meia-vida de 8 anos (escolha de desenho, a equilibrar depois da fase 6) |

Mais um multiplicador, **afinidade** (regime e gente), que não é influência por si: barateia ou
encarece a que os outros canais produzem.

## 3. Os canais: o que A faz e quanto vale em B

Todos medidos do estado do mundo, por par ordenado (A sobre B), em parcela do PIB de B:

| Canal | Medida | Entra em | Existe hoje? |
|---|---|---|---|
| **Crédito** | dívida de B a A no livro de dívidas entre Estados (#101), sobre o PIB de B | dependência | **sim** |
| **Investimento** | valor das plantas de B cujo dono é de A, sobre o PIB de B (`foreign_capital.rs`) | dependência | não: capital estrangeiro ainda fora do tick |
| **Comércio** | exportações de B para A, sobre o PIB de B, menos o mesmo de A para B escalado pelo PIB de A (a assimetria) | dependência | não: o comércio mundial ainda é um *pool* sem parceiro |
| **Proteção** | defesa prometida por A a B (`defence`), base de A em B (`base`), tropa de A em B | dependência | cláusulas dormentes até a fase 5 |
| **Ajuda** | doação do Tesouro de A ao de B (`payment`, `tribute` de A para B), perdão de dívida (`revise_debt` a favor de B) | boa vontade | **sim**, pelas cláusulas |
| **Cumprimento** | cláusula em favor de B cumprida (defesa honrada); quebra contra B | boa vontade (+/−) | com a quebra dos tratados (seção 9 deles) |
| **Regime** | distância entre as leis em vigor de A e de B (as mesmas leis e efeitos da política) | afinidade | **sim** |
| **Gente** | parcela da população de B que é da cultura principal de A, ou da diáspora de A (comunidades do #97) | afinidade | **sim** |

- **Dependência de B sobre A** = soma ponderada dos canais de dependência. **Boa vontade** soma os
  atos, cada um pelo seu valor em parcela do PIB de B, e decai pela meia-vida.
- **Influência de A sobre B** = (dependência + boa vontade) × afinidade. A afinidade vai de 0,5
  (regimes opostos, nenhuma gente em comum) a 1,5 (mesmo regime, comunidade grande); os pesos ficam
  em `data/diplomacy/influence.ron`.
- **Sem gasto abstrato.** A gasta influência só no sentido real: emprestou e pode perder o
  dinheiro, deu e não volta, prometeu defender e vai ter de ir. O custo está no Tesouro, no livro
  de dívida e nas cláusulas, onde o jogo já mede.
- **Canais que ainda não existem ficam dormentes e à vista** ("comércio: sem efeito até o comércio
  ter parceiro"), como as cláusulas dormentes dos tratados. Nada de estimar o comércio bilateral por
  gravidade para preencher: seria um número inventado com cara de medida (decisão 2).

## 4. O retorno: o que a influência compra

Cada retorno é um termo com nome numa conta que já existe ou vai existir, à vista na tela:

| Retorno | Onde age | Como | Vivo quando |
|---|---|---|---|
| **Sim numa proposta** | `treaty::evaluate`, termo novo `ValueTerm::Influence` | B dá a uma proposta de A um valor a mais igual à influência de A sobre B × o tamanho da proposta × peso; a IA aceita porque a conta fecha, e a tela mostra a linha "influência de A: +X" | etapa 4 dos tratados (proposta e avaliação) |
| **Preferência entre propostas rivais** | a mesma conta, quando dois países propõem a B o mesmo tipo de acordo | ganha a de maior valor; a influência entra no valor | idem |
| **Voto em organização** | votos da Liga e da ONU, quando as organizações entrarem no tick | cada membro vota pelo próprio interesse mais a influência de cada lado; o teste é o de Kuziemko e Werker | organizações no tick (fase 4) |
| **Acesso a recurso** | `foreign_capital::Site::relation` | a relação, que já é termo do retorno do investidor estrangeiro, passa a ser lida da influência e da boa vontade, não de um número fixo | capital estrangeiro no tick |
| **Lado numa crise** | escada de crise (fase 5) e decisão de terceiros (fase 6) | o terceiro entra do lado de quem tem mais influência sobre ele, menos o custo medido de entrar | fases 5 e 6 |

**Forma do termo, combinada com o thread dos tratados (2026-11-17):**

- **É uma linha da proposta, não de uma cláusula.** A tela da mesa mostra "influência de A: +X"
  separada das cláusulas.
- **O valor vem pronto, por par.** O sistema de influência monta uma vez por mês um mapa (A, B) →
  valor da influência de A sobre B, em unidades de referência (`InfluenceMap`):
  `influência × PIB de B × proposal_weight`. A avaliação só lê o mapa.
- **Tem teto.** O termo é no máximo `proposal_cap` (25 %) do tamanho da proposta, a soma dos
  valores absolutos das outras linhas de B (`InfluenceMap::proposal_term`). A influência faz um
  país aceitar uma perda pequena por um patrono, nunca qualquer perda.
- **Par fora do mapa vale zero.** Sem ato nenhum, o mapa só tem os pares do livro de dívida
  (as dívidas de guerra de 1925). O teste de neutralidade do tick é: com `proposal_weight` zero, o
  hash é o da `main`; com o peso, a diferença vem só desses pares, e o PR a mostra.

Na primeira entrega no tick, o único retorno vivo é o termo `Influence` na avaliação de proposta.
Ele já basta para a regra "sem retorno não entra", porque toda proposta da IA passa por essa conta.
Os outros quatro entram com a frente de que dependem, sem desenho novo.

## 5. Blocos

Um **bloco** é um conjunto de tratados entre um **líder** e os **membros**, cada tratado com o
mesmo feixe de cláusulas (como os graus de súdito dos tratados, seção 7 deles), e um nome. Não é
uma estrutura nova de regras: é um rótulo sobre acordos que já existem, para a tela, para a
influência e para a IA.

| Tipo | Feixe (de `clauses.ron`) | Exemplos |
|---|---|---|
| **Aduaneiro** | `customs_union` ou `tariff_preference` entre todos | Zollverein, CEE, Comecon (como preferência e plano) |
| **Militar** | `defence` mútua, `consultation`, às vezes `base` e `troops_quota` do líder | Pequena Entente, OTAN, Pacto de Varsóvia |
| **Monetário** | `currency_peg` à moeda do líder, depósito de reservas | área da libra, franco CFA |
| **De súditos** | os feixes de sujeição | Império Britânico e domínios (1925) |

- **O que o membro ganha e cede** é a soma das cláusulas do feixe, na mesma conta de valor dos
  tratados (dinheiro, comércio, segurança, soberania), com a influência do líder como termo a mais.
  Nada de bônus fixo de "estar no bloco".
- **Entrar:** uma proposta de tratado ao líder com o feixe do bloco, avaliada pela regra de
  sempre. Para o jogador, é uma decisão rara em Agendas ("Entrar no bloco X"), com o que ganha, o
  que cede e o passo de confirmação, e um ponto de restauração antes, como "Libertar território".
- **Sair:** denúncia pelo aviso do tratado (OTAN: 1 ano depois de 20). Sair contra a vontade do
  líder sob a regra dos súditos é a saída medida da seção 7 dos tratados. Também é decisão rara em
  Agendas.
- **Organização não é bloco.** A Liga e a ONU são fóruns de voto, abertos, sem líder. A
  Pequena Entente é bloco (aliança com tratados). O dado de 1925 já separa os dois
  (`OrganizationKind::Alliance` contra `Global` e `Regional`).
- **Referência para teste, não roteiro:** os blocos de 1925 (Império Britânico e domínios, Pequena
  Entente, União Pan-Americana como organização regional, a Liga como global) entram do cenário. Os
  que a história criou (Comecon, CEE, OTAN, Varsóvia, Não Alinhados) só entram se a conta dos
  países os formar. O placar pergunta quando e com quem se formam, como pergunta das leis.

## 6. O país pequeno

O país pequeno não ganha guerra nem compra influência grande: ele **escolhe a quem se liga e cobra
pela escolha**. O que pode querer e conseguir, com a conta aberta:

| Objetivo | Como se mede | Como consegue | Exemplo |
|---|---|---|---|
| **Neutralidade garantida** | ameaça medida dos vizinhos (fase 5) sobre a própria força | `border_guarantee` de várias potências, sem base estrangeira; a garantia vale o que os garantidores pagariam para cumprir | Suíça; Bélgica (garantia que falhou em 1914) |
| **Proteção** | mesma ameaça | `defence` de um patrono em troca de soberania (base, política externa): o país cede o que vale menos que a segurança que ganha | Kuwait de 1899, Coreia do Sul pós-1953 |
| **Renda de posição** | estreitos e portos que controla (o grafo marítimo já guarda `holders` de estreitos e os portos) | `base` e `passage` com `tribute`: o aluguel é renda do Tesouro | Panamá, Egito (Suez), Turquia (estreitos), Malta |
| **Jogar um bloco contra o outro** | influência de dois rivais sobre o país, as duas altas | a avaliação de proposta já compara as ofertas; com dois rivais, a melhor oferta sobe, e o país aceita a maior sem fechar a porta à outra | Iugoslávia, Egito de 1955–56, Índia |

- **Nada aqui é regra especial de país pequeno.** São as mesmas cláusulas e a mesma conta, que
  dão resultado diferente para quem tem pouca força e posição boa. O "objetivo" é o termo de maior
  valor na conta do próprio país, que a tela mostra ("o que o Egito quer: renda do canal e um
  patrono que não ponha base").
- **Leilão sem regra de leilão:** quando dois rivais disputam, cada um propõe; a conta do pequeno
  aceita a de maior valor e registra a outra. A fase 6 dá aos rivais o motivo para cobrir a oferta.

## 7. Onde entra no agendador

- **Sistema `influence`, mensal**, depois da política e da demografia do mês.
  - **Lê:** livro de dívidas (`Economy::ledger`), tratados em vigor e o registro de cumprimento
    (`GameWorld::treaties`), leis em vigor (política), comunidades e diáspora (demografia), PIB dos
    mercados, súditos (`borders`).
  - **Escreve:** `GameWorld::influence`.
- **Estado salvo:** `InfluenceState` com a boa vontade por par (esparsa: só pares com ato
  registrado) e o registro dos atos. A dependência e a afinidade **não são salvas**: são
  recalculadas do mundo e ficam num cache do sistema, para a avaliação de proposta ler. O estado
  só entra no hash quando não está vazio. **Sem ato de influência, o hash é o da `main`** (o termo
  da avaliação é zero quando a influência é zero).
- **Custo:** cerca de 150 países vivos dão até 22 mil pares ordenados. Só se calculam os pares com
  algum canal não nulo (dívida, tratado, comunidade ou regime próximo de vizinho), algumas centenas
  a poucos milhares, uma vez por mês: abaixo de 1 ms por mês.
- **O que a fase 6 lê daqui:** `Interest { dependence, goodwill, affinity, influence }` por par
  ordenado, e os blocos com líder e membros. É o "interesse medido de cada país em cada outro" de
  `history-rails.pt-BR.md`, parte econômica e política; a parte de ameaça vem da fase 5.

## 8. Tela

- **Lente "Influência" no mapa:** com um país selecionado, pinta os outros pela influência dele
  sobre cada um (ou, num segundo modo, pela de cada um sobre ele). A dica mostra os canais ("crédito
  3,1 % do PIB, ajuda há 4 anos, regime próximo").
- **Lente "Blocos":** cor do bloco militar, com hachura do aduaneiro, e o líder destacado.
- **Painel de leitura do país:** quem mais influencia o país e quem ele mais influencia, cada um com
  os canais. É só leitura.
- **Decisões raras em Agendas:** entrar em bloco e sair de bloco, com o que ganha e cede, passo de
  confirmação e ponto de restauração antes. Ajuda e crédito saem da mesa de negociação dos
  tratados, não de um botão próprio.

## 9. Decisões do dono (decididas em 2026-11-17)

O dono disse sim às seis, como recomendadas. Ele pediu ainda duas coisas: conferir a decisão 1
contra a teoria (seção 12) e pensar o limite real da diplomacia que os jogos representam com
diplomatas (seção 13). As decisões que sobraram dos dois estudos estão na seção 14.

1. **Influência em duas partes, dependência sem memória e boa vontade com meia-vida.**
   Recomendação: sim, com meia-vida de 8 anos para a boa vontade. É como a vida real se comporta
   (rompimento iugoslavo, ajuda e voto), e um estoque único faria a influência sobreviver ao fim
   dos laços. Teste: cortar o crédito de A a B zera a dependência no mês seguinte; uma ajuda cai à
   metade em 8 anos. Custo: um estoque por par com ato e uma recomputação mensal.
2. **Canais que ainda não existem (comércio bilateral, investimento estrangeiro) ficam dormentes**,
   sem estimativa. Recomendação: sim, e o comércio com parceiro sobe de prioridade na fase 4, porque
   é o canal que mais pesou em 1925–45 (Hirschman). Alternativa: estimar o comércio bilateral por
   gravidade (PIB e distância) até ele existir, um número de cara medida que não é medida. Teste: a
   lente mostra os canais dormentes como dormentes. Custo: nenhum agora; o canal liga sozinho
   quando o comércio tiver parceiro.
3. **Nenhum botão de gastar influência.** Recomendação: sim. A influência nasce de atos que custam
   no Tesouro, no livro ou nas cláusulas, e não há moeda abstrata de diplomacia. Alternativa: uma
   ação "melhorar relações" que custa pontos. Teste: sem nenhum ato, a influência de todos é a
   dependência e a afinidade estruturais, e o hash é o da `main`. Custo: nenhum.
4. **Primeiro retorno vivo: o termo `Influence` na avaliação de proposta dos tratados.**
   Recomendação: sim. Os outros (voto, acesso a recurso, lado na crise) entram com a frente de que
   dependem. Teste: a mesma proposta de A a B com e sem crédito de A em B muda de recusa para
   aceite, e a tela mostra a linha. Custo: um termo na `Valuation`, combinado com o thread dos
   tratados (quem escreve: eu calculo, eles somam).
5. **Bloco é rótulo sobre tratados com o mesmo feixe e um líder**, não regra nova. Entrar e sair
   são decisões raras em Agendas, com confirmação e ponto de restauração. Recomendação: sim.
   Alternativa: bloco como entidade com regras próprias (votação interna, tarifa comum fora das
   cláusulas). Teste: a Pequena Entente de 1925 aparece como bloco militar com os seus três
   membros; um país que denuncia o tratado sai do bloco no fim do aviso. Custo: um campo `bloc` no
   tratado (que o dado de 1925 já tem) e a lente.
6. **O país pequeno sem regra especial.** Recomendação: sim. Neutralidade, proteção, renda de
   posição e leilão saem das mesmas cláusulas e da mesma conta; a tela mostra o termo que mais pesa
   como "o que o país quer". Alternativa: objetivos escritos por país (agendas da fase 7). Teste:
   com a renda de base e de trânsito ligada, o Egito e o Panamá avaliam acordos de base de um jeito
   que a Hungria não avalia; com dois rivais propondo, o país aceita a maior oferta. Custo: nenhum
   além das cláusulas.

## 10. Etapa 2, depois do sim

1. **Contrato e dados fora do tick:** `influence.rs` (canais, `Interest`, funções puras com termos
   nomeados) e `data/diplomacy/influence.ron` (pesos, meia-vida, faixa da afinidade); blocos de
   1925 lidos de `diplomacy.ron`. Testes de unidade dos canais e do decaimento. Hash igual ao da
   `main`.
2. **No tick:** sistema mensal, `InfluenceState` com degrau de save antigo, termo `Influence` na
   avaliação de proposta (com o thread dos tratados). Determinismo de 100 anos (threads e salvar e
   recarregar), CPU de 10 anos contra a `main`, e o teste de neutralidade (sem ato, hash da `main`).
3. **Tela:** lentes de influência e de blocos, painel de leitura, decisões de bloco em Agendas.

## 12. Conferência com a teoria (pedido do dono na decisão 1)

O dono perguntou se é assim que a diplomacia funciona, já que jogos do gênero somam "opinião" e
"favor devido". O que a literatura diz, e o que muda no desenho:

- **Dependência é vulnerabilidade, não tamanho do fluxo.** Keohane e Nye separam a
  *sensibilidade* (o custo imediato de uma mudança) da *vulnerabilidade* (o custo que sobra depois
  que o país se ajusta, procurando outro parceiro). Hirschman já media o poder pela dificuldade do
  parceiro pequeno de trocar de mercado.
  - **Muda:** cada canal passa a pesar o fluxo vezes a **concentração**, a parte de A no total
    daquele canal em B. Quem deve a muitos credores depende pouco de cada um. Quem deve tudo a um
    só depende dele. Quem está excluído do mercado de crédito (calote recente) depende ainda mais.
  - **País isolado ou de fluxos pequenos** depende pouco de qualquer um, pela própria conta: o
    fluxo é pequeno. Responde à observação do dono na decisão 2.
- **"Opinião" e "favor" são reciprocidade difusa e específica.** Keohane (1986) separa a
  reciprocidade *específica* (troca de valores equivalentes entre parceiros definidos, num prazo
  definido) da *difusa* (equivalência frouxa, obrigação geral, entre quem compartilha normas). A
  cooperação que se repete sem polícia vem da reciprocidade (Axelrod, 1984).
  - **A boa vontade com meia-vida é a difusa:** fica como está.
  - **O favor devido é a específica:** discreto, cobrável uma vez, sem meia-vida. Ele é melhor
    modelado como **cláusula de promessa num acordo** (por exemplo "B apoia A na próxima votação
    da organização X" ou "B consente no próximo acordo de A com C"), cobrada ou vencida, do que como
    número. Fica com o thread dos tratados, num tipo novo de cláusula.
- **O que os jogos chamam de opinião, na teoria é ameaça e reputação.**
  - Walt (1987): os países se alinham **contra quem ameaça** (poder, proximidade, intenção). A
    ideologia pesa bem menos do que se esperava. **Muda:** a afinidade fica com peso pequeno, de
    0,8 a 1,2 em vez de 0,5 a 1,5. A **ameaça** entra como canal dormente, negativo, ligado pela
    guerra (fase 5) e pela IA (fase 6).
  - Tomz (2007): na dívida soberana, o que pesa é a **reputação de cumprir**, aprendida com o
    comportamento. A reputação por país já é dos tratados (registro de quebras, seção 9 deles), e a
    avaliação de proposta já a lê como termo "confiança". A influência não duplica isso.
- **Influência vale por assunto.** Baldwin (1979) critica tratar o poder como moeda que serve em
  qualquer assunto (a fungibilidade). Um credor tem peso numa renegociação de dívida e pouco numa
  aliança; quem defende tem peso numa aliança e pouco numa tarifa.
  - **Muda:** o mapa passa a guardar o valor de cada par **por assunto**. O assunto é o termo
    dominante da proposta (dinheiro, comércio, território, segurança, soberania), e cada assunto
    pesa os canais de um jeito (`data/diplomacy/influence.ron`): o crédito pesa em dinheiro, a
    proteção em segurança, o comércio em comércio. A boa vontade pesa igual em todos.
  - Os retornos por frente da seção 4 (voto, recurso, crise) já seguem essa linha; isto leva a mesma
    ideia para o termo da proposta.

**A diferença importa.** Sem a concentração, um país grande com dívidas espalhadas pareceria tão
dependente quanto um pequeno preso a um só credor. Sem o assunto, emprestar a um país compraria a
aliança dele. Os dois erros dariam uma diplomacia que não se reconhece.

## 13. Por que não se melhora a relação com todo mundo ao mesmo tempo (pedido do dono na decisão 3)

Os jogos do gênero limitam a diplomacia com uma contagem de diplomatas. O dono pediu o limite real
por trás disso. Não há cópia de mecânica aqui: são cinco limites medidos, cada um com o seu custo.

1. **Serviço exterior é uma repartição do governo.** Cada par de países tem ou não tem missão:
   nenhuma, legação ou embaixada. A missão custa no orçamento (folha de funcionários públicos no
   exterior) e ocupa gente qualificada (profissionais). Com missão, o ato rende inteiro, a
   negociação anda no ritmo normal e a avaliação do outro lado vem exata. Sem missão, o ato rende
   metade, a negociação leva o dobro dos meses e a avaliação que o jogador vê do outro lado vem com
   faixa de incerteza (determinística: a faixa é larga, não o número sorteado).
   - **Na vida real:** em 1925 muitos países tinham legação em poucas capitais, e só as potências
     tinham embaixada (a hierarquia de embaixador e ministro). A rede cresceu com a descolonização e
     com o orçamento.
   - **Dado de 1925:** o Correlates of War Diplomatic Exchange cobre 1920 e 1925, mas os termos de
     uso proíbem uso comercial e **redistribuição a terceiros sem permissão escrita**. Não pode ir
     para o repositório público. Proposta: derivar as missões de 1925 por regra (potência tem
     embaixada em toda capital reconhecida; os outros têm legação nos vizinhos, no suserano e nas
     potências) e usar o COW, se for o caso, só para conferir fora do repositório.
2. **Negociações simultâneas são poucas.** Cada negociação aberta ocupa uma equipe por meses. O
   número de equipes sai do tamanho do serviço exterior (folha e profissionais). A **eficiência da
   administração** multiplica: é o lugar do pedido antigo do dono de computação como método de
   eficiência. A tecnologia de informática e comunicação aumenta o que cada equipe faz.
3. **Rivalidade e exclusividade.** Boa vontade com B custa boa vontade em C quando B e C são
   hostis: o ato a favor de B registra um ato negativo menor em C. Algumas cláusulas são
   exclusivas por natureza: reconhecer uma China exclui a outra (doutrina Hallstein, "uma só
   China"), e uma aliança defensiva exclui a aliança com o inimigo dela. A hostilidade vem da fase
   5/6; até lá valem os pares que não se reconhecem e os blocos rivais.
4. **Promessa a muitos vale menos.** Quem promete defender todo mundo não defende ninguém: o valor
   de uma cláusula de defesa cai com o número de defesas que o país já prometeu, pesadas pela
   ameaça de cada uma. É o dilema da aliança (Snyder, 1984): o medo de ser abandonado contra o medo
   de ser arrastado.
5. **Política interna cobra a aproximação.** Os grupos de interesse cobram legitimidade quando o
   governo se aproxima de um regime que eles rejeitam (os dois níveis de Putnam, 1988). Usa a
   máquina da ratificação dos tratados: a postura dos grupos sobre a distância de regime.

**A diferença importa.** Sem o primeiro e o segundo limites, a IA e o jogador abririam negociação
com o mundo inteiro todo mês, e o tamanho do país não pesaria na diplomacia. Sem o terceiro, não há
escolha de lado. Sem o quarto, as alianças se espalham sem custo. Sem o quinto, mudar de patrono
não custa nada por dentro.

**Como se testa:**
- um país sem missão em B leva o dobro dos meses para fechar o mesmo acordo;
- com o serviço pequeno, a terceira negociação simultânea espera vaga;
- uma ajuda a B baixa a boa vontade em C quando os dois não se reconhecem;
- a quinta aliança defensiva de um país vale menos para quem a recebe do que a primeira;
- a aproximação com um regime oposto baixa a legitimidade quando os grupos que contam o rejeitam.

**Quanto custa manter:** as missões são uma tabela por par (esparsa, algumas centenas de pares) e
uma linha do orçamento; as equipes são um contador por país; os outros três limites são termos da
avaliação que já existe.

## 14. Decisões que sobraram dos dois estudos

Propostas, aguardando o dono.

1. **Dependência como vulnerabilidade:** fluxo vezes a concentração de A no canal (a parte de A no
   total de B), mais forte para quem está fora do mercado de crédito. Recomendação: sim. Teste: o
   mesmo crédito pesa mais em quem deve só a A do que em quem deve a dez credores. Custo: uma soma
   por canal e por país.
2. **Favor devido como cláusula de promessa**, cobrada uma vez ou vencida, com o thread dos
   tratados. A boa vontade com meia-vida segue como a reciprocidade difusa. Recomendação: sim.
   Alternativa: um número de favor por par. Teste: B que prometeu apoio vota com A na votação
   seguinte e a cláusula se encerra. Custo: um tipo de cláusula.
3. **Afinidade fraca, ameaça dormente, influência por assunto.** A afinidade fica de 0,8 a 1,2. A
   ameaça é um canal negativo dormente até as fases 5 e 6. O mapa guarda o valor por assunto da
   proposta. A reputação continua dos tratados. Recomendação: sim. Teste: um empréstimo não compra
   uma aliança; o mesmo par tem valor maior numa renegociação de dívida do que numa aliança. Custo:
   uma tabela de pesos por assunto.
4. **Serviço exterior com missões pagas e pessoal**; as missões de 1925 derivadas por regra, sem os
   dados do COW (termos de uso). Recomendação: sim. Alternativa: pedir permissão ao COW para usar o
   dado de 1925. Teste: sem missão, o ato rende metade e a negociação leva o dobro. Custo: uma
   tabela por par e uma linha de orçamento.
5. **Limites de quantidade:** negociações simultâneas pelo tamanho do serviço, com a eficiência
   administrativa (informática) multiplicando; rivalidade e cláusulas exclusivas; diluição da
   promessa de defesa. Recomendação: sim, os três. Teste: os da seção 13. Custo: um contador por
   país e dois termos da avaliação.
6. **Custo interno da aproximação** pela postura dos grupos sobre a distância de regime, na
   máquina da ratificação. Recomendação: sim. Teste: a mesma aproximação custa legitimidade numa
   república com grupos contra e não custa numa autocracia alinhada. Custo: um termo na ratificação.

## 11. Fontes

- Hirschman, A. O. *National Power and the Structure of Foreign Trade*. University of California
  Press, 1945. Dependência comercial assimétrica como poder; o caso alemão nos Bálcãs.
- Kuziemko, I. e Werker, E. "How Much Is a Seat on the Security Council Worth? Foreign Aid and
  Bribery at the United Nations". *Journal of Political Economy* 114(5), 2006, 905–930. Ajuda sobe
  no mandato e cai depois; alinhamento de voto acompanha.
- Dreher, A., Nunnenkamp, P. e Thiele, R. "Does US Aid Buy UN General Assembly Votes? A
  Disaggregated Analysis". *Public Choice* 136, 2008, 139–164. O efeito existe e é pequeno.
- Keohane, R. O. e Nye, J. S. *Power and Interdependence*. Little, Brown, 1977. Sensibilidade e
  vulnerabilidade na interdependência.
- Walt, S. M. *The Origins of Alliances*. Cornell University Press, 1987. Alinhamento contra a
  ameaça, não contra o poder.
- Fox, A. B. *The Power of Small States: Diplomacy in World War II*. University of Chicago Press,
  1959. Como os pequenos se mantiveram neutros ou escolheram o patrono.
- Keohane, R. O. "Reciprocity in International Relations". *International Organization* 40(1),
  1986, 1–27. Reciprocidade específica e difusa.
- Axelrod, R. *The Evolution of Cooperation*. Basic Books, 1984.
- Tomz, M. *Reputation and International Cooperation: Sovereign Debt across Three Centuries*.
  Princeton University Press, 2007.
- Baldwin, D. A. "Power Analysis and World Politics: New Trends versus Old Tendencies". *World
  Politics* 31(2), 1979, 161–194. Crítica da fungibilidade do poder.
- Snyder, G. H. "The Security Dilemma in Alliance Politics". *World Politics* 36(4), 1984,
  461–495. Abandono e arrastamento.
- Putnam, R. D. "Diplomacy and Domestic Politics: The Logic of Two-Level Games". *International
  Organization* 42(3), 1988, 427–460.
- Bayer, R. *Diplomatic Exchange Data set, v2006.1*. Correlates of War Project, 2006 (1817–2005,
  de cinco em cinco anos entre 1920 e 1940). Termos de uso do COW: sem uso comercial e sem
  redistribuição a terceiros sem permissão; **não** entra no repositório.
- Keohane, R. O. "Lilliputians' Dilemmas: Small States in International Politics". *International
  Organization* 23(2), 1969, 291–310.
