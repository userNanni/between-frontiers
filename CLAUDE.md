# Between Frontiers: regras de trabalho

Vale para pessoas e agentes. Código, commits e nomes em inglês; documentos e PRs em português do
Brasil.

## Onde trabalhar

- O checkout principal fica na `main`: não edite nele. Cada tarefa tem a sua worktree, ao lado
  dele: `git worktree add -b <tipo>/<tema> ../bf-wt-<tema> main`.
- Worktree de branch já integrada à `main` é removida (`git worktree remove ../bf-wt-<tema>`),
  com o `target/` dela.
- Antes de mexer num arquivo quente (`econ/week.rs`, `econ/monthly.rs`, `bf-bench/src/game.rs`,
  `bf-client/src/sim.rs`), veja se outra branch aberta já mexe nele:
  `git diff --stat main...<branch>`. Se sim, mantenha a sua mudança pequena e diga no PR.
- Instrumentação provisória (`eprintln!`, contadores estáticos, `dbg_add`) não entra no commit. O
  `bf-bench` já imprime e grava no JSON o custo por sistema; `BF_VERBOSE=1` imprime todo ano.
  `python3 tools/bench/compare.py base.json novo.json` compara duas rodadas.

## Simulação

- **Chave de dado se resolve na carga, nunca no tick.** Nada de `cat.good("grain")`,
  `key == "..."` ou `match` por nome de grupo dentro de sistema semanal ou mensal: o índice ou o
  valor vira campo do `Catalogue` (ver `grain`, `construction`, `staple_need`, `public_goods`,
  `Method::output_value`, `Building::labor_share`). Uma chave errada falha na carga, em vez de
  desligar a regra em silêncio.
- **O que não depende do pop sai do laço dos pops**; o que não depende da semana sai da semana
  (exemplo: `NeedMixes`, uma vez por estado e semana).
- **Sem alocação por pop ou por planta** em laço quente quando um buffer por estado basta.
- **Reuse antes de escrever.** Regra que já existe tem um lugar só:
  - preço que anda em direção a um alvo: `econ::PriceBand`;
  - escala de uma planta na semana: `week::plant_run`;
  - preços e parcelas que um estado vê: `local::View::new`;
  - compra por cesta de pesos por mil: `week::basket_orders`;
  - valor adicionado e lucro de um método: `Method::value_added`, `Method::profit`;
  - razão limitada a 1: `econ::fill`.
  Se a regra nova é variação de uma dessas, estenda a existente.
- Toda conta de estado em `Fixed`; reduções em ordem de id, nunca `rayon::reduce`.

## Dados

- Todo dado público pode ser usado, inclusive de licença não comercial. A licença vai anotada
  junto do dado: no `.md` de fontes da pasta e no bloco de fontes do arquivo. Dado sob licença NC
  ou GPL mantém a licença dele e não passa a valer como CC BY-SA.
- Nada copiado de mod ou jogo sem licença (o CWE, o Tech & Res): deles, só se comparam sistemas e
  ordens de grandeza.

## Desempenho

- Bench do `--game` roda um por vez na máquina, sob trava (`flock /tmp/bf-bench.lock ...`).
  Vale sobretudo para os 100 anos.
- PR que toca `bf-sim` ou `laplace-schelling` mede contra a `main`, os dois binários em seguida:
  `flock /tmp/bf-bench.lock env BF_SYSTEMS=1 /usr/bin/time -f '%U s' target/release/bf-bench --game --years 10 --threads 1`.
  Compare o tempo de CPU (`%U`) e a tabela por sistema, não o tempo de parede: a máquina é
  dividida com outras sessões e o tempo de parede varia o dobro.
- **Refatoração e otimização não mudam o hash.** O `final hash` tem de ser igual ao da `main`; se
  mudar, é mudança de comportamento e o PR diz por quê.
- Ao fechar uma fase, atualize a linha de desempenho de `docs/benchmarks.pt-BR.md` com a medição
  de 100 anos.

## Commits e PRs

- Commit no formato `tipo(escopo): resumo` (`feat(sim)`, `fix(client)`, `perf(engine)`,
  `feat(data)`, `docs`), um assunto por PR.
- Corpo do PR, nesta ordem: **O que muda**; **Teste de 100 anos** (tabela contra a `main`, quando
  muda a simulação); **Desempenho** (CPU e sistema mais caro, contra a `main`); **Testes**;
  **Docs**. Achado que não é deste PR vai para o plano de melhoria, não para o código.
- Antes de abrir: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`.
