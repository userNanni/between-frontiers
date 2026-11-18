# O que o jogador decide

Cada decisão de um país que o jogador pode tomar, com o comando da simulação, onde fica na tela e o
atalho. O teste `crates/bf-client/tests/player_commands.rs` garante que nenhum comando de jogador
novo chega sem tela e sem atalho: comando novo em `GameCommand` falha nele até ter os dois, ou até
entrar na lista de exceções com a razão. Quando esta tabela muda, o teste muda junto.

Atalhos do perfil padrão; todos se mudam no editor de atalhos (Configurações → Editar atalhos).

## No jogo

| Decisão | Comando | Onde fica | Atalho | Estado |
|---|---|---|---|---|
| Escolher o país da campanha | `ChoosePlayer` | Tela de escolha de país: clique no mapa, ou "Procurar país" (lista com busca pelo nome ou pela sigla), e "Começar" | Ctrl+F abre a lista; Enter escolhe o primeiro; S começa | no jogo (#8; teclado neste PR) |
| Jogar com outro país | `SwitchCountry` | Campanhas → "…" da campanha → "Jogar com outro país" | Shift+F10 abre o "…"; J escolhe | no jogo (#96, #108) |
| Pesquisa: foco | `SetResearch` | Pesquisa: clique numa tecnologia; "Limpar foco" | R abre; Delete limpa o foco | no jogo |
| Impostos por classe, ou regra automática | `SetTaxRates` | Governo → Orçamento | G abre; B a aba | no jogo |
| Prioridades das obras públicas | `SetPublicPriority` | Governo → Orçamento | G, B | no jogo |
| Lei de migração | `SetMigrationLaw` | Governo → Orçamento | G, B | no jogo |
| Campanha de alfabetização de adultos: começar ou parar | `LiteracyCampaign` | Governo → Orçamento → "Campanha de alfabetização", com confirmação; só com a regra do ensino ligada (`data/rules.ron`) | G, B | no jogo, regra desligada (PR do ensino) |
| Taxa de juros do banco central, ou regra automática | `SetPolicyRate` | Governo → Moeda e dívida | G abre; M a aba | no jogo |
| Banco central: histórico, autônomo ou subordinado | `SetCentralBank` | Governo → Moeda e dívida | G, M | no jogo |
| Propor uma lei | `SetLaw` | Governo → Leis: escolher a lei e "Propor" | G, L; Ctrl+Enter propõe | no jogo |
| Libertar território | `Liberate` | Agendas → Decisões → "Libertar território", com confirmação | J abre as Agendas | no jogo (#96) |
| Mobilizar e desmobilizar a reserva | `Mobilize` | Agendas → "Forças armadas em paz" → "Mobilizar a reserva…" (ou desmobilizar), com confirmação e ponto de restauração; a aba Forças do governo (T) mostra os reservistas que se apresentam | J abre as Agendas | em revisão (#115) |
| Responder à exigência de independência de uma colônia: governo próprio, domínio, independência ou recusar | `AnswerDemand` | Agendas → Decisões → "{colônia} exige uma resposta": mobilização contra as marcas, o que segura o movimento, a conta de manter a colônia, o preço de cada resposta, o prazo e confirmação; ponto de restauração antes (motivo "exigência") | J abre as Agendas; a dica de tecla escolhe a exigência, 1 a 4 a resposta, R revisa, Y confirma | no jogo (#136 e a tela empilhada nele) |

Exceção: `Borders` (operação de fronteira) não é do jogador. Vem de tratados, da paz, de testes e
de ferramentas; o jogador liberta a própria terra com `Liberate`.

Exceção provisória: `Demand`, `AnswerCrisis` e `Declare` (crise e declaração, fatia B, rascunho)
ainda não têm tela. A entrada da crise no diário e as decisões de exigir e declarar em Agendas vêm
com as telas da fatia, e a exceção sai com elas.

## Em PR

| Decisão | Comando | Onde fica | Estado |
|---|---|---|---|
| Compras do Estado e a parte militar (D3) | `SetStatePurchases` | Governo → Orçamento → "Folha e Tesouro": regra histórica ou fixado, com o teto militar da semana | em revisão |
| Onde emitir a dívida: em casa, fora em moeda própria, fora em dólar (D9) | `SetIssuance` | Governo → Moeda e dívida → "Onde emitir": regra histórica ou fixado; o vendido e o sem vender de cada praça; aviso quando a moeda própria pedida passa do que os de fora aceitam | em revisão |
| Ordenar a fila de obras públicas, pôr e tirar obra (D8) | `SetWorksOrder`, `AddWork`, `CancelWork` | Governo → Obras (Shift+B): ordem de chegada ou a minha, ↑ ↓, cancelar em dois cliques, nova obra por estado com o custo; "só a minha lista" zera as prioridades | em revisão |
| Folha do Estado: servidores e clero por mil habitantes (D1) | `SetPayroll` | Governo → Orçamento → "Folha e Tesouro": regra histórica ou fixado | em revisão |
| Salário público sobre o privado (D2) | `SetPublicPay` | Governo → Orçamento → "Folha e Tesouro" | em revisão |
| Reserva do Tesouro e destino do excesso (D4) | `SetTreasury` | Governo → Orçamento → "Folha e Tesouro" | em revisão |
| Teto de déficit (D5) | `SetDeficitCap` | Governo → Orçamento → "Folha e Tesouro" | em revisão |
| Mobilizar e desmobilizar | `Mobilize { on }` | Agendas, decisão em dois passos, com ponto de restauração antes (motivo "guerra") | em revisão (#115) |
| Encomendar tropas, navios e aviões | `OrderForces` | Governo → Forças → "Nova encomenda": tipo, elementos, colonial; o custo de construir e o de todo ano; a encomenda grande pede um segundo clique | em revisão |
| Cancelar uma encomenda | `CancelOrder` | Governo → Forças → "Encomendas": "Cancelar" e confirmação; o construído vai ao depósito | em revisão |
| Dissolver tropas, dar baixa em navios e aviões | `DisbandForces` | Governo → Forças → "Dissolver…" na linha do tipo: quantos elementos, o que volta ao depósito (valor e bens), quantos homens vão à reserva ou para casa, e o aviso de que o exército de paz volta ao tamanho de paz em janeiro; "Dissolver" confirma, com ponto de restauração antes (motivo "dissolver forças"). Tira primeiro as formações encomendadas. Reservistas chamados voltam por `Mobilize` | em revisão |
| Propor um acordo a outro país | `ProposeTreaty` | Agendas → Decisões → "Propor acordo a {país selecionado}…": a mesa, com o que cada lado dá em sua coluna (cláusulas do catálogo, parâmetros dentro dos limites), prazo e aviso, a conta de cada lado por termo, a resposta provável do outro, e por que a proposta não pode ir (limite, reconhecimento, igual recusada há menos de 6 meses); ponto de restauração antes (motivo "tratado"). Só com a regra `negotiation` | J abre; T a mesa; A e S acrescentam cláusula, K escolhe, R tira, V muda parâmetro, W prazo e aviso, P propõe | em revisão, regra desligada |
| Aceitar, recusar ou contrapropor um acordo recebido | `AnswerProposal` | Agendas → Decisões → "{país} propõe um acordo (N semanas)": o que cada lado dá, a conta do jogador, o prazo; aceitar e recusar em dois passos, contrapropor na mesma mesa (até a terceira rodada); ponto de restauração antes de aceitar ou contrapropor | J; P escolhe a proposta; A, R, C; Y confirma | em revisão, regra desligada |
| Renegociar uma dívida entre governos | `ProposeTreaty` (cláusula `revise_debt`) | Governo → Dívidas entre governos → a linha → "Renegociar": abre a mesa nas Agendas com a revisão da dívida, a partir dos termos de hoje, e o outro lado fixo. Os parâmetros da revisão e o preço dela na conta chegam com a simulação (pedido ao thread dos tratados) | G, Shift+M; Z escolhe a linha; N renegocia | em revisão, regra desligada |
| Postura de cada frente na guerra: segurar ou atacar | `SetPosture` | Janela "Guerra" (Shift+W, ou a linha da guerra nas Agendas): uma frente por país inimigo, com "Segurar" e "Atacar", os homens e o poder de fogo dos dois lados, as baixas e o avanço da semana e cada segmento com os termos do poder e quanto já entrou em cada estado. O padrão é segurar. "Propor paz…" abre a mesa nas Agendas com o líder inimigo. Só com a regra `war` | Shift+W; F + número troca a postura; S + número abre os segmentos; Q propõe a paz | em revisão, regra desligada |

## Desenhadas, sem comando ainda

| Decisão | Onde deve ficar | Depende de |
|---|---|---|
| Denunciar um tratado | Agendas, sem aviso em dois passos e com ponto de restauração | comando `Denounce`, atrás de `negotiation`; falta a tela |
| Honrar ou não um pacto de defesa | Agendas, decisão com ponto de restauração | registro de quebras |
| Exigir e declarar guerra | Agendas, com ponto de restauração (motivo "guerra") | fase 5 |
| Entrar e sair de um bloco | Agendas | blocos como rótulo sobre tratados (#132) e proposta no tick |
| Vetar capital estrangeiro e impor embargo | Governo | fluxo de capital (#117) e comércio por par (#113) |
| Abrir missão diplomática | Agendas | serviço exterior (#112) |

## O que o país decide por regra

Estas decisões o país toma sozinho, por uma regra, sem alavanca do jogador. Se alguma virar decisão
do jogador, é o autor quem decide.

### O que o autor decidiu (2026-11-17)

O autor leu esta lista e decidiu o que passa para a mão do jogador. Em cada uma, a regra de hoje
continua como padrão ("Regra histórica"), e o jogador pode fixar o valor ("Fixado pelo governo"),
como já faz com os impostos.

| | Vira decisão do jogador | Nota do autor | Simulação | Tela |
|---|---|---|---|---|
| D1 | Tamanho da folha | O clero nem sempre é folha do governo: depende da lei de religião e Estado. | thread do ensino | Orçamento |
| D2 | Salário público | | thread do ensino | Orçamento |
| D3 | Compras do Estado e a parte militar | | thread do ensino | Orçamento |
| D4 | Excedente e reserva do Tesouro | | thread do ensino | Orçamento |
| D5 | Teto de déficit | | thread do ensino | Orçamento |
| D6 | Calote e reestruturação | Decisão rara, em Agendas. | thread do ensino | Agendas |
| D7 | Câmbio | Quando o padrão-ouro entrar. | fase 4 | Moeda e dívida |
| D8 | Fila de obras públicas | "Uma lista de prioridades que o jogador define e em que pode ordenar tudo." | thread do ensino | Orçamento |
| D9 | Dívida interna ou externa | O mercado decide quem compra; o jogador escolhe onde emite. | thread do ensino | Moeda e dívida |
| D13 | Construir tropas de propósito | O efetivo de paz continua pela lei. | thread da guerra | Forças |

**Ficam por regra:** a correção fiscal, o recorte de zeros e a bolsa colonial.

### A lista de regras de hoje

- **Tamanho da folha do Estado:** servidores, soldados e clero são uma fração fixa da população: 1,2 %, 1 % e 0,5 % (`econ::monthly::STATE_SHARE`). O Estado não contrata nem demite.
- **Salário público:** o salário privado médio vezes o peso da profissão vezes 0,7 (`econ::public::PUBLIC_PAY`).
- **Compras do Estado** (armas, papel, transporte): a receita mais o déficit que a regra da dívida permite, menos a folha, os serviços, as obras e os juros. O piso é 60 % da folha (`GOODS_PER_PAYROLL`), cortado quando o Tesouro não conseguiu tomar emprestado ou quando os juros pesam (`goods_budget`, `austerity`, piso de 25 %).
- **Gastar o excedente:** acima do dobro da reserva, o Tesouro gasta um décimo do excesso por semana (`SURPLUS_SPENDING`). Se a dívida passar de 60 % do PIB, ele paga dívida com esse excesso em vez de gastar (`money::REPAY_SHARE`, `REPAY_ABOVE`).
- **Reserva do Tesouro:** 13 semanas de receita antes de começar obras (`RESERVE_WEEKS`). Abaixo de 8 semanas, paga a folha em atraso, até o piso de 10 % (`PAY_RESERVE_WEEKS`, `PAY_FLOOR`).
- **Quanto se endivida:** o déficit que a regra da dívida permite por semana e o crédito para obras por mês (`money::deficit_room`, `borrowing_room`).
- **Com quem se endivida:** os títulos são comprados pela poupança ociosa de casa, pelo fundo de investimento e pela poupança ociosa de fora, em frações fixas por semana (`HOME_ABSORB`, `POOL_ABSORB`, `FOREIGN_ABSORB`). A parte em moeda estrangeira cresce para países pobres, com inflação ou com calote (`fx_share`).
- **Calote e reestruturação:** se o serviço em moeda estrangeira passa de 30 % da receita, ou se o Tesouro deve um quarto da receita do ano que não conseguiu tomar enquanto deve fora, metade da dívida externa é perdoada e o país fica três anos fora do crédito externo. Se os juros tomam demais da receita, o Estado reestrutura a dívida (`money.rs`).
- **Correção fiscal:** com os impostos na regra automática, a taxa sobe um degrau quando os juros passam de 12 % da receita e dois quando passam de um quarto, e desce com pouca dívida e caixa folgado (`yearly_taxes`). O que os tetos da lei de impostos cortam vira imposto sobre o consumo, até o teto (`cap_taxes`, `INDIRECT_CEILING`).
- **Quais obras públicas fazer:** a infraestrutura onde o produto é escasso (preço acima de 110 % do de referência), até 200 obras (`public_works`). As prioridades por categoria são do jogador; a escolha de cada obra é da regra.
- **Pesquisa paga pelo Estado:** a meta de gasto em pesquisa e a parte do Tesouro (`research_target`, `STATE_RESEARCH_SHARE`). A prioridade de pesquisa vem das leis.
- **Bolsa colonial:** o que as colônias pagam e recebem da metrópole (`colonial_purse`).
- **Câmbio:** a moeda deriva para a paridade enfraquecida pelo prêmio de risco (`exchange_target`). Não há âncora nem câmbio fixo escolhido (regime internacional na fase 4).
- **Recorte de zeros:** a moeda perde oito zeros quando o índice de preços passa do limite (`REDENOMINATION`).
- **Forças em paz** (#115, entra desligado; ligar é o PR seguinte): o efetivo de terra segue, todo janeiro, a fração de 1925 dos homens em idade de serviço, ajustada pela lei de conscrição. As encomendas têm teto na parte militar das compras do Estado.
