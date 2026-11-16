# Referências de comparação

Toda fase fecha comparada com o mod **Cold War Era (CWE)** do Victoria 3, a referência fixa do
projeto (<https://github.com/settintotrieste/Victoria-3-Cold-War-Era-Mod-CWE>), e com o
**Tech & Res**, que cobre o mesmo século. Para cada sistema, a pergunta é: o que eles têm que
nós não temos, o que temos que eles não têm, e o que fazer com a diferença.

O CWE não tem licença no repositório, o que quer dizer todos os direitos reservados. Ele serve só
para comparar sistemas e ordens de grandeza. Nada dele entra no projeto: dado, texto, mapa,
ícone ou nome de evento. O mesmo vale para o Tech & Res.

## Como usar

- **Ao fechar uma fase:** atualize a tabela da fase abaixo (números nossos e deles) e diga, no PR,
  onde ficamos atrás e se isso entra no plano de melhoria (`architecture.pt-BR.md`).
- **Para medir:** os números do CWE saem da contagem das definições em `common/` do repositório
  dele (tecnologias, bens, edifícios, leis), conferida no commit lido. Recontar quando o mod
  avançar.
- **Desempenho:** o CI compara cada PR com a base (mesma máquina, 5 anos, 1 thread), põe o custo
  de cada sistema no resumo do job e avisa quando a simulação fica mais de 15 % mais cara. A
  rodada noturna de 100 anos publica a mesma tabela. Ao fechar uma fase, atualize a linha
  "Desempenho" abaixo com uma rodada de 100 anos sob `flock /tmp/bf-bench.lock`.
- **Não é meta copiar.** Um sistema deles que contraria um princípio nosso (pouco sorteio, causa
  visível, determinismo) fica de fora, com o motivo escrito aqui.

## Visão geral (contagem de 2026-11-16)

| | CWE | Tech & Res | Between Frontiers |
|---|---|---|---|
| Período | 1950–2092 (eras de 1900 a 2100) | 1936–1999 e depois | 1925–2025 |
| Motor | Victoria 3 | Victoria 3 | Próprio (Laplace-Schelling), determinístico |
| Unidade de simulação | Estados do Victoria 3 | Estados do Victoria 3 | 870 estados, 83 mercados nacionais |
| Bens | 59 | Victoria 3 + 30 | 81 |
| Edifícios | 133 (1213 métodos de produção) | Victoria 3 + 30 | 82 (213 métodos) |
| Tipos de população | 15 | do Victoria 3 | 13 profissões |
| Tecnologias | 270 | 200+ | 168 |
| Leis | 211 em 60 grupos | do Victoria 3 + algumas | 99 em 24 grupos, forma de governo inclusa (fase 3d, ligadas) |
| Grupos de interesse / ideologias | 8 / 89 | do Victoria 3 | 8 / nenhuma (postura pelos efeitos) |
| Movimentos políticos | 27 | do Victoria 3 | 23 tipos (fase 3d, ligados) |
| Eventos (journal entries) / decisões | 135 / 74 | — | na fase 7, entradas históricas com condição de entrada e de saída e agenda da IA (`history-rails.pt-BR.md`) |
| Desempenho | autor relata 5 min 26 s numa amostra de 1951 e 4 min no save de referência de 1900 do jogo base, sem dizer o que foi medido | — | 100 anos em 134 s com 12 threads, tick mais lento 98 ms (`main` 662bb7a, 2026-11-16; na fase 2 eram 105 s e 46 ms) |

## Fase 1: economia

| Sistema | CWE | Nós | Diferença |
|---|---|---|---|
| Recursos | carvão, petróleo, ferro, cobre, chumbo, urânio, enxofre, ouro, borracha | cerca de 20 minerais com jazidas de 1925, terras raras e bauxita desde o início | à frente |
| Serviços modernos | serviços corporativos, profissionais, financeiros, comunicação, mídia, software, lazer, imóveis, robôs | serviços, finanças, telecomunicações, transporte | atrás: mídia, software e imóveis vão para o plano (fases 3 e 4) |
| Perspectiva econômica | sistema "Economic Outlook" | preços e utilização por mercado, sem indicador agregado | plano: ciclo econômico com confiança e investimento |
| Corrupção | sistema próprio | não há | fase 3 |

## Fase 3b: infraestrutura

| | Victoria 3 e CWE | Nós |
|---|---|---|
| Unidade | pontos de infraestrutura por estado, dados pelas ferrovias e portos; os edifícios gastam pontos e, sem eles, o acesso ao mercado cai | transporte e eletricidade como bens do estado, com preço próprio, frete pago pelas minas, fazendas e fábricas, e parte da sobra passando às vizinhas por terra |
| Eletricidade | bem do mercado nacional | do estado até a rede nacional (tecnologia); depois, do país |
| Quem constrói | o jogador e o investimento | o investimento, pelo preço do estado, e a obra pública, onde a demanda não é atendida |

O CWE herda a infraestrutura do Victoria 3; não há o que ele faça a mais nisso.

## Fase 2: pesquisa

| | CWE | Nós |
|---|---|---|
| Estrutura | três árvores (produção 120, militar 80, sociedade 70), 10 eras de 20 anos, 27 tecnologias por era | uma árvore, 12 áreas, 6 eras, 13 níveis |
| Dependências | cada tecnologia pede uma só, da mesma árvore: são linhas, como no Hearts of Iron IV; nenhuma cruza de árvore | 225 ligações, 47 % cruzando de área; toda área alimenta outras |
| Custo | por era, de 5000 a 50000 (10× em dez eras) | por nível, de ≈ 40 a ≈ 6000; cada tecnologia custa ao menos 10 % mais que seus requisitos |
| Maturidade | pesquisada ou não | ciência, protótipo, indústria, difusão, por país; vale da morte, imitação, difusão |
| Calibração | não publicada | ano de cada tecnologia contra a data histórica (atraso médio de 2,6 anos no protótipo) |

O CWE escolheu linhas para fugir do "espaguete" de centenas de tecnologias. A nossa escolha é a
oposta: as áreas se alimentam umas das outras, e a tela de grafo (filtro por área, caminho aceso)
é a resposta ao espaguete.

## Fase 3c: moeda, banco central e dívida

| | CWE | Nós | Diferença |
|---|---|---|---|
| Moeda | uma só, a do Victoria 3 (sem inflação nem câmbio) | por país: nível de preços, inflação, câmbio por paridade com inércia e risco | à frente |
| Dívida | a do Victoria 3: limite de crédito pelo ouro e pelo PIB, juro, falência | títulos por credor (casa, banco central, exterior, dólar), prêmio de risco, calote com perdão parcial e histórico | à frente no mecanismo |
| Banco central | não há | regra de Taylor, autonomia histórica, taxa do jogador | à frente |
| Perspectiva econômica | sistema "Economic Outlook" | inflação, juros e dívida por país, ainda sem confiança nem ciclo de investimento | atrás no ciclo; o plano continua |
| FMI | dentro da ONU funcional | não há; o calote só fecha o crédito por três anos | fase 4, com Bretton Woods |

O CWE só serviu de comparação: nada dele entrou.

## Fase 3: política nacional

Contagem nas definições de `common/` do CWE no commit `4d301a8` (2026-11-15): 211 leis em 60
grupos, 8 grupos de interesse, 89 ideologias, 27 movimentos políticos. Desenho nosso em
`politics.pt-BR.md`; desde a fase 3d, roda na simulação.

| | CWE (Victoria 3) | Nós |
|---|---|---|
| Leis | 211 em 60 grupos; muitos grupos são instituições do Estado (chefe de governo, legislativo, judiciário, partidos, centralização) | 99 em 24 grupos; a forma de governo é um grupo de leis (10 formas), e o voto num grupo só |
| O que a lei faz | modificadores de script e gatilhos por lei | efeitos declarados em 37 números sobre sistemas nomeados; 10 já caem em código que existe (migração, impostos, escola, hospital, renda da terra) |
| Postura dos grupos | ideologias atribuídas aos grupos (89), trocadas com o líder | soma dos interesses materiais do grupo sobre os efeitos da lei, lida termo a termo; sem ideologia |
| Base dos grupos | tipos de população com atração por grupo | 13 profissões repartidas sem sobra nem dupla contagem; comunidade opcional por id |
| Movimentos | 27, criados por gatilho e com peso de criação; apoio de personagens por ideologia | 22 tipos; nascem só da causa medida (indicador ou efeito de lei), crescem pela margem da condição mais fraca, decaem com meia-vida; troca de regime e revisionismo inclusos |
| Mudança de lei pela IA | aprovação com chance por mês, apoio dos grupos no governo | determinística: movimento empurra o próximo degrau; cede quando mobilização × 4 + apoio ponderado dos grupos passa de um limiar que cai com a legitimidade |
| Efeito na economia | modificadores por lei | 10 efeitos ligados (migração, tetos de imposto, arrecadação, parte do Estado em escola e hospital, meta de matrícula, renda da terra) |
| Corrupção | grupo de leis próprio e movimentos a favor e contra | grupo `civil_service`, efeito `corruption` no orçamento, movimento anticorrupção |
| Descolonização | política colonial como grupo de leis e eventos | governo e trabalho coloniais como leis; independência como movimento que mede a discriminação e o autogoverno |
| Mídia | — | imprensa e radiodifusão como leis, com censura e mídia estatal; alcance da mídia como indicador |
| 1925 | começa em 1950 | 83 países com lei por grupo e fonte legal, 132 mudanças datadas de referência |

Ficamos atrás em instituições do Estado (judiciário, legislativo, partidos, centralização,
política externa e armas de destruição em massa), que o CWE trata como leis. Partidos e forma de
governo entram quando a fase 3 ligar o peso político; política externa e armas, nas fases 4 e 5.
Não entram de propósito: ideologia sorteada com o líder e movimento com peso de criação, que
contrariam causa visível e pouco sorteio.

## Fase 4: diplomacia (só o contrato e os dados de 1925)

| Sistema do CWE | Nós (`diplomacy.pt-BR.md`) | Diferença |
|---|---|---|
| ONU funcional (adesão, FMI, missões de paz, sanções) | Liga das Nações com membros datados, Conselho permanente e territórios que governa (Sarre) ou protege (Danzig); sem regra ainda | atrás até ligar; a Liga de 1925 é fraca de propósito, e o FMI casa com o regime monetário já registrado por país |
| Estados clientes como esfera de influência | sujeição de `countries.ron` com o instrumento citado, e proteção sem sujeição (Cuba, Panamá, Haiti, Egito, Nejd) como tratado desigual | à frente nos dados: a tutela informal tem nome, data e lado forte |
| Superestados (formação de uniões) | uniões aduaneiras e monetárias com data (BLEU, Suíça–Liechtenstein, União Monetária Latina) | atrás: falta a regra de formação |
| — | regime monetário por país (ouro, divisa-ouro, fixo, prata, papel, moeda alheia) e tarifa média de 1925 | só nosso |

## Fases seguintes: o que o CWE tem e nós ainda não

| Sistema do CWE | Nossa fase | Nota |
|---|---|---|
| Leis e formas de governo refeitas, grupos de interesse e ideologias por posição em temas (estilo Victoria 2) | 3 | ligado na fase 3d (acima) |
| Corrupção | 3 | lei e efeito no catálogo; falta ligar ao orçamento |
| Descolonização | 3 | movimento político, não cadeia de eventos; no catálogo |
| ONU funcional (FMI, missões de paz, sanções, adesão) | 4 | o FMI casa com Bretton Woods e a dívida |
| Estados clientes como esfera de influência | 4 | |
| Superestados (formação de uniões) | 4 | |
| Corrida espacial | 2 e 5 | já há satélite e foguete reutilizável na árvore; falta o prestígio |
| Guerra nuclear | 5 | arma nuclear com consequência é meta desde a fase 0 |
