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

### Ganhos da rodada de 2026-11-17

Todos com o hash igual ao da `main`. Base de 101 anos: 1174 s de CPU em `7d851c1`, 2 threads.

| PR | mudança | CPU |
|---|---|---|
| #141 | o bench confere o save a partir de um save da rodada principal | −25 % da rodada |
| #146 | `Fixed` em linha entre crates | −8 % (2 e 10 anos) |
| #150 | multiplicação de `Fixed` em 64 bits quando cabe | −6 % em 10 anos, −3,5 % em 101 |
| #155 | planta parada não passa pelas somas de zero | −18 % em 10 e em 101 anos |
| #156 | lista de pedidos do pop alocada uma vez | −7 % em 2 e 10 anos (incerto) |

Lições para sistema novo:
- **O que só soma zero, pule.** Somar zero com saturação não muda nada; testar e pular é exato.
  Mas o que a entidade parada ainda precisa atualizar (utilização, razão de insumos) continua.
  O teste compara as duas variantes na transição de zero para não zero.
- **A conta de `Fixed` é o piso.** Perto de metade do semanal é multiplicação e divisão. O ganho
  vem de fazer menos contas, não de contas mais baratas.

## Orçamento de CPU por sistema novo

- **Sistema novo:** até **5 % da simulação** (cerca de 10 s em 101 anos).
- **Mudança dentro de um sistema que já existe:** até **2 % do custo daquele sistema**.
- Acima disso, o PR traz a justificativa e o ganho que compensa, ou a regra entra desligada.
- **Como provar o 2 %.** Com ruído de ±7 % uma medida só não prova nada.
  - Use **5 pares** intercalados no mesmo slot (`main`, branch, `main`, branch…), 10 anos em 1
    thread, e compare as **medianas**.
  - O par vale se a carga ficar abaixo de 12.
  - Para regra desligada, prefira a **prova estrutural**: o sistema sai na primeira linha, antes
    de tocar em estado ou alocar, e o hash é igual ao da `main`. Isso vale mais que uma medida.
- **Regra desligada custa zero.** A chave em `game::Switches` (ou `rules.*`) vem desligada por
  padrão. Desligada, a regra não aloca estado, não pré-calcula nada e o sistema sai na primeira
  linha. A CPU com a regra desligada fica a até 2 % da `main`.

Em média por execução, medido em 1 thread:

| cadência | execuções em 101 anos | orçamento por execução |
|---|---|---|
| diária | 36 890 | 0,25 ms |
| semanal | 5 270 | 2 ms |
| mensal | 1 212 | 8 ms |
| anual | 101 | 100 ms |

Exemplo do livro de rodadas: com o capital ligado (#117), `economy-investment` vai de 14 para
43 s em 101 anos, 23 ms por execução mensal contra o orçamento de 8 ms. A causa são 220 mil pares
fundo × lugar avaliados por mês, quando a conta do lugar não depende do fundo.

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
3. **`clone` de estrutura grande que quase não muda.** O `compute` lê o mundo por referência e
   as `Changes` levam o que muda.
   - Copiar um `Market` inteiro para mudar três campos é desperdício.
   - Quando o sistema reescreve todo campo da estrutura toda semana (o `country_week` da guerra
     com o `CountryWar`), devolvê-la nova é o certo: a cópia é o próprio resultado.
   - Separe as partes que não mudam (catálogo, tabelas, nomes) e passe-as por referência ou por
     índice; elas não entram na cópia.
4. **Mapa onde vetor serve.** Id denso (país, mercado, estado, bem, profissão) indexa `Vec` ou
   matriz plana. `BTreeMap` só para chave esparsa; `HashMap` não entra no estado (a ordem de
   iteração muda o resultado).
5. **Busca linear e chave por nome no tick.** `iter().position(|c| c.tag == ...)`, `find` por
   chave, `== "military"`, `law("...")`: o índice se resolve na carga (regra do `CLAUDE.md`).
   Uma busca dentro de laço por planta ou por país vira O(n²).
6. **Pares O(n²) por semana.** 83 países dão 6 889 pares; 870 estados, 757 mil. Par país × país
   por semana só com trabalho trivial por par, e o que depende só de um lado sai do laço interno.
   - Guarde só os pares acima do piso, numa lista esparsa ordenada por id.
   - Não expanda a esparsa em matriz densa toda semana. O `dense()` do #113 fazia isso; uma tabela
     de preferência que muda por mês se refaz no mês.
   - Medida do comércio por par (#113 ligado, 2026-11-18): o custo não estava nos pares que
     comerciam, e sim nas varreduras da matriz inteira. Por bem e por semana, eram montar e
     normalizar a matriz vendedor × comprador (uma divisão por célula) e, em cada passada, zerar,
     somar por linha, somar por coluna e acumular, umas 12 varreduras de ~300 mil células. Zerar
     metade das preferências não baixou o tempo. O que baixa é varrer menos: acumular as somas no
     mesmo laço que escreve, não zerar o que não se usou e não somar coluna de matriz por linha. A
     divisão de 128 bits e os pesos zero não eram o custo. Meça com relógio por trecho antes de
     otimizar: com a máquina carregada, o amostrador de pilha pega poucas amostras.
7. **Lista que só cresce percorrida no tick.** Histórico, coorte por ano de construção, fila nunca
   podada: o custo por ano não pode crescer com o tempo de jogo. Funda, pode ou indexe.
   - Vale também para o que o jogador faz crescer, como a fila de encomendas e os pedidos: ponha
     um teto ou pode.
   - Fundir coortes ou podar uma lista quase sempre muda o hash (a regra 9 vale). Então entra
     atrás de regra desligada, como toda mudança de resultado, e se julga pelos agregados.
   - Exemplo medido: plantas que perderam todos os níveis ficavam na lista do estado e eram
     custeadas toda semana, 11 % das plantas a partir de 2000. Pular as somas de zero, sem tirar a
     planta da lista, não mudou o hash e tirou 18 % da CPU de 101 anos (#155).
8. **Trabalho pesado no `apply`.** O `apply` é serial e a conta vai no `compute`, que é paralelo
   por estado. Exemplo: um `apply` semanal que soma num vetor de mercados × mercados (o
   `trade.add` do #113) é O(n²) serial toda semana. Prefira somar por mercado no `compute` e
   aplicar a linha pronta.
9. **Ordem de soma.** Reduções em ordem de id, sem `rayon::reduce`. Trocar a ordem de uma soma ou
   o ponto de arredondamento de um produto de `Fixed` muda o hash: é mudança de comportamento, não
   otimização.
10. **Estado global ou atômico dentro de `par_iter`.** Contador estático, `AtomicU64` ou
    `Mutex` tocado por tarefa paralela tem contenção entre núcleos e é compartilhado entre
    simulações do mesmo processo. Devolva o número na saída da tarefa e some em ordem.
11. **Tabela plana grande lida fora de ordem no laço quente.** Uma matriz de fundo × lugar ou de
    mercado × mercado varrida por coluna dentro do laço por linha erra o cache a cada leitura;
    no investimento do capital (#117) isso deu cerca de 10 % das amostras do mensal. Percorra na
    ordem da memória, ou transponha uma vez por execução.

## Regra que muda o hash

Regra nova que muda o resultado de todos entra **desligada**, num campo de `rules.*` (ou do
catálogo) com o valor que deixa o hash da `main` igual. Ligar é um PR separado, com o teste de
100 anos contra a `main`. Assim o PR do código prova que não mexeu no resto, e o PR que liga mede
só o efeito da regra.

## Como medir

- **Custo do sistema, não da trajetória.** Ligar uma regra muda o mundo: com o capital ligado o
  século termina com 1,4× os níveis de planta, e o semanal fica mais caro só por isso. Comparar a
  rodada ligada com a desligada mistura as duas coisas. Para isolar o custo do código:
  - compare no mesmo mundo: 10 anos a partir de 1925, quando as duas trajetórias ainda quase não
    se separaram, e olhe a tabela por sistema (`BF_SYSTEMS=1`), não só o total;
  - dentro do sistema, use o amostrador (`sample.sh` e `profile.py`) e some a parte das funções
    do próprio sistema;
  - o resto da diferença em 101 anos é o mundo maior. Ele entra no julgamento dos agregados, não
    no orçamento do código.
- **Carga à parte.** O `%U` do `/usr/bin/time` inclui a carga dos dados (`GameData::load`). O
  que não é simulação sai de `%U − 1,5 × simulação impressa`; uma conta pesada na carga aparece
  ali, em toda rodada e em todo jogo aberto. Conta de carga de regra desligada é preguiçosa: no #113, `Geography::build` calculava as
  distâncias em toda carga, mesmo com o comércio desligado.
- **CPU, não parede.** A máquina é dividida com outras sessões e a carga passa de 15:
  `BF_SYSTEMS=1 /usr/bin/time -f '%U s' target/release/bf-bench --game --years 2 --threads 1`.
  Compare com um binário da `main` compilado por você, os dois em seguida, e repita (3 a 5 vezes,
  use a mediana).
  - O ruído chega a ±7 % com a máquina cheia, e passa disso com os três slots ocupados.
  - Rode as duas medidas no mesmo slot, uma depois da outra (`bf-bench-slot sh -c 'A; B'`).
  - **`user` de momentos diferentes não se compara.** Duas rodadas de 101 anos do mesmo binário
    deram 1 785 s e 1 080 s, com carga de 75 na mais lenta. Só vale o par no mesmo slot, um
    binário depois do outro.
  - **1 thread ou 2.** O orçamento por sistema é a média por execução em 1 thread. A rodada de 10
    anos em 1 thread mede o começo do século, e o fim custa cerca de 1,45× o começo por ano. Para
    o fim do século, compare a parte de cada sistema (custo do sistema sobre a simulação) na
    rodada de 101 anos em 2 threads contra a base guardada com as mesmas threads. A parte não
    depende do número de threads do jeito que o tempo depende.
  - Descarte o par se a carga de 1 min (`/proc/loadavg`) passar de **12** em qualquer das duas:
    acima disso há mais threads prontas que núcleos lógicos.
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
- **Saída cedo:** a cada década a rodada confere que o mundo tem gente e produto; com
  `BF_BAND=<base>/game.json[:PCT]` confere também a população e o PIB mundiais contra a base, a
  ±PCT % (padrão 50), e sai com código 3 na primeira década fora. A faixa larga pega mundo
  quebrado (uma regra que zera o PIB em 1935), não o efeito de uma regra; para isso, a seção de
  agregados abaixo.
- **Livro de rodadas:** `~/Projects/between-frontiers/target/runs/ledger.jsonl`, uma linha por
  rodada longa. Tem commit, ramo, hash, threads, CPU, CPU por sistema, PIB e população por década
  e a carga da máquina. `python3 tools/bench/ledger.py show [--branch B]` mostra a tendência de
  CPU por commit e a dispersão dos agregados por década.

## Julgar agregados em 101 anos

Uma mudança de regra muda o mundo. A pergunta é se a mudança sai da faixa que um empurrão
mínimo já produz. Essa faixa cresce muito ao longo do século: o 2σ do PIB da Itália é de 0,3
ponto em 1935 e de 33 pontos em 2024.

- **Três amostras não decidem.** Com 3 rodadas, o desvio-padrão estimado erra por até ±50 %; com
  8, por cerca de ±25 %. Use **pelo menos oito amostras** (`BF_PERTURB_PERMILLE` = ±1, ±2, ±3,
  ±4) sobre a `main` atual, e não sobre a `main` de quando a branch nasceu.
- **A faixa sai da base guardada, não de cada thread.** Ela é feita uma vez por commit da `main`
  que muda a simulação, guardada ao lado da base (`target/baseline/<sha>/`) e registrada no livro
  de rodadas. Cada PR roda só a própria rodada e compara país a país, por década, com a faixa.
  Oito rodadas de 101 anos ocupam um slot por cerca de 3 h, uma vez por commit, em vez de 3 h por
  thread.
- **Como:** `tools/bench/band.sh [commit]` faz as oito amostras ao lado da base (em
  `target/baseline/<sha>/band/p±N/`), uma vez por sha. `tools/bench/band.py <base> <nova>/history.json`
  compara território a território, por década, com a média ± 2σ da base e das amostras, e
  resume quantos ficam fora por série. Por acaso, cerca de 5 % ficam fora a 2σ.
- **Fora da faixa não é erro:** é efeito da regra, e o PR diz qual e por quê.

## Compilar

- `~/.local/bin/bf-cargo` é o `cargo` com `sccache` (`~/.cache/sccache-bf`). Reaproveita as
  dependências de terceiros entre worktrees, mas não os crates do workspace: o `sccache` põe no
  hash as variáveis `CARGO_*`, e `CARGO_MANIFEST_DIR` muda de worktree para worktree.
  - **Não force o reaproveitamento.** `bf-bench`, `bf-client` e `bf-mapgen` gravam no binário o
    caminho dos dados (`env!("CARGO_MANIFEST_DIR")`), e um acerto de cache vindo de outra
    worktree leria os dados dela.
- `bf-build-governor` segura em 4 os `rustc` simultâneos do projeto; os excedentes ficam parados.
  Compile com `-j 2`. Com 8 compilações ao mesmo tempo a máquina fica sem memória.
- **Teste e clippy em release:** `cargo test --release -j 2` e
  `cargo clippy --release -j 2 ...`. Os `target/debug` das worktrees chegaram a 20 GB; o build de
  release já existe para o bench e é reaproveitado.
