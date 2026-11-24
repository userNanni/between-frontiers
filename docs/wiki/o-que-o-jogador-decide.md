# O que o jogador decide

Cada decisão de um país que o jogador pode tomar, com o comando da simulação, onde fica na tela e o
atalho. O teste `crates/bf-client/tests/player_commands.rs` garante que nenhum comando de jogador
novo chega sem tela e sem atalho: comando novo em `GameCommand` falha nele até ter os dois, ou até
entrar na lista de exceções com a razão. O mesmo teste confere que a seção "No jogo" desta página
nomeia cada comando e cada exceção e que nenhuma linha dela está "em revisão": o PR que
traz a decisão a escreve aqui com o próprio número, sem tabela à parte para PR aberto.

Atalhos do perfil padrão; todos se mudam no editor de atalhos (Configurações → Editar atalhos). Numa janela mais alta que a tela, Shift+PageDown e Shift+PageUp rolam a página, e as dicas de tecla do que entrou à vista aparecem.

O caminho inteiro da crise à paz (exigir, declarar depois da recusa, atacar numa frente, propor a paz pela mesa) se faz só pelo teclado; o teste de tela `crisis_to_peace_by_keyboard` o percorre.

## No jogo

| Decisão | Comando | Onde fica | Atalho | Estado |
|---|---|---|---|---|
| Escolher o país da campanha | `ChoosePlayer` | Tela de escolha de país: clique no mapa, ou "Procurar país" (lista com busca pelo nome ou pela sigla), e "Começar" | Ctrl+F abre a lista; Enter escolhe o primeiro; S começa | no jogo (#8; teclado #137) |
| Jogar com outro país | `SwitchCountry` | Campanhas → "…" da campanha → "Jogar com outro país". A troca passa o controle: o país novo responde às crises, à mesa, às chamadas e à postura das frentes; o deixado volta às regras da IA na semana seguinte (as posturas e a ordem de expedição dele caem) | Shift+F10 abre o "…"; J escolhe | no jogo (#96, #108) |
| Pesquisa: foco | `SetResearch` | Pesquisa: clique numa tecnologia; "Limpar foco" | R abre; Delete limpa o foco | no jogo |
| Impostos por classe, ou regra automática | `SetTaxRates` | Governo → Orçamento | G abre; B a aba | no jogo |
| Prioridades das obras públicas | `SetPublicPriority` | Governo → Orçamento | G, B | no jogo |
| Lei de migração | `SetMigrationLaw` | Governo → Orçamento | G, B | no jogo |
| Campanha de alfabetização de adultos: começar ou parar | `LiteracyCampaign` | Governo → Orçamento → "Campanha de alfabetização", com confirmação; só com a regra do ensino ligada (`data/rules.ron`) | G, B | no jogo, regra desligada (PR do ensino) |
| Taxa de juros do banco central, ou regra automática | `SetPolicyRate` | Governo → Moeda e dívida | G abre; M a aba | no jogo |
| Banco central: histórico, autônomo ou subordinado | `SetCentralBank` | Governo → Moeda e dívida | G, M | no jogo |
| Propor uma lei | `SetLaw` | Governo → Leis: escolher a lei e "Propor" | G, L; Ctrl+Enter propõe | no jogo |
| Folha do Estado: servidores e clero por mil habitantes (D1) | `SetPayroll` | Governo → Orçamento → "Folha e Tesouro": regra histórica ou fixado | G, B | no jogo (#165) |
| Salário público sobre o privado (D2) | `SetPublicPay` | Governo → Orçamento → "Folha e Tesouro" | G, B | no jogo (#165) |
| Compras do Estado e a parte militar (D3) | `SetStatePurchases` | Governo → Orçamento → "Folha e Tesouro": regra histórica ou fixado, com o teto militar da semana | G, B | no jogo (#186) |
| Reserva do Tesouro e destino do excesso (D4) | `SetTreasury` | Governo → Orçamento → "Folha e Tesouro" | G, B | no jogo (#165) |
| Teto de déficit (D5) | `SetDeficitCap` | Governo → Orçamento → "Folha e Tesouro" | G, B | no jogo (#165) |
| Ordenar a fila de obras públicas, pôr e tirar obra (D8) | `SetWorksOrder`, `AddWork`, `CancelWork` | Governo → Obras: no topo, quantas obras há, quanto falta pagar, o ritmo e o que trava a fila; uma linha por obra com a data em que fica pronta, ↑ ↓ e cancelar em dois cliques; as privadas da mesma fila, só leitura; "Mandar construir": primeiro o prédio, depois o estado, com o que ele já tem, o custo e a data; "só a minha lista" zera as prioridades | G, Shift+B | no jogo (#176; tela refeita no #276) |
| Onde emitir a dívida: em casa, fora em moeda própria, fora em dólar (D9) | `SetIssuance` | Governo → Moeda e dívida → "Onde emitir": regra histórica ou fixado; o vendido e o sem vender de cada praça; aviso quando a moeda própria pedida passa do que os de fora aceitam | G, M | no jogo (#190) |
| Tarifa por bem e por parceiro (CT1, CT2) | `SetGoodsTariff`, `SetPartnerTariff` | Governo → Comércio: os bens pelas 9 categorias, cada uma abre e fecha (fechada, mostra a faixa das tarifas, "26,0 % a 40,0 %", e quantos bens estão fixados); cada bem na "Regra histórica" ou "Fixado", de 0 a 200 %, de ponto em ponto; "Aplicar ao grupo" e "Voltar à regra" por categoria. Por parceiro, os 10 maiores vendedores ao país e a busca pelo nome: o acordo em vigor, o ajuste em pontos sobre a tarifa de cada bem (−100 a +100) e a tarifa efetiva; quando o acordo corta a sobretaxa, a tela diz até onde. Passar do teto de propósito (quebra de cláusula) vem depois. A receita aduaneira é uma linha nova do Orçamento | G, Shift+T; K abre a categoria; F escolhe regra ou fixado; S + número muda a tarifa do bem; G aplica ao grupo; P + número muda o ajuste do parceiro; A procura o país | no jogo |
| Encomendar tropas, navios e aviões (D13) | `OrderForces` | Governo → Forças → "Nova encomenda": tipo, elementos, colonial; o custo de construir e o de todo ano; a encomenda grande pede um segundo clique | G, T | no jogo (#151) |
| Cancelar uma encomenda | `CancelOrder` | Governo → Forças → "Encomendas": "Cancelar" e confirmação; o construído vai ao depósito | G, T | no jogo (#151) |
| Dissolver tropas, dar baixa em navios e aviões | `DisbandForces` | Governo → Forças → "Dissolver…" na linha do tipo: quantos elementos, o que volta ao depósito (valor e bens), quantos homens vão à reserva ou para casa, e o aviso de que o exército de paz volta ao tamanho de paz em janeiro; "Dissolver" confirma, com ponto de restauração antes (motivo "dissolver forças"). Tira primeiro as formações encomendadas. Reservistas chamados voltam por `Mobilize` | G, T | no jogo (#209) |
| Composição das forças: parcela sobre rodas e em lagarta | `SetComposition` | Governo → Forças → "Composição das forças": a parcela de hoje sobre rodas e em lagarta; o alvo pela "Regra histórica" ou "Fixado pelo governo", com as duas parcelas em passos de um ponto (cada passo vale na hora, como as alavancas do Orçamento); a infantaria em conversão e, quando para, o motivo em frase (o país não sabe fazer o tipo, o teto militar não sobra, não há infantaria, os preços). A infantaria troca de equipamento até 2 % dos homens de terra por ano, sem crescer o exército | G, T; K + R ou F escolhe regra ou fixado, K + 1 a 4 os passos | no jogo (#235), regra desligada (`war`) |
| Programa nuclear: parcela do PIB por ano | `NuclearProgramme` | Governo → Forças → "Programa nuclear" (e a entrada nas Agendas, grupo Forças): começar com 0,1% do PIB por ano, ajustar em passos de 0,1% até 2%, ou encerrar. A tela mostra o custo por ano, o que já foi entregue, cada passo da cadeia (fissão, enriquecimento, bomba, termonuclear) com o estágio, os meses do passo em curso no ritmo do último mês, as ogivas prontas e encomendadas, as usinas de enriquecimento e o material físsil (preço e quanto o mercado entregou). Só com a regra `war` | G, F; U + S começa, U + 1 e U + 2 mudam a parcela, U + E encerra | PR do programa nuclear, regra desligada (`war`) |
| Testar a arma nuclear | `NuclearTest` | Governo → Forças → "Programa nuclear": "Testar a arma", quando há ao menos uma ogiva. O teste acontece na semana seguinte e torna o arsenal conhecido de todos (N3); até ele, os outros contam a metade das ogivas. Não se desfaz. Só com a regra `war` | G, F; U + T | PR do degrau nuclear na crise, regra desligada (`war`) |
| Usar a arma nuclear | `NuclearUse` | Janela "Guerra", sob o "Balanço de dissuasão" de uma guerra em que você tem ogivas prontas nos lançadores: "Uso nuclear", com contraforça (as forças de terra do inimigo, um décimo dos mortos civis) ou contravalor (as cidades), e quantas ogivas. A tela mostra os mortos estimados, os homens postos fora da guerra, a mancha na reputação de segurança, a resposta esperada do inimigo (N5: o mesmo degrau e o mesmo número) e o tabu com os anos desde o último uso (N7). "Lançar…" pede a confirmação, que diz em frase o que muda, e grava um ponto de restauração antes. O lançamento sai na semana seguinte. Só com a regra `war` | Shift+W; N + F contraforça, N + V contravalor, N + L lançar | rascunho local da fatia do uso, regra desligada |
| Libertar território | `Liberate` | Agendas → Decisões → "Libertar território", com confirmação | J abre as Agendas | no jogo (#96) |
| Mobilizar e desmobilizar a reserva | `Mobilize` | Agendas → "Forças armadas em paz" → "Mobilizar a reserva…" (ou desmobilizar), com confirmação e ponto de restauração; a aba Forças do governo (T) mostra os reservistas que se apresentam | J abre as Agendas | no jogo (#115) |
| Prolongar ou encurtar o serviço militar (item 9 da guerra) | `SetServiceMonths` | Agendas → "Forças armadas em paz" → "Prolongar o serviço…": os meses entre o mínimo e o máximo declarados (6 e 36), em dois passos, com ponto de restauração; antes de confirmar, a tela diz quanto o exército de paz cresce (ou encolhe), em quantos anos e quanto custa por ano. A aba Forças mostra a reserva por turma (ano de saída, homens, treino) e o total mobilizável. Encurtar usa o mesmo comando. A IA não mexe no prazo (fase 6). Só com a regra `reserve_cohorts` | J abre as Agendas; na entrada das forças, com Alt: S prepara, L e M mudam de 6 em 6 meses, Y confirma, N cancela | feito; regra desligada |
| Terra arrasada contra um inimigo: dar e retirar a ordem | `SetScorchedEarth` | Agendas → Decisões → "Terra arrasada contra {inimigo}…": em dois passos, com ponto de restauração; antes de confirmar, o que está em risco (as regiões do país em luta, os níveis de edifício e a parcela do PIB delas) e a parcela que queima pelas semanas de luta (até a metade). A mesma decisão retira a ordem. A janela "Guerra" mostra, por frente, o estado da ordem e o que já queimou, sem dar a ordem. A IA não usa (fase 6). Só com a regra `scorched_earth` | J abre as Agendas; o atalho vem com a tela | tela no mesmo trem (thread da interface); até lá, exceção no teste, regra desligada, rascunho |
| Responder à exigência de independência de uma colônia: governo próprio, domínio, independência ou recusar | `AnswerDemand` | Agendas → Decisões → "{colônia} exige uma resposta": mobilização contra as marcas, o que segura o movimento, a conta de manter a colônia, o preço de cada resposta, o prazo e confirmação; ponto de restauração antes (motivo "exigência") | J abre as Agendas; a dica de tecla escolhe a exigência, 1 a 4 a resposta, R revisa, Y confirma | no jogo (#144), regra desligada |
| Exigir de outro país: estados, o país inteiro, pagamento, tributo | `Demand` | Agendas → Decisões → "Exigir de {país}…": o país selecionado no mapa ou escolhido pelo nome ("Escolher país pelo nome…", C, digitar e Enter); a conta do alvo, o prazo, revisar e enviar; ponto de restauração antes (motivo "guerra"). Só com a regra `crisis` | J; E abre; C procura o país; U o país inteiro; S + número os estados; M muda pagamento e tributo; R revisa; Y envia | no jogo (#157), regra desligada |
| Aceitar ou recusar uma exigência feita ao país | `AnswerCrisis` | Agendas → Crises → "{país} exige…": o que pede, o prazo, a resposta provável; em dois passos, com ponto de restauração | J; K + número abre; A aceita, R recusa; Y confirma | no jogo (#157), regra desligada |
| Depois da recusa: declarar guerra ou recuar | `Declare` | Agendas → Crises → a crise recusada: declarar ou recuar, em dois passos, com ponto de restauração; quem entraria pelos acordos | J; K + número; A declara, R recua; Y confirma | no jogo (#157), regra desligada |
| Honrar ou não a chamada de um aliado à guerra | `AnswerCall` | Agendas → Crises → "{aliado} chama…": as duas saídas e o preço de cada uma; em dois passos, com ponto de restauração | J; H + número; H honra, R recusa; Y confirma | no jogo (#184), regra desligada |
| Propor um acordo a outro país | `ProposeTreaty` | Agendas → Decisões → "Propor acordo a {país selecionado}…": a mesa, com o que cada lado dá em sua coluna (cláusulas do catálogo, parâmetros dentro dos limites), prazo e aviso, a conta de cada lado por termo, a resposta provável do outro, e por que a proposta não pode ir (limite, reconhecimento, igual recusada há menos de 6 meses); ponto de restauração antes (motivo "tratado"). Só com a regra `negotiation` | J abre; T a mesa; A e S acrescentam cláusula, K escolhe, R tira, V muda parâmetro, W prazo e aviso, P propõe | no jogo (#210), regra desligada |
| Aceitar, recusar ou contrapropor um acordo recebido | `AnswerProposal` | Agendas → Decisões → "{país} propõe um acordo (N semanas)": o que cada lado dá, a conta do jogador, o prazo; aceitar e recusar em dois passos, contrapropor na mesma mesa (até a terceira rodada); ponto de restauração antes de aceitar ou contrapropor | J; P escolhe a proposta; A, R, C; Y confirma | no jogo (#210), regra desligada |
| Renegociar uma dívida entre governos | `ProposeTreaty` (cláusula `revise_debt`) | Governo → Dívidas entre governos → a linha → "Renegociar": abre a mesa nas Agendas com a revisão da dívida, a partir dos termos de hoje, e o outro lado fixo. Os parâmetros da revisão e o preço dela na conta chegam com a simulação (pedido ao thread dos tratados) | G, Shift+M; Z escolhe a linha; N renegocia | no jogo (#213), regra desligada |
| Postura de cada frente na guerra: segurar ou atacar | `SetPosture` | Janela "Guerra" (Shift+W, ou a linha da guerra nas Agendas): uma frente por país inimigo, com "Segurar" e "Atacar", os homens e o poder de fogo dos dois lados, as baixas e o avanço da semana e cada segmento com os termos do poder, quanto já entrou em cada estado, a largura da frente, os homens por km de cada lado e o entrincheiramento. Acima das frentes, o apoio à guerra de cada lado (os termos, os grupos de interesse, a velocidade sob o voto e a censura, a contagem e a queda), a ocupação (resistência, guarnição e o que o ocupante leva) e a devastação. O padrão é segurar. "Propor paz…" abre a mesa nas Agendas com o líder inimigo. Só com a regra `war` | Shift+W; F + número troca a postura; S + número abre os segmentos; Q propõe a paz | no jogo (#219), regra desligada |
| Expedição a uma colônia em revolta: quantos homens mandar | `SetExpedition` | Janela "Guerra", na guerra colonial da sua metrópole: "Expedição a X", com os homens desembarcados contra o teto (metade dos homens de terra) numa barra com a meta marcada, os amarrados na unidade (e a parcela do exército) e os irregulares armados do inimigo. A meta é uma alavanca: "Regra histórica" (a IA manda o bastante para cobrir 20 por mil da população que você segura, O3) ou "Fixado", com passos de 10 mil homens até o teto. Os homens desembarcam, ou voltam, pelos portos, no ritmo de cada semana. Só com a regra `war` | Shift+W; E + R regra, E + F fixado; E + 1 menos, E + 2 mais | no jogo (#241), regra desligada |
| Denunciar um acordo em vigor | `Denounce` | Agendas → Decisões → "Acordos do país (N em vigor)": cada acordo com as partes, o ano, o prazo, o aviso e as cláusulas; "Denunciar…" — com aviso, diz o ano em que acaba, sem quebra; sem aviso, diz que é quebra de cada cláusula que o país dá e pede dois passos. Ponto de restauração antes (motivo "denunciar um acordo"). A mesma tela mostra as quebras, as causas e a reputação geral e de segurança de cada país. A denúncia só é processada com a mesa (`negotiation`) | J; O abre; U + número denuncia; Y confirma, N mantém | no jogo (#221), regra desligada |
| Impor e levantar embargo (CT6) | `Embargo` | Agendas → Decisões → "Embargo comercial (N em vigor)": o país pelo nome ("Escolher país pelo nome…", C, digitar e Enter); a confirmação diz em palavras o que fecha (o comércio nos dois sentidos a partir da próxima semana, o que o país vendeu ao alvo e comprou dele no ano passado, e que levantado volta devagar, uns seis meses); "Impor o embargo" confirma, com ponto de restauração antes (motivo "impor um embargo"). Abaixo, os embargos em vigor do país e contra ele; os do país com "Levantar…" em dois passos, sem ponto de restauração. O embargo contra o país só quem impôs levanta | J; M abre; C procura o país; Y confirma, N cancela; L + número levanta | no jogo |
| Aderir ao padrão-ouro, sair, segurar a paridade sob controle cambial, ou devolver à regra (PO6) | `SetGoldStandard` | Agendas → Decisões → "Aderir ao padrão-ouro…" ou "Sair do padrão-ouro…": cada escolha com o que daria nesta semana (`gold::preview`): a paridade e o quanto a moeda fica acima do mercado, a cobertura que teria contra a legal e o piso, a folga ou falta e os juros acima da regra de Taylor; a paridade de antes da guerra sem dado aparece desabilitada com o motivo; dois passos, com ponto de restauração antes (motivo "câmbio"). Só com `gold_standard` | J; U abre; U + número escolhe; R devolve à regra; Y confirma, N mantém | no jogo (#297), regra desligada |
| No ouro, abaixo do piso: regra histórica (sai), defender a paridade ou desvalorizar (PO6) | `SetGoldPosture` | Governo → Moeda e dívida → "Câmbio e reservas": "Regra histórica: sai do ouro" ou "Fixado pelo governo" (Defender, Desvalorizar), cada um com a consequência em uma linha; a seção mostra a cobertura contra a legal e o piso, as semanas abaixo e as reservas por moeda | G, M; O + R ou F, O + D ou V | no jogo (#292, #297), regra desligada |
| Controle de capital: ligar e desligar (PO5) | `SetCapitalControls` | Governo → Moeda e dívida → "Câmbio e reservas" → "Capital privado": as posições longas e curtas nos dois sentidos; "Ligar o controle de capital…" e o segundo passo dizem o que muda (posições congeladas, sem capital novo, o prêmio sobe); ponto de restauração antes se os Ajustes pedirem. Só com `capital_follows_rate` | G, M; K pede, Y confirma, N mantém | no jogo (#292), regra desligada |

Exceção: `Borders` (operação de fronteira) não é do jogador. Vem de tratados, da paz, de testes e
de ferramentas; o jogador liberta a própria terra com `Liberate`.

Exceção provisória: `SetScorchedEarth` (terra arrasada) chega antes da tela, atrás da regra
`scorched_earth`, desligada; a decisão nas Agendas vem do thread da interface no mesmo trem.

## Desenhadas, sem comando ainda

| Decisão | Onde deve ficar | Depende de |
|---|---|---|
| Entrar e sair de um bloco | Agendas | blocos como rótulo sobre tratados (#132) e proposta no tick |
| Vetar capital estrangeiro | Governo | fluxo de capital (#117) |
| Calote e reestruturação (D6) | Agendas | thread do ensino |
| Câmbio (D7) | Governo → Moeda e dívida | padrão-ouro (fase 4) |
| Abrir missão diplomática | Agendas | serviço exterior (#112) |

### Comércio: o que o dono decidiu (2026-11-18)

Desenho em `docs/trade.pt-BR.md`, seções 13 e 14. Tudo entra atrás de chave. A ordem é o alcance a
100 %, o comércio bruto, CT3, CT4 e CT5 (#220), depois a tabela por bem com a tela, depois embargo
e guerra.

| | Decisão | Simulação | Tela |
|---|---|---|---|
| CT1 | A tarifa é por bem, uns 60, não por grupo. Na "Regra histórica" vale a taxa da lei, igual para todos os bens; em "Fixado pelo governo" o jogador dá a taxa de cada bem. | thread do comércio | Governo → Comércio (no jogo), os bens pelas 9 categorias do catálogo |
| CT2 | O jogador dá sobretaxa ou preferência a um parceiro dentro do que os acordos em vigor permitem. Passar de um teto é quebra de cláusula, com o preço de reputação. | thread do comércio, com o dos tratados | Governo → Comércio |
| CT3 | A receita aduaneira vai ao Tesouro, numa linha nova do orçamento. | #220 | Orçamento → "Receita aduaneira" (no jogo) |
| CT4 | A tarifa encarece o importado e reduz a importação, com a elasticidade da literatura. | #220 | — |
| CT5 | O ponto de partida é a taxa de 1925 de cada país (`diplomacy.ron`); a lei move a taxa a partir dela. | #220 | — |
| CT6 | Embargo unilateral por comando, nos dois sentidos, que reabre devagar. Decisão rara, em Agendas, com ponto de restauração. | thread do comércio (`Geography::closures`) | Agendas → "Embargo comercial" (no jogo) |
| CT7 | A guerra fecha o comércio entre os beligerantes. | thread da guerra (`war_pairs`) e do comércio (o fechamento do par) | — |
| V1 | O volume do comércio se trata agora (opção A): o saldo chega inteiro ao mundo (alcance a 100 %) e cada mercado também troca variedades do que produz e consome (comércio bruto), calibrado para uns 15 % do PIB mundial em 1925, com o custo no tick otimizado. | thread do comércio | — |

### Padrão-ouro e balanço de pagamentos: o que o dono decidiu (2026-11-19)

Desenho em `docs/state.pt-BR.md`, "Padrão-ouro, reservas e balanço de pagamentos (fase 4)". O dono
aceitou as oito recomendações. Tudo entra atrás de chave e desligado; ligar é PR próprio, contra a
faixa.

| | Decisão | Simulação | Tela |
|---|---|---|---|
| PO1 | O comércio passa a mover dinheiro entre países: quem importa paga a quem exporta. O que a reserva do país não cobre é pago na moeda dele, e os países em superávit a guardam como divisa: é a dívida que a regra da fatia 3 cobra. | fatia 2, feita atrás da chave `payments_settle`, desligada (`econ::payments::settle`) | — |
| PO2 | Quem está no ouro em 1925 é dado com fonte; as voltas depois de 1925 saem da regra da IA, e a data histórica fica só como conferência. | fatia 3, feita atrás da chave `gold_standard`, desligada: `data/economy/gold.ron` (dez países no ouro em 1925, com a paridade), a volta pela regra no câmbio do dia (`econ::gold`) | — |
| PO3 | Um país sai do ouro pela regra: cobertura abaixo do piso por meses seguidos, ou a saída do principal país-reserva. | fatia 3, desligada: 13 semanas abaixo de metade da cobertura legal, ou a saída do emissor de um quarto ou mais das reservas | — |
| PO4 | Defender a paridade é juro acima da regra de Taylor por ponto de cobertura que falta, e moeda retirada quando a falta continua. | fatia 3, desligada: 0,3 ponto de juro por ponto de cobertura que falta, até 6; sem emissão abaixo da cobertura, retirada depois de 26 semanas | — |
| PO5 | Controle de capital é decisão do jogador e entra na regra da IA. | fatia 4, feita atrás da chave `capital_follows_rate`, desligada (`econ::capital`): o capital privado, longo e de curto prazo, segue o retorno esperado de cada país; o controle fecha a conta financeira (a renda continua paga, o prêmio sobe 1 ponto); a IA o liga quando segura a paridade por controle cambial; só a simulação: `SetCapitalControls` | Governo → Moeda e dívida (fatia 5) |
| PO6 | Aderir e sair do ouro é decisão rara, com confirmação e ponto de restauração; a tela mostra o custo de cada paridade. | fatia 3, só a simulação: `SetGoldStandard`, `SetGoldPosture` | Agendas (fatia 5) |
| PO7 | Divisas contam como reserva, e quem guarda a moeda que desvaloriza perde. | fatia 2 (a reserva por moeda); fatia 3: a divisa vale o que o câmbio do emissor diz, e quem a guarda perde o que o emissor deixa de dever | — |
| PO8 | Ouro novo das minas entra pela produção de cada país, com fonte. | fatia 2, desligada: 62,5 % da produção vai ao banco central (Delegação do Ouro da Liga das Nações, 1932) | — |

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

### Apoio à guerra e ocupação: o que o autor decidiu (2026-11-18)

O autor ficou com a recomendada nas seis escolhas. Cada uma é dado em `data/military/war.ron`, e
trocar uma é trocar o dado (#217).

| | Decisão | Referência |
|---|---|---|
| A1 | O apoio à guerra é medido por grupo de interesse, pesado pela influência de cada grupo. | Grupos de interesse e influência da fase 3d (`politics`). |
| A2 | A censura atrasa a queda do apoio. Abaixo do colapso, ela alimenta o movimento de troca de regime: a queda vem de repente. | Rússia de 1917 e Alemanha de 1918 (Reiter e Stam, 2002). |
| A3 | Com o apoio abaixo de 100 por mil, o país cede mesmo com o jogador. A rendição passa pela política: o movimento de troca de regime vence, e o governo novo pede a paz. Numa república cai só o gabinete, sem corte de reputação. | Novembro de 1918; a P3 lê a troca de forma de governo. |
| O1 | O ocupante leva do território ocupado de 20 % do PIB no primeiro ano até o teto de 33 %, como na França ocupada, vezes a eficiência da guarnição. | Milward, 1970; Occhino, Oosterlinck e White, 2008. |
| O2 | O combate devasta toda semana, em proporção ao fogo, os dois lados da frente, com mortes civis. A fila de obras reconstrói. | Harrison, 1998. |
| O3 | A guarnição vai de 2 a 20 homens por mil habitantes, conforme a resistência. Faltando guarnição, a extração cai na mesma proporção e a resistência cresce. | Quinlivan, 1995. |

**Também decidido (CT7):** a guerra fecha o comércio entre os países em guerra. A guerra informa
os pares (`war::fight::war_pairs`), e o comércio fecha o par (`econ::trade::Geography::closures`).

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
