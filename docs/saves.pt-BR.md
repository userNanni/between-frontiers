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
anual, ~0,5 %. O padrão continua anual. Quem quiser pontos mais finos tem o trimestral por menos
de 2 %. Uma campanha com o teto de 30 pontos ocupa ~55 MB.

**A interface não trava.** A gravação roda no thread da simulação (`Command::Save`): a interface
segue desenhando o último snapshot e só lê a resposta (`try_recv`) a cada quadro; o trabalho dela
depois disso fica abaixo de 1 ms. O que pausa é a simulação, pelo tempo de gravar. A única espera
na interface é ao voltar a um ponto (salvar rápido, F9, ou "Voltar a este ponto"): ela espera os
saves na fila e grava o ponto de onde o jogador estava, o que é a troca de tela de qualquer jeito.
