# Fronteiras: mudança de dono, nascimento e morte de países

Estado: desenho (parte A, integrada no #73) com as decisões do dono de 2026-11-17 (seção 6). Nada
aqui roda no tick ainda. A operação entra na parte B, o primeiro gatilho (independência por
movimento) na parte C.

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
| Pesquisa | `ResearchState::stage`, `progress`, `focus`, `points`, `effects` | país novo herda (decisão 6); `firsts` é mundial e não muda |
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
Secede    { from, states, tag, overlord: Option<tag> }        estados viram país novo (ou renascido),
                                                              independente ou súdito
Annex     { from, into, colony: bool }                        um país deixa de existir
Status    { tag, overlord: Option<tag> }                      súdito vira soberano ou o contrário

toda operação, termos opcionais (decisões 1 e 6):
  debt:       [ { amount, debtor, creditor, currency } ]      dívida transferida ou criada por acordo
  techniques: [ { tech, from, to, stage, people } ]           técnica levada (paz) ou roubada (espionagem)
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

## 6. Decisões do dono (decididas em 2026-11-17)

As seis recomendações foram aceitas como estavam, com quatro acréscimos (marcados). Direção geral
do dono no mesmo dia: funcionalidade antes de equilíbrio. A operação se julga por funcionar e
conservar (população, dinheiro, dívida, estoques, plantas; soma mundial igual antes e depois), por
determinismo de 100 anos, por save antigo carregando e por CPU, não por célula de país no placar;
o equilíbrio fino fica para depois da fase 6.

1. **Dívida.** Padrão por tipo de operação: colônia que se emancipa nasce sem dívida pública e a
   metrópole fica com a dívida inteira; secessão de metrópole reparte toda a dívida por **w**;
   anexação total passa tudo; transferência de estados não mexe na dívida.
   **Acréscimo: dívida criada por acordo.** Toda operação aceita um termo opcional `debt`, uma
   lista de linhas de dívida a criar ou transferir. Cobre a paz com reparação de guerra (como a
   de 1919), a independência que nasce devendo ao antigo dono (a dívida da independência do Haiti
   é o caso extremo; as colônias que herdaram empréstimos coloniais, o caso comum) e o acordo de
   reconhecimento. Dois usos, combinados com o thread dos tratados (`docs/treaties.pt-BR.md`,
   seção 5):
   - **dívida de mercado que muda de devedor** (`DebtTerm::Market`: quanto, de quem para quem):
     fica em `Money` (`debt_home`, `debt_bank`, `debt_abroad`, `debt_fx`), sai de cada detentor em
     proporção; é da operação de fronteira;
   - **dívida criada entre Estados** (`DebtTerm::Agreement`, quando o contrato dos tratados
     entrar): uma linha do **livro nomeado de dívida entre Estados** dos tratados, fonte da
     verdade (devedor, credor, moeda, principal, juro, calendário, teto de transferência,
     garantia). A operação só chama `treaty::ledger_create`; o crédito do credor é a soma do livro
     (`treaty::claims_of`), nunca um campo guardado à parte. Na anexação e na secessão a operação
     chama `treaty::ledger_on_annex` e `treaty::ledger_on_secede`, que os tratados escrevem.

   A soma mundial fecha com as duas partes: dívida de mercado e `foreign_assets` em `Money`, e o
   livro, em que dívida e crédito são iguais por construção (teste).
2. **Moeda.** Moeda própria na hora, com nível de preços, inflação, expectativa e câmbio do antigo
   dono no dia (nada salta). A união monetária entra com a moeda comum da fase 4.
3. **O jogador liberta ou cria país do próprio território**, pelo mesmo comando das operações:
   colônia ou estado próprio vira país **independente ou súdito** do jogador. Tirar território de
   outro só por tratado ou paz. Recusas testadas: estado alheio, estado que não existe, libertar
   todos os estados do próprio país, etiqueta viva.
4. **O jogador anexado continua**, e **acréscimo: o jogador troca de país a qualquer momento.**
   - Derrotado por anexação ou por revolta, escolhe outro país, inclusive o que nasceu da revolta.
   - Fora de qualquer operação, troca quando quiser: começar com a URSS, libertar uma república e
     passar a jogar com ela; começar com o Brasil, libertar São Paulo e jogar com São Paulo; jogar
     como súdito e sair do controle do suserano.
   - **Estado de sessão, fora do hash.** Quem joga com qual país é do jogador, não do mundo: a
     simulação não lê a escolha (os países do jogador e da IA seguem as mesmas regras), então a
     troca não muda o hash. A sessão guarda o mapa emissor → país; o save grava esse mapa no
     cabeçalho, ao lado do registro de comandos; o comando de troca entra no registro (o replay
     mostra quem jogava com quem e quando) mas não muda o mundo. O `GameWorld::player` dos saves
     antigos é lido uma vez na carga como a escolha do emissor local e deixa de ser escrito
     (continua no hash enquanto existir, com o valor da primeira escolha, para os saves antigos
     carregarem e conferirem). No multiplayer futuro, cada emissor tem a sua entrada; dois
     jogadores no mesmo país é decisão do servidor, não da simulação.
   - **Súdito.** O país criado como súdito guarda o suserano (`borders::Subject`: suserano, tipo
     da sujeição, desde quando), lido do cenário para os súditos de 1925 (`countries.ron`,
     `Status::Subject`). O ponto de leitura para romper é `GameWorld::overlord_of(país)`; a regra
     de quando o súdito rompe (força, legitimidade, movimento de independência, guerra) é da
     diplomacia (fase 4). A operação `Status` muda o vínculo; romper é uma `Status` com suserano
     vazio.
5. **Granularidade.** Só estado inteiro. A operação recusa estado repetido ou que não existe; o
   que é menor que um estado fica em `partial` na referência.
6. **Herança tecnológica.** O país novo herda os estágios do antigo dono limitados a Protótipo,
   salvo as técnicas usadas por plantas que existem nos seus estados, que ficam no estágio do
   antigo dono; o progresso das limitadas zera. A colônia que vira país herda, assim, o que a
   metrópole sabe e o que a colônia já opera, sem herdar a indústria da metrópole. (Enquanto o
   estágio colonial de `economy.pt-BR.md`, "Colônias: salário e técnica", estiver estacionado, a
   base é o estágio do país; quando entrar, a base passa a ser o estágio colonial do antigo dono.)
   **Acréscimo: técnicas transferidas por evento.** Toda operação aceita um termo opcional
   `techniques: Vec<TechniqueTerm>` (técnica, de quem, para quem, estágio a que chega, e se o
   doador perde gente: o estágio dele não cai, mas o progresso da próxima etapa recua na parte
   levada). Não é usado agora; os usos desenhados:
   - **Paz (fase 5):** o vencedor leva cientistas, engenheiros e projetos (a Operação Paperclip,
     os especialistas alemães levados para a URSS). A exigência "técnicas" entra na mesa como as
     outras; aceita, vira o termo, com as técnicas em que o derrotado está acima do vencedor.
   - **Espionagem (fase 4):** os soviéticos e a bomba. Uma ação diplomática com custo e risco
     medidos (relação, contraespionagem do alvo); quando dá certo, um termo de uma técnica com o
     espião subindo um estágio, nunca acima do estágio do alvo. Fica fora da operação de
     fronteira: a espionagem chama só a parte do termo, sem estados.

## 7. O que a interface vai precisar ler

Para o thread de interface, sem tela nova aqui:

- dono e estatuto (metrópole ou colônia) de cada estado: `Region::market`, `Region::colony`, a
  cada leitura mensal;
- países vivos e lápides, e a linha de cada país novo (etiqueta, `CountryDef` do catálogo);
- o registro de mudanças (`GameWorld::borders.changes`: data, operação, de, para, estados, gatilho)
  para o diário e para a lente "fronteiras" (mapa de 1925 contra o de agora);
- o movimento de independência por unidade colonial (parte C): apoio, pressão, a condição mais
  fraca, e o limite que dispara a secessão.

## 8. Implementação (parte B)

Código em `crates/bf-sim/src/borders/op.rs`; testes em `crates/bf-sim/tests/borders_ops.rs` e
`borders_conservation.rs`.

- **Comandos.** `GameCommand::Borders { op }` enfileira qualquer operação (testes, ferramentas e,
  depois, tratados e paz). `GameCommand::Liberate { by, states, tag, subject }` é o do jogador:
  `by` liberta estados seus como país independente ou súdito seu. Quem pode emitir o comando por
  `by` é assunto da sessão (decisão 4); a simulação confere só que os estados são de `by` e que
  `by` fica com pelo menos um.
- **Sistema `borders`**, diário e último do dia. Sem pedido pendente, não faz nada. Com pedido, a
  operação é conferida inteira antes de qualquer mudança; recusada, entra em
  `GameWorld::borders.refused` com o motivo e nada muda.
- **Operações:** `Transfer`, `Secede` (com `overlord` opcional), `Annex`, `Status` e `Terms` (só
  os termos, para um acordo sem território). Cada uma aceita `Terms { debt, techniques }`.
- **Etiqueta do país libertado** sem etiqueta dada: a do catálogo cujo núcleo tem mais da metade
  da população dos estados; senão `X00`, `X01`… (`op::tag_for`).
- **Leis do país novo:** as do antigo dono sem as coloniais; o modelo do catálogo
  (`Candidate::laws`) fica para quando o catálogo tiver modelos. Quem recebe a primeira colônia
  ganha as leis coloniais de quem a cedeu.
- **Pesos:** **w** é a renda da semana dos pops dos estados que saem sobre a do país; **p**, a
  população.
- **Demografia:** a diáspora reparte como na seção 1. Quem estava no exterior e passou a morar no
  próprio país (origem e destino viraram o mesmo território) volta para casa e é contado em
  `ChangeRecord::repatriated`. As ondas abertas dos territórios mudados terminam com
  `WaveEnd::Border`, e os fatos dos territórios são recalculados.
- **Súditos:** `BorderRules::overlord_of(world, país)` lê o suserano. Até a primeira operação ele
  vem do cenário; depois, de `GameWorld::borders.subjects`. A operação `Status` só registra a
  mudança. Quem decide quando o súdito sai é a regra dos tratados, por negociação cláusula a
  cláusula ou por denúncia, que só vale quando a pressão de saída passa o controle do suserano
  (decisão do dono de 2026-11-17).
- **Reconhecimento** (decisão do dono de 2026-11-17): o país novo nasce reconhecido só por quem
  assinou a criação dele. `Secede` leva a lista `signatories`, e ao libertar o próprio território
  o antigo dono é o signatário. Os pares que não se reconhecem ficam com os tratados.
- **Hash:** o estado de fronteiras só entra no hash depois da primeira operação ou recusa. Teste:
  meio ano com e sem o sistema dá o mesmo hash.
- **Conservação** (`borders::Totals`): pessoas, poupança, fundo, Tesouro, as quatro dívidas de
  mercado, créditos no exterior, diáspora (mais os repatriados), níveis de planta e obras na fila.
  Igualdade exata antes e depois de cada operação nos testes.
- **Save antigo:** o postcard não aceita campo faltando no fim; `game::decode_world` completa com estado vazio o snapshot escrito antes do campo `borders` (teste).
- **Determinismo:** campanha com libertação, anexação e independência dá o mesmo hash com 1 e 2
  threads e com save e load no meio.

### Onde ficam as ações (decisão do dono de 2026-11-17)

Libertar território e jogar com outro país não são decisões comuns. Um fluxo de um toque ocupa
espaço na tela e deixa o jogador fazer isso sem querer. Por isso:

- **O painel do país só informa.** Mostra os súditos, mas não tem botão de fronteira.
- **Libertar território é uma decisão da janela de Agendas (J)**, seção "Decisões" no topo da
  lista (`bf-client/src/screens/liberate.rs`).
  - O jogador escolhe o que libertar: um país do catálogo cujo território é dele (só a parte que
    ele tem) ou um estado só. Escolhe também se sai independente ou súdito.
  - Antes de confirmar, vê o que sai do país: pessoas, níveis de fábricas e fazendas, parte do
    fundo, dívida que vai junto. Como súdito, vê também o que o país novo cede e recebe pelo
    feixe de cláusulas.
  - A confirmação é um passo próprio ("Revisar a decisão", depois "Confirmar a libertação"). Tudo
    tem dica de tecla e texto nos dois idiomas.
  - As opções e os números vêm de `borders::op::liberation_options` e `borders::op::preview`,
    lidos na thread da simulação uma vez por mês e quando as fronteiras ou o jogador mudam.
- **Jogar com outro país fica fora do jogo em andamento**, na tela de Campanhas: menu "…" da
  campanha → "Jogar com outro país".
  - Abre a escolha de país (`screens::pick::State::switching`) com as fronteiras e os números do
    próprio save, e grava a escolha na campanha.
  - Ao abrir a campanha, o jogo emite `GameCommand::SwitchCountry` se a escolha difere da sessão.
- **Quando é necessária, a escolha aparece sozinha.** Se o país do jogador deixa de existir
  (anexado, ou derrotado por uma revolta), o jogo salva e abre a mesma escolha.

## 9. Rodada de 100 anos com a história aplicada por data (só para exercitar as operações)

`bf-bench --game --borders-history` aplica `data/history/borders.ron` nas datas reais. É desligada
por padrão e não é a rodada de referência: na fase 7 a história acontece por condição, não por
data. Antes de enviar cada operação, o bench a roda numa cópia do mundo e compara as somas
(`borders::Totals`). `tools/history/borders.py` compara colônias e metrópoles por década contra a
rodada sem mudanças, pelas unidades de 1925 fixas por estados.

Rodada de 101 anos, semente 1925, 2 threads, contra a `main` 7c43500:

- **Operações:** 260 mudanças da referência submetidas, 253 aplicadas, 7 recusadas e nenhuma que
  mude as somas.
  - Cinco das recusas são dissoluções em que o último herdeiro leva todos os estados que restam
    (Tchecoslováquia em 1939 e em 1993, Federação do Mali, Iugoslávia); a operação recusa
    esvaziar um país por secessão. As outras duas são a Sérvia e Montenegro, que não nasceu por
    causa da recusa iugoslava.
  - A dissolução precisa de uma operação própria, que entra com os tratados e a paz.
- **Determinismo:** mesmo hash com 1 e 2 threads; salvar em 1975 e recarregar dá o mesmo hash
  final.
- **Placar**, comparado pela faixa de reembaralhamento de `noise.py compare`:

| 2025 (PIB de 2022) | sem mudanças | com mudanças |
|---|---|---|
| Erro log. médio do PIB por habitante | 1,09 | 0,90 |
| Correlação de ranking do PIB por habitante | 0,53 | 0,61 |
| Erro log. médio da população | 0,52 | 0,55 |
| PIB mundial, simulado/real | 0,87 | 0,73 |

| PIB por habitante, simulado/real | 1975 | 2025 |
|---|---|---|
| Colônias britânicas | 2,28 → 1,53 | 2,13 → 0,75 |
| Colônias americanas (Filipinas) | 2,05 → 1,37 | 3,69 → 1,19 |
| Colônias francesas | 2,62 → 1,99 | 2,26 → 1,85 |
| Congo Belga | 3,06 → 2,00 | 3,78 → 3,08 |
| Colônias portuguesas | 1,24 → 1,36 | 0,52 → 0,86 |
| Metrópole britânica | 1,52 → 1,49 | 1,89 → 1,24 |
| Metrópole holandesa | 1,30 → 1,20 | 1,47 → 1,09 |
| Metrópole japonesa | 0,98 → 0,79 | 1,22 → 1,10 |

O excesso de PIB por habitante das colônias, estudado em `economy.pt-BR.md` ("Colônias: salário e
técnica"), cai pela metade ou mais quando a colônia vira país. A técnica fica no teto da decisão
6, e a colônia sai do fundo e dos métodos da metrópole. As metrópoles também se aproximam do
real, porque perdem a renda colonial.

O PIB mundial cai 15 %: melhor até 2005, pior depois.

Fora da faixa de ruído ficam 1.323 dos 3.196 números (734 mais perto do real, 541 mais longe).
É uma mudança de mundo, não de ajuste. Nada aqui é calibração: o equilíbrio fino fica para depois
da fase 6.
