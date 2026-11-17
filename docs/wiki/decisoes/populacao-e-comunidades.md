# População e comunidades

Quem vive em cada estado, em que língua e religião, quantos filhos tem, quando morre e para onde
vai. O detalhe está em [`population.pt-BR.md`](../../population.pt-BR.md) e em
[`architecture.pt-BR.md`, "Modelo de população"](../../architecture.pt-BR.md#modelo-de-população).

## Decisões

### Identidade não vira percentual

- **Decisão:** cultura e religião juntas formam uma **comunidade**, que é linha própria da
  população, com gente por profissão. O que se agrupa para poupar cálculo é o local de trabalho, não
  a identidade.
- **O que o autor pensou:** desenho de arquitetura aceito no início do projeto. A pergunta era o que
  o século pede: segregação, expulsão, genocídio, partilha e cidadania diferenciada miram um grupo.
- **Como o jogo funciona por causa disso:** uma lei pode mirar os judeus da Polônia ou os alemães da
  Tchecoslováquia, e o jogo sabe onde eles estão e em que trabalham. "30 % irlandeses, 40 % católicos"
  nunca perde que os irlandeses são os católicos.
- **Como é na vida real:** de 1925 a 2025, o ato dirigido a um grupo está no centro da história:
  Holocausto, partilha da Índia, apartheid, direitos civis, Iugoslávia, Ruanda.
- **Estado:** no jogo (#97).
- **Onde está o detalhe:** [`architecture.pt-BR.md`, "O que não pode virar percentual"](../../architecture.pt-BR.md#o-que-não-pode-virar-percentual).

### Quem ganha linha: tamanho vezes relevância

- **Decisão:** a comunidade ganha linha pela parcela no estado vezes a relevância (elite, status
  legal, organização, patrono, concentração). Entram as que somam 95 % da importância, com teto de 8
  por estado; quem é importante no país ganha linha onde tiver 2 % ou mais. O resto fica em "outras
  minorias", que ainda guarda quem é cada um.
- **O que o autor pensou:** o autor aceitou as três decisões de comunidades como propostas.
- **Como o jogo funciona por causa disso:** uma elite de 5 % (os brancos na Rodésia, os europeus nas
  Índias Holandesas) pesa como um grupo de 20 %, porque tem terra, capital e Estado. Ingleses e
  indianos do Quênia ou chineses das Índias Holandesas existem como linha só por isso. Ninguém some:
  quem está em "outras minorias" mantém a identidade.
- **Como é na vida real:** cada número de 1925 tem fonte por estado (censo, misto ou estimativa); a
  China, sem censo até 1953, e boa parte da África e da Ásia são estimativa, anotada linha a linha.
- **Estado:** no jogo (#41, #97); a promoção e o rebaixamento de linha durante a partida, ainda não.
- **Onde está o detalhe:** [`population.pt-BR.md`, "Quais comunidades viram linha"](../../population.pt-BR.md#quais-comunidades-viram-linha-decisão-1-regra).

### Assimilação é fluxo explícito, lento, e a língua anda mais que a fé

- **Decisão:** uma vez por ano, parte de cada comunidade passa para a cultura principal do país
  (mesma religião), e uma parte bem menor passa para a religião da maioria (mesma cultura). A lei, a
  discriminação, a concentração e a distância da língua mudam o ritmo.
- **O que o autor pensou:** proposta aceita (decisão 3 das comunidades).
- **Como o jogo funciona por causa disso:** o italiano na Argentina vira argentino em algumas
  décadas; o catalão e o flamengo, maioria no próprio estado, quase não assimilam; quem é
  discriminado assimila menos, porque a porta está fechada dos dois lados. Na colônia não há
  assimilação cultural sem lei que a empurre.
- **Como é na vida real:** o ritmo de base dá a ordem do que se viu com imigrantes europeus nos EUA
  e na Argentina de 1900 a 1970; a segregação de Jim Crow não assimilou os afro-americanos.
- **Estado:** no jogo (#97); as leis com alvo por comunidade, à espera de um PR de dados.
- **Onde está o detalhe:** [`population.pt-BR.md`, "Comunidades na simulação"](../../population.pt-BR.md#comunidades-na-simulação).

### Demografia sem taxa por país nem por data

- **Decisão:** nascimentos, mortes e migração saem de parâmetros mundiais e da economia de cada
  estado (conforto, escolaridade, trabalho fora do campo, hospitais, remédio, contracepção). Nenhuma
  taxa é escrita por país.
- **O que o autor pensou:** princípio de causa visível do projeto.
- **Como o jogo funciona por causa disso:** a transição demográfica acontece quando o país enriquece,
  escolariza e urbaniza, não numa data. A tela de população mostra a pirâmide, as taxas e por que os
  nascimentos sobem ou descem.
- **Estado:** no jogo (#54, #80).
- **Onde está o detalhe:** [`population.pt-BR.md`, "Demografia"](../../population.pt-BR.md#demografia).

### Cada classe tem a sua fecundidade

- **Decisão:** cada pop (estado × profissão) tem a própria fecundidade, pela mesma curva, com o
  conforto da classe e o trabalho fora do campo.
- **O que o autor pensou:** decisão técnica do thread da demografia.
- **Como o jogo funciona por causa disso:** dentro do mesmo estado, a cidade e a classe mais
  confortável têm menos filhos que o campo pobre.
- **Estado:** no jogo (#88).
- **Onde está o detalhe:** [`population.pt-BR.md`, "Natalidade por classe"](../../population.pt-BR.md#natalidade-por-classe).

### Cultura entra como difusão da transição, nunca como "tem mais filhos"

- **Decisão:** a cultura pesa na fecundidade só como difusão: quanto da mesma família linguística
  já vive em lugares com fecundidade baixa. A religião só entra com efeito medido líquido de
  escolaridade e renda; sem essa medida, não entra.
- **O que o autor pensou:** as três decisões desse desenho foram tomadas em 2026-11-17, como estavam propostas.
- **Como o jogo funciona por causa disso:** a queda da fecundidade se espalha por quem fala a mesma
  família de línguas, centrada na média mundial: ninguém ganha ou perde fecundidade só pelo nome da
  cultura.
- **Estado:** o termo de difusão já roda com as comunidades (#97); o de religião, sem número, não
  entra.
- **Onde está o detalhe:** [`population.pt-BR.md`, "Cultura e religião na natalidade"](../../population.pt-BR.md#cultura-e-religião-na-natalidade-desenho-sem-código).

### Migração entre países por atração, diáspora e ondas

- **Decisão:** a gente vai para onde a renda ou a terra puxa, pesada pela distância, pelas vagas,
  pela lei de imigração do destino e pela rede de quem já foi; as diásporas viram correntes, e os
  grandes movimentos, ondas.
- **O que o autor pensou:** decisão técnica do thread da demografia.
- **Como o jogo funciona por causa disso:** quem chega entra na linha da sua comunidade, ou em
  "outras minorias" com a identidade. Uma lei de imigração fechada corta a atração.
- **Estado:** no jogo (#54, #97).
- **Onde está o detalhe:** [`population.pt-BR.md`, "Migração entre países"](../../population.pt-BR.md#migração-entre-países).

## O que ainda não existe

- Migração interna por comunidade: quem muda de estado ainda não leva a comunidade.
- Riqueza, alfabetização e atitudes por comunidade; profissões repartidas por comunidade com fonte.
- Os alvos das leis (quem cada lei de discriminação ou assimilação atinge) nos dados de 1925.
