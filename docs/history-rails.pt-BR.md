# Trilho histórico, interesses, hostilidade e IA estratégica

Estado: proposta de desenho, **sem código**. É o último bloco do plano (fase 7, ver
`architecture.pt-BR.md`): depende de política (fase 3), diplomacia (4), guerra (5) e da confiança
(`confidence.pt-BR.md`).

## O pedido e a tensão

O pedido: sem o jogador tocar em nada, a história real acontece (a Alemanha produz um partido
nacionalista que quer expansão, os Estados Unidos financiam a reconstrução da Europa, a China tem a
revolução); quando o jogador começa a mexer, tudo vai desviando.

Os princípios do repositório dizem: pouco sorteio, causa visível, determinismo, nenhuma cadeia de
eventos datados. As duas coisas só cabem juntas se os grandes eventos nascerem de **condições e
interesses** medidos e calibrados, não de data. Este documento propõe esse mecanismo e diz, fio a
fio, onde ele não basta.

## As opções

| | Como funciona | Prós | Contras |
|---|---|---|---|
| **A. Roteiro datado** | Evento dispara na data, com condição mínima (Victoria 2, Hearts of Iron IV com foco histórico) | A história acontece sempre, na data | Quebra causa visível; o desvio vira "o evento não disparou" em vez de "a causa sumiu"; o jogador aprende a esperar a data |
| **B. Condições com pesos históricos** | Nenhum evento; os sistemas (confiança, movimentos, interesses, IA) têm parâmetros calibrados para que o mundo de 1925 produza, sozinho, o século | Princípios intactos; todo desvio é explicável | Não garante nada: ano, protagonista e forma podem sair outros; fios que dependem de pessoas (Hitler, Mao, Deng, Gorbachev) e de acaso militar ficam fracos |
| **C. Agenda histórica da IA** | Cada país da IA tem metas declaradas (as que teve na história), com pré-condições e condições de abandono; a IA as persegue com as ações legais que todos têm. Nada dispara: a agenda pesa na escolha | Conduz os atores sem conduzir o mundo; a agenda é visível ("a Alemanha quer rever Versalhes"); se as condições somem, a meta cai | Escrita à mão por país; risco de virar roteiro disfarçado se as metas forem muito específicas |
| **D. Mistura (recomendada)** | B para tudo o que é estrutura (crise, movimentos, descolonização, blocos); C para as escolhas dos atores (expansão, ajuda, revolução organizada); um **empurrão histórico** explícito, visível e que se desfaz com o desvio, só onde B e C não bastam | Cada fio usa o mínimo de condução que precisa; o desvio é medido | Mais peças; a calibração é longa; exige o placar de eventos para saber se funciona |

**Recomendação: D**, com o empurrão como regra de jogo que o jogador pode ver e desligar.

### O empurrão histórico

Quando B e C não chegam ao evento, entra um termo nomeado, **pressão histórica**, somado ao mesmo
cálculo que a causa medida já faz (a margem de um movimento, a pontuação de uma meta da IA):

```text
pressão = peso do fio × proximidade da história
proximidade = 1 − distância(estado atual, pré-condições históricas do fio) / limite
```

- **Só soma, nunca cria.** A pressão aumenta a margem de uma condição que já existe; um movimento
  sem causa (zero) continua zero. A Alemanha sem desemprego, sem reparações e sem território
  perdido não ganha partido expansionista por pressão histórica.
- **Desfaz-se com o desvio.** A distância mede as pré-condições do fio no mundo simulado contra as
  da história (desemprego, dívida externa, território perdido, regime de quem decide). Mexa nelas e
  a proximidade cai até zero; a pressão some sem que nada seja desligado à mão.
- **Visível.** A tela da causa mostra o termo pelo nome: "pressão histórica +120 (o mundo está a
  18 % da Alemanha de 1932)". Nada escondido.
- **Desligável.** Opção de partida: *histórico* (peso cheio), *plausível* (metade), *livre* (zero).
  O placar de eventos mede as três.
- **Determinística.** Sem sorteio, como o resto.

O peso de cada fio é decisão de calibração: começa em zero e só sobe onde o placar mostrar que a
causa medida não chega.

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
| Crise | exigência com prazo (o que quer e o que oferece); terceiros escolhem lado | mobilização; confiança dos mercados cai (contágio pelo medo) |
| Guerra | só se o valor esperado do que se exige passa do custo esperado, para quem decide | fase 5 |

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
3. **Agenda histórica** (opção C): metas escritas à mão para os países que fizeram o século, cada
   uma com pré-condições, condição de abandono e o peso que soma à pontuação. Exemplo: "Japão: zona
   econômica própria no continente" exige dependência de matéria-prima acima de X e militares com
   peso acima de Y; abandona se a dependência cair (comércio aberto garantido) ou se os militares
   perderem o governo.

A agenda nunca escolhe a ação: soma ao valor das ações que servem a meta. Se as ações custam mais
do que valem, a IA não as toma.

## Os grandes fios do século

Para cada fio: pré-condições; o que o move; o que o jogador pode mudar e o que acontece com o resto
da cadeia; os sistemas que exige; e o veredito: **causa** (chega só pela causa medida), **agenda**
(precisa da agenda histórica da IA) ou **empurrão** (precisa da pressão histórica). Datas e números
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
- **Veredito: causa.** O ano exato não é garantido (janela de 1928 a 1931).

### 2. Nacionalismo expansionista na Alemanha, Itália e Japão

- **Pré-condições:** Alemanha: território perdido e minorias fora da fronteira (Versalhes),
  reparações em moeda estrangeira, memória da inflação de 1923 na classe média, desemprego de massa
  (perto de 6 milhões no início de 1932), democracia com poderes de emergência. Itália: já
  autoritária em 1925 (o cenário tem as leis), interesse em colônia e no Mediterrâneo. Japão:
  dependência de matéria-prima importada, interesse no continente, militares com autonomia diante
  do gabinete.
- **Move:** um movimento novo, **revisionismo** (ou nacionalismo expansionista), que mede
  território perdido, minoria fora e desemprego nos grupos que recruta (classe média urbana,
  militares, proprietários); e um movimento de **troca de regime** (autoritário), que mede o
  bloqueio do governo (leis travadas, crise sem resposta). Quando os dois juntos passam do peso
  dos grupos que sustentam o regime, a forma de governo e as leis mudam. O governo novo pondera os
  interesses de outro jeito e a agenda de expansão sobe.
- **O jogador muda:** perdoar reparações (Hoover e Lausanne vieram tarde), evitar a Depressão,
  garantir mercado e matéria-prima ao Japão, ou, como Alemanha, governar de outro modo.
  **Resto da cadeia:** sem regime revisionista em Berlim, a guerra europeia perde a causa; a
  Itália e o Japão sozinhos fazem guerras regionais (Etiópia, China), não mundial.
- **Exige:** política 3d ligada (peso dos grupos, mudança de lei e de forma de governo), dois
  movimentos novos (o catálogo de 19 não tem revisionismo nem troca de regime; tem `nativism`, que
  é outro), comunidades fora da fronteira (catálogo pronto), reparações como dívida (fase 4).
- **Veredito: causa + agenda.** A causa medida chega a "regime revisionista e autoritário na
  Alemanha nos anos 1930" se a Depressão vier. Que o vencedor seja a direita e não o partido
  comunista (que também cresceu) depende de quais grupos pesam: dá para calibrar sem empurrão. A
  autonomia do exército japonês é dado do cenário (como o governo), não empurrão. O nome, a pessoa
  e o genocídio não vêm de condição: o jogo não tem personagens; a perseguição sai das leis
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
- **Veredito: agenda + empurrão.** A causa medida leva a "uma guerra de grande porte na Europa no
  fim dos anos 1930 com a Alemanha agressora" se o fio 2 vier. A forma (o pacto com a URSS, o
  ataque à URSS, o ataque aos Estados Unidos) depende de decisões de poucas pessoas e do resultado
  militar; nenhuma regra sem sorteio reproduz isso por causa. Recomendação: o placar mede a guerra
  (houve? quem contra quem? quando?) e não a forma; o empurrão só na decisão de atacar, e só se o
  placar pedir.

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
- **Veredito: causa + agenda.** O interesse medido explica a ajuda; o tamanho (US$ 13,3 bilhões em
  quatro anos, cerca de 5 % de um PIB americano) é alvo de calibração. A virada do isolamento dos
  anos 1930 para a liderança de 1947 sai da memória de guerra e da ameaça medida, sem data.

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
- **Veredito: causa + agenda**, com risco. A causa agrária é forte e medida. Quem vence a guerra
  civil depende do resultado militar e da ajuda externa: agenda da URSS (apoio a revolucionários)
  e dos Estados Unidos (apoio ao governo). Se o placar não chegar à vitória revolucionária perto de
  1949, empurrão na moral e na deserção do exército do governo, nunca no resultado da batalha.

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
- **Veredito: causa + agenda.** A bipolaridade sai da distância de regime e da força; a agenda da
  URSS (zona de segurança no Leste Europeu) e a dos Estados Unidos (contenção) entram como agenda
  histórica com pré-condição (ter sido invadido pelo Ocidente; o rival estar se expandindo).

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
- **Veredito: agenda + empurrão.** O cartel sai do interesse, mas a data depende da guerra de 1973
  e da revolução de 1979, que são eventos de poucos atores. Sem empurrão, aceita-se que o choque
  venha quando a condição de cartel e uma crise regional coincidirem.

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
- **Veredito: causa + empurrão fraco.** A causa medida existe (a distância de renda é grande nos
  anos 1970). O ano depende da morte de Mao, e o jogo não tem pessoas: sem empurrão, a reforma vem
  quando a margem passar, com janela larga (proposta: 1970 a 1990).

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
- **Veredito: causa.** É o fio em que a causa medida mais se sustenta: estagnação, petróleo e
  nacionalidades são números. A forma (dissolução negociada em dezembro de 1991) é alvo, não regra.

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
- **Veredito: causa + agenda.** O comércio e a segurança explicam a união; a moeda comum de 1999 é
  decisão política com custo visível, que a agenda da França e da Alemanha pode carregar.

### Resumo

| Fio | Veredito | Fases que exige | O que falta hoje |
|---|---|---|---|
| Depressão | causa | 4, 7 | dívida privada, confiança, padrão-ouro, tarifa |
| Nacionalismo expansionista | causa + agenda | 3, 4, 7 | peso político, mudança de regime, movimentos de revisionismo e troca de regime, interesses |
| Segunda Guerra | agenda + empurrão | 4, 5, 6, 7 | exército, frentes, escada de hostilidade |
| Reconstrução americana | causa + agenda | 4, 5, 6 | ajuda entre governos, blocos, papel de hegemon |
| Revolução chinesa | causa + agenda (empurrão se preciso) | 3, 5, 6 | insurgência com território, guerra civil, fragmentação |
| Descolonização | causa | 3, 4, 5 | movimentos ligados, país novo |
| Guerra Fria | causa + agenda | 4, 5, 6 | blocos, arma nuclear, guerra por procuração |
| Choques do petróleo | agenda + empurrão | 4, 5 | cartel, reservas, guerra regional |
| Reformas chinesas | causa + empurrão fraco | 3 | ligar `market_reform`, zona econômica |
| Colapso soviético | causa | 3, 4, 5 | separação pacífica, nacionalidades |
| Integração europeia | causa + agenda | 4, 5 | união aduaneira e moeda comum ligadas |

Onde a causa não basta, o motivo é quase sempre o mesmo: **pessoas e batalhas**. O jogo não tem
personagens e a guerra será determinística; a ascensão de um líder específico e o resultado de uma
campanha não têm causa medida que os reproduza. A proposta é medir o que é estrutura e aceitar a
janela no que é decisão de poucos.

## Como medir "sem tocar em nada, a história acontece"

### O placar de eventos

Ao lado do placar de números (`tools/history/`), uma lista de **eventos como predicados sobre o
estado**, avaliados todo ano na rodada sem jogador. Um evento não dispara nada: é uma pergunta.

```text
evento: regime_revisionista_alemanha
  quando: forma de governo autoritária E movimento revisionismo no poder
  história: 1933 (janela 1930–1936)
  com quem: GER
evento: guerra_europeia_geral
  quando: guerra com 3+ dos 6 maiores PIBs europeus, um deles agressor revisionista
  história: 1939 (janela 1936–1942)
  com quem: agressor GER; lados
```

Para cada evento o placar responde: aconteceu? quando (distância à janela)? com quem (os
protagonistas certos)? A nota da rodada soma os acertos com peso por fio. Os eventos e as janelas
ficam num arquivo de dados com fonte, como as 132 mudanças de lei de `laws.ron`, que viram eventos
do placar sem mudança (a lei X mudou no país Y perto da data Z?).

Rodar as três opções de partida (histórico, plausível, livre) e publicar as três notas: a
diferença entre *livre* e *histórico* é o quanto o empurrão está fazendo. Meta: *livre* acertar os
fios marcados "causa"; *histórico* acertar todos dentro da janela.

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

## Comparação de abordagem com o Victoria 3 e o Cold War Era

Só sistema. Nada copiado; o CWE não tem licença (`benchmarks.pt-BR.md`).

| | Victoria 3 | Cold War Era (CWE) | Nossa proposta |
|---|---|---|---|
| Unidade da história | *journal entry*: objeto com condição de aparecer, de completar e de falhar, prazo e efeitos; dentro dele, eventos sorteados com peso | os mesmos *journal entries*, usados como **sistemas** (Bretton Woods, FMI, OTAN, Pacto de Varsóvia, cartel do petróleo, descolonização, guerra civil, tensão mundial, perspectiva econômica) | sistemas de verdade na simulação; o placar de eventos só mede, não age |
| Guerra | não se declara: toda guerra começa por uma jogada diplomática com fases de manobra, terceiros e recuo | herda | escada de hostilidade com exigência, terceiros e recuo, pela conta de interesses |
| IA | três estratégias por país (administrativa, política, diplomática), sorteadas com peso e trocadas por acúmulo de mudanças | estratégias próprias | interesse medido + papel + agenda histórica, sem sorteio |
| História | pesos nos arquivos de cenário, sem IA histórica separada: um empurrão probabilístico | começa em 1950 (com submod de 1946): o pós-guerra já está dado | causa medida, agenda e pressão histórica visível e desligável |
| Ciclo econômico | não há | "Economic Outlook": acompanha o crescimento do PIB em meio ano e aplica faixas de efeito | confiança por classe de agente com Minsky, Fisher e corrida bancária (`confidence.pt-BR.md`) |

O Hearts of Iron IV é a comparação mais próxima da opção C: a regra de partida "comportamento da
IA" deixa cada país seguir o foco histórico, um não histórico ou um sorteado. O que tiramos dele é
a ideia de a condução ser uma **opção do jogador**, não o mecanismo (lá o foco histórico é uma
sequência fixa de passos; aqui a agenda pesa e cai com a condição).

O Victoria 2 usava eventos com tempo médio para acontecer e decisões históricas com requisitos: a
opção A, que os princípios recusam.

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
- Victoria 3: wiki do jogo (páginas *Journal*, *Diplomatic play*, *Interest*) e diário de
  desenvolvimento 59 (IA); Hearts of Iron IV: wiki, página *Game rules*; Victoria 2: wiki, páginas
  *Event modding* e *Decisions*. CWE: nomes dos arquivos em `common/journal_entries` do
  repositório, para dizer quais sistemas existem; nenhum conteúdo lido para uso.

## Decisões do dono

1. **Qual opção de condução?** A (roteiro datado), B (só condições), C (agenda da IA) ou D
   (mistura com pressão histórica visível). Recomendação: **D**.
2. **A pressão histórica é opção de partida (histórico, plausível, livre)?** Recomendação: sim,
   com *histórico* como padrão para quem quer ver o século e *livre* para o placar de causa.
3. **O jogo terá personagens (líderes)?** Recomendação: **não**. Os fios que dependem de pessoas
   (Hitler, Mao, Deng, Gorbachev) ficam com janela larga em vez de pessoa; personagem traz sorteio
   de traço e de morte, que os princípios recusam.
4. **O placar exige o protagonista certo ou aceita equivalente?** (Uma ditadura revisionista na
   Hungria em vez da Alemanha conta?) Recomendação: exigir o protagonista nos fios 1, 2, 4, 5, 9
   e 10; aceitar equivalente nos outros.
5. **O empurrão pode atuar na guerra (decisão de atacar), ou só na política?** Recomendação: só na
   decisão de atacar, nunca no resultado militar.
6. **Dois movimentos novos no catálogo da fase 3: revisionismo e troca de regime?**
   Recomendação: sim, já no catálogo (sem efeito até a fase 7), porque a política 3d decide como a
   forma de governo muda e eles são a principal entrada.
7. **A agenda histórica é escrita para quantos países?** Recomendação: só para as potências e os
   protagonistas dos 11 fios (cerca de 12 países); o resto fica com interesse e papel.
