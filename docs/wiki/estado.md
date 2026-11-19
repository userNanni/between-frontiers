# Estado do jogo, fase a fase

Atualizado em **2026-11-19**, com o `main` em `f8515ae`. Ainda não há versão jogável para o público.
A lista das fases com a diferença de cada uma está em
[`architecture.pt-BR.md`, "Fases"](../architecture.pt-BR.md#fases).

## Como ler esta página

- **No jogo e ligado:** está no `main` e roda numa partida normal.
- **No jogo e desligado:** está no `main`, mas uma chave de [`data/rules.ron`](../../data/rules.ron)
  o deixa parado. Regra que muda os números de todos os países entra desligada; ligar é um PR
  próprio, julgado pelos agregados do mundo
  ([Princípios](principios.md#regra-que-muda-o-mundo-inteiro-entra-desligada)). Quem quer ver a regra
  rodar liga a chave com `BF_RULES=nome` (ou `bf-bench --rule nome`).
- **Em PR:** pull request aberto, ainda fora do `main`.
- **Desenhado:** só documento.

A coluna "Andamento" é uma **estimativa da coordenação por contagem de itens do plano** de cada
fase, não uma medida de quanto do jogo está pronto. Onde há dois números, o segundo conta também os
PRs abertos.

## O que está ligado hoje

Das chaves de `data/rules.ron`, só três estão ligadas:

| Chave | O que liga | Desde |
|---|---|---|
| `forces` | forças armadas em paz, com os soldados das formações de 1925 | #198 |
| `trade_by_pair` | comércio por par de países, pelo custo de chegada | #203 |
| `treaties_in_force` | os tratados de 1925 em vigor como acordos (chamam terceiros, registram quebras) | #208 |

Todo o resto que entrou desde 2026-11-17 está no `main` **desligado**: crise e declaração
(`crisis`), guerra terrestre (`war`), mesa de negociação (`negotiation`), reputação (`reputation`),
teste mensal dos acordos (`clause_tests`), registro de cada linha do livro de dívida
(`ledger_record`), independência por movimento (`independence`), capital entre países
(`foreign_flow`, `foreign_contract`, `foreign_technique`), canais de comércio e investimento da
influência (`influence_ties`), ensino por coorte (`schooling`), capacidade fiscal
(`fiscal_capacity`), serviços na união aduaneira (`union_services`) e duas regras de trabalho e
salário (`jobs_follow_use`, `small_market_wage_floor`).

## As fases

| Fase | O que é | Andamento | Estado |
|---|---|---|---|
| 0. Fundação | engine determinística, mapa de 1925 com 870 estados, i18n, teste de 100 anos | 100 % | no jogo (#4 a #22, #7 a engine) |
| 1. Economia | mercados, fábricas, pops, investimento autônomo, comércio, transporte e energia por região | ~88 % | no jogo; comércio por par ligado (#203), alcance do comércio a 100 % (#228); comércio bruto (#231) e tarifa (#220) em PR, desligados |
| 2. Pesquisa | árvore de 1925 a 2025, maturidade por país, laboratórios que custam | ~85 % | no jogo (#30 a #33, #107) |
| 3. Política nacional | leis, grupos de interesse, movimentos, orçamento, moeda, banco central e dívida | ~80 % | no jogo; alavancas do orçamento e da dívida (#165, #186, #189) e fila de obras (#176), com a regra histórica como padrão; ensino por coorte (#152) e capacidade fiscal (#165) desligados |
| 4. Diplomacia | fronteiras, tratados, livro de dívida entre Estados, influência, blocos, capital entre países | ~60 %; ~70 % com os PRs abertos | veja "Fase 4 em detalhe" |
| 5. Guerra | forças, encomenda, crise, frentes, apoio à guerra, ocupação, paz | ~45 %; ~55 % com os PRs abertos | veja "Fase 5 em detalhe" |
| 6. IA estratégica | interesses medidos, hostilidade, escada de crise, agendas | ~10 % | desenhado ([`history-rails.pt-BR.md`](../history-rails.pt-BR.md)) |
| 7. Confiança e história | medo do mercado e crise de 29, trilho histórico, personagens, casus belli | ~10 % | desenhado (#46); entra por último |

## Fase 4 em detalhe

| Frente | No jogo e ligado | No jogo e desligado | Em PR | Desenhado |
|---|---|---|---|---|
| Fronteiras | operações de fronteira (#95, #102); trocar de país e libertar território (#96, #108) | independência por movimento e a resposta da metrópole (#136, #144) | ligar a independência, com o estudo da onda (#173) | — |
| Tratados | cláusulas e livro de dívida (#93, #101, #104); reconhecimento (#145); os tratados de 1925 em vigor (#208); união aduaneira de San Marino com a Itália (#222) | acordos em vigor e quebras (#181), teste mensal (#192), reputação (#194), registro de cada linha (#158), mesa de negociação (#196, #199, #210, #212), preço da revisão de dívida (#216) | — | cláusula de promessa |
| Influência e blocos | influência no tick (#110), serviço exterior (#112), blocos lidos das cláusulas (#132), lentes de influência e de blocos (#164, #167, #171) | canais de comércio e investimento (#143) | — | — |
| Capital entre países | camadas no save (#109), capital por método (#111) | fluxo e remessa, fábricas da colônia na independência, técnica pela fábrica estrangeira (#166, #175) | ligar o fluxo de capital (#174) | — |
| Comércio | comércio por par (#203); tarifa lida dos acordos em vigor (#205, #207); alcance a 100 % (#228) | serviços na união aduaneira (#211) | comércio bruto (#231), tarifa que conta (#220) | — |

## Fase 5 em detalhe

| Frente | No jogo e ligado | No jogo e desligado | Em PR | Falta |
|---|---|---|---|---|
| Forças | forças em paz (#198); encomenda de forças (#151); dissolver tropas e navios (#202) | — | composição das forças (ramo `feat/war-composition`, sem PR) | — |
| Crise | — | crise e declaração (#157); terceiros pelos acordos em vigor (#184) | — | — |
| Guerra em terra | — | guerra no tick (#206); ritmo da frente pela densidade e pelo entrincheiramento (#224); o exército no passo da maior parte (#227); apoio à guerra, rendição política e ocupação (#217) | geração do exército, cultura de comando e ruptura (#230, rascunho, à espera de decisão do autor); a tela dela (#232) | — |
| Outros domínios | — | — | — | guerra colonial, mar e ar, bloqueio (desenho em #197) |

## Interface e saves

No jogo antes de 2026-11-17: menu, campanhas e escolha de país (#8), HUD modular (#58), teclado e
editor de atalhos (#59), painel lateral e Agendas (#56), escala de interface (#63), pontos de
restauração (#98), menu "…" da campanha (#108), save versionado com o salvamento automático
trimestral (#114).

Entraram desde então:
- escolher o país pelo teclado (#137) e o X que fecha as janelas (#134, #139);
- aba de dívidas entre governos (#134) e "Renegociar", que abre a mesa com a revisão (#213);
- mesa de negociação nas Agendas: propor, responder, contrapropor (#210);
- os acordos do país nas Agendas: em vigor, quebras, reputações e denúncia (#221);
- dissolver tropas na aba Forças (#209);
- comércio por parceiro no painel do país (#215);
- janela da guerra e lente do controle (#219), com apoio à guerra, ocupação e devastação (#229);
- da crise à paz só pelo teclado (#225);
- passe de consistência: números que nunca cortam, a wiki conferida contra os comandos (#226).

Várias dessas telas leem sistemas desligados: abrem, mas só mostram movimento com a chave ligada.

## O que vem depois

- **A primeira rodada de otimização** foi adiantada, só com mudanças que deixam o resultado
  idêntico (#140 e seguintes). Está **em pausa** enquanto a máquina roda em carga reduzida.
- Depois da fase 6, nesta ordem ([Princípios](principios.md#otimizar-antes-de-calibrar)):
  1. as demais rodadas de otimização e simplificação do código;
  2. calibração (equilíbrio fino), cujas rodadas servem também de teste de desempenho.
- Referência gerada dos dados de `data/` (bens, prédios, tecnologias, leis, cláusulas), por script.
- Guias de como jogar, depois da primeira versão estável.
