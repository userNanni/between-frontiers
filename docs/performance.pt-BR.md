# Desempenho do tick

Guia curto para quem escreve ou muda um sistema da simulação. Sai do perfil da rodada de
otimização de 2026-11-17 e é atualizado a cada otimização que entra. A regra de fundo continua a
do `CLAUDE.md`: otimização não muda o hash.

## Onde o tempo vai hoje

Rodada de 101 anos na `main` (`7b10b48`, 2 threads, tempo de simulação de 229 s):

| sistema | cadência | total | parte | média por execução |
|---|---|---|---|---|
| `economy-week` | semanal | 185 s | 81 % | 35 ms |
| `demography` | mensal | 19 s | 8 % | 16 ms |
| `economy-investment` | mensal | 13 s | 6 % | 11 ms |
| `economy-labour` | mensal | 3,8 s | 2 % | 3,1 ms |
| `research` | mensal | 2,4 s | 1 % | 2,0 ms |
| `politics` | diária | 2,1 s | 1 % | 0,06 ms |
| `influence` | mensal | 1,0 s | 0,5 % | 0,9 ms |
| `economy-year` | anual | 0,9 s | 0,4 % | 8,6 ms |

- **O custo cresce com o século:** de 16 s por década (1925–35) para 35 s (2015–25). Mais
  plantas, mais bens desbloqueados e mais pops ativos por estado. Tudo o que é por planta ou por
  bem desbloqueado pesa mais no fim do que no começo: meça também os últimos anos.
- **Dentro do semanal** (amostragem de pilha, 1925–27): `settle_region` 28 %, `plan_pop` 18 %,
  `need_mix` 5 %, `region_orders` e `plant_run` o resto. Cerca de 40 % do tempo total é aritmética
  `Fixed` (multiplicação e divisão com arredondamento) e cerca de 10 % é `malloc`/`free`/`memcpy`.
- **Escala:** os sistemas rodam um de cada vez (quase todo tick tem um sistema só; o semanal
  roda sozinho na segunda-feira). O paralelismo está dentro do sistema: os dois `par_iter` por
  estado do semanal, que somam 82–85 % das amostras. Seriais: o trecho entre os dois (soma por
  mercado, comércio mundial, `clear_market`, `local::finish`, `market_wages`) e todo `apply`,
  15–18 % do tick. Pela lei de Amdahl, o teto é 1,7× com 2 threads, 2,5× com 4 e 2,9× com 6. O
  hash é o mesmo com 1, 2, 4 e 6 threads.
- **O bench conferia o save simulando o século duas vezes.** Desde o #141 a conferência retoma de
  um save feito durante a rodada principal: a CPU de uma rodada é cerca de 1,5 × o tempo de
  simulação impresso (era 2 ×).

## Orçamento de CPU por sistema novo

Um sistema novo não deve passar de **5 % da simulação** (cerca de 10 s em 101 anos) sem
discussão no PR. Em média por execução, medido em 1 thread:

| cadência | execuções em 101 anos | orçamento por execução |
|---|---|---|
| diária | 36 890 | 0,25 ms |
| semanal | 5 270 | 2 ms |
| mensal | 1 212 | 8 ms |
| anual | 101 | 100 ms |

Código novo dentro de `economy-week` (comércio, capital, guerra) entra no orçamento do semanal:
diga no PR quanto o `economy-week` subiu, em 10 anos e nos últimos anos do século.

## O que evitar

1. **Cadência mais rápida que o dado.** Tabela que só muda por mês, por ano ou quando um evento
   acontece (preferência de comércio, tratados vigentes, relações, participação estrangeira por
   região) não se recalcula toda semana ou todo dia. Guarde a tabela derivada e refaça quando o
   dado de origem mudar. Sistema diário que só age em certas datas devolve na primeira linha.
2. **Alocação em laço quente.** Nada de `Vec`, `String`, `format!`, `collect` ou mapa novo por
   pop, por planta, por bem ou por par de países em sistema diário ou semanal. Use um buffer por
   estado ou por mercado, um array fixo (`[Fixed; P]`) ou uma matriz plana `n × m` alocada uma vez
   por execução e reaproveitada entre bens.
3. **`clone` de estrutura grande.** O `compute` lê o mundo por referência; as `Changes` levam só
   o que muda, não a cópia inteira de um `Market`, de um `CountryWar` ou de uma lista de pops.
4. **Mapa onde vetor serve.** Id denso (país, mercado, estado, bem, profissão) indexa `Vec` ou
   matriz plana. `BTreeMap` só para chave esparsa; `HashMap` não entra no estado (a ordem de
   iteração muda o resultado).
5. **Busca linear e chave por nome no tick.** `iter().position(|c| c.tag == ...)`, `find` por
   chave, `== "military"`, `law("...")`: o índice se resolve na carga (regra do `CLAUDE.md`).
   Uma busca dentro de laço por planta ou por país vira O(n²).
6. **Pares O(n²) por semana.** 83 países dão 6 889 pares; 870 estados, 757 mil. Par país × país
   por semana só com trabalho trivial por par; o que depende só de um lado sai do laço interno.
7. **Lista que só cresce percorrida no tick.** Histórico, coorte por ano de construção, fila nunca
   podada: o custo por ano não pode crescer com o tempo de jogo. Funda ou pode, ou indexe.
8. **Trabalho pesado no `apply`.** O `apply` é serial; a conta vai no `compute`, que é paralelo
   por estado.
9. **Ordem de soma.** Reduções em ordem de id, sem `rayon::reduce`. Trocar a ordem de uma soma ou
   o ponto de arredondamento de um produto de `Fixed` muda o hash: é mudança de comportamento, não
   otimização.

## Regra que muda o hash

Regra nova que muda o resultado de todos entra **desligada**, num campo de `rules.*` (ou do
catálogo) com o valor que deixa o hash da `main` igual. Ligar é um PR separado, com o teste de
100 anos contra a `main`. Assim o PR do código prova que não mexeu no resto, e o PR que liga mede
só o efeito da regra.

## Como medir

- **CPU, não parede.** A máquina é dividida com outras sessões e a carga passa de 15:
  `BF_SYSTEMS=1 /usr/bin/time -f '%U s' target/release/bf-bench --game --years 2 --threads 1`.
  Compare com um binário da `main` compilado por você, os dois em seguida, e repita (3 a 5 vezes,
  use a mediana): o ruído chega a ±7 % com a máquina cheia.
- **Por sistema:** a tabela que o `bf-bench` imprime e grava no JSON;
  `python3 tools/bench/compare.py base.json novo.json` compara duas rodadas. `BF_VERBOSE=1`
  imprime o tempo de cada ano.
- **Dentro de um sistema:** `perf` não roda nesta máquina. Use o amostrador de pilha:

  ```
  SKIP=6 GAP=0.05 tools/bench/sample.sh target/prof target/release/bf-bench --game --years 2 --threads 1
  python3 tools/bench/profile.py target/prof target/release/bf-bench
  ```

  A saída dá o custo próprio por função e por linha e, mais útil, a linha do jogo mais interna de
  cada amostra. O terceiro argumento filtra (`week::`, `demography`).
- **Rodada longa** (mais de 10 anos) vai por `bf-bench-slot` (atende por ordem de chegada) e
  numa unidade `systemd-run --user`, com arquivo de status no fim. Use
  `tools/bench/run-long.sh OUTDIR BIN [argumentos]`. Ele grava `game.json`, `history.json`,
  `cpu.txt` e `status`, e acrescenta uma linha ao livro de rodadas.
- **A base da `main` não se roda de novo.** `tools/bench/baseline.sh [commit]` devolve
  `~/Projects/between-frontiers/target/baseline/<sha>/`, com `bf-bench`, `game.json`,
  `history.json`, `hash.txt`, `cpu.txt` e `systems.txt`. Só roda se ainda não existe, com
  `flock` por sha: dois threads que pedem o mesmo commit esperam um pelo outro.
- **Livro de rodadas:** `~/Projects/between-frontiers/target/runs/ledger.jsonl`, uma linha por
  rodada longa. Tem commit, ramo, hash, threads, CPU, CPU por sistema, PIB e população por década
  e a carga da máquina. `python3 tools/bench/ledger.py show [--branch B]` mostra a tendência de
  CPU por commit e a dispersão dos agregados por década.

## Compilar

- `~/.local/bin/bf-cargo` é o `cargo` com `sccache` (`~/.cache/sccache-bf`). Reaproveita as
  dependências de terceiros entre worktrees, mas não os crates do workspace: o `sccache` põe no
  hash as variáveis `CARGO_*`, e `CARGO_MANIFEST_DIR` muda de worktree para worktree.
  - **Não force o reaproveitamento.** `bf-bench`, `bf-client` e `bf-mapgen` gravam no binário o
    caminho dos dados (`env!("CARGO_MANIFEST_DIR")`), e um acerto de cache vindo de outra
    worktree leria os dados dela.
- `bf-build-governor` segura em 4 os `rustc` simultâneos do projeto; os excedentes ficam parados.
  Compile com `-j 2`. Com 8 compilações ao mesmo tempo a máquina fica sem memória.
