# Ensino

Quem está na escola, em que nível, e como isso vira alfabetização anos depois. O detalhe está em
[`population.pt-BR.md`, "Ensino por coorte"](../../population.pt-BR.md#ensino-por-coorte).

Esta página nasceu de um pedido do autor: uma campanha de educação não pode subir a alfabetização
na hora. Ela sobe primeiro a quantidade de gente na escola, e a escolaridade vem nos anos seguintes
([Princípios](../principios.md#gradual-como-na-vida-real-mas-não-tanto-assim)).

## Decisões

### A escola anda por coorte, e o jogador vê a matrícula antes do resultado

- **Decisão:** o jogo guarda, por estado e por grupo de idade, o nível mais alto que cada um
  alcançou, e a matrícula de cada nível. A matrícula anda devagar até as vagas e a meta da lei; a
  coorte leva para a vida adulta o que estudou nos últimos anos.
- **O que o autor pensou:** ele aceitou a alfabetização anual, porque a escola anda por ano letivo,
  mas pediu ver primeiro quanta gente está na escola. Perguntou também o que conta como alfabetizado
  e se havia níveis. Sobre o pacote de decisões, disse: "Concordo com as decisões, por enquanto." O
  "por enquanto" é dele: pode rever depois de ver funcionando.
- **Como o jogo funciona por causa disso:** dobrar as vagas do fundamental sobe a matrícula num ano
  e mais nos anos seguintes; a escolaridade de quem tem 15 a 19 anos sobe aos poucos em cinco e em
  quinze anos; a alfabetização adulta, mais devagar ainda (os números do teste estão no documento).
  O painel de população mostra a pirâmide por nível, os alunos e os formandos que faltam.
- **Como é na vida real:** a escola nova enche em um a três anos, porque as famílias demoram; quem
  termina o fundamental hoje estudou nos anos anteriores.
- **Estado:** dados e contrato no jogo (#106); a escola no tick e o painel estão num ramo ainda sem
  PR (`feat/schooling-sim`).
- **Onde está o detalhe:** [`population.pt-BR.md`, "Ensino por coorte"](../../population.pt-BR.md#ensino-por-coorte);
  `crates/bf-sim/src/demography/education.rs`.

### Três níveis, mais quem lê sem escola

- **Decisão:** fundamental (5 a 14 anos), médio (15 a 19) e superior (20 a 24), mais "lê sem escola"
  (aprendeu em casa, no Exército ou numa campanha). Alfabetizado é quem tem 15 anos ou mais e lê e
  escreve.
- **O que o autor pensou:** respondeu à pergunta dele sobre níveis e sobre o que conta como
  alfabetizado; aceitou a proposta.
- **Como o jogo funciona por causa disso:** um país pode ter muita gente que lê sem escola (depois de
  uma campanha) e pouca gente com o fundamental; a tela separa os dois.
- **Como é na vida real:** é a definição das referências do placar (OWID e UNESCO) e dos dados de
  1925.
- **Estado:** no jogo (#106).
- **Onde está o detalhe:** [`population.pt-BR.md`, "Ensino por coorte"](../../population.pt-BR.md#ensino-por-coorte).

### Um prédio por nível, com o custo de cada um

- **Decisão:** escola, escola de nível médio e universidade são prédios públicos, cada um com alunos
  por professor e custo por aluno com fonte. A lei de educação dá a meta de matrícula de cada nível.
- **O que o autor pensou:** proposta aceita no pacote.
- **Como o jogo funciona por causa disso:** universalizar o médio custa bem mais que universalizar o
  fundamental, e a universidade mais ainda; o jogador escolhe a lei e paga a obra.
- **Como é na vida real:** as fontes do custo por aluno (Banco Mundial, OCDE, Psacharopoulos) estão
  no documento.
- **Estado:** dados no jogo (#106); obra pública por nível no ramo sem PR.
- **Onde está o detalhe:** [`population.pt-BR.md`, "Ensino por coorte"](../../population.pt-BR.md#ensino-por-coorte);
  `data/politics/laws.ron`.

### Campanha de alfabetização de adultos é programa do orçamento

- **Decisão:** a campanha é um programa com custo e prazo, aprovado no orçamento com confirmação;
  alcança parte dos analfabetos adultos por ano, e só parte deles fica alfabetizada de vez.
- **O que o autor pensou:** veio do exemplo dele: a campanha não pode subir a alfabetização na hora.
- **Como o jogo funciona por causa disso:** a campanha é a alavanca rápida que existe, mas tem custo
  e perda: metade evade, parte regride. Ela aumenta "lê sem escola", não o fundamental.
- **Como é na vida real:** Cuba em 1961, a liquidação do analfabetismo soviética, o Mobral; a perda
  vem de Abadzi (2003).
- **Estado:** regra e dado no jogo (#106, `data/economy/literacy_campaign.ron`); o comando e a tela
  no ramo sem PR.
- **Onde está o detalhe:** [`population.pt-BR.md`, "Ensino por coorte"](../../population.pt-BR.md#ensino-por-coorte).

### 1925 nasce dos 60 anos de escola anteriores

- **Decisão:** a escolaridade de cada estado em 1925 é reconstruída vivendo, ano a ano, a matrícula
  de 1865 a 1925 na pirâmide de 1925, e conferida com a escolaridade publicada para 1925.
- **O que o autor pensou:** decisão técnica da coordenação, pela regra de nenhum número de memória.
- **Como o jogo funciona por causa disso:** um país com escola antiga começa com adultos escolarizados
  em todas as idades; um país com escola recente, só nos mais jovens.
- **Como é na vida real:** Lee e Lee (2016), 111 países de 1870 a 2010, também a referência do placar.
- **Estado:** no jogo (#106).
- **Onde está o detalhe:** `data/scenario/1925/education.ron`; `data/history/education.csv`.
