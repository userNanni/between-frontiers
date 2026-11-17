# Tempo e ordem de cálculo

O que o jogo calcula todo dia, toda semana, todo mês e todo ano, em que ordem, e por quê. A fonte
é o código: a função `schedule` em
[`crates/bf-sim/src/game.rs`](../../../crates/bf-sim/src/game.rs) e a cadência de cada sistema.

## Decisões

### Cada processo na cadência em que anda na vida real

- **Decisão:** o tick é diário, mas cada sistema roda na cadência do processo que simula: o mercado
  fecha por semana; trabalho, investimento, demografia e pesquisa, por mês; troca de método e ano
  letivo, por ano.
- **O que o autor pensou:** "Gostei das separações mensais: as coisas são dinâmicas e graduais, mas
  não tanto assim." Ele pediu que toda regra confira se o processo real é mais gradual e se isso
  muda a simulação ([Princípios](../principios.md#gradual-como-na-vida-real-mas-não-tanto-assim)).
- **Como o jogo funciona por causa disso:** o preço reage à falta na semana; a fábrica nova é
  decidida no mês e fica pronta quando a obra recebe todos os pontos; a escola enche ano a ano. O
  jogador vê a matrícula antes da alfabetização e a obra antes da fábrica.
- **Estado:** no jogo.

### A ordem é fixa, e o resultado não depende do número de núcleos

- **Decisão:** os sistemas do dia rodam na ordem em que foram registrados. Os que não mexem nos
  mesmos dados calculam em paralelo, mas o resultado é sempre o de rodar um por vez, nessa ordem.
- **O que o autor pensou:** princípio do projeto: determinismo. Desempenho é requisito, medido toda
  noite, e o paralelismo não pode mudar o mundo.
- **Como o jogo funciona por causa disso:** a mesma campanha, com a mesma semente, dá o mesmo mundo
  com 1 ou com 16 núcleos, e depois de salvar e carregar. Isso permite o replay, o teste de 100 anos
  e, no futuro, o multiplayer.
- **Estado:** no jogo (#7, a engine Laplace-Schelling).
- **Onde está o detalhe:** [`architecture.pt-BR.md`, "Laplace-Schelling: a engine"](../../architecture.pt-BR.md#laplace-schelling-a-engine);
  `crates/laplace-schelling/src/schedule.rs`.

### Fronteira muda por último no dia

- **Decisão:** a operação de fronteira é o último sistema do dia.
- **O que o autor pensou:** decisão técnica: uma operação vê os resultados do dia, e o dia seguinte
  já vê o mapa novo.
- **Estado:** no jogo (#95).

## A ordem no `main`

Na ordem em que os sistemas estão registrados (é a ordem em que rodam quando caem no mesmo dia):

| # | Sistema | Quando | O que faz |
|---|---|---|---|
| 1 | censo | 1º de janeiro | registra os totais do mundo (população e PIB) |
| 2 | política | todo dia; contas no dia 1º; movimentos e leis em 1º de janeiro | todo dia: as leis pedidas por comando; dia 1º: as contas do mês (peso dos grupos de interesse, alíquotas da regra automática); 1º de janeiro: os movimentos andam e os países sem jogador mudam leis sob pressão |
| 3 | mercado semanal | segunda-feira | pedidos, fechamento de preço por estado e por país, comércio mundial, liquidação; depois moeda, banco central, títulos e o livro de dívida entre Estados; obras prontas entram em funcionamento |
| 4 | trabalho | dia 1º | contratação e demissão por profissão, migração interna |
| 5 | demografia | dia 15 | nascimentos, mortes, pirâmide, migração entre países, comunidades; assimilação em janeiro |
| 6 | investimento | dia 8 | o investidor privado escolhe obras novas |
| 7 | pesquisa | dia 22 | esforço, estágios, difusão, imitação |
| 8 | ano | 1º de janeiro | troca de método, desgaste do capital, escola, estatísticas |
| 9 | fronteiras | todo dia, por último | executa as operações pedidas (por comando ou por tratado) |

Os dias diferentes no mês (1, 8, 15, 22) espalham o trabalho mensal pelas semanas, em vez de juntar
tudo no dia 1º.

## O que os PRs abertos acrescentam

| Sistema | Quando | PR |
|---|---|---|
| forças | segunda-feira, antes do mercado | #115 |
| influência | dia 16, depois da demografia | #110 (rascunho) |
| capital para fora | no fim do investimento do mês; remessa na liquidação semanal | #117 (rascunho) |
| escola no tick | ano letivo em janeiro | ramo `feat/schooling-sim`, sem PR |

Esta tabela muda a cada sistema que entra: quem acrescenta um sistema ao `schedule` atualiza esta
página no mesmo PR ([Como manter](../como-manter.md)).
