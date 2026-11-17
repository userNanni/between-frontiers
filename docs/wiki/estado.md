# Estado do jogo, fase a fase

Atualizado em **2026-11-17**. "No jogo" é o que está na `main`; "em rascunho" é PR aberto; "desenhado"
é só documento. Ainda não há versão jogável para o público. A lista das fases com a diferença de
cada uma está em [`architecture.pt-BR.md`, "Fases"](../architecture.pt-BR.md#fases).

| Fase | O que é | Estado |
|---|---|---|
| 0. Fundação | engine determinística, mapa de 1925 com 870 estados, i18n, teste de 100 anos | no jogo (#4 a #22, #7 a engine) |
| 1. Economia | mercados, fábricas, pops, investimento autônomo, comércio mundial, transporte e energia por região | no jogo (#23, #29, #36 e seguintes); comércio com parceiro em rascunho (#113) |
| 2. Pesquisa | árvore de 1925 a 2025, maturidade por país, laboratórios que custam | no jogo (#30 a #33, #107) |
| 3. Política nacional | leis, grupos de interesse, movimentos, regimes, orçamento, moeda, banco central e dívida | no jogo (#34, #39, #40, #45, #53) |
| 4. Diplomacia | fronteiras, tratados por cláusulas, livro de dívida entre Estados, influência, blocos, capital entre países | em andamento: veja abaixo |
| 5. Guerra | forças, mobilização, crise, frentes, suprimento, paz | contrato e dados de 1925 no jogo, fora do tick (#64, #72, #75); forças em paz no tick em revisão (#115); crise e declaração desenhadas |
| 6. IA estratégica | interesses medidos, hostilidade, escada de crise, agendas | desenhado ([`history-rails.pt-BR.md`](../history-rails.pt-BR.md)) |
| 7. Confiança e história | medo do mercado e crise de 29, trilho histórico, personagens, casus belli | desenhado (#46); entra por último |

## Fase 4 em detalhe

| Frente | No jogo | Em rascunho ou desenhado |
|---|---|---|
| Fronteiras | operações no tick: transferir, separar, anexar, dissolver (#95, #102); trocar de país e libertar território no cliente (#96, #108) | fábricas da colônia na independência: em estudo |
| Tratados | tratados como cláusulas, os de 1925 convertidos (#92, #93); livro de dívidas no passo semanal (#101); cláusula vira operação de fronteira (#104) | proposta e avaliação no tick (etapa 4 dos tratados); cláusula de promessa: desenhada |
| Influência e blocos | desenho decidido (#103); contrato fora do tick (#105) | influência no tick (#110) e serviço exterior (#112), rascunhos |
| Capital entre países | contrato fora do tick (#70); capital por método (#111) | capital entre países no tick (#109), rascunho |
| Comércio com parceiro | — | rascunho (#113) |

## Interface e saves

No jogo: menu, campanhas e escolha de país no mapa (#8), HUD modular (#58), teclado e editor de
atalhos (#59), painel lateral e Agendas (#56), escala de interface (#63), pontos de restauração e
salvamento automático configurável (#98). Em revisão: formato de save versionado, com o salvamento
automático trimestral por padrão (#114).

## O que vem depois

- Equilíbrio fino, depois da fase 6 ([Princípios](principios.md#funcionalidade-antes-de-equilíbrio)).
- Referência gerada dos dados de `data/` (bens, prédios, tecnologias, leis, cláusulas), por script.
- Guias de como jogar, depois da primeira versão estável.
