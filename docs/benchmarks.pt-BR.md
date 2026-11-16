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
| Leis | 211 em 60 grupos | do Victoria 3 + algumas | fase 3 |
| Grupos de interesse / ideologias | 8 / 89 | do Victoria 3 | fase 3 |
| Eventos (journal entries) / decisões | 135 / 74 | — | fora do escopo por ora (pouco sorteio) |
| Desempenho | autor relata 5 min 26 s numa amostra de 1951 e 4 min no save de referência de 1900 do jogo base, sem dizer o que foi medido | — | 100 anos em 105 s, tick mais lento 46 ms |

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

## Fases seguintes: o que o CWE tem e nós ainda não

| Sistema do CWE | Nossa fase | Nota |
|---|---|---|
| Leis e formas de governo refeitas, grupos de interesse e ideologias por posição em temas (estilo Victoria 2) | 3 | nossa meta: movimentos persistentes, causa visível |
| Corrupção | 3 | |
| Descolonização | 3 | movimento político, não cadeia de eventos |
| ONU funcional (FMI, missões de paz, sanções, adesão) | 4 | o FMI casa com Bretton Woods e a dívida |
| Estados clientes como esfera de influência | 4 | |
| Superestados (formação de uniões) | 4 | |
| Corrida espacial | 2 e 5 | já há satélite e foguete reutilizável na árvore; falta o prestígio |
| Guerra nuclear | 5 | arma nuclear com consequência é meta desde a fase 0 |
