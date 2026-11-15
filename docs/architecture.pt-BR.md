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

- **Recomendação:** Electron com interface em React/TypeScript e mapa em WebGL2, núcleo carregado
  como módulo nativo. É o mesmo Chromium nos dois sistemas, então o que roda no Linux roda igual no
  Windows.
- **Alternativa nativa:** winit + wgpu + egui. Dá um binário só e mais controle, ao custo de
  construir toda tela densa em um toolkit menos produtivo.
- **Descartado:** Tauri, porque no Linux usa WebKitGTK, onde o WebGL pode cair em rasterização por
  software sem aviso.
- **Garantia:** o CI compila e roda testes e benchmark em Linux e Windows desde o primeiro PR com
  código. macOS não é meta do primeiro dia.

Como o núcleo não conhece o cliente, trocar Electron por cliente nativo depois não toca a simulação.

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
- **Cauda longa:** comunidade abaixo de um limiar no estado entra numa linha "outras minorias", que
  guarda a composição. Ela volta a ser linha própria se crescer.

É o modelo do Europa Universalis V (location × cultura × religião × classe, sem local de trabalho).

### O que se perde

Dois operários do mesmo estado e da mesma comunidade têm a mesma renda, mesmo que um trabalhe numa
siderúrgica lucrativa e o outro numa tecelagem falida. A diferença entre setores continua existindo
na tabela de emprego (greve, falência e salário são por setor); o que some é a diferença entre
pessoas da mesma linha. Um índice de desigualdade por linha repõe parte disso para consumo e
política.

### Tamanho

Ilustração, não medida: 800 estados × 6 comunidades acima do limiar × 12 profissões dá cerca de 58
mil linhas demográficas; a tabela de emprego fica abaixo de 800 × 12 × 10 = 96 mil células. O total
não é fixo, mas é limitado pelo limiar da cauda longa, e o benchmark de 100 anos acusa se ele
crescer.

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
- Falha se o tempo total ou o tick mais lento passarem do orçamento, ou se o hash divergir.
- Verifica a saúde do mundo: nenhum país com dívida infinita, nenhum bem com preço preso no limite
  por décadas, população mundial dentro de uma faixa plausível.

## Fases

| Fase | Entrega | Diferença para o Victoria 3 |
|---|---|---|
| 0. Fundação | Engine com tick, hash e save; mapa navegável; i18n; benchmark | — |
| 1. Economia | Mercado nacional, necessidades, propriedade e investimento autônomos, mercado mundial | Demanda sensível a preço, estoques, serviços comercializáveis; depois moeda, câmbio, inflação e banco central |
| 2. Pesquisa | Árvore do século XIX como base, mais as eras de 1925 a 2025 | Maturidade por país: ciência, protótipo, industrialização, difusão |
| 3. Política nacional | Grupos de interesse, leis, movimentos persistentes | Causa visível, pouco sorteio, sem funil ideológico |
| 4. Diplomacia | Blocos, tratados, influência com retorno | Objetivo próprio para país pequeno; arma nuclear com consequência |
| 5. Guerra | Frentes sobre a logística definida na fase 1 | Guerra mundial e paz negociada como sistema próprio |
| 6. IA estratégica | Países que interpretam seu papel | — |

Dentro da economia, a ordem é: primeiro o tick que fecha preço e zera, depois estoque, depois
moeda. Um de cada vez, cada um com o benchmark verde.

### Pesquisa e maturidade tecnológica

Cada tecnologia passa, por país, por quatro estágios inspirados na escala TRL.

- **Operar e produzir são estados distintos.** Um país pode comprar e usar o que não fabrica.
- **Vale da morte:** sem comprador, do mercado ou do Estado, a tecnologia estaciona antes da
  industrialização.
- **Custo** cai com a produção acumulada; **difusão** depende de educação e comércio.
- **Licença e engenharia reversa** pulam estágios com teto de eficiência; **embargo** corta o canal.

## Mapa

- Textura de ids de província e paleta por modo de mapa, em um draw call; seleção lendo a textura.
- Estados são conjuntos de províncias; o número de estados define o orçamento do tick.
- **Fronteiras de 1925:** o repositório é aberto e o projeto pode cobrar por servidor de
  multiplayer, então dado com cláusula de uso não comercial fica de fora. A base é desenhada sobre
  dado de domínio público.

## Licença

- Código: AGPL-3.0-or-later. Quem hospedar uma versão modificada tem de publicar o código, o que
  protege um eventual serviço pago de multiplayer sem fechar nada.
- Dados e textos do jogo: CC BY-SA 4.0.

## Decisões em aberto

1. Cliente: Electron (recomendado) ou nativo.
2. Número de estados e limiar da cauda longa de comunidades.
3. Licença da engine: AGPL junto com o jogo, ou permissiva para que outros projetos a usem.
4. Fonte de dados das fronteiras e da demografia de 1925.
