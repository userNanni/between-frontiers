# Estado do jogo, fase a fase

Atualizado em **2026-11-17**. "No jogo" é o que está na `main`; "em rascunho" é PR aberto; "desenhado"
é só documento. Ainda não há versão jogável para o público. A lista das fases com a diferença de
cada uma está em [`architecture.pt-BR.md`, "Fases"](../architecture.pt-BR.md#fases).

A coluna "Andamento" é uma **estimativa da coordenação por contagem de itens do plano** de cada
fase, não uma medida de quanto do jogo está pronto. Onde há dois números, o segundo conta também os
PRs abertos.

| Fase | O que é | Andamento | Estado |
|---|---|---|---|
| 0. Fundação | engine determinística, mapa de 1925 com 870 estados, i18n, teste de 100 anos | 100 % | no jogo (#4 a #22, #7 a engine) |
| 1. Economia | mercados, fábricas, pops, investimento autônomo, comércio mundial, transporte e energia por região | ~85 % | no jogo (#23, #29, #36 e seguintes); comércio com parceiro em rascunho (#113) |
| 2. Pesquisa | árvore de 1925 a 2025, maturidade por país, laboratórios que custam | ~85 % | no jogo (#30 a #33, #107) |
| 3. Política nacional | leis, grupos de interesse, movimentos, regimes, orçamento, moeda, banco central e dívida | ~70 % | no jogo (#34, #39, #40, #45, #53) |
| 4. Diplomacia | fronteiras, tratados por cláusulas, livro de dívida entre Estados, influência, blocos, capital entre países | ~30 %; ~55 % com os PRs abertos | em andamento: veja abaixo |
| 5. Guerra | forças, mobilização, crise, frentes, suprimento, paz | ~20 %; ~35 % com os PRs abertos | contrato e dados de 1925 no jogo, fora do tick (#64, #72, #75); forças em paz no tick em revisão (#115); crise e declaração decididas e desenhadas |
| 6. IA estratégica | interesses medidos, hostilidade, escada de crise, agendas | ~5 % | desenhado ([`history-rails.pt-BR.md`](../history-rails.pt-BR.md)) |
| 7. Confiança e história | medo do mercado e crise de 29, trilho histórico, personagens, casus belli | ~10 % | desenhado (#46); entra por último |

## Fase 4 em detalhe

| Frente | No jogo | Em rascunho ou desenhado |
|---|---|---|
| Fronteiras | operações no tick: transferir, separar, anexar, dissolver (#95, #102); trocar de país e libertar território no cliente (#96, #108) | fábricas da metrópole na independência: decididas, desenhadas |
| Tratados | tratados como cláusulas, os de 1925 convertidos (#92, #93); livro de dívidas no passo semanal (#101); cláusula vira operação de fronteira (#104) | proposta e avaliação no tick (etapa 4 dos tratados); cláusula de promessa; renegociar e não cumprir como regras gerais de todo acordo: desenhados |
| Influência e blocos | desenho decidido (#103); contrato fora do tick (#105) | influência no tick (#110) e serviço exterior (#112), rascunhos |
| Capital entre países | contrato fora do tick (#70); camadas no save (#109); capital por método (#111) | fluxo mensal e remessa (#117), rascunho; fábricas da metrópole na independência decididas (estudo no #118) |
| Comércio com parceiro | — | rascunho (#113) |

## Interface e saves

No jogo: menu, campanhas e escolha de país no mapa (#8), HUD modular (#58), teclado e editor de
atalhos (#59), painel lateral e Agendas (#56), escala de interface (#63), pontos de restauração e
salvamento automático configurável (#98), cartão da campanha com o menu "…" (#108), formato de save
versionado com cadeia de migração e chaves do catálogo, e o salvamento automático trimestral por padrão
(#114).

## O que vem depois

- Equilíbrio fino, depois da fase 6 ([Princípios](principios.md#funcionalidade-antes-de-equilíbrio)).
- Referência gerada dos dados de `data/` (bens, prédios, tecnologias, leis, cláusulas), por script.
- Guias de como jogar, depois da primeira versão estável.
