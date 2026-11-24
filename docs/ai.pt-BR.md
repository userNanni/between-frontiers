# IA estratégica (fase 6): desenho

Desenho, sem código; a fatia 1 (as medidas) está autorizada, as outras esperam o dono. Este documento
junta o que a fase 6 precisa: o que a IA lê, o que decide e com que frequência, o que já existe e
ela só precisa chamar, a ordem das fatias e as decisões que são do dono.

A base é a seção "IA estratégica: países que interpretam o papel" de
`docs/history-rails.pt-BR.md` (interesse medido, papel, agenda histórica; nenhuma estratégia
sorteada), com o que as fases 4 e 5 construíram desde então.

## Princípio

**A IA reage ao que mede, não a um roteiro.** Calibração sem forçar a história: nenhuma ação
acontece porque aconteceu em 1935. A Alemanha reintroduz a conscrição se a ameaça medida e os
grupos no poder pedirem, não porque o placar espera 1935.

As agendas históricas de `history-rails.pt-BR.md` continuam como foram decididas: somam ao valor das
ações que servem a meta, só enquanto as condições medidas da agenda valem, e nunca escolhem a ação.

**A IA usa os comandos do jogador** (`GameCommand`), com as mesmas restrições e os mesmos limites.
O enum já diz "What players (and later the AI) can do" (`game.rs`). Uma ação que o jogador não pode
tomar a IA também não toma.

## Hoje: o que um país sem jogador já decide

Tudo por regra fixa, sem olhar adiante.

| Onde | O que decide | Quando | Código |
|---|---|---|---|
| Crise | o alvo cede ou recusa pela razão de forças contra a parte pedida; nunca cede metade do país | na primeira semana | `war::crisis::crisis_week`, `weigh_crisis` |
| Crise | quem exigiu sempre recua depois da recusa; nunca declara | na recusa | `crisis.rs` (o recuo) |
| Chamada à guerra | sempre honra e entra mobilizado | na declaração | `war::third` (o comentário diz "its choice comes with the strategic AI") |
| Apoio à guerra | pede paz abaixo de 300; cai o governo abaixo de 100, depois de 6 semanas | semanal | `war::support::support_week` (a única proposta de tratado que a IA faz) |
| Frente | ataca o segmento onde tem 1,5 vez o fogo (G5) | semanal | `fight.rs`, `AI_ATTACK_RATIO` |
| Encomendas | navios e aviões com a sobra do teto | anual | `war::ai_orders` (`share` 0: desligado) |
| Guarnição | encomenda formações de guarnição depois de 4 semanas de falta | semanal | `occupation::garrison_order` |
| Expedição colonial | desembarca até cobrir 20 por mil (O3) | semanal | `war::colonial::week` |
| Independência | recusa, governo próprio, domínio ou independência pela conta | anual | `borders::independence::answer` |
| Arma nuclear | retalia no mesmo degrau (N5); primeiro uso quando ocupado além do limiar (N7) | semanal | `nuclear_use::answer`, `first_uses` |
| Mesa de tratados | aceita se a conta fecha, contrapropõe, recusa | semanal | `treaty_table::respond` |
| Leis | cede ao empurrão do movimento mais forte, ou os grupos mudam a lei | anual | `polity::system::year` |
| Orçamento, impostos, banco central, pesquisa | regra histórica (`None` nas alavancas), regra de Taylor, "a mais barata aberta" | — | `econ::levers`, `money.rs`, `research_system.rs` |
| Padrão-ouro | sai abaixo do piso, volta quando calmo | semanal | `econ::gold::step` |
| Prazo de serviço | fixo no de 1925 | — | `war::reserve` |
| Mobilização | só quando a guerra abre ou quando entra numa | na guerra | `open_war`, `third::join` |

**O que nenhum país sem jogador faz:** exigir, declarar, embargar, começar ou mudar programa
nuclear, testar a arma, mobilizar numa crise, mudar o prazo de serviço ou a lei de conscrição, mexer
no orçamento militar, propor tratado (fora o pedido de paz), quebrar ou denunciar acordo, escolher
defender ou desvalorizar no ouro. Por isso, na rodada de 101 anos com a guerra ligada, nenhuma
guerra começa.

## O que ela lê

**Já medido:**
- **Influência** (`influence.rs`, mensal): `interest` (dependência, boa vontade, afinidade,
  influência; a struct já diz "what the AI of phase 6 takes"), `dependence`, `trade_dependence`,
  `investment_dependence`, `affinity`, `worth` por tema, `goodwill`, blocos (`blocs_on`,
  `bloc_views`, `alliance_label`).
- **Reputação** (`treaty.rs`): geral e de segurança, com o decaimento e o corte na troca de regime;
  a mancha do uso nuclear.
- **Quebras e causas** (`TreatyState::breaches`, `causes`, `cause(holder, against)`): o casus belli
  que a exigência já lê.
- **Avaliação de proposta** (`treaty::evaluate`, `treaty_table::account`), termo a termo
  (dinheiro, comércio, território, segurança, soberania, política, confiança).
- **Balanço nuclear** (`war::balance::balance`): dissuasão (segura, mútua, incentivo ao primeiro
  golpe, segundo golpe) e os tetos da escada (`EscalationLimits`, `Rung`: tensão, rivalidade,
  corrida armamentista, crise, guerra, coerção nuclear, uso nuclear).
- **Forças e crise:** `crisis::land_men`, `weigh_crisis`, `states_asked`, `money_asked`,
  `third::who_would_join`, `forces_cost_year`.
- **Guerra em curso:** apoio por grupo, queda, contagem, controle, ocupação, perdas, o movimento
  contra a guerra (rascunho #362).
- **Política:** legitimidade, movimentos, o revisionismo.

**A medir (não existe no código):**
- **Hostilidade por par** (`history-rails.pt-BR.md`, "Hostilidade"): a escada tensão → guerra por
  par de países, que sobe com causas, disputa de território, interesses opostos e corrida, e desce
  com comércio, acordos e tempo.
- **Ameaça:** o poder militar dos vizinhos hostis (terra por fronteira, mar por zona, ar, arma
  nuclear) contra o próprio, e a tendência.
- **Território perdido:** o indicador `lost_territory` está declarado sem função que o calcule;
  o `revisionism` não cresce por isso.
- **Papel:** a posição no sistema (ranking e tendência), como `history-rails.pt-BR.md` descreve.

## O que decide, e com que frequência

Cada decisão pontua as ações candidatas pela soma dos interesses ponderados pelos grupos no poder,
mais a agenda se as condições dela valem, menos o custo medido (o mesmo `evaluate` dos tratados
onde couber). Toma a ação se o valor passa do custo; senão, não faz nada.

| Decisão | Frequência | Lê | Chama |
|---|---|---|---|
| Medir hostilidade, ameaça e papel | mensal | influência, causas, forças, balanço | — (estado novo, só leitura) |
| Orçamento militar pela ameaça | anual | ameaça, apoio dos grupos, caixa | `SetStatePurchases` (parcela militar) |
| Lei de conscrição e prazo de serviço | anual | ameaça, grupos (os militares e os camponeses), legitimidade | `SetLaw`, `SetServiceMonths` |
| Encomendas de navios e aviões | anual | ameaça no mar e no ar, tetos dos tratados | `ai_orders` com `share` pela ameaça |
| Programa nuclear e teste | anual | balanço nuclear do rival, ameaça, custo | `NuclearProgramme`, `NuclearTest` |
| Exigir (abrir a crise) | mensal | hostilidade, interesse no que pede, causa, força prevista (`weigh_crisis` com uma crise hipotética), quem entraria (`who_would_join`), teto nuclear | `Demand` |
| Escalar ou recuar na crise | semanal, dentro do prazo | força, aliados, apoio previsto | `Mobilize`, `Declare` (ou o recuo) |
| Ceder ou recusar (alvo) | na crise | o que já existe (W3), mais aliados e ameaça | `AnswerCrisis` |
| Honrar a chamada | na chamada | preço de trair (reputação), força relativa, prontidão, interesse no aliado | `AnswerCall` |
| Frentes | semanal, em guerra | ameaça por frente, objetivo pelo interesse | `SetFrontShare`, `SetFrontObjective`, `SetReserve` (rascunho #371) |
| Pedir ou aceitar a paz | semanal, em guerra | o que já existe (apoio), mais a guerra que vai mal | `ProposeTreaty`, `AnswerProposal` |
| Expedição colonial | mensal | custo, apoio, cobertura | `SetExpedition` |
| Propor acordo (aliança, garantia, comércio) | mensal | ameaça comum, interesse, reputação do outro | `ProposeTreaty` |
| Quebrar ou denunciar acordo | mensal | quebrar vale mais que o custo (`treaties.pt-BR.md`) | `Denounce` |
| Embargo | mensal | hostilidade, dependência do alvo, custo próprio | `Embargo` |
| Ouro: defender ou desvalorizar | semanal, abaixo do piso | reservas, desemprego, grupos | `SetGoldPosture` |

## O que já existe e ela só precisa chamar

- A avaliação de proposta e a resposta provável (`treaty_table::respond`, `account`), com o
  bloqueio de 6 meses depois de recusa igual e os limites de cláusula.
- A previsão de quem cede numa crise (`weigh_crisis`; precisa de uma crise hipotética).
- A escada de crise inteira: exigência, prazo, resposta, declaração (`crisis_decide`), terceiros
  pelos acordos (`third::who_would_join`, `join`, `betray`), o teto nuclear (`nuclear.blocked`).
- A mobilização (`Mobilize`), as encomendas (`OrderForces`, `ai_orders`), a postura (`SetPosture`),
  a expedição (`SetExpedition`), o programa, o teste e o uso nuclear, o prazo de serviço, as leis,
  o embargo, a tarifa por parceiro, o ouro e o controle de capital.

## Ordem das fatias

Cada fatia atrás da sua chave, desligada, com a série de 101 anos contra a faixa.

1. **Medidas, só leitura:** hostilidade por par, ameaça, papel, o `lost_territory`. Uma tela que
   mostra cada par e por quê. Não muda a simulação.
2. **Armar-se pela ameaça, sem guerra:** orçamento militar, conscrição e prazo, encomendas. Resolve
   as dependências da fase 5 que não pedem guerra (abaixo).
3. **Alianças:** honrar a chamada pela conta, propor aliança e garantia contra a ameaça comum,
   quebrar e denunciar pela conta.
4. **Crise e guerra:** exigir, escalar, declarar. É a fatia em que alguma guerra começa.
5. **Guerra conduzida:** frentes pela ameaça, mobilização na crise, expedição dosada, paz pedida
   antes do colapso.
6. **Arma nuclear:** programa e teste pela ameaça, a escada nuclear com o risco semanal.
7. **Economia de poder:** embargo, tarifa por parceiro como arma, ouro.

## Decisões

**Já decididas pelo dono** (`history-rails.pt-BR.md`):
- **IA2, sem sorte:** pontuação determinística, empate pelo índice do país; nenhuma estratégia
  sorteada.
- **IA3, a agenda histórica:** soma ao valor das ações que servem a meta, só enquanto as condições
  medidas dela valem; nunca escolhe a ação.

**Decididas pelo orquestrador (2026-11-24),** por serem engenharia ou regra do dono já existente:
- **IA1, frequência:** medidas mensais; exigir e propor, mensal; o que está em curso (crise e
  guerra), semanal; orçamento e leis, anual.
- **IA6, comandos:** a IA usa só os comandos do jogador, com as mesmas restrições.
- **IA7, chaves:** uma por fatia, desligadas, ligadas uma a uma depois da faixa.

**Abertas, com o dono:**

| # | Pergunta | Recomendado | Alternativa |
|---|---|---|---|
| IA4 | Honrar a chamada: conta ou taxa? | a conta (preço de trair contra o custo de entrar); a taxa de cerca de 75 % de alianças honradas (Leeds, Long e Mitchell) mede, não fixa | uma probabilidade fixa de 75 % |
| IA5 | Orçamento militar pela ameaça: como? | uma parcela do PIB que segue a razão entre o poder dos vizinhos hostis e o próprio, com teto e devagar (anos), e que cai na paz | a corrida simétrica de Richardson (cada um responde ao gasto do outro) |

**Fatia 1 autorizada** (o orquestrador, 2026-11-24), depois do helicóptero: as medidas, só leitura,
atrás de chave desligada. As fatias 2 em diante esperam o dono.

## Dependências da fase 6 anotadas nas fases anteriores

- **O gasto militar não cai na paz:** com a guerra ligada, sem nenhuma guerra, o gasto militar
  sobe a 8 a 13 % do PIB em 2000, pelo teto do orçamento (`war.pt-BR.md`, roteiro de calibração).
  O orçamento pela ameaça (IA5) é o que deveria baixá-lo; o teto é calibração.
- **A conscrição alemã de 1935:** a Alemanha fica com 95 mil homens em 1939, porque a lei e o prazo
  não mudam sem jogador (reserva por coorte, #356).
- **Rearmamento:** a encomenda da IA (`ai_orders`) está em `share` 0 até o orçamento militar ser
  recalibrado; a fatia 2 a liga pela ameaça.
- **Orçamento militar pela ameaça:** citado nas forças em paz e na encomenda.
- **A legitimidade é anual:** o movimento contra a guerra pesa pouco numa guerra curta (#362).
- **O ocupante constrói só o que decidir:** construir em terra ocupada é decisão futura do jogador
  e da IA (obras em terra ocupada, #367).
- **A Europa como refém, a razão de forças no teatro (variante b da N10), terceiros na guerra
  colonial, o risco semanal de escalada, o custo de reputação do teste nuclear** (`war.pt-BR.md`,
  "Dependências de outros donos").
- **A IA recusa a própria vitória** (defeito aberto em `treaties.pt-BR.md`): a fatia 5 o corrige,
  com a correção já proposta lá.
- **`lost_territory` sem medida:** o revisionismo não cresce pela terra perdida; a fatia 1 o mede.
- **A pesquisa militar não responde à ameaça** (`war.pt-BR.md`): entra na fatia 2.
- **Neutros prejudicados pelo bloqueio como casus belli** (os EUA de 1917): entra na fatia 4.

## Custo de CPU

Minha conta, sem medida: 83 países dão 3.403 pares. Medir a hostilidade de cada par uma vez por
mês, com leituras que já existem, é da ordem das medidas de influência, que já rodam mensais. As
decisões de exigir e propor olham poucos pares por país (os vizinhos e os hostis). A fatia 1 mede o
custo antes de qualquer decisão entrar.
