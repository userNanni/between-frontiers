# Capital entre países: o fundo que investe fora (fase 4)

Estado: desenho, para o dono decidir. O estudo e a conta do retorno estão em
`late-industrialization.pt-BR.md` (parte 2). O contrato puro está em
`crates/bf-sim/src/foreign_capital.rs`, fora do tick. Este documento diz como ele entra no tick:
em que passo do agendador, o que o save guarda, como se testa e o que o dono decide.

## 1. Como é na vida real, e se a diferença importa

- **O fluxo é gradual, mas feito de projetos.**
  - O investimento direto chega como fábricas, cada uma com data, dono e contrato.
  - O estoque cresce por camadas. Cada camada guarda as condições do ano em que entrou: a lei,
    a parcela do sócio e o prazo da empresa mista.
  - Na China, o investimento direto recebido no ano passou de 0,03 % do PIB em 1980 a 1,1 % em
    1991 e 6,2 % em 1993 (`data/history/fdi.csv`), depois que a lei e as zonas abriram. Subiu
    ano a ano, não de uma vez.
- **O fluxo responde ao retorno com atraso.** Entre decidir e produzir há a obra. No jogo, o
  atraso é a fila de construção que já existe; não é um atraso inventado.
- **O contrato não muda com a lei.** Uma empresa mista de 1985 segue com o prazo de 1985 mesmo
  que a lei mude em 1990. Só a estatização mexe na fábrica existente
  (`late-industrialization.pt-BR.md`, "A nacionalização futura").
- **A diferença importa?** Sim, em dois pontos.
  - A passagem da fábrica ao sócio no fim do prazo depende do ano de cada camada.
  - Uma estatização mede quanto de cada dono foi tomado.

  Um estoque único por país, sem camadas, não sabe quando cada fábrica passa ao sócio, nem de
  quem era o que foi tomado. Por isso o estoque é guardado por camada: lugar, tipo de edifício,
  dono, ano e contrato.

## 2. O que entra no estado

`GameWorld::foreign`, campo novo **no fim** do mundo salvo, com duas partes:

| Parte | O que guarda | Tamanho |
|---|---|---|
| `stakes` | camadas: estado, tipo de edifício, país dono, níveis de propriedade estrangeira, parcela do sócio local, ano, prazo e modo de passagem | esparso: só onde houve investimento |
| `expropriations` | por país: as estatizações sem pagamento (ano, parcela tomada), para o prêmio que dura uma geração | algumas linhas por país, no século |

- As camadas **não entram na `Plant`**. A planta continua sendo estado × tipo, com níveis; a camada
  diz quantos desses níveis são de quem.
- Quando a planta perde níveis (depreciação, fechamento por prejuízo), as camadas perdem na mesma
  proporção, em ordem fixa.
- O estado só entra no hash quando não está vazio. **Sem nenhum investimento feito, o hash é o da
  `main`.**

## 3. Onde entra no agendador

| Passo | Sistema e cadência | O que faz |
|---|---|---|
| **Escolher onde investir** | `economy-investment`, mensal (dia 8), depois das obras em casa | Lê a sobra de cada fundo (`InvestmentChanges::leftover`, já exposta e fora do hash). Monta os lugares candidatos e reparte com `foreign_capital::allocate`. Põe as obras na fila do país de destino, pagas pelo fundo da origem, com a camada criada quando a obra termina |
| **Remessa do lucro** | `economy-week`, semanal (segunda), onde o lucro se divide entre donos, Estado e fundo (`week.rs`, `to_owners`) | A parte estrangeira do lucro de cada planta com camada sai do país de destino e entra no fundo da origem (`foreign_capital::remittance`), menos o que é reinvestido lá |
| **Contrato e estatização** | `economy-year`, anual (1º de janeiro), depois da política | Passa ao sócio as camadas cujo prazo venceu (`local_share`). Quando a lei de capital vira fechada com camadas no país, ou a economia vira planificada, as camadas viram do Estado e a estatização vai para a memória |
| **Técnica transferida** | `research`, mensal (dia 22) | O canal de imitação do #79 ganha o termo da planta estrangeira: `transfer_rate`, a parcela estrangeira dos níveis de cada grupo |
| **Influência** | `influence`, mensal, quando entrar no tick (desenho em `influence.pt-BR.md`) | O canal "investimento", hoje dormente, lê as camadas: o valor das plantas de B cujo dono é A, sobre o PIB de B |

**Quem pode investir onde:**
- a lei do destino abre a porta: `foreign_investment` maior que zero, com a parcela estrangeira
  permitida pela lei;
- o reconhecimento é condição: capital não vai de quem não reconhece para quem não é reconhecido
  (`treaties.pt-BR.md`, "Reconhecimento");
- a cláusula `capital_protection` de um tratado em vigor baixa o prêmio de risco entre as partes;
  estatizar contra ela é quebra de tratado.

**Candidatos a destino** (o custo depende disto):
- só os tipos de edifício que rodam cheios no país de origem (a planta estrangeira vende para fora:
  `late-industrialization.pt-BR.md`, seção 1 da parte 2);
- só os estados com trabalhadores ociosos;
- só os países cuja lei não fecha a porta.

**Dinheiro:** transferências em unidades reais a preço-base, as mesmas do fundo e do Tesouro. O
efeito no câmbio fica para quando entrar o balanço de pagamentos (decisão 4).

## 4. Como se testa

- **Conservação** (teste de unidade e uma semana do jogo):
  - investir fora tira do fundo da origem exatamente o que entra como obra no destino;
  - a remessa tira do destino exatamente o que entra no fundo da origem;
  - a soma mundial de fundos, tesouros e poupanças, antes e depois de cada passo, muda só pelo
    que a produção e o consumo da semana mudam, e não pelo movimento entre países;
  - a renda mundial da semana é a mesma com e sem a remessa: só muda quem recebe.
- **Neutralidade:**
  - com todas as portas fechadas por lei, o hash de 10 e de 101 anos é o da `main`;
  - com o estado vazio, nada entra no hash.
- **Direção** (critérios de `late-industrialization.pt-BR.md`, seção 7 da parte 2, rodadas com
  `BF_LAWS`):
  - China fechada: zero de investimento recebido;
  - China com empresa mista em 1979 e zonas em 1980: investimento recebido acima de 1 % do PIB
    antes de 1995;
  - Índia aberta em 1991: perto de 1 % nos anos 2000;
  - país rico e aberto não vira destino por salário;
  - quem estatiza recebe menos da metade do fluxo de antes por pelo menos dez anos;
  - referência: `data/history/fdi.csv`.
- **Save antigo:** o campo novo vai no fim do mundo salvo e vazio. O `decode_world` já tenta o save
  antigo completado com zeros no fim, e um vetor vazio é um zero em postcard. O hash não muda
  (estado vazio fora do hash). Entra no teste `saves_of_every_layout_load_with_their_hash`.
- **Determinismo:** a repartição já não depende da ordem dos destinos (teste do contrato). No tick,
  as camadas ficam em ordem de criação, e as reduções seguem a ordem de id. Rodada de 101 anos com
  troca de threads e salvar/recarregar.
- **CPU:**
  - por mês, poucos países com sobra, vezes os tipos cheios na origem, vezes os estados com
    ociosos nos países abertos: dezenas de milhares de avaliações de `site_return`, cada uma
    poucas multiplicações;
  - por semana, a remessa percorre as camadas, algumas centenas a poucos milhares no fim do
    século;
  - meta: menos de 5 % no sistema `economy-investment` e menos de 1 % na semana, contra a `main`.

## 5. Decisões do dono

1. **Camadas fora da planta, por lugar, tipo, dono e ano.**
   - **Recomendação:** sim, como na seção 2.
   - **Teste:** a passagem ao sócio no fim do prazo e a estatização parcial dão os números do
     contrato, camada por camada.
   - **Custo:** uma tabela esparsa no save. A alternativa, um dono na própria planta, não guarda
     camadas de anos diferentes nem dois donos no mesmo estado.
2. **Estoque de 1925: zero ou semeado.**
   - **Recomendação:** começar em zero. Não há tabela aberta de investimento direto por país antes
     de 1970: Dunning e Twomey são livros pagos, e Lipsey só dá a parte de cada país no estoque
     mundial de 1960 (`late-industrialization.pt-BR.md`, "O que ficou de fora").
   - **Teste:** com zero, o hash dos primeiros meses é o da `main`.
   - **Custo:** o capital britânico na Argentina, as ferrovias da Índia e o petróleo do México de
     1925 não existem no jogo até alguém investir. Semear depois, com dado de fonte, é um PR de
     dados.
3. **O que o jogador comanda.** Já decidido em 2026-11-16: a lei do próprio país e uma lista curta
   de países vetados.
   - **Recomendação:** o veto entra como comando (`SetCapitalEmbargo`) e vale nos dois sentidos:
     não investir lá e não aceitar de lá.
   - **Teste:** com o veto, nenhuma camada entre os dois.
   - **Custo:** baixo, um comando e uma lista no estado do país.
4. **Câmbio.**
   - **Recomendação:** transferências em unidades reais agora, sem efeito no câmbio. O câmbio passa
     a ler o saldo de capital quando o balanço de pagamentos entrar com o comércio.
   - **Teste:** conservação em unidades reais.
   - **Custo:** um país que recebe muito capital não valoriza a moeda por isso até lá; fica
     anotado.
5. **Técnica pela planta estrangeira, já neste passo.**
   - **Recomendação:** sim. O termo `transfer_rate` já existe e é pequeno.
   - **Teste:** com 20 % dos níveis de um grupo em mãos estrangeiras, a difusão anda 1,2 vez mais
     rápido; com a exigência de entrega ao sócio, 1,4 vez.
   - **Custo:** uma leitura das camadas por mês no sistema de pesquisa.

## 6. Ordem dos PRs

1. Este desenho, em rascunho.
2. Estado e camadas no save, sem fluxo, com hash igual ao da `main`.
3. Fluxo mensal e remessa semanal, com os testes de conservação.
4. Contrato, estatização e técnica.
5. Canal de influência e rodadas de direção (China e Índia).

Cada passo fecha com o hash ou com a rodada, e o placar de investimento direto contra `fdi.csv`
entra no passo 5.
