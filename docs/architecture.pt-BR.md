# Between Frontiers: proposta de arquitetura

Estado: proposta. Nada aqui está implementado. As decisões em aberto estão no fim.

Between Frontiers (Entre Fronteiras) é um simulador político e econômico de grande estratégia que
vai de 1925 a 2025. A referência de gênero é o Victoria 3: grupos populacionais simulados, mercados
de bens, mapa-múndi por estados. O jogo é e continuará gratuito e de código aberto.

## Princípios

1. **A paz é o jogo.** Economia e política sustentam décadas sem guerra.
2. **Mesmas regras para todos.** A IA não conjura recurso; atores autônomos (investidor privado,
   comerciante) existem desde a primeira fase.
3. **Cada fase sai jogável sozinha**, com 100 anos rodando sem jogador e sem o mundo quebrar.
4. **Desempenho é requisito de projeto**, medido toda noite, não uma etapa de otimização no fim.
5. **Português do Brasil e inglês desde o primeiro commit com texto de jogo.**
6. **Linux e Windows desde o primeiro dia.** Linux é a plataforma de desenvolvimento.

## Laplace-Schelling: a engine

O nome homenageia Laplace (o demônio: dado o estado e as regras, o futuro é calculável, e daí o
núcleo determinístico) e Thomas Schelling (micromotivos que viram macrocomportamento, do modelo de
segregação à teoria dos jogos).

A engine é escrita do zero onde o controle importa e usa biblioteca onde não importa.

| Parte | Do zero | Motivo |
|---|---|---|
| Agendador do tick (grafo de sistemas com leitura e escrita declaradas) | Sim | É onde o Victoria 3 perdeu paralelismo |
| Tabelas de estado (arrays por propriedade, ids tipados) | Sim | Layout de dados decide o custo do tick |
| Aritmética (ponto fixo, inteiro de 64 bits), RNG com semente | Sim | Determinismo entre máquinas |
| Comandos, save (snapshot + log), hash de estado | Sim | Replay, multiplayer e teste de regressão |
| Carga de dados e expressões declarativas | Sim | Script interpretado é gargalo conhecido |
| Renderizador do mapa | Sim, sobre uma API gráfica pronta | Técnica específica do gênero |
| Janela, entrada, abstração de GPU, texto, toolkit de interface | Não | Custo alto, ganho nenhum para a simulação |

O núcleo é uma biblioteca Rust sem janela, sem I/O e sem dependência de engine de terceiros. Ele
não sabe que cliente o está rodando: o cliente gráfico, o servidor de multiplayer e o benchmark são
três consumidores da mesma biblioteca.

### Camadas

1. **Núcleo (`laplace-schelling`):** estado, agendador, aritmética, comandos, save, hash.
2. **Regras do jogo (`between-frontiers-sim`):** os sistemas (mercado, demografia, pesquisa,
   política) escritos sobre a engine.
3. **Dados:** bens, edifícios, tecnologias, leis e países em arquivos de texto, compilados na carga
   para tabelas densas.
4. **Fronteira:** o núcleo entrega o view-model do que está na tela e os deltas. Nunca o estado
   inteiro por frame.
5. **Cliente:** interface e mapa.
6. **i18n:** o núcleo emite chave e argumentos; o cliente formata. Arquivos Fluent (`.ftl`) em
   `pt-BR` e `en`. O CI falha se uma chave faltar em uma das duas línguas.

### Cliente em Linux e Windows

O cliente é nativo: um executável só, sem navegador embutido.

- **Janela e GPU:** winit e wgpu (Vulkan no Linux, DirectX 12 ou Vulkan no Windows).
- **Quadro:** o renderizador do mapa desenha primeiro; a interface é composta por cima, no mesmo
  dispositivo de GPU.
- **Simulação fora da thread da interface.** A interface lê um view-model pronto e emite comandos;
  nunca espera o tick.
- **Garantia:** o CI compila e roda testes e benchmark em Linux e Windows desde o primeiro PR com
  código. macOS não é meta do primeiro dia.

#### Toolkit de interface

| Toolkit | Mapa próprio em wgpu | Aparência | Risco |
|---|---|---|---|
| Slint | Integração oficial: desenha por baixo da interface ou importa textura | Linguagem declarativa de componentes, estilo livre, prévia ao vivo | API de wgpu ainda marcada como instável |
| egui | A mais simples: a interface entra no nosso passo de renderização | Estilo global, layout básico; cara de ferramenta sem muito trabalho | Baixo |
| GPUI + gpui-component | Não há API estável para embutir textura; só em forks | A mais próxima do shadcn: mais de 75 componentes prontos | Alto: feito para o editor Zed, pré-1.0 |
| Dioxus Native (Blitz) | Integração recente | HTML e CSS sem navegador | Alto: beta, desempenho não é foco |

- **Decidido: egui**, na interface do jogo e nas ferramentas de desenvolvimento (inspetor de
  estado, painel do benchmark). Um toolkit só, com a integração mais simples ao nosso
  renderizador e o menor risco.
- **O custo conhecido** é a aparência: o estilo do egui é global e o layout é básico. O sistema de
  design abaixo existe para pagar esse custo uma vez, nos componentes, e não em cada tela.
- **Licença:** egui é MIT ou Apache-2.0.

#### Sistema de design

Não existe um shadcn pronto para egui; há crates de tema e de tokens, não um kit de componentes.
O que o shadcn tem de reaproveitável é o método: os componentes moram no repositório e são nossos.

- **Nenhuma tela chama widget cru do egui.** Toda tela usa os componentes do kit, que aplicam os
  tokens. É o que impede o estilo global do egui de vazar de uma tela para outra.

- **Tokens** em um arquivo só: cores, espaçamento, raio, tipografia, duração de animação.
- **Cerca de quinze componentes próprios:** painel, botão, aba, tabela, tooltip aninhado, barra de
  progresso, gráfico de linha, ícone, lista virtualizada, janela arrastável, alerta.
- **Fontes embutidas** com todos os acentos do português; ícones em SVG.
- **Tela densa primeiro:** o componente só entra no kit depois de provado numa tela real do jogo.

## Modelo de população

### O problema

No Victoria 3, um pop é uma combinação ocupada de estado × cultura × religião × profissão × local
de trabalho. O custo do tick escala com o número de combinações ocupadas, não com a população. Esse
número cresce durante a partida porque a migração espalha culturas pequenas por todos os estados e
cada edifício novo multiplica as combinações.

### O que não pode virar percentual

Cultura e religião não podem ser reduzidas a uma distribuição dentro de um grupo. Três perdas:

- **Correlação.** "30% irlandeses, 40% católicos" não diz que os irlandeses são os católicos.
- **Estado próprio.** Riqueza, alfabetização e radicalização diferentes por grupo deixam de existir:
  todos na mesma célula compartilham um valor.
- **Ato dirigido.** Segregação legal, expulsão, genocídio, partilha e cidadania diferenciada miram
  um grupo. De 1925 a 2025 isso é o centro da história: Holocausto, partilha da Índia, apartheid,
  direitos civis, Iugoslávia, Ruanda.

O Stellaris 4.3 confirma o critério. Ele rebaixou a percentual a ética e a facção, que são opinião,
e manteve espécie e estrato como chave, que são identidade com estado próprio.

### O desenho

A dimensão rebaixada é o **local de trabalho**, não a identidade. São duas tabelas que se encontram
em estado × profissão:

| Tabela | Chave | Guarda |
|---|---|---|
| Demográfica | estado × comunidade × profissão | pessoas, riqueza, alfabetização, pirâmide etária, atitudes políticas |
| De emprego | estado × profissão × setor | vagas, ocupação, salário |

- **Comunidade** é o par cultura + religião, tratado como uma identidade só. A correlação fica
  preservada por construção.
- **Renda de uma linha demográfica** = média dos salários da profissão no estado, ponderada pela
  ocupação de cada setor, vezes o fator de discriminação da comunidade naquele país.
- **Desemprego** recai primeiro sobre quem a lei e o preconceito discriminam.
- **Idade** é um vetor de faixas dentro da linha. **Ideologia e apoio partidário** são percentuais
  dentro da linha.
- **Cauda longa:** comunidade de baixa importância no estado entra numa linha "outras minorias",
  que guarda a composição. A regra está em "Quais comunidades viram linha".

É o modelo do Europa Universalis V (location × cultura × religião × classe, sem local de trabalho).

### Quais comunidades viram linha

Curva ABC sobre a importância, não sobre o tamanho.

**Importância = fatia da população × relevância**, com relevância mínima de 1.

- Grupo grande sem poder aparece: a relevância nunca zera o tamanho.
- Grupo pequeno com poder aparece: a relevância multiplica.
- Grupo pequeno sem poder não aparece: vai para "outras minorias".

O que aumenta a relevância de uma comunidade num país:

| Fator | Exemplo |
|---|---|
| Fatia da elite maior que a fatia da população (terra, capital, oficialato, burocracia) | Brancos na África do Sul |
| Organização própria (partido, igreja, movimento, milícia) | Católicos na Irlanda do Norte |
| Estatuto legal distinto, privilegiado ou perseguido | Judeus na Alemanha depois de 1933 |
| Pátria ou patrono no exterior | Alemães dos Sudetos |
| Concentração territorial (maioria em algum estado) | Catalães, curdos |

Regras:

- **Corte:** ordena por importância, dentro do país e dentro do estado. Classes A e B (até 95% da
  importância acumulada) viram linha; a classe C entra em "outras minorias".
- **Alvo de perseguição conta como relevância.** Ser perseguido é tão relevante para a simulação
  quanto mandar.
- **A relevância muda durante a partida.** Uma lei que mira um grupo ou um movimento que nasce
  promove a comunidade a linha própria. Ao ser promovida, ela herda as médias de "outras
  minorias", porque até ali não tinha estado próprio.
- **Histerese:** o limite para rebaixar é menor que o de promover, para a linha não piscar.
- **Sem fixação por retrospecto.** O cenário de 1925 não marca um grupo porque ele ficou
  importante depois. Tutsis e hutus já são linha em Ruanda-Urundi pelo tamanho; o que vier depois
  tem de sair da simulação.
- **Teto por estado:** o orçamento de linhas vale mesmo quando a curva pediria mais.

### Ninguém some em "outras minorias"

A linha "outras minorias" de um estado guarda a contagem de pessoas de cada comunidade. O que ela
não guarda é estado próprio por comunidade (riqueza, alfabetização, pirâmide etária).

- **Imigração lenta acumula.** Cada chegada soma na contagem da comunidade dentro da linha. Os
  nascimentos da linha são repartidos pela composição.
- **A importância é recalculada todo ano** sobre essa contagem. Quando passa do limite, por
  tamanho ou por relevância, a comunidade vira linha própria.
- **Onda grande já chega como linha.** Um fluxo de refugiados ou de colonos acima do lote mínimo
  nasce com estado próprio (pobre, jovem, sem terra), em vez de herdar a média das minorias.
- **Assimilação é regra explícita, nunca efeito do agrupamento.** A taxa depende da lei, da
  discriminação e da concentração: comunidade concentrada e discriminada assimila menos.
- **A comunidade é a mesma no mundo inteiro.** A diáspora e a população da pátria têm o mesmo
  identificador, o que permite remessa, retorno e movimento de exilados.

Caso de teste: os tutsis exilados em Uganda a partir de 1959. Entram como minoria pequena, não
somem, organizam-se décadas depois e voltam armados em 1990. O modelo tem de permitir os três
passos: persistir na contagem, ganhar relevância por organização e agir sobre a pátria.

### O que se perde

Dois operários do mesmo estado e da mesma comunidade têm a mesma renda, mesmo que um trabalhe numa
siderúrgica lucrativa e o outro numa tecelagem falida. A diferença entre setores continua existindo
na tabela de emprego (greve, falência e salário são por setor); o que some é a diferença entre
pessoas da mesma linha. Um índice de desigualdade por linha repõe parte disso para consumo e
política.

### Tamanho

Referência do Victoria 3: 675 regiões de estado para cerca de um bilhão de pessoas, e relatos de
jogadores de 140 a 160 mil pops entre 1900 e 1936. É nessa faixa que o jogo fica lento.

Ilustração, não medida: 900 estados × 6 comunidades acima do limiar × 12 profissões dá cerca de 65
mil linhas demográficas; a tabela de emprego fica abaixo de 900 × 12 × 10 = 108 mil células.

- O total não é fixo, mas é limitado pelo limiar da cauda longa.
- **O limiar cresce com a população.** O mundo vai de 2 para 8 bilhões de pessoas; o número de
  linhas não pode acompanhar. Limiar e lote mínimo de migração são proporção da população do
  estado, não número absoluto.
- O benchmark de 100 anos registra a contagem de linhas por década e falha se ela passar do
  orçamento.

### O segundo meio século

Levado além de 1936, o Victoria 3 esvazia o campesinato e vira desemprego em massa, e seus
agregados estouram 32 bits com PIB na casa dos bilhões. De 1925 a 2025 isso precisa de regra
própria, não de extensão da regra de 1925:

- transição demográfica (fecundidade e mortalidade caem com renda, educação e urbanização);
- serviços como setor dominante e empregador da mão de obra que sai do campo e da fábrica;
- todo agregado (população, dinheiro, PIB, bens) em 64 bits.

## Desempenho

- **Tick como grafo de sistemas** com leitura e escrita declaradas. Cada sistema calcula em paralelo
  num buffer privado e aplica em série, em ordem fixa.
- **Cadências escalonadas:** preço semanal, demografia mensal, política mensal ou anual.
- **Modificador recalculado por evento**, sem gatilho de script dentro do cálculo.
- **IA por orçamento:** filtra candidatos antes de pontuar e avalia menos os países pequenos.
- **Aritmética de 64 bits** para dinheiro e população.

### O benchmark de 100 anos

Existe a partir do primeiro PR com código e roda toda noite em Linux e Windows.

- Simula de 1925 a 2025 sem jogador, com semente fixa.
- Grava o hash do estado por tick. Duas execuções com a mesma semente têm de dar o mesmo hash, nas
  duas plataformas.
- Mede o custo por sistema e por década, e o número de linhas das duas tabelas de população.
  Grava o custo por sistema no JSON; no CI, cada PR roda contra a base na mesma máquina e o job
  avisa quando a simulação fica mais de 15 % mais cara (`tools/bench/compare.py`).
- Falha se o tempo total ou o tick mais lento passarem do orçamento, ou se o hash divergir.
- Verifica a saúde do mundo: nenhum país com dívida infinita, nenhum bem com preço preso no limite
  por décadas, população mundial dentro de uma faixa plausível.

## Fases

| Fase | Entrega | Diferença para o Victoria 3 |
|---|---|---|
| 0. Fundação | Engine com tick, hash e save; mapa navegável; i18n; benchmark | — |
| 1. Economia | Mercado nacional, necessidades, propriedade e investimento autônomos, mercado mundial | Demanda sensível a preço, estoques, serviços comercializáveis; depois moeda, câmbio, inflação e banco central |
| 2. Pesquisa | Árvore do século XIX como base, mais as eras de 1925 a 2025 | Maturidade por país: ciência, protótipo, industrialização, difusão |
| 3. Política nacional | Grupos de interesse, leis, movimentos persistentes (`politics.pt-BR.md`) | Causa visível, pouco sorteio, sem funil ideológico |
| 4. Diplomacia | Blocos, tratados, influência com retorno | Objetivo próprio para país pequeno; arma nuclear com consequência |
| 5. Guerra | Frentes sobre a logística definida na fase 1 (`war.pt-BR.md`) | Guerra mundial e paz negociada como sistema próprio |
| 6. IA estratégica | Países que interpretam seu papel: interesses medidos, hostilidade e escada de crise (`history-rails.pt-BR.md`) | Interesse com a conta aberta, sem estratégia sorteada |
| 7. Confiança e história (último bloco) | Ciclo de confiança e medo do mercado (`confidence.pt-BR.md`); entradas históricas com marcos, personagens, agendas e casus belli, medidas pelo placar (`history-rails.pt-BR.md`) | Crise que nasce da euforia (Minsky), não de evento; a história real acontece por roteiro com pontos e condições, não por data, e desvia com o esforço do jogador |

Toda fase fecha comparada com o mod Cold War Era do Victoria 3 (e com o Tech & Res): o que eles
têm, o que temos, o que fazer com a diferença. Ver `benchmarks.pt-BR.md`.

Dentro da economia, a ordem é: primeiro o tick que fecha preço e zera, depois estoque, depois
moeda. Um de cada vez, cada um com o benchmark verde.

### Plano de melhoria: o que a fase 1 deixou simplificado

O teste de 100 anos da fase 1 fecha determinístico: população 2,1 → 7,6 bilhões, PIB mundial 4,8 →
56 trilhões, alfabetização 0,48 → 0,83. O que ficou simplificado entra nas fases seguintes:

| Simplificação da fase 1 | Como fica | Fase |
|---|---|---|
| Produtividade cresce por fórmula provisória | Vem das tecnologias difundidas, com serviços ganhando menos | 2 |
| Métodos e bens liberados por ano | Liberados pelo estágio industrial da tecnologia no país; bem novo existe quando alguém o industrializa | 2 |
| Mortalidade cai por calendário a partir de 1935 | Feito: vacinas, antibióticos e saneamento como tecnologias; mortalidade por idade, fecundidade e migração entre países pelo estado de cada país (`population.pt-BR.md`, "Demografia") | 2 |
| Nível novo aparece na hora; sem fila de obras | Feito: construção civil como edifício próprio, com empregados e insumos, que gera pontos de construção; fila por país, obra pública e privada; o nível só funciona quando a obra termina, e falta de material atrasa (`state.pt-BR.md`) | 3 |
| Só o capital privado constrói | Feito em parte: o Estado tem orçamento (imposto por classe, folha, compras) e investe o que passa da reserva em infraestrutura, escolas e hospitais. Falta a indústria estatal | 3 |
| Urbanização cai: a população cresce mais rápido que os empregos (a mecanização da fase 2 piorou: 0,44 → 0,29) | Feito: migração interna campo-cidade por renda (Harris–Todaro), moradia urbana como necessidade e limite, construída pela fila (privada e pública), e lei de migração; 0,45 em 1935 → 0,60 em 2025 (`state.pt-BR.md`) | 3 |
| Moeda única mundial com "banco central automático" por mercado | Feito: moeda por país com nível de preços, inflação e câmbio flutuante por paridade com inércia; banco central com regra de Taylor, autonomia histórica (subordinado até 1990) e comando do jogador; a emissão invisível no Tesouro acabou (`state.pt-BR.md`, "Moeda, banco central e dívida") | 3 |
| Dívida não existe | Feito: déficit vira título comprado em casa, pelo banco central subordinado e no exterior (parte em dólar); juro = taxa mundial + prêmio (dívida, inflação, moeda estrangeira, calote) − poupança interna; calote com perdão parcial e prêmio por anos; correção fiscal pelo custo da dívida. O caso japonês e o calote em dólar saem das regras (teste); no século da IA nenhum país grande chega a eles | 3 |
| Investimento só doméstico; o efeito `foreign_investment` não cai em lugar nenhum; a imitação copia sem canal | Capital que cruza fronteira atrás de salário × técnica, com lei, relação, diáspora e memória de expropriação; dono e remessa na planta; imitação com canal (planta estrangeira, licença, fábrica comprada, ajuda); a via do plano constrói sem planta cheia (`late-industrialization.pt-BR.md`) | 3 (leis), 4 |
| Comércio mundial em pool, sem tarifa nem custo de transporte | Tarifas, acordos, blocos comerciais, frete por distância e porto | 4 |
| Sem regime monetário internacional (a fatia 3c deixou o gancho em `money::exchange_target`) | Padrão-ouro em 1925 (o Reino Unido volta ao ouro em 1925), saída em 1931–33, Bretton Woods de 1944 a 1971 (dólar a US$ 35 a onça, FMI e Banco Mundial), choque Nixon e câmbio flutuante a partir de 1973, euro em 1999; moeda de reserva barateia a dívida de quem a emite | 4 |
| Sem estoques: falta e excesso se resolvem no mesmo tick | Estoques por mercado e reservas estratégicas (petróleo, grãos) | 4 |
| HUD mostra dados de 1925, não da simulação | Feito: barra superior e painel do país leem a simulação uma vez por mês (população, urbanização, alfabetização, PIB do último ano completo em dólares a preços-base, posições no mundo), com metrópole e colônias à parte; a expectativa de vida segue a de 1925 até a saúde ser simulada. Menu e escolha de país mostram o cenário | 2 |
| Transporte e eletricidade valem no país inteiro: uma ferrovia em São Paulo atende uma mina no Amazonas | Feito: transporte e, até a rede nacional, eletricidade são bens do estado, com metade da sobra oferecida às vizinhas por terra; minas, fazendas e fábricas pagam frete, e sem ele produzem menos; empresas e obra pública constroem onde falta; lente "Infra" no mapa (`infrastructure.pt-BR.md`) | 3 |
| Sem mídia | Imprensa, rádio, TV e internet como edifícios e serviço consumido; a mídia pesa na opinião, na propaganda e na censura | 3 |
| Saúde só pela tecnologia; colônias paradas | Remédios e hospitais ligados às leis de saúde; descolonização como movimento político persistente | 3 |
| Petróleo refinado vira um combustível só | Refino em frações: óleo pesado (navios, usinas), leves (gasolina, diesel, querosene), nafta para a petroquímica e lubrificantes; craqueamento térmico e catalítico mudam a proporção | 4 |
| Componentes eletrônicos são um bem só | Semicondutores separados dos componentes, com fábrica cara e poucos países capazes; baterias e cabos como bens próprios (veículo elétrico, rede) | 4 |
| Informática só acelera a pesquisa | Serviços digitais (datacenter, software) como insumo das empresas, com ganho de produtividade para quem usa | 4 |
| Água não é recurso | Água por região para irrigação, hidrelétrica e cidades, com escassez | 4 |

### Plano do último bloco: confiança, interesses e trilho histórico

Pedido do dono: depois da simulação simples, a crise de 1929 pelo medo do mercado, as hostilidades
e guerras com os interesses de cada país, e a história real acontecendo sozinha e desviando quando
o jogador mexe. Fica por último, como ele pediu. Desenho em `confidence.pt-BR.md` e
`history-rails.pt-BR.md`.

Decidido pelo dono em 2026-11-16: a mistura "como o Victoria 3", com entradas históricas que têm
condição de entrada e de saída, marcos e esforço para desviar; personagens com janela de datas;
casus belli com pressão popular como único empurrão na guerra; agendas por ondas; dívida privada e
três confianças por país; um modo só, o histórico, com a chave da pressão histórica, sem promessa
histórica com ela desligada.

| O quê | Depende de | Fase |
|---|---|---|
| Guardar as séries que a confiança vai ler (lucro do fundo, fechamentos por prejuízo, crédito bancário, juro do título, utilização, índice de preços, desemprego por país) no `YearRecord` | nada; campos de leitura | **adiantável já** |
| Movimentos `revisionism` e `regime_change` no catálogo | catálogo da fase 3 | em andamento (PR #53) |
| Fronteiras de 1914 por país, para o indicador `lost_territory` | dados do cenário | **adiantável já** (dados) |
| Na política: margem do movimento e postura do grupo como lista de termos nomeados; variantes "personagem" e "entrada" no registro de mudanças; lugares vazios de chefe de governo e de líder de grupo e de movimento no estado | 3d | **adiantável já** (3, ver `history-rails.pt-BR.md`, "O que os personagens e as entradas pedem da política agora") |
| Reparações e dívidas de guerra como dívida em moeda estrangeira; padrão-ouro com reservas e saída por perda de reservas | contrato da diplomacia | 4 |
| Ajuda e empréstimo entre governos, blocos, cartel com cota, união aduaneira e moeda comum | diplomacia | 4 |
| Guerra que começa por crise (exigência com casus belli, terceiros, recuo), não por declaração seca; insurgência com território; memória de guerra | guerra | 5 |
| Interesses medidos, papel (hegemon, potência em ascensão, pequena), hostilidade por par | 3, 4, 5 | 6 |
| Dívida privada, três confianças por país, corrida bancária, contágio, alavancas do jogador (emprestador de última instância, seguro de depósito) | 3c, 4 | 7 |
| Entradas históricas com marcos e pressão histórica (`data/history/entries.ron`); casus belli e pressão popular | 5, 6 | 7 |
| Personagens: agitador, líder de grupo, chefe de governo, com janela, condição e sorteio pela semente (`data/history/characters.ron`, Wikidata) | 3d (lugares e termos), 7 | 7 |
| Agendas por ondas: GER, ITA, JAP, SOV, USA, GBR, FRA, CHN primeiro | 6 | 7, depois contínuo |
| Marcos das entradas no `events.json` do placar; testes de desvio no CI | `tools/history/` | 7 |
| Regra de partida "pressão histórica" (ligada por padrão): com ela desligada, só fumaça de 100 anos por noite e determinismo nos PRs, e a tela avisa que não há promessa histórica | 7 | 7 |

Nada disto muda a ordem das fases em curso. Três coisas pedem cuidado agora, para não refazer
depois:

- **Política (PR #53):** termos abertos na margem e na postura, lugares de personagem no estado e
  as variantes novas no registro de mudanças, todos vazios até a fase 7.
- **Guerra (fase 5):** nascer da escada de crise com casus belli, que é por onde a pressão
  histórica e a IA de interesses vão entrar.
- **Diplomacia (fase 4):** reparações como dívida em moeda estrangeira e o padrão-ouro com
  reservas, pré-requisitos da confiança e do fio alemão.

### Pesquisa e maturidade tecnológica

Cada tecnologia passa, por país, por quatro estágios inspirados na escala TRL.

- **Operar e produzir são estados distintos.** Um país pode comprar e usar o que não fabrica.
- **Vale da morte:** sem comprador, do mercado ou do Estado, a tecnologia estaciona antes da
  industrialização.
- **Custo** cai com a produção acumulada; **difusão** depende de educação e comércio.
- **Licença e engenharia reversa** pulam estágios com teto de eficiência; **embargo** corta o canal.

Detalhe em `research.pt-BR.md`.

## Mapa

- Textura de ids de província e paleta por modo de mapa, em um draw call; seleção lendo a textura.
- Estados são conjuntos de províncias. O estado é a unidade da simulação; a província é só visual
  e, mais tarde, de frente de guerra. O número de estados define o orçamento do tick.

### Quantos estados

Decidido: cerca de 900. Fica entre o Victoria 3 (675 regiões de estado) e o Hearts of Iron IV
(1.081 estados). Os estados brasileiros, americanos e mexicanos entram inteiros; só unidades de
dezenas de milhões de pessoas são divididas.

| Cenário | Média em 1925 | O que representa |
|---|---|---|
| ~500 | 4,0 milhões | Estados dos EUA e do Brasil, províncias chinesas e indianas inteiras; cada colônia africana é um estado |
| ~900 | 2,2 milhões | O anterior, com Bengala, Províncias Unidas e Sichuan divididas, os maiores estados principescos, URSS por governorado, Nigéria, Congo e Sudão por província |
| ~1.500 | 1,3 milhão | O anterior, com departamentos franceses, prefeituras japonesas e São Paulo e Minas divididos |

- Com 900, toda unidade histórica reconhecível existe sem exigir desenho abaixo do primeiro nível
  administrativo, onde não há fonte livre.
- Em uso: 870 estados em 1925, agrupados por `data/scenario/1925/states.ron`. As regras, as
  aproximações e o que falta para os 900 estão em `docs/map-pipeline.pt-BR.md`, "Estados".
- Em 2025 a média por estado chega a cerca de 9 milhões. Os extremos são grandes em qualquer
  cenário: dezenas de milhões em Bengala, poucos milhares no Yukon.
- O mundo de 1925 tinha de 62 a 76 Estados soberanos, conforme o critério, e mais de uma centena de
  dependências.

### Fontes de dados

O repositório é aberto e o projeto pode cobrar por servidor de multiplayer, então dado com cláusula
de uso não comercial fica de fora, e dado sob GPL não entra nos dados sob CC BY-SA.

| Fonte | Licença | Uso |
|---|---|---|
| Natural Earth | Domínio público | Base física: costa, rios, relevo |
| OpenHistoricalMap | CC0 | Países e colônias em 1925; subdivisões onde existem |
| Cliopatria (Seshat) | CC BY 4.0 | Conferência das fronteiras políticas |
| geoBoundaries | CC BY 4.0 | Subdivisões atuais, redesenhadas para 1925 onde o OpenHistoricalMap é vazio |
| HYDE 3.3 | CC BY-NC-SA 4.0 (o mais restritivo de três textos divergentes; decisão de 2026-11-17, `data/scenario/1925/LAND.md`) | População de 1920 e 1930 em grade, somada por estado |
| Maddison Project 2023 | CC BY 4.0 | PIB per capita por país |
| Wikidata | CC0 | Identificadores, datas, cidades |
| Atlas de época no Wikimedia Commons (Stieler, Andrees) | Por arquivo | Referência de desenho |

Ficam de fora: CShapes (não comercial), GADM (não comercial), historical-basemaps (GPL-3.0), os
scans da coleção David Rumsey (não comercial) e os livros de McEvedy & Jones, Chandler e Modelski
(direito autoral).

O OpenHistoricalMap cobre bem as Américas, a Alemanha, a Polônia e a Itália em 1925. China, Japão,
França, o interior da África e as Índias Holandesas estão quase vazios e terão de ser desenhados.
As correções podem voltar para o OpenHistoricalMap.

## Licença

- Código, engine incluída: AGPL-3.0-or-later. Quem hospedar uma versão modificada tem de publicar o
  código, o que protege um eventual serviço pago de multiplayer sem fechar nada.
- Dados e textos do jogo: CC BY-SA 4.0, com arquivo de atribuição para as fontes CC BY.

## Decisões em aberto

1. Parâmetros da curva ABC das comunidades: pesos da relevância, corte e teto de linhas por estado.
   Proposta, para o dono decidir: pesos 3/2/1/1/1 (elite, status legal, organização, patrono,
   concentração), corte de 95 %, teto de 8 linhas e piso nacional de 2 %
   (`docs/population.pt-BR.md`).
2. Lista de comunidades (pares cultura + religião) de 1925. Proposta, para o dono decidir: 552
   culturas, 24 religiões e 852 comunidades, com a composição dos 870 estados e a fonte de cada um
   (`docs/population.pt-BR.md`, `data/scenario/1925/communities.ron`).
3. Regra de assimilação: taxa base e peso de lei, discriminação e concentração. Proposta, sem
   código, para o dono decidir: 1,5 % ao ano para a cultura e 0,2 % para a religião, vezes lei,
   (1 − discriminação), (1 − concentração)² e família linguística (`docs/population.pt-BR.md`).
4. ~~Trilho histórico e confiança (último bloco)~~: decididos em 2026-11-16. Um modo só, o
   histórico, com a chave "pressão histórica: ligada (padrão) ou desligada"; só o padrão é
   calibrado e coberto pelo placar, e desligada o projeto não promete resultado histórico
   (`history-rails.pt-BR.md`, `confidence.pt-BR.md`).
