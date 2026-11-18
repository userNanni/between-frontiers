# Campanhas, pontos de restauração e custo do save

Salvar e voltar atrás é um jeito legítimo de jogar: arriscar uma lei, uma guerra ou um acordo e ver
no que dá, ou procurar, num país difícil, o caminho estreito que cumpre uma agenda. O cliente
facilita isso com pontos de restauração dentro da campanha. O formato do save da simulação não
muda: cada ponto é o mesmo snapshot do `world.lss`, gravado como arquivo a mais.

## Pasta da campanha

```
saves/<campanha>/
  campaign.ron        nome, país, data do jogo, quando foi jogada
  world.lss           estado atual (o que "Carregar" abre)
  points/
    <id>.lss          snapshot do ponto, igual ao world.lss
    <id>.ron          o que a lista mostra: tipo, motivo, nome, data do jogo, quando foi gravado, tamanho
```

`crates/bf-client/src/points.rs` cuida dos pontos: `list`, `record`, `rename`, `delete`, `prune`,
`restore`. Voltar a um ponto copia o `.lss` dele sobre o `world.lss` e abre a campanha naquela data;
os outros pontos ficam.

## Tipos de ponto

| Tipo | Quem cria | Sai sozinho? |
|---|---|---|
| Automático | o salvamento automático, a cada virada do calendário (mensal, trimestral, semestral, anual) | sim, na rotação |
| Antes de decisão | `State::restore_point_before(Reason)`, antes de uma decisão arriscada ou irreversível | sim, na rotação |
| Salvar rápido | `campaign.quick_save` (F5); cada um substitui o anterior | só pelo próximo salvar rápido |
| Salvo à mão | `campaign.save_named` (Ctrl+Shift+S), com nome | nunca |

O salvamento automático grava o ponto e o `world.lss` de uma vez (o mundo é serializado uma vez,
`SimHandle::save_to`). O ponto "antes de decisão" grava só o ponto.

**Ponto de entrada único.** Todo sistema que toma decisão arriscada chama
`restore_point_before(reason)` **antes** de mandar o comando: o pedido de save entra na fila da
simulação antes do comando, e o ponto guarda o mundo de antes. Liga e desliga em Configurações
("Ponto antes de decisão arriscada"). Hoje chama: mudança de lei (`GameCommand::SetLaw`). Os motivos
já existem para libertar território, aceitar acordo, declarar guerra e assinar tratado
(`Reason::Liberate`, `Accept`, `War`, `Treaty`), e esses sistemas chamam quando chegarem ao cliente.

**Carregar rápido** (F9) volta ao salvar rápido. Antes de voltar, grava onde o jogador estava como
ponto "Antes de voltar a outro ponto", para desfazer a volta.

## Rotação e disco

- **Pontos automáticos mantidos:** 3, 5 (padrão), 10 ou 20. Os mais velhos saem primeiro.
- **Teto de pontos por campanha:** 10, 30 (padrão), 60 ou 100. Passando dele saem os automáticos
  mais velhos. Pontos salvos à mão e o salvar rápido nunca saem sozinhos, mesmo além do teto.
- **Aviso de pasta grande:** 250, 500, 1000 (padrão) ou 2000 MB. A tela de Campanhas mostra o
  tamanho de cada pasta (em vermelho quando passa) e o jogo avisa uma vez por sessão.
- Configurações mostram "Cada ponto: ~X MB · N automáticos: ~Y MB · teto de M: ~Z MB", calculado
  do save mais recente no disco.

## Tela de Campanhas

À esquerda, as campanhas. Cada uma tem um botão "…" com Renomear, Apagar campanha e Abrir pasta
(`screens::saves::MenuItem`, ids `saves.menu.rename`, `saves.menu.delete`,
`saves.menu.open_folder`; o id `saves.menu.play_other` fica reservado para "Jogar com outro país").
À direita, os pontos da campanha selecionada: data do jogo, quando foi gravado (na hora local do
jogador; o arquivo guarda segundos Unix, em UTC), tipo e motivo; Voltar a este ponto, Renomear e Apagar. Tudo tem dica de tecla.

## Custo do save

Medido com `cargo test -p bf-client --release --test save_cost -- --ignored --nocapture` (dois
threads, Brasil, nesta máquina, com outras rodadas em paralelo). "Gravar" é serializar e escrever
com troca atômica (`.tmp` e `rename`); "carregar" é ler e reconstruir a sessão; "mês" é o tempo
médio de simular um mês nos três meses antes da data; "interface" é o que roda no thread da
interface depois que o ponto está no disco (registrar, girar a rotação, somar a pasta).

| Ano | Tamanho | Gravar | Carregar | Um mês | Automático mensal / mês | Interface |
|---|---|---|---|---|---|---|
| 1925 | 1,2 MB | 5 ms | 4 ms | 153 ms | 3,3 % | 0,3 ms |
| 1975 | 1,7 MB | 7 ms | 6 ms | 130 ms | 5,5 % | 0,3 ms |
| 2025 | 1,8 MB | 8 ms | 7 ms | 143 ms | 5,5 % | 0,3 ms |

O automático mensal custa 5,5 % do tempo de um mês de 1975 em diante; o trimestral, ~1,8 %, e o
anual, ~0,5 %. O padrão é o **trimestral** (abaixo de 2 %); as Configurações mostram o custo de
cada opção (`points::MONTHLY_COST_PERCENT`, o mensal medido em 2025, dividido pelos meses do
intervalo). Uma campanha com o teto de 30 pontos ocupa ~55 MB.

**A interface não trava.** A gravação roda no thread da simulação (`Command::Save`): a interface
segue desenhando o último snapshot e só lê a resposta (`try_recv`) a cada quadro; o trabalho dela
depois disso fica abaixo de 1 ms. O que pausa é a simulação, pelo tempo de gravar. A única espera
na interface é ao voltar a um ponto (salvar rápido, F9, ou "Voltar a este ponto"): ela espera os
saves na fila e grava o ponto de onde o jogador estava, o que é a troca de tela de qualquer jeito.

## Formato do mundo e migração

O contêiner do save (`laplace_schelling::save`) já tem versão e checksum. O snapshot do mundo,
dentro dele, é postcard, que não se descreve: um campo novo no `GameWorld`, ou em qualquer parte
dele, não lê como padrão, e o save antigo deixa de abrir. Por isso o snapshot tem cabeçalho e
versão (`bf_sim::save_format`):

```
| "BFWD" | versão: u32 LE | postcard(GameWorld) |
```

O cabeçalho do contêiner guarda a versão do jogo que gravou (`GAME_VERSION`, versão do pacote e
commit, como `0.0.1+92093bd4e`), para a mensagem de erro.

| Versão | O que entrou | Struct congelada |
|---|---|---|
| 0 | antes das comunidades na população | `compat::V0` |
| 1 | comunidades | `compat::V1` |
| 2 | estado das fronteiras (`borders`) | `compat::V2` |
| 3 | livro de dívidas entre Estados (`Economy::ledger`) | lê como a 4 |
| 4 | dívida acordada em tratado (`DebtTerm::Agreement`) e dissolução de país (`Operation::Dissolve`) | `compat::V4` (vale para a 3) |
| 5 | pesquisa que custa (obra pública de pesquisa, prioridade de pesquisa nas leis) | `compat::V5` |
| 6 | capital estrangeiro (`GameWorld::foreign`, #109); a primeira gravada com cabeçalho | `compat::V6` (economia `EconomyBeforeTrade`) |
| 7 | influência (`GameWorld::influence`, #110) | `compat::V7` (economia `EconomyBeforeTrade`) |
| 8 | estado dos tratados (`GameWorld::treaties`: reconhecimento entre Estados, #145) | `compat::V8` (economia `EconomyBeforeTrade`) |
| 9 | forças armadas (`GameWorld::war`, #115), vazias com as forças desligadas | `compat::V9` (economia `EconomyBeforeTrade`) |
| 10 | comércio com parceiro: `Economy::trade` (`econ::trade::TradeState`: número de países, valor exportado por par no ano corrente e no último ano, preferências por comprador e grupo), último campo do `Economy`, #113. Vazio e fora do hash enquanto a chave `trade_by_pair` estiver desligada | `compat::V10` |
| 11 | ensino por coorte e nível (`GameWorld::education`); vazio na migração. Com a regra `schooling` desligada continua vazio; ligada, a demografia monta no mês seguinte | `GameWorld` |

`decode_world` lê a versão e aplica a cadeia até a atual: `v0_to_v1 → v1_to_v2 → v2_to_now`
(a economia da 2 converte direto para a da 5: `v2_to_v5`), `v4_to_v5` (a 3 lê direto como a 4) e
`v5_to_v6`, `v6_to_v7`, `v7_to_v8`, `v8_to_v9`, `v9_to_v10` e `v10_to_v11` (o comércio por par começa vazio; com a chave
ligada, se preenche na primeira semana; o ensino também começa vazio)
(`save_format::migrate`). Os saves sem cabeçalho, de antes deste formato, continuam abrindo pelas
tentativas de antes (da versão mais nova para a mais velha, com a cauda de listas vazias
completada). Um save de versão mais nova que o jogo não abre: a tela de Campanhas diz qual
versão gravou e qual o jogo lê. Um save danificado também aparece com explicação.

### A regra, em três linhas

1. Todo PR que muda o que o `GameWorld` grava sobe `save_format::WORLD_VERSION`.
2. E acrescenta um degrau: a struct da versão anterior congelada em `compat` e a função `vN → vN+1`.
3. O teste `world_schema` falha enquanto isso não for feito.

### Como acrescentar um campo ao mundo

Qualquer mudança no que o `GameWorld` grava (campo novo, removido, reordenado ou de outro tipo,
no mundo ou em qualquer parte dele) é uma versão nova:

1. Suba `save_format::WORLD_VERSION`.
2. Congele o formato que sai em `save_format::compat` como `V<antiga>`. Se a mudança foi numa
   parte (economia, demografia), congele o formato antigo da parte ao lado dela, como
   `econ::state::EconomyBeforeLedger`, e use-o na struct congelada.
3. Escreva `v<antiga>_to_v<nova>` em `compat` e o degrau em `save_format::migrate`.
4. Acrescente o formato antigo ao teste `game::tests::every_version_loads_with_its_hash`: um
   mundo de 1925 gravado nele abre com o mesmo hash, com e sem cabeçalho.
5. Grave o esquema novo: `BF_BLESS=1 cargo test -p bf-sim --test world_schema` escreve
   `crates/bf-sim/tests/schema/world-v<nova>.txt`. O teste falha enquanto o `GameWorld` não
   bater com o esquema da versão atual: mudar o mundo sem subir a versão não passa. O esquema de
   uma versão que já existe nunca é reescrito.
6. Variante nova **no fim** de um enum não muda os bytes do que já existe: suba a versão e leia a
   anterior como a nova (`3 | 4 => …` em `migrate`), sem struct congelada. Variante no meio,
   reordenada ou removida muda os índices: aí é struct congelada e migração.
7. Um enum novo no mundo aparece no teste do esquema como variantes faltando: acrescente-o à
   lista `trace!` de `tests/world_schema.rs`.

Dois PRs que sobem a versão ao mesmo tempo colidem no número e no esquema: quem entra depois faz
rebase, sobe para o número seguinte e regrava o esquema.
