# Fronteiras: mudança de dono, nascimento e morte de países

Estado: desenho (parte A). Nada aqui roda no tick ainda. A operação entra na parte B, o primeiro
gatilho (independência por movimento) na parte C.

Hoje o mapa político é o de 1925 para sempre: em 2025 a Índia ainda é o Raj, a África ainda é
colônia, a URSS não se dissolve. Este documento define **a operação de mudar o dono de um
território**, de que dependem a descolonização (fase 3), a paz com perda de território (fase 5) e
os fios históricos da fase 7 (`history-rails.pt-BR.md`).

Princípios, os de sempre (`architecture.pt-BR.md`): causa visível, pouco sorteio, determinismo.
**Nenhuma mudança de fronteira acontece por data.** As datas reais de
`data/history/borders.ron` são referência para o placar, como os anos das tecnologias.

## 1. Auditoria: o que é indexado por país, mercado ou território

O índice de país é o mesmo em toda parte: a ordem do cenário, que é o índice de
`GameWorld::countries`, de `econ.markets`, de `politics.*` e de `research.*`. `Region::market`
aponta para ele; `Region::colony` separa metrópole e colônias do mesmo mercado. A demografia nova
(PR #54) acrescenta o **território** (`demography::TerritoryId`: metrópole ou colônias de um mercado).

Daí a primeira regra: **o índice de país é estável.** País novo entra no fim de todas as tabelas;
país que morre fica no lugar, sem estados (lápide), e pode renascer com a mesma etiqueta (os
bálticos em 1991, a Áustria em 1945). Nada é removido, nada é renumerado, e um save antigo continua
apontando para o mesmo país.

Legenda das regras de repartição: **p** = parcela da população do antigo dono que sai com os
estados; **w** = parcela econômica, o valor adicionado a preço-base das plantas dos estados que
saem (`Method::value_added` × níveis × ocupação, mais a subsistência), sobre o do antigo dono
inteiro. As duas se calculam na hora da operação, do estado salvo, sem sorteio.

### Vai com o estado (nada a fazer)

| O quê | Onde | Observação |
|---|---|---|
| Pops (tamanho, emprego, renda, poupança, satisfação) | `Region::pops` | a poupança das famílias é dinheiro delas e vai junto |
| Plantas (níveis, método, desgaste, lucro) | `Region::plants` | o dono da planta é o mercado da região: hoje não existe planta de dono estrangeiro (ver "Capital estrangeiro") |
| Jazidas, terra, subsistência, produtividade, alfabetização | `Region` | |
| Bens locais (transporte, eletricidade antes da rede) | `Region::local` | preço e razões continuam; a oferta às vizinhas passa a valer só dentro do novo mercado |
| Pirâmide etária e escolaridade por estado (PR #54) | `DemographyState::ages`, `schooling` | por estado; vão com ele |
| Comunidades de 1925 por estado | `data/scenario/1925/communities.ron` | dado estático por estado |

### Precisa ser refeito, repartido ou criado

| O quê | Onde | Regra |
|---|---|---|
| Dono do estado | `Region::market` | o novo dono |
| Metrópole ou colônia | `Region::colony` | transferência: mantém, salvo se a operação disser (anexação de metrópole vizinha vira metrópole); secessão: o país novo nasce sem colônias, tudo vira metrópole |
| Vizinhas do mesmo mercado | `Region::neighbours` | recalculadas, para os dois donos, do grafo de fronteiras terrestres do mapa (`StateDef::neighbours`), que passa a viver nas regras (`Rules`), fora do mundo salvo |
| Preços, ofertas, demandas, razões do mercado | `Market::price` e vizinhos | país novo copia os do antigo dono; a semana seguinte fecha o mercado dele sozinha |
| Fundo de investimento privado | `Market::pool` | reparte por **w** (é lucro retido das empresas daqueles estados) |
| Tesouro (caixa) | `Market::treasury` | fica com o antigo dono (caixa do Estado que continua); país novo nasce com zero e toma emprestado na semana |
| Produtividade do país e por grupo | `Market::tfp`, `tech_tfp` | copiadas |
| Fila de obras | `Market::queue`, `construction` | as obras nos estados que saem passam para a fila do novo dono com os pontos que faltam; a obra pública passa a ser paga pelo novo Tesouro; `construction` (bens ainda a comprar) reparte pela parcela dos pontos que mudaram |
| PIB do ano | `gdp_running`, `gdp_last_year` | repartido por **w** (é fluxo, não conserva, mas é base de razões de dívida e de imposto) |
| Funcionalismo | `gov_workers` | recontado no mês seguinte |
| Orçamento | `Market::budget` | regras (alíquotas, ajuste, prioridades) copiadas; contas da semana zeradas; `revenue_avg` × **w** |
| Lei de migração e efeitos das leis | `Market::laws` | migração copiada; efeitos reescritos pela política no dia seguinte |
| Moeda, banco central | `Money` (nível de preços, inflação, câmbio, taxa, autonomia) | país novo nasce com moeda própria e os números do antigo dono (decisão 2) |
| Dívida pública | `Money::debt_home`, `debt_bank`, `debt_abroad`, `debt_fx`, `cost_*` | decisão 1; o que passa sai do antigo dono na mesma quantia (soma mundial igual) |
| Créditos no exterior | `Money::foreign_assets` | poupança aplicada fora: reparte por **p** (é das famílias) |
| Calotes e exclusão | `defaults`, `default_history`, `excluded_until` | país novo nasce limpo; transferência não mexe |
| Leis em vigor | `PoliticsState::laws` | país novo: decisão das leis abaixo; quem recebe território aplica as suas; quem recebe a primeira colônia ganha as leis coloniais do antigo dono (as que valiam ali) |
| Alvos das leis dirigidas | `PoliticsState::targets` | país novo: vazio; o resto, sem mudança |
| Peso dos grupos de interesse | `PoliticsState::clout` | país novo copia; recalculado no mês seguinte pelos pops |
| Movimentos | `PoliticsState::movements` | país novo: os do antigo dono no escopo que valia ali (o colonial vira metropolitano), com o de independência zerado (venceu); antigo dono: o movimento colonial perde o apoio na parcela de recrutas que saiu |
| Legitimidade | `PoliticsState::legitimacy` | país novo: 1.000 (refeita no fim do ano); antigo dono: sem mudança |
| Pedidos de lei pendentes, registro de mudanças | `pending`, `changes` | intocados; o registro de fronteiras é próprio |
| Contexto de imposto | `PoliticsState::taxes` | país novo copia; refeito no mês |
| Lugares de personagem | `leaders`, `MovementState::leader` | vazios até a fase 7 |
| Pesquisa | `ResearchState::stage`, `progress`, `focus`, `points`, `effects`, `colony_stage`, `colony_progress` | país novo herda (decisão 6; de colônias, o estágio colonial); `firsts` é mundial e não muda |
| Linha de país | `GameWorld::countries` (`CountryState`) | etiqueta nova; os números de 1925 do cenário ficam zerados |
| Escolha do jogador | `GameWorld::player` | decisão 4 |
| Série anual | `Economy::history` (`YearRecord` por mercado) | intocada; o país novo começa a sua no ano seguinte |
| Diplomacia: organizações, tratados, regime monetário, tarifa | `diplomacy.rs` (dado, fora do tick) | quando entrar no tick (fase 4): o país novo não herda tratado nem assento; quem morre sai de todos; o regime monetário do país novo é o do antigo dono até decisão própria |
| Zonas marítimas e estreitos | `sea.rs`, controle pelos donos das margens | já derivado do dono do estado na hora; nada a guardar |
| Forças de 1925 | `forces.rs` (dado) | quando entrar no tick: unidades ficam com quem as mantém; guarnição nos estados que saem passa ao país novo (fase 5) |

### A demografia do PR #54

Ela guarda por estado (pirâmide, escolaridade), o que vai junto sem trabalho, e por **território**
(`demography::TerritoryId`: metrópole ou colônias de um mercado, com `new(market, colony)`,
`of(&Region)`, `market()`, `colony()` e `count(markets)`; a operação usa só esse tipo, nunca a
aritmética do índice), o que precisa de regra:

- `territories` (fatos fixos: latitude, longitude, família linguística, cultura principal): ganha
  as entradas do país novo (`TerritoryId::count` cresce com os mercados) e é recalculada para os
  territórios que mudaram (`demography::territories` já calcula do mapa e das comunidades).
- `vital` por território: o país novo começa com os números do território de onde saiu.
- `diaspora` é guardada por (território de origem, território de destino, cultura), não por estado.
  - **Como destino:** a parcela que vai para o novo dono é a da **população urbana** dos estados
    que mudam sobre a do território, que é onde a demografia põe os imigrantes (regra proposta pelo
    thread da demografia).
  - **Como origem:** a mesma regra, pela população urbana e rural da cultura nos estados movidos
    (comunidades por estado); sem gente da cultura nos estados movidos, a origem não muda.
  - Repartição em `Fixed`, resto de arredondamento com o antigo dono: ninguém some (teste de
    conservação das pessoas na diáspora).
- `homes` (minorias com rota própria por território) se refazem das comunidades dos estados
  movidos, para os dois donos.
- `waves` abertas que saem de ou vão para o território: terminam com uma variante nova
  (`WaveEnd::Border`) e reabrem no ciclo seguinte se a pressão continuar.
- As leis `immigration` e `emigration` lidas em `Demography::migration_laws` já são por país.

A parte B é escrita contra essa API, sem depender do tipo enquanto o #54 não entra na `main`.

### O cliente

Hoje o cliente lê o dono **do cenário**, uma vez (`bf-client/src/world.rs`: `Province::owner`,
`State::owner`, `Country::states`; `map/mod.rs`: `owner_of`, campos de distância e rótulos; textura
de donos em `map/gpu.rs`). Nome (`country-<TAG>` em `assets/i18n/*/countries.ftl`), bandeira
(`assets/flags/<TAG>.png`) e cor vêm da etiqueta e do `CountryDef`.

Para não quebrar: a cada leitura mensal (`World::live`), comparar `Region::market` e `colony` com o
último visto; se mudou, refazer `owner_of`, a tabela de donos da GPU, os campos de distância e os
rótulos, e as listas `Country::states`. O `CountryDef` do país novo vem do catálogo de países
possíveis (seção 3). O que o thread de interface vai precisar ler está na seção 7.

## 2. As operações

Quatro operações, todas sobre **estados inteiros** (decisão 5), todas determinísticas, todas
disparadas por um comando que só enfileira (`GameCommand::Borders`) e executadas por um sistema
próprio (`borders`), que tem as regras (grafo do mapa, catálogo, árvore, leis). O comando não tem
acesso às regras; por isso não executa ele mesmo, como `SetLaw`.

```
Transfer  { from, to, states, colony: Option<bool> }          estados entre dois países que existem
Secede    { from, states, tag, laws: Option<template> }       estados viram país novo (ou renascido)
Annex     { from, into, colony: bool }                        um país deixa de existir
Status    { tag, overlord: Option<tag> }                      súdito vira soberano ou o contrário
```

Renomear (Pérsia → Irã, URSS → Rússia) não é operação de fronteira: é a etiqueta de exibição
(`display`) do país, que o catálogo e a fase 7 trocam. Fica de fora daqui.

### Transferência

Cessão, anexação parcial, devolução. Os estados mudam de mercado; o resto segue a tabela da
auditoria. Dinheiro: o `pool` reparte por **w**, `foreign_assets` por **p**; o Tesouro e a dívida
ficam com quem cede (o Estado que perde território não deixa de dever; a paz da fase 5 pode pôr a
dívida na mesa como exigência). Leis: as de quem recebe. Pesquisa: a de quem recebe. Fábricas: vão
junto, com o método que têm, mesmo que quem recebe não saiba construí-lo (operar não é produzir,
`research.pt-BR.md`).

### Secessão ou independência

Um conjunto de estados vira país novo. É a colônia que se emancipa e, com vários herdeiros, a
partilha da Índia e a dissolução da URSS e da Iugoslávia (uma secessão por herdeiro; o que sobra
continua sendo o antigo dono).

| Item | Regra |
|---|---|
| Etiqueta, nome, cor, bandeira, capital | do catálogo de países possíveis; fora dele, a regra da seção 3 |
| Mercado | novo, no fim da tabela, com preços e produtividade do antigo dono |
| Tesouro | zero |
| Dívida | decisão 1 |
| Moeda | própria, com os números do antigo dono (decisão 2) |
| Leis | o modelo (`template`) do catálogo; sem ele, os grupos de economia que valiam no território (os da metrópole, que já valiam nas colônias) e, nos outros grupos, o modelo `new_state` (a definir em `laws.ron` na parte B: sufrágio restrito, liberdades parciais, nenhuma lei colonial) |
| Forma de governo | do catálogo; sem ele, `presidential_republic` |
| Tecnologia | decisão 6 |
| Fábricas de dono estrangeiro | hoje não existem; quando existirem (`foreign_capital.rs`, fase 4) ficam com o dono; o país novo decide pela lei `foreign_capital` se expropria, e a expropriação entra na memória (`expropriation_premium`) como qualquer outra |
| População e diáspora | vão com os estados; diáspora como na auditoria |
| Tratados e organizações | nenhum; entra pelas regras da fase 4 |
| Placar | o país novo tem série própria com a etiqueta ISO do país moderno (seção 4) |

**Renascimento:** se a etiqueta já existe como lápide, o país volta no mesmo índice, com o
mercado zerado e as regras de país novo. Não há memória do Estado anterior além da etiqueta.

### Anexação ou união total

Um país deixa de existir: Áustria em 1938, os bálticos em 1940, a reunificação alemã, o Vietnã em
1976, o Iêmen em 1990. É uma transferência de todos os estados mais:

- **Tesouro e dívida** passam inteiros a quem anexa (sucessão de Estado: o caso alemão de 1990);
  a dívida em moeda própria do anexado vira dívida em moeda de quem anexa, ao câmbio do dia.
- **Pool** e **créditos no exterior** passam inteiros.
- O país vira lápide: mercado sem regiões, linhas de política e pesquisa congeladas, fora dos
  sistemas (eles já toleram mercado vazio? teste na parte B; se não, o sistema pula lápide).
- O jogador que era esse país: decisão 4.

União de iguais (Hejaz e Négede em 1932, Tanganica e Zanzibar em 1964) é anexação mais troca de
nome do que fica.

### Divisão por ocupação

Alemanha e Coreia em 1945, Áustria até 1955. **Não precisa de operação própria agora.** Na
referência histórica a zona é registrada como `occupier` na secessão que cria o Estado do
pós-guerra (RFA, RDA, as duas Coreias). A ocupação militar em si é controle, não soberania, e
pertence à guerra (fase 5: estados ocupados, governo militar, exigências da paz); quando existir,
a paz poderá terminar numa secessão com ocupante, que é uma secessão cujo país novo nasce com um
`Status` de súdito do ocupante.

### Conservação

Em toda operação, antes e depois, somados sobre o mundo: população, poupança dos pops, `pool`,
`treasury`, dívida em cada forma (em unidades reais, ao câmbio do dia), `foreign_assets`, pessoas
na diáspora. Teste na parte B: igualdade exata em `Fixed`, com o resto de arredondamento atribuído
ao antigo dono.

### De onde vem a mudança (gatilhos)

Neste trabalho as operações existem e são disparáveis por comando. Os gatilhos ficam desenhados,
cada um na sua fase:

| Gatilho | Fase | Como chama a operação |
|---|---|---|
| Movimento de independência com causa medida | 3 (**parte C deste trabalho**) | `independence` de uma unidade colonial passa de um limite de mobilização e a metrópole não o reprime nem responde: `Secede` |
| Tratado (cessão, compra, devolução, união) | 4 | o tratado assinado executa `Transfer` ou `Annex` na entrada em vigor |
| Paz depois de guerra | 5 | exigência territorial aceita: `Transfer`, `Secede` com ocupante, ou `Annex` |
| Entrada histórica | 7 | marco da entrada é um dos três acima com condição; nunca a operação seca |
| Jogador | já | decisão 3 |

## 3. Países que ainda não existem

Arquivo `data/borders/candidates.ron`: catálogo de países possíveis, os que a história produziu de
1925 a 2025 e que não estão no cenário.

```
(
    tag: "NGA",
    wikidata: "Q1033",
    color: (0, 135, 81),
    capital_lon_lat: (3.3792, 6.5244),
    flag: Some("NGA"),             // assets/flags/NGA.png, ATTRIBUTION.md
    laws: Some("new_state"),       // modelo de data/scenario/1925/laws.ron
    government: "parliamentary_republic",
    states: ["NGA-WESTERN", "NGA-EASTERN", ...],   // núcleo: estados que a história lhe deu
    source: "...",
)
```

- **Nome** em `assets/i18n/{en,pt-BR}/countries.ftl` (`country-NGA`, `.article`, `-capital`), como
  os de 1925.
- **Bandeira** só de domínio público ou licença livre, pelo mesmo `tools/flags/fetch.py`, numa
  lista própria (`tools/flags/flags_new.csv`, com a revisão do Commons fixada por SHA-1 e a licença
  conferida a cada rodada); a atribuição vai para `assets/flags/ATTRIBUTION.md`.
- **Núcleo** (`states`): de `data/history/borders.ron` quando a história fez o país; senão, os
  estados de 1925 cuja maior parte da área está no país moderno (a mesma conta do placar,
  `state_countries` em `tools/history/scorecard.py`).
- **Cor** distinta das vizinhas no mapa de 1925 e de 2025, escolhida na geração do catálogo.

**O que a história não produziu** (o jogador ou um movimento separa um pedaço que nunca foi país):

1. Se os estados que saem batem com o núcleo de um candidato em mais da metade da população, usa
   o candidato.
2. Senão, um país genérico: etiqueta `X00` a `X99` em ordem de criação; nome pelo estado mais
   populoso do conjunto (`"{state}"`, com o artigo do idioma), capital no centro desse estado,
   cor derivada da do antigo dono por regra fixa, bandeira genérica desenhada pelo cliente (três
   faixas nas cores do país, sem arte de terceiros). O jogador poderá renomear (interface, depois).

## 4. Referência histórica e placar

`data/history/borders.ron`: as mudanças de soberania de 1925 a 2025 na resolução dos estados do
mapa, uma linha por operação, com data, tipo (`Transfer`, `Secession`, `Annexation`, `Status`,
`Rename`), de, para, estados, estados mudados só em parte, ocupante, item do Wikidata e fonte.
Mudança menor que um estado fica de fora e anotada (`partial`).

**Como o placar passa a comparar** depois que as unidades mudam:

- **Unidade de 1925 fixa por estados.** O bench já grava a série de cada um dos 870 estados. A
  unidade `GBR/colonies` passa a ser a soma dos estados que eram colônias britânicas em 1925, seja
  qual for o dono hoje. A comparação de população e PIB que já existe continua idêntica, com ou sem
  mudança de fronteira (no mundo sem mudanças dá os mesmos números de hoje).
- **País atual contra país moderno.** Os países novos têm etiqueta ISO do país moderno, então a
  série deles (PIB, população, impostos, dívida, inflação, que só existem por país) se compara
  direto com a referência do país moderno, a partir do ano em que os dois existem.
- **Soberania.** Uma tabela nova: para cada linha de `borders.ron`, se aconteceu na simulação,
  quando (adiantada ou atrasada em anos), ou não; e o contrário, mudanças simuladas que a história
  não teve. Casamento por (tipo, de, para) e pela sobreposição de estados (mais da metade da
  população de 1925 em comum).
- `history_mapping.rs` passa a exigir também que toda etiqueta de `candidates.ron` tenha linha em
  `mapping.csv`.

## 5. Desempenho e determinismo

- O sistema `borders` é diário e não faz nada sem pedido pendente: custo zero na rodada sem
  operação. Roda depois dos outros no dia (fila de pedidos aplicada antes da semana seguinte).
- O estado novo (`GameWorld::borders`: pedidos pendentes, registro de mudanças, estatuto de
  súdito) só entra no hash quando não está vazio; **a rodada sem nenhuma operação dá o mesmo hash
  da `main`.**
- O grafo de vizinhança fica nas regras (carregado do mapa), não no mundo salvo.
- Teste de 100 anos com roteiro de operações em datas fixas (só no teste): mesmo hash com 1 e 2
  threads e depois de salvar e recarregar no meio.

## 6. Decisões para o dono

Propostas, aguardando o dono. A parte B só começa depois da resposta.

1. **Dívida do país novo.** Recomendação: **colônia que se emancipa nasce sem dívida pública**
   (o modelo põe as contas coloniais no Tesouro da metrópole, que fica com a dívida inteira e perde
   a receita: a metrópole sai mais endividada, como saíram); **secessão de metrópole** (Bangladesh,
   repúblicas soviéticas, Iugoslávia) reparte toda a dívida por **w**; anexação total passa tudo.
   Teste: soma mundial de dívida igual antes e depois. Custo: uma regra por tipo de operação.
2. **Moeda.** Recomendação: **moeda própria na hora**, com nível de preços, inflação e câmbio do
   antigo dono no dia (nada salta); a união monetária (zona do franco, CFA, rublo até 1993) entra
   com a moeda comum da fase 4. Alternativa: ficar na moeda do antigo dono por um tempo, que exige
   moeda compartilhada entre mercados agora. Teste: preços e câmbio do país novo iguais aos do
   antigo dono na semana da operação. Custo: nenhum além da cópia.
3. **O jogador pode libertar ou criar país por decisão?** Recomendação: **sim, só do próprio
   território** (libertar colônia ou estado como país independente ou súdito), pelo mesmo comando;
   tirar território de outro só por tratado ou paz. Teste: comando do jogador sobre estado alheio é
   recusado. Custo: um botão na interface depois; a regra já é a operação.
4. **O jogador continua se o país dele for anexado?** Recomendação: **sim, como observador**, com a
   escolha de outro país (inclusive o herdeiro); o país dele vira lápide e pode renascer. Teste:
   anexar o país do jogador não derruba a sessão e o save carrega. Custo: uma tela de escolha que já
   existe.
5. **Granularidade.** Recomendação: **só estado inteiro** (870 estados). Corredor de Danzig, Memel,
   Hatay, Oder-Neisse e Caxemira ficam com a maioria da área e anotados em `partial`; dividir
   estado exige mexer no gerador do mapa e só entra se o placar mostrar que pesa. Teste: a
   operação recusa província solta. Custo: zero agora; dividir estado depois custa a regeneração do
   mapa.
6. **Herança tecnológica do país novo.** Recomendação: **herda os estágios do antigo dono limitados
   a Protótipo, salvo as técnicas usadas por plantas que existem nos seus estados**, que ficam no
   estágio do antigo dono (a técnica vai com a fábrica e com a escola, não com a bandeira). Copiar
   tudo faria a Índia de 1947 industrializar o que só a Grã-Bretanha fazia. Teste: país novo sem
   planta química não constrói química no dia seguinte. Custo: uma regra de cópia. Desde o
   estágio colonial (`research.pt-BR.md`, "Colônias"), a secessão de colônias parte de
   `ResearchState::stages(antigo dono, true)`, que já é o que a colônia usava, e aplica o teto por
   cima.

## 7. O que a interface vai precisar ler

Para o thread de interface, sem tela nova aqui:

- dono e estatuto (metrópole ou colônia) de cada estado: `Region::market`, `Region::colony`, a
  cada leitura mensal;
- países vivos e lápides, e a linha de cada país novo (etiqueta, `CountryDef` do catálogo);
- o registro de mudanças (`GameWorld::borders.changes`: data, operação, de, para, estados, gatilho)
  para o diário e para a lente "fronteiras" (mapa de 1925 contra o de agora);
- o movimento de independência por unidade colonial (parte C): apoio, pressão, a condição mais
  fraca, e o limite que dispara a secessão.
