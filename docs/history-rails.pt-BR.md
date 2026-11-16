# Trilho histórico, interesses, hostilidade e IA estratégica

Estado: desenho, **sem código**, com as decisões do dono de 2026-11-16. É o último bloco do plano
(fase 7, ver `architecture.pt-BR.md`): depende de política (fase 3), diplomacia (4), guerra (5) e
da confiança (`confidence.pt-BR.md`).

## O pedido e o que foi decidido

O pedido: sem o jogador tocar em nada, a história real acontece (a Alemanha produz um partido
nacionalista que quer expansão, os Estados Unidos financiam a reconstrução da Europa, a China tem a
revolução); quando o jogador começa a mexer, tudo vai desviando.

Os princípios do repositório dizem: pouco sorteio, causa visível, determinismo. A decisão do dono
(2026-11-16) foi a mistura, "como o Victoria 3": **um roteiro com pontos**, em que cada grande
evento tem condições para entrar e para sair; o jogador pode mudar o curso, mas tem de ir contra
as forças que impelem, e isso custa esforço. O resto deste documento é esse desenho:

- **Entradas históricas** (abaixo): o roteiro com pontos.
- **Pressão histórica:** a força que a entrada soma às causas medidas, visível pelo nome, que some
  quando o mundo se afasta da história.
- **Personagens** com janela de datas e condição medida.
- **Casus belli** como forma do empurrão na guerra.
- **Agendas** dos países principais primeiro, os demais aos poucos.

Fica uma decisão aberta: as regras de partida (seção própria, mais abaixo).

## Entradas históricas: o roteiro com pontos

Uma entrada é um objeto de dados (`data/history/entries.ron`, a criar) com o mesmo espírito das
*journal entries* do Victoria 3: aparece sob condição, tem progresso, marcos e saída. Nenhum texto
ou nome de evento de jogo ou mod entra; os nomes são os fatos históricos.

| Parte | O que é | Exemplo (a ascensão revisionista na Alemanha) |
|---|---|---|
| Protagonista | tag de 1925 ou papel (metrópole com colônias, hegemon) | GER |
| Janela | datas de referência, com margem; antes dela a entrada não abre, depois dela a pressão zera | 1928–1938 |
| Condição de entrada | predicados sobre o estado medido, os mesmos tipos de condição dos movimentos (`politics.pt-BR.md`) | território perdido > 0; reparações em moeda estrangeira; desemprego ≥ 15 % |
| Marcos | os pontos do roteiro, cada um com a própria condição e a data de referência para o placar | movimento `revisionism` com 10 % de apoio; `regime_change` vence (lei `government` vira partido único); serviço militar universal; primeira exigência territorial |
| Forças | os termos que empurram o próximo marco: causas medidas e a pressão histórica | desemprego, legitimidade, reparações, minoria fora da fronteira, pressão histórica |
| Saída por sucesso | o último marco alcançado | |
| Saída por falha | as condições de entrada falsas por N anos seguidos, ou a janela vencida | desemprego abaixo de 8 % por 3 anos com legitimidade alta |
| Efeitos permitidos | só três: somar pressão a termos que já existem (margem de movimento, postura de grupo, pontuação da IA); abrir uma ação (casus belli, oferta de ajuda); dar nome (regime, personagem) | pressão em `revisionism` e `regime_change`; casus belli sobre o corredor polonês no último marco |

A entrada **nunca escreve o estado** ("a Alemanha vira partido único"): quem muda a lei é a
política, quem declara a guerra é a IA ou o jogador. A entrada só soma força ao que as regras já
fazem e mede se o marco veio. É o que separa um roteiro com pontos de um roteiro datado.

### Pressão histórica

```text
pressão de um marco = peso da entrada × proximidade × fator da janela
proximidade = 1 − distância(estado medido, pré-condições históricas) / limite   (0 a 1)
fator da janela = 0 antes da janela, 1 dentro, cai a 0 em 2 anos depois do fim
```

- **Só soma, nunca cria.** A pressão multiplica a margem de uma condição que já existe; margem
  zero continua zero. A Alemanha sem desemprego, sem reparações e sem território perdido não
  ganha partido expansionista por pressão histórica.
- **Desfaz-se com o desvio.** A distância compara as pré-condições do marco no mundo simulado com
  as da história. O jogador que muda as condições afasta a proximidade de 1 e a pressão cai sem
  nada ser desligado à mão.
- **Visível.** A tela do movimento ou da decisão mostra o termo pelo nome: "pressão histórica +120
  (o mundo está a 18 % da Alemanha de 1932)".
- **Determinística.** Sem sorteio.

### Forças e esforço

O dono pediu que o jogador possa mudar o curso, mas contra as características que impelem. O
esforço é medido na mesma conta: **vencer uma entrada é levar a soma das forças abaixo do limite
do próximo marco**. Cada força tem uma alavanca e um custo. A tela da entrada mostra as forças em
ordem e, para cada uma, a alavanca e quanto falta.

```text
Ascensão revisionista (Alemanha)                     próximo marco: regime de partido único
  desemprego 31 %                   +310   alavanca: evitar a Depressão, gasto público
  legitimidade 340                  +180   alavanca: leis que os grupos apoiam
  reparações 9 % da receita         +90    alavanca: perdão (diplomacia, só os credores)
  minoria fora da fronteira         +60    fixo em 1925
  pressão histórica                 +120   cai se o mundo se afastar de 1932
  total 760 / limite 600            falta: −160 para parar o marco
```

Para cada fio, o veredito diz quais forças impelem e o esforço para vencê-las, em três graus
(estimativa de desenho, para calibrar):

- **Baixo:** uma alavanca do próprio país, usada a tempo, basta.
- **Médio:** duas ou mais alavancas, ou uma que custa caro (dívida, legitimidade).
- **Alto:** precisa de outro país (perdão de dívida, ajuda, aliança) ou de anos de política
  contra os grupos no poder.

## Personagens

Decidido em 2026-11-16: o jogo tem personagens, com janela de datas e coisas prováveis, como os
agitadores do Victoria 3 (lá, os agitadores históricos surgem sob condição histórica e não são
garantidos em toda partida; Marx é um agitador). O desenho mínimo:

### Tipos

| Tipo | Onde vive | O que faz |
|---|---|---|
| Agitador | um movimento de um país (ou exilado em outro) | soma um termo à margem do movimento (até +30 %, estimativa); pode ser preso, exilado ou anistiado pela lei de liberdades civis |
| Líder de grupo | um grupo de interesse (ou partido, quando houver) | desloca a postura do grupo sobre as leis da sua agenda (até ±150 milésimos) e soma ao peso do grupo |
| Chefe de governo | o país | dá nome ao regime ("a Alemanha de partido único de X"), carrega a agenda histórica do país enquanto governa e soma à legitimidade |

Um personagem não tem traço sorteado nem ideologia rotulada: a agenda dele é uma lista de leis e
movimentos (o que ele empurra), como a meta de um movimento. Hitler empurra `revisionism`,
`regime_change` e as leis dirigidas; Lenin já não está vivo em 1925; Mao empurra `agrarian_reform`
e a insurgência; Deng, `market_reform`.

### Como surge

- **Histórico** (`data/history/characters.ron`, a criar): nome, ano de nascimento e de morte de
  referência, país e comunidade de origem, tipo, agenda, **janela** de atividade (com margem) e
  **condição medida** (por exemplo, movimento `revisionism` com 5 % de apoio na Alemanha).
- **Sorteio determinístico:** a cada mês da janela em que a condição vale, o personagem surge com
  chance fixa (proposta: 1/24 por mês, para cair em média em dois anos). O número sai de um hash
  de (semente, id do personagem, mês), uma corrente própria que não consome o gerador de nenhum
  outro sistema: acrescentar um personagem não muda o resto da rodada.
- **Sem condição, não surge.** Se a janela passa sem a condição, ele não aparece. Se um movimento
  passa de um limite de apoio sem líder, surge um **genérico** (nome tirado das listas da
  comunidade, sem agenda além da do movimento, efeito menor).
- **Fim:** pela mortalidade do país a partir do ano de nascimento (o mesmo hash), pela troca de
  regime, pela prisão ou pelo exílio. A morte não é a data histórica: Mao pode morrer em 1970 ou
  em 1982.

### Como conversa com o pouco sorteio

O sorteio decide **quem e quando dentro da janela**, nunca se a causa existe. O efeito de um
personagem é limitado e aparece pelo nome nos termos. A mesma semente dá a mesma rodada. O preço é
que o placar passa a ter variância entre sementes: o CI roda uma semente, e uma rodada periódica
com cinco sementes mede a dispersão de cada marco.

### Dados

Nomes, datas e cargos do Wikidata (CC0), com a fonte por linha, sob a regra de dados do
repositório (todo dado público pode entrar, licença anotada). A lista é escolhida por critério
nosso (chefes de governo e líderes de movimento dos 83 países de 1925 a 2025 que movem uma
entrada), não copiada da lista de nenhum jogo ou mod.

### O que os personagens e as entradas pedem da política agora

A política da fase 3d (PR #53, em andamento) já tem estado por país, forma de governo como lei,
movimentos com margem medida, legitimidade e o registro de cada mudança. Para os personagens e as
entradas entrarem na fase 7 sem refazer nada, o que convém deixar pronto lá:

1. **Termos abertos na margem do movimento e na postura dos grupos.** Hoje a margem vem da
   condição mais fraca; a fase 7 soma termos nomeados (agitador, pressão histórica). Se a conta já
   for uma lista de termos `(chave, valor)` que a tela lê, personagem e pressão são só mais dois
   termos, sem mudar a fórmula. O mesmo para a postura de um grupo (o líder desloca a postura sobre
   as leis da agenda dele).
2. **Quem mudou a lei.** O registro de mudanças tem "comando, movimento ou grupos"; reservar as
   variantes "personagem" e "entrada histórica" no tipo, para o save não mudar de formato depois.
3. **Lugares vazios no estado:** chefe de governo por país, líder por grupo de interesse e líder
   por movimento (`Option<id>`), em `PoliticsState`, com `#[serde(default)]`. Vazios até a fase 7.
4. **Nenhum sorteio na política.** Os personagens usam uma corrente própria (hash de semente,
   personagem e mês); a política continua sem gerador.
5. **Exílio:** o movimento precisa saber a comunidade que recruta, para o agitador exilado agir de
   fora (já previsto com as comunidades nos pops).
6. **Placar:** as entradas gravam no mesmo `events.json` que o PR #53 já produz; manter o formato.

## Regras de partida (decisão aberta)

O dono duvidou dos três modos propostos antes (histórico, plausível, livre): "manutenibilidade e
testar são pontos reais". Ele tem razão: três modos calibrados são três jogos para manter. A
proposta enxuta:

- **Um jogo só, um caminho de código.** Nada de modo: a pressão histórica, as agendas e os
  personagens históricos são sempre o mesmo código.
- **No máximo duas regras de partida em dados**, cada uma um escalar que o código multiplica, sem
  `if` em sistema nenhum:
  - `historical_pressure` (padrão 1, desligada 0): o peso das entradas;
  - `historical_characters` (padrão ligado; desligado, só genéricos).
- **Só a configuração padrão é calibrada e coberta pelo placar.** As outras não prometem
  história: têm só teste de fumaça e de determinismo.

| | O que o CI roda | Custo de manter |
|---|---|---|
| Padrão | toda noite: os 100 anos do bench e o placar histórico (números e eventos), uma semente; por semana, cinco sementes | é o jogo; toda calibração mira aqui |
| `historical_pressure = 0` | em cada PR que toca `history/`: 10 anos, duas contagens de threads, mesmo hash, nenhum país quebrado; toda noite: 100 anos de fumaça (sem placar) | uma rodada a mais por noite (cerca de 2 a 4 min de parede na máquina atual); nenhuma regra de jogo própria; o risco é alguém calibrar um parâmetro que só funciona com a pressão ligada, e o placar não pega |
| `historical_characters = off` | o mesmo, 10 anos por PR e 100 anos de fumaça por noite | idem; mais a garantia de que os genéricos sozinhos não travam um movimento |

Cada regra a mais dobra as combinações possíveis; com duas, o CI testa cada uma sozinha contra o
padrão, não as quatro combinações. Uma terceira regra só entraria com o mesmo critério: escalar em
dados, sem caminho de código próprio, sem promessa histórica.

**Comparação com o que existe:**

- **Victoria 3:** não tem modo histórico. Tem regras de partida (por exemplo, ligar ou desligar o
  investimento autônomo), três estratégias de IA por país sorteadas com peso e trocadas por
  acúmulo de mudanças (diário de desenvolvimento 59), *journal entries* com condição de entrada,
  de conclusão e de falha, e agitadores históricos que surgem sob condição e não são garantidos
  (diários 82 e 83). A história entra por pesos no cenário, não por uma chave.
- **Hearts of Iron IV:** tem a chave, por país: o comportamento da IA pode ser o padrão, o histórico
  (segue a árvore de foco real), um alternativo ou sorteado (wiki, *Game rules*). É o modelo de
  "modo histórico" que o dono questionou, e o custo aparece lá: cada caminho de foco é conteúdo a
  manter.

**Recomendação:** um jogo, padrão calibrado, as duas regras escalares acima numa tela de opções
sem promessa histórica. Alternativa mais barata: nenhuma regra (só o padrão), e a pressão
desligada fica só como parâmetro de teste do CI para medir quanto da história vem da causa.

## Interesses: o que cada país quer

Mesmo espírito da postura dos grupos de interesse (`politics.pt-BR.md`): nenhum rótulo, a conta
aberta. Um interesse é um **objetivo sobre o mundo** com um valor calculado a partir de estado
medido, termo a termo.

| Interesse | Medido por | Exemplo de 1925 |
|---|---|---|
| Território perdido | estados que o país tinha em data de referência (antes de 1914, no cenário) e não tem; peso pela população da própria comunidade neles | Alemanha: corredor polonês, Alsácia-Lorena, colônias; Hungria: Trianon |
| Minoria fora da fronteira | população da comunidade nacional em estados vizinhos (o catálogo de comunidades já tem) | Alemães nos Sudetos e na Áustria; húngaros na Romênia |
| Matéria-prima | parcela das importações de um bem crítico (petróleo, ferro, borracha, ferroligas) sem fonte própria, vezes o peso do bem na indústria | Japão: petróleo, ferro, borracha; Itália: carvão e petróleo |
| Mercado e capital | exportações e investimento no exterior por destino (fase 4) | Estados Unidos e Reino Unido: mercado aberto, dívida paga |
| Dívida | serviço da dívida externa e de reparações sobre a receita (3c) | Alemanha: Plano Dawes |
| Segurança | força militar dos vizinhos contra a própria, aliados, fronteira terrestre (fase 5) | França diante da Alemanha; Polônia entre Alemanha e URSS |
| Regime | distância entre as leis do país e as dos outros, nos efeitos que os grupos dominantes mais pesam (a mesma conta da postura) | URSS contra os países de mercado; o inverso também |
| Posição | lugar no ranking de PIB, indústria e força, e a tendência (subindo ou caindo) | Reino Unido em declínio relativo; Estados Unidos subindo |
| Colônias | valor das colônias para a metrópole (mercado, matéria-prima, prestígio) contra o custo (guarnição, movimento de independência) | Reino Unido, França, Países Baixos, Portugal, Bélgica |

- **Quem pesa:** o valor de um interesse é ponderado pelos grupos de interesse que mandam. O mesmo
  território perdido pesa pouco num governo de comerciantes e muito num de militares e
  proprietários. É assim que um mesmo país muda de agenda quando muda de governo, sem rótulo.
- **A tela** mostra, por país, a lista de interesses com a conta: "A Alemanha quer o corredor
  polonês: 2,1 milhões de alemães (peso 0,4), perdido em 1920 (peso 0,3), governo atual pesa ×0,6
  (comércio e indústria no poder)".

## Hostilidade: de interesse em conflito a guerra, e de volta

Dois países são rivais quando o interesse de um é a perda do outro (o território que um quer é do
outro; a matéria-prima de que um precisa está na esfera do outro; os regimes se ameaçam). A
hostilidade é um número por par, alimentado por esses termos e mais:

- **Corrida armamentista:** o gasto militar de um sobe a ameaça medida pelo outro (fase 5).
- **Incidentes:** ação de um que fere interesse do outro (ocupação, sanção, tarifa contra ele,
  apoio a um movimento separatista dele). Cada um soma uma vez e decai.
- **Memória:** a guerra anterior entre os dois deixa um piso que decai em décadas.
- **Recua com:** comércio entre os dois (custo da ruptura), tratado, bloco comum, inimigo comum,
  dissuasão (arma nuclear dos dois lados, fase 5), troca de governo.

Escada, por faixas da hostilidade, com causa visível em cada degrau:

| Degrau | O que a IA pode fazer | O que muda no mundo |
|---|---|---|
| Tensão | retórica, tarifa, recusa de tratado | influência e comércio caem |
| Rivalidade | aliança contra o rival, apoio a movimentos dentro dele | blocos se formam |
| Corrida armamentista | gasto militar, recrutamento (lei `universal_service`) | orçamento e indústria bélica |
| Crise | exigência com prazo (o que quer e o que oferece), apoiada num casus belli; terceiros escolhem lado | mobilização; confiança dos mercados cai (contágio pelo medo) |
| Guerra | só se o valor esperado do que se exige passa do custo esperado, para quem decide | fase 5 |

### Casus belli e pressão popular: o empurrão na guerra

Decidido em 2026-11-16. Quando uma entrada histórica chega ao marco de guerra (a exigência sobre a
Polônia, a ofensiva no continente asiático), a pressão histórica faz duas coisas e nenhuma outra:

- **Casus belli:** abre ao protagonista o direito de fazer aquela exigência contra aquele alvo,
  sem o custo de legitimidade e de hostilidade de terceiros que uma exigência sem motivo teria. O
  casus belli tem prazo (a janela da entrada) e some se a condição de entrada cair.
- **Pressão popular:** um termo na pontuação da IA a favor de usar o casus belli, do tamanho da
  pressão histórica do marco ("a opinião pede a revisão de Versalhes +140").

O jogador que controla o país recebe **só o casus belli**: a decisão é dele, sem multa por não
usar. Nada disso toca o resultado militar: frentes, logística e paz seguem a fase 5 para todos.

A crise é o lugar onde se negocia: a exigência pode ser atendida (Munique é atender), recusada ou
trocada. O cálculo de quem cede é o mesmo para todos: força relativa, aliados que vêm de fato,
custo da guerra para o orçamento e para a confiança, memória da última guerra (o peso de 1914–18
na França e no Reino Unido de 1938 é um piso de aversão à guerra que decai).

## IA estratégica: países que interpretam o papel

A IA de país (fase 6) escolhe, a cada mês ou ano, entre as mesmas ações do jogador: leis,
orçamento, banco central, tratados, tarifas, exigências, guerra. Ela pontua cada ação candidata
pela soma dos interesses ponderados pelos grupos no poder, como a postura. Três camadas:

1. **Interesse medido** (acima): o que o estado do país pede.
2. **Papel:** o que o lugar do país no sistema pede: o hegemon estabiliza (Kindleberger), a
   potência em ascensão revisa, a pequena se protege ou se alinha. O papel é calculado pela posição
   (ranking e tendência), não escrito por país.
3. **Agenda histórica:** as entradas históricas de que o país é protagonista, e as metas do chefe
   de governo em exercício. Exemplo: "Japão: zona econômica própria no continente" exige
   dependência de matéria-prima acima de X e militares com peso acima de Y; sai se a dependência
   cair (comércio aberto garantido) ou se os militares perderem o governo.

A agenda nunca escolhe a ação: soma ao valor das ações que servem a meta. Se as ações custam mais
do que valem, a IA não as toma. Não há estratégia sorteada, ao contrário do Victoria 3.

### Ordem de entrada das agendas

Decidido em 2026-11-16: como no Victoria 3, os países principais têm agenda e sabor histórico
primeiro; os demais recebem aos poucos.

| Onda | Países | Por quê |
|---|---|---|
| 1 | GER, ITA, JAP, SOV, USA, GBR, FRA, CHN | protagonistas dos fios 1 a 5 e 7; sem eles o placar de eventos dos anos 1930–50 não tem o que medir |
| 2 | RAJ, NLD, BEL, POR, e os países que a descolonização cria; TUR, POL, ESP | descolonização, Guerra Fria regional, as mudanças de regime mais citadas em `laws.ron` |
| 3 | BRA, ARG, MEX e a América Latina; Oriente Médio produtor de petróleo | crise da dívida de 1982, choques do petróleo |
| 4 | o resto, por PIB | quando o placar mostrar desvio que uma agenda explicaria |

**O mínimo de um país sem agenda:** interesses medidos e papel (calculados para todos, sem
escrever nada), participação nas entradas escritas por papel e não por tag (independência de uma
colônia, crise da dívida, golpe), personagens genéricos dos seus movimentos e o nome do regime
pela forma de governo. Joga-se com ele; só não tem roteiro próprio.

**Sabor:** nome histórico do regime e dos cargos, texto das entradas e dos personagens, em
`pt-BR` e `en`. Texto sem efeito: entra por onda junto com a agenda.

## Os grandes fios do século

Cada fio é uma entrada histórica. Para cada um: pré-condições; o que o move; o que o jogador pode
mudar e o que acontece com o resto da cadeia; os sistemas que exige; entrada e saída; forças e
esforço; e o veredito: **causa** (chega só pela causa medida), **agenda** (precisa da agenda
histórica da IA), **personagem** (o momento vem de um personagem com janela) ou **pressão**
(precisa da pressão histórica). Datas e números
com fonte na seção "Fontes dos fios".

### 1. A Depressão (1929–33)

- **Pré-condições:** euforia longa nos Estados Unidos, crédito privado alto, padrão-ouro em vigor
  nos grandes (o Reino Unido volta em 1925), dívida externa curta na Europa Central e reparações
  alemãs, preço de produtos primários em queda.
- **Move:** o ciclo de confiança (`confidence.pt-BR.md`): aperto do banco central sobre a euforia,
  pânico, corrida bancária sem emprestador, contágio pelo ouro e pela dívida.
- **O jogador muda:** emprestador de última instância, seguro de depósito, sair do ouro cedo, gasto
  público. **Resto da cadeia:** Depressão rasa tira o desemprego alemão de 1932, que é a principal
  causa do fio 2; menos protecionismo, menos interesse em autarquia (Japão, Alemanha, Itália).
- **Exige:** confiança e dívida privada (fase 7); regime monetário internacional (fase 4); tarifa
  no comércio (fase 4, a corrida protecionista de 1930–32).
- **Entrada e saída:** entra com euforia e fragilidade acima de limites em um dos grandes
  (janela 1927–1932); marcos: pânico no primeiro país, contágio a três dos dez maiores PIBs, fundo;
  sai pelo fundo ou se a fragilidade cair antes do pânico.
- **Forças e esforço:** fragilidade, padrão-ouro, dívida externa curta, corrida sem emprestador.
  Esforço **médio**: emprestador de última instância e seguro de depósito a tempo, ou sair do ouro
  em 1929–30, nos Estados Unidos; para quem joga um país pequeno, só sair do ouro cedo (atenua, não
  evita).
- **Veredito: causa**, sem pressão histórica. Decidido em 2026-11-16: a crise vem na janela sem o
  jogador; uma intervenção grande pode evitá-la, adiantá-la ou atrasá-la.

### 2. Nacionalismo expansionista na Alemanha, Itália e Japão

- **Pré-condições:** Alemanha: território perdido e minorias fora da fronteira (Versalhes),
  reparações em moeda estrangeira, memória da inflação de 1923 na classe média, desemprego de massa
  (perto de 6 milhões no início de 1932), democracia com poderes de emergência. Itália: já
  autoritária em 1925 (o cenário tem as leis), interesse em colônia e no Mediterrâneo. Japão:
  dependência de matéria-prima importada, interesse no continente, militares com autonomia diante
  do gabinete.
- **Move:** o movimento `revisionism` (meta `Revision`), que mede território perdido, minoria
  fora e desemprego nos grupos que recruta (classe média urbana, militares, proprietários); e o
  `regime_change` (legitimidade baixa com desemprego alto, quer partido único). Os dois estão no
  PR #53 da política; o `revisionism` fica parado até existir o indicador `lost_territory`. Quando
  passam do peso dos grupos que sustentam o regime, a lei `government` e as outras mudam. O
  governo novo pondera os interesses de outro jeito e a agenda de expansão sobe.
- **O jogador muda:** perdoar reparações (Hoover e Lausanne vieram tarde), evitar a Depressão,
  garantir mercado e matéria-prima ao Japão, ou, como Alemanha, governar de outro modo.
  **Resto da cadeia:** sem regime revisionista em Berlim, a guerra europeia perde a causa; a
  Itália e o Japão sozinhos fazem guerras regionais (Etiópia, China), não mundial.
- **Exige:** política 3d (PR #53: peso dos grupos, forma de governo como lei, os dois
  movimentos), o indicador `lost_territory` com as fronteiras de 1914, comunidades fora da
  fronteira (catálogo pronto), reparações como dívida (fase 4), personagens (fase 7).
- **Entrada e saída:** a entrada do exemplo acima (janela 1928–1938, marcos `revisionism` com
  apoio, `regime_change` vence, serviço militar universal, primeira exigência). Itália: entra já no
  marco de regime (o cenário de 1925 tem as leis); marcos seguintes, a colônia na África Oriental.
  Japão: entra com dependência de matéria-prima e militares com peso; marcos: ação no continente
  sem ordem do gabinete, governo militar.
- **Forças e esforço:** desemprego, legitimidade, reparações, minoria fora, pressão histórica e o
  agitador (Hitler, se surgir). Esforço **alto** como Alemanha (anos de política contra a Depressão
  e os grupos), **alto** para os outros (perdão de reparações, que pede os credores).
- **Veredito: causa + agenda + personagem.** A causa medida chega a "regime revisionista e
  autoritário na Alemanha nos anos 1930" se a Depressão vier; o personagem dá nome e soma margem,
  não cria a causa. A autonomia do exército japonês é dado do cenário. A perseguição sai das leis
  dirigidas que o catálogo já tem.

### 3. A Segunda Guerra Mundial

- **Pré-condições:** regimes revisionistas com agenda de expansão, rearmamento, aversão à guerra
  nas democracias (memória de 1914–18), Liga sem dentes, isolamento americano.
- **Move:** a escada de hostilidade: exigências (Renânia, Áustria, Sudetos, Polônia), cálculo de
  quem cede, crise, guerra; e no Pacífico, o embargo de matéria-prima ao Japão como incidente que
  dispara o interesse de matéria-prima.
- **O jogador muda:** garantias firmes cedo, rearmamento cedo, aliança com a URSS, embargo ou não.
  **Resto da cadeia:** guerra mais cedo e menor, ou nenhuma guerra mundial; muda a reconstrução, a
  descolonização (a metrópole não sai quebrada) e os blocos.
- **Exige:** fase 5 inteira (exército, frentes, mobilização, paz negociada), diplomacia (garantia
  e aliança chamando de fato), interesses e hostilidade (fase 7).
- **Entrada e saída:** abre quando o fio 2 chega ao marco de exigência (janela 1936–1942); marcos:
  exigências atendidas ou recusadas, guerra entre potências europeias, entrada dos Estados Unidos e
  da URSS; sai pela paz (fase 5) ou se o regime revisionista cair.
- **Forças e esforço:** a agenda revisionista, a aversão à guerra das democracias (que adia a
  resposta), o casus belli e a pressão popular. Esforço **médio** para os rivais (garantia firme e
  rearmamento cedo mudam o cálculo de quem cede), **alto** para a própria Alemanha (precisa de outro
  governo).
- **Veredito: agenda + pressão (casus belli).** A forma (o pacto com a URSS, o ataque à URSS, o
  ataque aos Estados Unidos) depende de poucas pessoas e do resultado militar: o placar mede a
  guerra (houve? quem contra quem? quando?), não a forma. O empurrão é o casus belli com pressão
  popular, decidido em 2026-11-16; nunca o resultado.

### 4. A reconstrução financiada pelos Estados Unidos

- **Pré-condições:** Europa destruída e sem dólar, Estados Unidos com poupança sobrando e
  indústria intacta, rival de regime (URSS) com movimento comunista forte na Europa Ocidental,
  memória de Versalhes (reparação punitiva levou à guerra).
- **Move:** o interesse do hegemon (papel de estabilizador): o valor da ajuda é o mercado de
  exportação recuperado mais a segurança do bloco (a ameaça do rival medida pelos movimentos
  comunistas nos países ajudados), contra o custo no orçamento. Se passa, a IA americana empresta
  ou doa, com condição (cooperação entre os ajudados, que alimenta o fio 11).
- **O jogador muda:** como Estados Unidos, não ajudar ou ajudar com outras condições; como
  europeu, recusar. **Resto da cadeia:** sem ajuda, a Europa cresce mais devagar, os movimentos
  comunistas crescem e o bloco ocidental é menor.
- **Exige:** diplomacia (ajuda, empréstimo entre governos, condição), blocos (fase 4), a IA de
  papel (fase 6), a memória de guerra (fase 5).
- **Entrada e saída:** entra com guerra terminada há menos de cinco anos, Europa com capacidade
  destruída e o hegemon com poupança sobrando (janela 1945–1952); marcos: oferta, adesão, quatro
  anos de transferência; sai se o hegemon não tiver rival de regime na Europa ou se a Europa já
  tiver se recuperado.
- **Forças e esforço:** mercado de exportação, ameaça medida dos movimentos comunistas, memória de
  Versalhes, pressão histórica. Esforço **baixo** como Estados Unidos (não oferecer), **médio** como
  europeu (recusar tem custo de crescimento).
- **Veredito: causa + agenda.** A entrada abre a ação "oferta de ajuda"; o tamanho (US$ 13,3
  bilhões em quatro anos, cerca de 5 % de um PIB americano) é alvo de calibração. A virada do
  isolamento dos anos 1930 para a liderança de 1947 sai da memória de guerra e da ameaça medida.

### 5. A revolução chinesa

- **Pré-condições:** campesinato sem terra e renda da terra alta (os indicadores
  `landowner_income_share` e `rural_wage_gap` já existem), Estado central fraco (a China de 1925 é
  um país só no cenário, `military_dictatorship`, mas na prática fragmentada), invasão estrangeira
  que enfraquece o governo, partido revolucionário organizado.
- **Move:** o movimento `agrarian_reform` radicalizado (fase 3, item 6 do que falta ligar:
  movimento forte e sem resposta vira insurgência), com território próprio quando o Estado não o
  controla; guerra civil (fase 5); a invasão japonesa como desgaste do governo.
- **O jogador muda:** reforma agrária pelo governo (tira a causa), evitar a guerra com o Japão,
  ajuda externa a um lado. **Resto da cadeia:** sem revolução, não há aliança sino-soviética nem
  reformas de 1978 (fio 9) no mesmo formato; a Guerra da Coreia perde um lado.
- **Exige:** insurgência com território (fase 5: um movimento vira país rebelde em guerra civil),
  fragmentação do Estado (senhores da guerra como autonomia regional), política 3d.
- **Entrada e saída:** entra com renda da terra alta, Estado sem controle de parte do território e
  partido revolucionário organizado (janela 1927–1955); marcos: insurgência com território, base
  rural, guerra civil, governo revolucionário; sai por reforma agrária feita pelo governo ou pela
  derrota da insurgência.
- **Forças e esforço:** `landowner_income_share`, `rural_wage_gap`, a guerra com o Japão, a agenda
  soviética, o agitador (Mao, se surgir), pressão histórica. Esforço **alto** como China (reforma
  agrária contra os proprietários que sustentam o governo), **médio** como Japão (não invadir).
- **Veredito: causa + agenda + personagem**, com risco. Quem vence a guerra civil depende do
  resultado militar; a pressão só entra na moral e na deserção do exército do governo, nunca no
  resultado da batalha.

### 6. A descolonização

- **Pré-condições:** discriminação e falta de autogoverno nas colônias, alfabetização e
  urbanização subindo (elite colonial educada), metrópole enfraquecida (dívida, exército gasto na
  guerra), superpotências contra o império formal, a ONU.
- **Move:** o movimento `independence`, já no catálogo, que mede discriminação e autogoverno; o
  custo de manter (guarnição, revolta) contra o valor da colônia (interesse "colônias"); a pressão
  externa como incidente.
- **O jogador muda:** como metrópole, autogoverno cedo (domínio), repressão, guerra colonial. **Resto
  da cadeia:** a ordem e a violência da descolonização mudam (guerra longa na Argélia, ou saída
  negociada); os países novos entram nos blocos (fio 7) por interesse.
- **Exige:** movimentos ligados (fase 3), insurgência (fase 5), organizações (fase 4), país novo
  nascendo de colônia (separação de mercado e de moeda).
- **Entrada e saída:** uma entrada por papel (metrópole e colônia), não por tag: entra com o
  movimento `independence` acima de um limite; marcos: autogoverno, independência negociada ou
  guerra colonial; sai pela independência ou pelo fim da causa (igualdade de direitos de fato).
- **Forças e esforço:** discriminação, alfabetização, metrópole enfraquecida, pressão externa.
  Esforço **alto** para manter colônias depois de 1945 (guarnição, guerra colonial, legitimidade),
  **baixo** para soltar cedo.
- **Veredito: causa.** É o fio que mais se parece com o que a política já foi desenhada para fazer.
  A onda de 1960 (17 países africanos num ano) é alvo do placar.

### 7. A Guerra Fria e os blocos

- **Pré-condições:** duas superpotências com regimes opostos (a distância de leis é máxima entre
  economia planejada e mercado), o resto do mundo fraco depois da guerra, arma nuclear.
- **Move:** interesse de regime e de segurança: cada país pontua os dois blocos pela distância de
  leis dos grupos no poder, pela ameaça e pelo que cada bloco oferece (ajuda, mercado, proteção).
  Os blocos são organizações da fase 4 (aliança militar, área econômica). A dissuasão nuclear
  (fase 5) tira a guerra direta da escada entre os dois e empurra a rivalidade para guerras por
  procuração (apoio a movimentos e insurgências do outro lado).
- **O jogador muda:** alinhamento, neutralidade, troca de lado (um governo novo pondera de outro
  modo). **Resto da cadeia:** um bloco ocidental sem a Alemanha, ou um bloco soviético sem a
  China, mudam a corrida e os fios 9 e 10.
- **Exige:** blocos (fase 4), arma nuclear com consequência (fase 5), interesse de regime (fase 7).
- **Entrada e saída:** entra com duas potências de regime oposto no topo do ranking de força e o
  resto fraco (janela 1945–1950); marcos: blocos militares, corrida nuclear, primeira guerra por
  procuração; sai quando uma das duas deixa de ter regime oposto ou cai no ranking.
- **Forças e esforço:** distância de regime, ameaça, oferta de cada bloco, agendas soviética e
  americana. Esforço **médio** para um país médio ficar neutro (perde ajuda), **alto** para mudar o
  regime de uma superpotência.
- **Veredito: causa + agenda.** A bipolaridade sai da distância de regime e da força; a agenda da
  URSS (zona de segurança no Leste Europeu) e a dos Estados Unidos (contenção) entram com
  pré-condição (ter sido invadido pelo Ocidente; o rival estar se expandindo).

### 8. Os choques do petróleo (1973 e 1979)

- **Pré-condições:** petróleo do Oriente Médio descoberto (as jazidas já existem com ano de
  descoberta, `deposits.ron`), dependência dos importadores, produtores organizados em cartel,
  conflito árabe-israelense e apoio americano a Israel; em 1979, revolução no Irã.
- **Move:** interesse dos produtores (receita) e a margem de cartel quando a oferta está
  concentrada e a demanda é pouco sensível a preço: cortar produção paga. O embargo é a ação do
  cartel contra quem apoia o inimigo de um membro (incidente na escada de hostilidade). O preço é o
  mercado mundial que já existe. A recessão dos importadores sai da confiança.
- **O jogador muda:** reservas estratégicas, troca de fonte (nuclear, carvão), não apoiar um lado.
  **Resto da cadeia:** sem os choques, menos dívida em dólar emprestada à América Latina com o
  dinheiro dos produtores, e a crise de 1982 fica menor.
- **Exige:** cartel como organização com cota (fase 4), estoques e reservas (fase 4), guerra
  regional (fase 5), movimento religioso que derruba um regime (fase 3).
- **Entrada e saída:** entra com o cartel formado e a parcela dele na oferta acima de um limite
  (janela 1970–1982); marcos: guerra regional com apoio de um importador grande ao inimigo de um
  membro, embargo (o casus belli aqui é econômico: direito de embargar sem custo de hostilidade
  com terceiros), preço; sai se a parcela do cartel cair.
- **Forças e esforço:** concentração da oferta, demanda pouco sensível, conflito regional,
  pressão histórica. Esforço **médio** para o importador (reservas, troca de fonte, não apoiar um
  lado), **baixo** para um membro do cartel (furar a cota).
- **Veredito: agenda + pressão.** A data depende de uma guerra e de uma revolução; sem elas, o
  choque vem quando a condição de cartel e uma crise regional coincidirem.

### 9. As reformas chinesas de 1978

- **Pré-condições:** economia planejada estagnada, distância de renda para os vizinhos (Japão,
  Taiwan, Hong Kong, Coreia do Sul), fim da geração revolucionária no poder.
- **Move:** o movimento `market_reform`, já no catálogo, medindo a distância de renda para os
  vizinhos e as necessidades não atendidas; a mudança da lei econômica quando ele passa do peso dos
  grupos que sustentam o planejamento.
- **O jogador muda:** como China, reformar antes ou nunca; como Estados Unidos, abrir ou fechar
  mercado e tecnologia (licença, embargo, `research.pt-BR.md`). **Resto da cadeia:** a China que
  reforma cedo vira o motor da industrialização mundial antes; a que não reforma fica como a URSS.
- **Exige:** política 3d, licença e embargo de tecnologia (fase 2 já tem), zona econômica especial
  (lei regional; não existe ainda).
- **Entrada e saída:** entra com economia planejada e distância de renda para os vizinhos acima de
  um limite (janela 1970–1990); marcos: lei agrária de mercado, zona econômica especial, capital
  estrangeiro; sai se a distância fechar sem reforma ou se o regime cair.
- **Forças e esforço:** distância de renda, necessidades não atendidas, o chefe de governo (a morte
  do fundador abre espaço; Deng, se surgir, empurra `market_reform`), pressão histórica fraca.
  Esforço **médio** como China (manter o plano custa legitimidade), **baixo** para os Estados
  Unidos fecharem mercado e tecnologia.
- **Veredito: causa + personagem.** A causa medida existe; os personagens dão o momento dentro da
  janela.

### 10. O colapso soviético

- **Pré-condições:** produtividade parada na economia planejada (difusão de tecnologia lenta, fase
  2), receita dependente do petróleo (o preço cai de cerca de US$ 27 a menos de US$ 10 em 1986),
  gasto militar alto, movimentos de independência nas repúblicas (comunidades com concentração
  territorial), mercado sem bens.
- **Move:** crise fiscal (a 3c já faz calote e inflação), movimentos `independence` e
  `market_reform`, perda do peso do partido e do exército.
- **O jogador muda:** reformar cedo, cortar gasto militar, federalizar. **Resto da cadeia:**
  sem colapso, a Europa Oriental fica no bloco, não há expansão da OTAN nem a integração europeia
  ao leste.
- **Exige:** tudo de política e de comunidades, guerra fria (fio 7), petróleo (fio 8), separação
  pacífica de país (independência sem guerra).
- **Entrada e saída:** entra com produtividade parada, receita dependente do petróleo e movimentos
  de independência nas repúblicas (janela 1980–1995); marcos: reforma, crise fiscal, independência
  das repúblicas; sai se a produtividade voltar a crescer ou as nacionalidades perderem a causa.
- **Forças e esforço:** estagnação, preço do petróleo, gasto militar, nacionalidades. Esforço
  **alto** como URSS (reformar cedo e cortar o gasto militar contra os militares e o partido).
- **Veredito: causa.** Estagnação, petróleo e nacionalidades são números; a forma (dissolução
  negociada em dezembro de 1991) é alvo, não regra. Um chefe de governo reformista (Gorbachev, se
  surgir) muda o momento, não a causa.

### 11. A integração europeia

- **Pré-condições:** guerra recente entre França e Alemanha (hostilidade que se quer desarmar),
  ganho de comércio entre vizinhos ricos, ameaça comum (URSS), ajuda americana condicionada à
  cooperação.
- **Move:** interesse de mercado e de segurança: o valor de uma união aduaneira e depois de uma
  moeda comum, pela IA, contra a perda de tarifa e de política monetária própria.
- **O jogador muda:** ficar de fora, entrar cedo, bloquear. **Resto da cadeia:** sem integração, a
  rivalidade franco-alemã volta a pesar; sem a moeda comum, a crise de 2010 não tem a forma do euro.
- **Exige:** união aduaneira e moeda comum (fase 4: o contrato já tem `CustomsUnion` e o regime
  monetário), memória de guerra (fase 5).
- **Entrada e saída:** entra com guerra franco-alemã recente, ameaça comum e ajuda condicionada
  (janela 1948–1960); marcos: comunidade setorial, união aduaneira, mercado único, moeda comum; sai
  se a hostilidade franco-alemã voltar a passar de um limite.
- **Forças e esforço:** ganho de comércio, segurança, memória de guerra, agendas da França e da
  Alemanha. Esforço **baixo** para ficar de fora (perde comércio), **alto** para bloquear os outros.
- **Veredito: causa + agenda.** A moeda comum de 1999 é decisão política com custo visível, que a
  agenda da França e da Alemanha carrega.

### Resumo

| Fio | Veredito | Esforço para desviar | Fases que exige | O que falta hoje |
|---|---|---|---|---|
| Depressão | causa | médio | 4, 7 | dívida privada, confiança, padrão-ouro, tarifa |
| Nacionalismo expansionista | causa + agenda + personagem | alto | 3, 4, 7 | indicador `lost_territory` (fronteiras de 1914), reparações, interesses |
| Segunda Guerra | agenda + pressão (casus belli) | médio a alto | 4, 5, 6, 7 | exército, frentes, escada de hostilidade |
| Reconstrução americana | causa + agenda | baixo a médio | 4, 5, 6 | ajuda entre governos, blocos, papel de hegemon |
| Revolução chinesa | causa + agenda + personagem | alto | 3, 5, 6 | insurgência com território, guerra civil, fragmentação |
| Descolonização | causa | alto para manter | 3, 4, 5 | movimentos ligados, país novo |
| Guerra Fria | causa + agenda | médio a alto | 4, 5, 6 | blocos, arma nuclear, guerra por procuração |
| Choques do petróleo | agenda + pressão | médio | 4, 5 | cartel, reservas, guerra regional |
| Reformas chinesas | causa + personagem | médio | 3 | `market_reform` ligado, zona econômica |
| Colapso soviético | causa | alto | 3, 4, 5 | separação pacífica, nacionalidades |
| Integração europeia | causa + agenda | baixo a alto | 4, 5 | união aduaneira e moeda comum ligadas |

Onde a causa não basta, o motivo é quase sempre o mesmo: **pessoas e batalhas**. Os personagens
cobrem as pessoas dentro de uma janela; as batalhas ficam com a fase 5, sem pressão nenhuma no
resultado.

## Como medir "sem tocar em nada, a história acontece"

### O placar de eventos dentro do placar histórico

O placar histórico já existe (`tools/history/`, `docs/history.pt-BR.md`) e já fixou o formato de
entrada de eventos (`target/history/events.json`: `id`, `country`, `question`,
`reference_date`, `simulated_date`, `source`). A política (PR #53) já grava as 132 mudanças de lei
de `laws.ron` como perguntas com janela de ± 5 anos. As entradas históricas usam o mesmo formato:
**cada marco de cada entrada é um evento**, com `id` `<tag>-<entrada>-<marco>`, a data de
referência e a fonte.

```json
{ "id": "GER-revisionist-rise-regime", "country": "GER",
  "question": "Regime de partido único revisionista",
  "reference_date": "1933-07-14", "simulated_date": "1934-02-01",
  "source": "Law against the Formation of New Parties of 14 July 1933" }
```

Além do acerto e do atraso que o placar já conta, a entrada acrescenta duas colunas por marco:
quanto da força vinha da pressão histórica no mês em que o marco veio, e se o protagonista era o
certo (exigido nos fios 1, 2, 4, 5, 9 e 10, decidido em 2026-11-16).

O primeiro placar agrupou os desvios em seis causas; **"sem ciclo nem guerra" é a terceira, com
peso 0,59**: na Depressão (1929–33) o Ocidente de colonização europeia cresce +5,5 % ao ano
simulado contra −9,1 % real, e na guerra (1939–45) a Europa Ocidental cresce +2,4 % contra −3,3 %.
Este bloco é o que ataca essa causa: a confiança (Depressão), a guerra com casus belli e a
reconstrução. "Sem efeito de regime" (0,26: a URSS e o Leste Europeu de 1973–91) é o alvo do fio 10.
Meta do bloco: tirar "sem ciclo nem guerra" das três primeiras causas, sem piorar as outras.

### O teste do desvio

Rodadas contrafactuais curtas, com uma condição mudada por comando no início, e a pergunta: a
cadeia mudou, e os termos explicam por quê?

| Mudança | O que tem de acontecer | O que a tela tem de mostrar |
|---|---|---|
| Os Estados Unidos com seguro de depósito e emprestador desde 1925 | Depressão rasa; desemprego alemão de 1932 menor; regime revisionista tarde ou nunca | termo de corrida bancária ausente; margem do revisionismo menor no termo "desemprego" |
| Reparações alemãs perdoadas em 1925 | revisionismo mais fraco | termo "dívida" do interesse alemão zerado |
| Reforma agrária na China em 1930 | movimento agrário sem causa; revolução tarde ou nunca | causa do movimento abaixo do limite |
| Japão com acesso garantido a petróleo | agenda continental abandonada | condição de abandono atingida |
| Sem ajuda americana em 1948 | Europa Ocidental cresce menos, movimentos comunistas maiores | termos na política dos países europeus |

Cada teste fica no CI como regra (não como número): a direção da mudança e o termo que a explica.
Um teste a mais mede a pressão: a mesma rodada com `historical_pressure = 0` mostra quanto de
cada marco vem da causa medida; um marco que só vem com a pressão é candidato a causa nova.

## Comparação de abordagem com o Victoria 3 e o Cold War Era

Só sistema. Nada copiado; o CWE não tem licença (`benchmarks.pt-BR.md`).

| | Victoria 3 | Cold War Era (CWE) | Nossa proposta |
|---|---|---|---|
| Unidade da história | *journal entry*: objeto com condição de aparecer, de completar e de falhar, prazo e efeitos; dentro dele, eventos sorteados com peso | os mesmos *journal entries*, usados como **sistemas** (Bretton Woods, FMI, OTAN, Pacto de Varsóvia, cartel do petróleo, descolonização, guerra civil, tensão mundial, perspectiva econômica) | sistemas de verdade na simulação; o placar de eventos só mede, não age |
| Guerra | não se declara: toda guerra começa por uma jogada diplomática com fases de manobra, terceiros e recuo | herda | escada de hostilidade com exigência, terceiros e recuo, pela conta de interesses |
| IA | três estratégias por país (administrativa, política, diplomática), sorteadas com peso e trocadas por acúmulo de mudanças | estratégias próprias | interesse medido + papel + agenda histórica, sem sorteio |
| História | pesos nos arquivos de cenário, sem modo histórico; agitadores históricos que surgem sob condição e não são garantidos | começa em 1950 (com submod de 1946): o pós-guerra já está dado | entradas com marcos, causa medida, agenda, personagens com janela e pressão histórica visível |
| Ciclo econômico | não há | "Economic Outlook": acompanha o crescimento do PIB em meio ano e aplica faixas de efeito | confiança por classe de agente com Minsky, Fisher e corrida bancária (`confidence.pt-BR.md`) |

O Hearts of Iron IV tem a chave por país (IA histórica, alternativa ou sorteada); lá o foco
histórico é uma sequência fixa de passos. Aqui a agenda pesa e cai com a condição, e não há chave
por país (seção "Regras de partida").

O Victoria 2 usava eventos com tempo médio para acontecer e decisões históricas com requisitos;
daqui só vem a ideia do marco com condição, não o sorteio mensal.

## Fontes dos fios

Datas e números conferidos; Wikipedia em inglês usada só para fatos, como em `diplomacy.pt-BR.md`.

- NSDAP: 2,6 % em maio de 1928, 18,3 % em setembro de 1930, 37,3 % em julho de 1932 (artigos de cada
  eleição federal alemã); Hitler chanceler em 30 de janeiro de 1933; desemprego alemão de 5,3
  milhões em média em 1932 e perto de 6 milhões no pico ("Great Depression in Germany").
- Reparações: Plano Dawes de 1924, Plano Young ratificado em 1930, moratória Hoover proposta em
  20 de junho de 1931, Lausanne em 1932 ("World War I reparations", "Hoover Moratorium").
- Agressões: Mukden em 18 de setembro de 1931, ponte Marco Polo em 7 de julho de 1937, invasão da
  Polônia em 1º de setembro de 1939.
- Plano Marshall: assinado em 3 de abril de 1948, US$ 13,3 bilhões para 17 países; o PIB
  americano de 1948 era de US$ 258 bilhões ("Marshall Plan"). A razão de cerca de 5 % em quatro
  anos é conta nossa.
- China: Expedição do Norte de 1926 a 1928, massacre de Xangai em abril de 1927, Longa Marcha de
  1934–35, guerra civil em escala total a partir de junho de 1946, República Popular em 1º de
  outubro de 1949 ("Northern Expedition", "Long March", "Chinese Civil War").
- Descolonização: Indonésia soberana em 27 de dezembro de 1949; 17 países africanos independentes
  em 1960; resolução 1514 da ONU em 14 de dezembro de 1960, 89 a 0 com 9 abstenções ("Year of
  Africa", "United Nations General Assembly Resolution 1514").
- Guerra Fria: Comecon em janeiro de 1949; OTAN em 4 de abril de 1949.
- Petróleo: de US$ 3,01 a US$ 11,65 o barril entre outubro e dezembro de 1973; de US$ 15,85 em abril
  de 1979 a um pico de US$ 39,50; queda de cerca de US$ 27 para menos de US$ 10 em 1986 ("1973 oil
  crisis", "1979 oil crisis", "1980s oil glut").
- China: terceiro pleno do 11º Comitê Central de 18 a 22 de dezembro de 1978; zona econômica
  especial de Shenzhen em agosto de 1980.
- Bretton Woods: fim da conversibilidade do dólar em 15 de agosto de 1971 ("Nixon shock").
- Victoria 3: wiki do jogo (páginas *Journal*, *Diplomatic play*, *Interest*); diários de
  desenvolvimento 59 (IA), 71 (regra de investimento autônomo), 82 e 83 (agitadores; a resposta
  dos desenvolvedores de que os históricos surgem sob condição e não são garantidos); Hearts of Iron IV: wiki, página *Game rules*; Victoria 2: wiki, páginas
  *Event modding* e *Decisions*. CWE: nomes dos arquivos em `common/journal_entries` do
  repositório, para dizer quais sistemas existem; nenhum conteúdo lido para uso.

## Decisões do dono

Decididas em 2026-11-16:

1. ~~Condução~~: mistura, como o Victoria 3; entradas com condição de entrada e de saída, marcos, e
   esforço para ir contra as forças.
2. ~~Personagens~~: sim, com janela de datas, condição medida e sorteio determinístico.
3. ~~Protagonista no placar~~: exigido nos fios 1, 2, 4, 5, 9 e 10; equivalente aceito nos outros.
4. ~~Empurrão na guerra~~: casus belli com pressão popular para a IA; o jogador recebe só o casus
   belli; nunca no resultado.
5. ~~Movimentos `revisionism` e `regime_change`~~: no catálogo (já no PR #53 da política).
6. ~~Agendas~~: por ondas, os principais primeiro (tabela "Ordem de entrada das agendas").

Aberta:

7. **Regras de partida.** Proposta: um jogo, um caminho de código, o padrão calibrado e coberto
   pelo placar; no máximo duas regras escalares em dados (`historical_pressure`,
   `historical_characters`) com só fumaça e determinismo no CI, sem promessa histórica.
   Alternativa: nenhuma regra, a pressão desligada só como parâmetro de teste. Recomendação: as
   duas regras escalares, se o dono quiser a tela de opções; senão, nenhuma.
