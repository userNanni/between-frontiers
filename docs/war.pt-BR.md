# Guerra: forças, frentes, suprimento e paz

Estado: desenho, contrato e dados da fase 5, **fora da simulação**. O contrato está em
`crates/bf-sim/src/war/`, o catálogo em `data/military/units.ron`, as regras de combate em
`data/military/rules.ron`, as forças de 1925 em `data/scenario/1925/forces.ron` e as invariantes em
`crates/bf-sim/tests/war_1925.rs`. `GameData` carrega os três arquivos (campo `war`); nada no tick
os lê, nada entra no save nem no hash.

A guerra segue os princípios do projeto: pouco sorteio (nenhum, nas funções deste contrato), causa
visível (todo resultado sai com a decomposição termo a termo), determinismo e ponto fixo (`Fixed`).
Ela nasce da escada de crise de `history-rails.pt-BR.md`, nunca de declaração seca, e a pressão
histórica age só na decisão de atacar, nunca no resultado.

## A ideia em uma frase

Uma força é **gente** (homens em idade militar, tirados da pirâmide e do trabalho) mais
**equipamento** (os bens militares que a economia fabrica) mais **treino**; mantê-la custa bens
toda semana, e esses bens têm de **chegar** à frente pela rede de transporte. Quem fabrica, paga e
entrega mais ganha, salvo terreno, ar e mar contrários. A guerra é ganha ou perdida na economia.

## Forças em quatro domínios

| Domínio | Elemento (unidade de conta) | O que o catálogo dá por elemento |
|---|---|---|
| Terra | 1.000 homens | equipamento por bem, consumo semanal, semanas de treino, classe de mobilidade, papéis (antiaérea, engenharia) |
| Mar | 1 navio | tripulação, tonelagem, custo por tonelada, papéis (canhão, torpedo, antissubmarino, antiaérea, aviões a bordo, carga), velocidade |
| Ar | 1 avião | tripulação e pessoal de terra, custo, papéis (ar-ar, ataque ao solo, bombardeio estratégico, ataque naval, reconhecimento, carga), alcance |
| Estratégico | 1 arma, lançador ou satélite | custo, guarda e manutenção; fora das frentes |

O elemento é só a unidade de conta. Formação (divisão, frota, ala) é agrupamento de comando e de
tela, decidido abaixo; as funções recebem elementos somados.

### Tipos e gerações

Um tipo é um **papel** que atravessa o século; a tecnologia não cria tipo novo, cria **geração** do
mesmo tipo, com outro equipamento e outros atributos. O caça de 1925 e o de 2025 são o mesmo tipo
em gerações diferentes. É o corte contra a planilha: 33 tipos, cada um com 1 a 5 gerações.

| Domínio | Tipos (tecnologia de entrada → gerações) |
|---|---|
| Terra (11) | `irregular` (milícia e guerrilha; sem tecnologia); `garrison`; `infantry` (`machine_gun` → `arms_mass_production`, `guided_missile`); `light_infantry` (montanha, selva, fuzileiro; `machine_gun` → `arms_mass_production`, `guided_missile`); `cavalry`; `artillery` (→ `motor_vehicle`, `guided_missile`, `precision_guided_munitions`); `engineers` (→ `earthmoving_machinery`); `anti_air` (`aviation` → `radar`, `guided_missile`); `armor` (`tank` → `armored_warfare`, `composite_armor`); `mechanized` (`motor_vehicle`, motorizada → `armored_warfare`, mecanizada → `composite_armor`); `airborne` (`stressed_skin_aircraft` → `gas_turbine`, aeromóvel) |
| Mar (9) | `battleship` (encouraçado e cruzador de batalha); `carrier` (`aircraft_carrier` → `jet_engine`, `nuclear_power`); `cruiser` (→ `guided_missile`); `destroyer` (→ `radar`, `guided_missile`); `submarine` (→ `nuclear_submarine`, `cruise_missile`); `escort` (aviso, corveta, fragata; → `radar`, `guided_missile`); `coastal` (navio de defesa costeira, torpedeiro, canhoneira, monitor); `mine_warfare`; `transport` (transporte e desembarque) |
| Ar (8) | `fighter` (`aviation` → `stressed_skin_aircraft`, `jet_engine`, `guided_missile`, `stealth_aircraft`); `attack` (bombardeiro tático; → `stressed_skin_aircraft`, `jet_engine`, `precision_guided_munitions`, `satellite_guided_munitions`); `strategic_bomber` (`strategic_bomber` → `jet_engine`, `cruise_missile`, `stealth_aircraft`); `patrol` (reconhecimento e patrulha marítima; → `radar`, `jet_engine`); `air_transport` (→ `stressed_skin_aircraft`, `jet_engine`); `naval_aviation` (`aircraft_carrier` → `stressed_skin_aircraft`, `jet_engine`); `helicopter` (→ `precision_guided_munitions`); `drone` (`armed_drone` → `loitering_munition`) |
| Estratégico (5) | `ballistic_missile` (→ `hypersonic_glide_vehicle`); `missile_submarine` (`nuclear_submarine` + `ballistic_missile`); `nuclear_warhead` (`atomic_bomb` → `thermonuclear_weapon`); `satellite` (`artificial_satellite` → `gps`); `missile_defense` |

**Quantos o jogo aguenta.** O Victoria 3 tem cerca de 20 tipos de unidade e o Cold War Era 34
(abaixo); o Hearts of Iron passa de 100 com variantes. A conta da tela: o jogador lê uma formação
por linha com ícone por tipo, e acima de uns 10 tipos por domínio a linha deixa de caber e a escolha
vira planilha. Proposta: **33 tipos no catálogo, nunca mais de 11 por domínio, e no máximo 20 à
mão em qualquer década**, porque as gerações de um tipo substituem umas às outras e os tipos de
entrada tardia (drone, satélite, míssil) chegam quando os de cavalaria e costeiro já saíram de uso.
A primeira fatia jogável da fase 5 usa os 22 tipos de 1925–1945 (sem os estratégicos, o
helicóptero e o drone).

### De onde sai cada atributo

Nenhum atributo de combate é um número solto por tipo: ou é equipamento (que a economia fabrica e
o jogador paga), ou é uma regra única de `rules.ron` aplicada a ele.

| Atributo | Regra | Onde |
|---|---|---|
| Poder de fogo em terra | soma do equipamento que a unidade de fato tem (o preenchimento fica à vista: o que tem sobre o que deveria ter), na unidade de cada bem, vezes o peso de fogo do bem (`fire_per_unit`) | `rules.ron`, um peso por bem; o lote é US$ 1.000 a preço-base, então equipamento mais caro (geração nova) é mais fogo |
| Proteção | parcela do valor do equipamento em blindados, vezes `protection_per_armor_share`, até o teto | `rules.ron` |
| Fogo naval | valor do equipamento do navio em lotes vezes a parcela do papel (canhão, torpedo, antissubmarino, antiaérea, antinavio): um encouraçado é casco e canhão, um contratorpedeiro é torpedo e antissubmarino; a geração nova custa mais e atira mais | papéis no tipo |
| Resistência do navio | tonelagem: dano em toneladas afunda o navio quando passa da sua tonelagem | tipo |
| Poder aéreo | custo do avião em lotes vezes a parcela do papel | papéis no tipo |
| Treino | semanas de treino do tipo; o fator vai do piso (recruta) a 1 (treino completo) | `rules.ron`, `training_floor` |
| Mobilidade | classe (a pé, cavalo, roda, lagarta, estática) → velocidade e efeito do terreno | `rules.ron`, tabela por classe e terreno |
| Consumo | consumo explícito do tipo (comida, forragem, combustível, munição de treino) mais a manutenção do equipamento, `maintenance_per_mille` do valor por semana; em combate a munição multiplica por `combat_ammunition`, em marcha o combustível por `moving_fuel` | tipo e `rules.ron` |

Exemplo da conta aberta, um elemento de infantaria de 1925 (`infantry`, 1.000 homens): equipamento
100 lotes de `small_arms` e 20 de `artillery`; fogo = 100 × 1 + 20 × 4 = 180 (com os pesos de
`rules.ron`); proteção 0; consumo semanal: 5 t de `processed_food`, 1 lote de `clothes`, 1 lote de
`ammunition` em paz, mais 0,24 lote de manutenção (2 por mil de 120 lotes); em combate, 10 lotes de
munição.

Os números de equipamento, consumo, tonelagem e custo por tonelada são **parâmetros de desenho**,
não dados históricos: ordens de grandeza a calibrar contra a produção bélica e o gasto militar dos
anos 1930 (o teste de calibração abaixo). Eles ficam marcados assim em `units.ron`.

## Como se luta

### Gente: quem pode ser mobilizado

`war::mobilizable` recebe a pirâmide de homens por faixa de idade (qualquer faixa; dentro de uma
faixa a idade é uniforme) e a lei de conscrição:

- **Em idade:** homens entre `service_age` (18 a 45 em `rules.ron`), com as faixas cortadas na
  proporção.
- **Chamáveis:** em idade × efeito `conscription` da lei em vigor (`volunteer_army` 2 %,
  `selective_service` 30 %, `universal_service` 80 %, de `data/politics/laws.ron`).
- **Reservados:** parte dos chamáveis que fica no trabalho essencial (`reserved_share`).
- **Disponíveis:** chamáveis − reservados − já em serviço. Cada homem mobilizado sai da força de
  trabalho; a economia sente a falta na semana seguinte.

A pirâmide entra como lista de faixas, sem depender do tipo da demografia nova (PR #54): quando ela
entrar na `main`, a ligação é uma conversão.

**Reserva e milícia.** São homens treinados fora do serviço: a milícia suíça, a Guarda Nacional e
as reservas organizadas americanas, o Exército Territorial e a reserva britânicos, a milícia ativa
canadense e as forças cidadãs australianas. Em `forces.ron` são linhas com `reserve: true`, à
parte do exército em serviço. `war::call_up` chama a reserva:
- ela se apresenta em partes iguais ao longo de `reserve.call_up_weeks` (4 em `rules.ron`);
- chega com `reserve.training` de treino (70 %), parâmetro de desenho;
- cada reservista que se apresenta sai da força de trabalho, como o mobilizado.

É assim que a Suíça, sem exército permanente, tem meio milhão de homens em um mês.

### Suprimento: o que chega à frente

`war::supply` recebe a necessidade semanal da frente em toneladas e o caminho até ela como trechos
(modo, quilômetros, capacidade em toneladas-quilômetro por semana):

- **Vazão de um trecho:** capacidade ÷ distância. A frente recebe a vazão do **gargalo** (o menor
  trecho), menos a perda de cada trecho por modo (o caminhão gasta parte do que leva, `road` perde
  mais que `rail`; o mar perde para submarino e bloqueio, que entram como perda extra do trecho).
- **Preenchimento:** entregue ÷ necessário, até 1. Sai também qual trecho é o gargalo, que é o
  que a tela mostra ("a ferrovia de Brest a Minsk leva 60 % do que a frente pede").
- A capacidade vem do bem `transportation` dos estados por onde o caminho passa
  (`infrastructure.pt-BR.md`): a guerra compra transporte no estado como qualquer comprador, e o
  que a frente leva falta às fábricas. `tkm_per_transport_lot` converte o lote de transporte em
  toneladas-quilômetro.

Mais capacidade nunca entrega menos (teste de propriedade).

### Terreno e clima

Cada estado tem a sua mistura de terreno, montada por `war::state_terrain`:
- **cidade:** a população urbana de `population.ron` dividida pela densidade urbana de 1925 em
  `rules.ron` (22.000 hab/km², de Angel et al. 2010), sobre a área do estado;
- **montanha e pântano:** de `assets/map/terrain.ron`, gerado pelo `bf-mapgen terrain` a partir do
  ETOPO5 e do GLWD v2 (`map-pipeline.pt-BR.md`, "Terreno");
- **floresta:** de `land.ron`;
- **o resto:** pelos climas de Köppen, que `rules.ron` leva a aberto, selva, deserto ou ártico.

Cada parcela toma o que pede até onde sobrar, nessa ordem. O fator de defesa é a média dos fatores
das classes, pesada pela parcela de cada uma. Inverno multiplica o consumo e corta a velocidade
(`winter`).

O **rio grande na fronteira** do segmento soma à defesa: `river_defense` (30 %) vezes a parcela da
fronteira que corre ao longo do rio (`TerrainFile::river_share`). Só conta o rio na fronteira entre
estados; o rio dentro do estado espera a frente ter posição dentro dele. A **altitude média** de cada
estado também fica gravada (o Tibete passa de 5.000 m), para pesar no suprimento e na aviação
quando a guerra ligar.

### Frente e segmento: onde se manda e onde se luta

Dois níveis, para o jogador não comandar dezenas de pedaços de fronteira:

- **Segmento:** um par de estados vizinhos por terra em lados opostos. É onde rodam o passo de
  combate e o suprimento. Ninguém comanda segmento.
- **Frente:** toda a fronteira contínua em terra com um inimigo ou uma coalizão (os segmentos
  ligados entre si). É onde o jogador e a IA dão objetivo, postura, formações e prioridade de
  suprimento. Alemanha contra União Soviética é uma frente com dezenas de segmentos.

`war::allocate` reparte os elementos da frente entre os seus segmentos por uma regra só, com a
conta aberta:

> **peso do segmento** = extensão × (1 + poder inimigo do segmento ÷ poder inimigo médio da frente)
> × (1 + `objective_pull` se o segmento leva ao objetivo)

Cada segmento recebe primeiro o piso de `min_share` dos elementos (nenhum trecho fica vazio) e o
resto na proporção do peso. Os elementos são inteiros: a sobra do arredondamento vai, um a um, aos
maiores restos, com empate resolvido pela ordem do segmento, o que torna a repartição determinística
e estável: mesma frente, mesmo comando e mesmos números dão a mesma repartição de uma semana para
a outra (teste). A tela mostra, por segmento, a extensão, o poder inimigo, se puxa o objetivo, o
peso e os elementos recebidos.

### Doutrina: as gerações da guerra

Não confundir com a geração de um tipo de unidade (o equipamento de cada época, acima). Aqui,
geração é a doutrina do exército inteiro.

Decisão 1. O dono pediu a leitura de William Lind sobre as gerações da guerra moderna. Para ele,
a 2ª e a 3ª são as duas principais entre países: a 2ª, marcada pela conquista de território e
pelo avanço e recuo; a 3ª, pela ação independente das frações, com foco nos centros de gravidade.
Foram lidos:
- Lind, Nightengale, Schmitt, Sutton e Wilson, "The Changing Face of War: Into the Fourth
  Generation" (*Marine Corps Gazette*, out. 1989, pp. 22–26), inteiro;
- Lind, "Understanding Fourth Generation War" (2004), inteiro;
- Echevarria, *Fourth Generation War and Other Myths* (Strategic Studies Institute, 2005),
  inteiro;
- Lind, *Maneuver Warfare Handbook* (1985), só em parte: a prévia da editora e páginas da prévia
  do Google Books. Por isso, nas citações dele, "PT" é o código de página do Google.

O que as fontes dizem e o que o desenho tira:

- **2ª geração.** Resposta ao poder de fogo. As táticas de fogo e movimento são lineares, a defesa
  tenta parar toda penetração e o fogo indireto concentrado substitui a massa de homens: "a
  artilharia conquista, a infantaria ocupa" (Lind 1989, p. 23). Mais tarde Lind a descreve como
  doutrina francesa da 1ª Guerra em diante. Nela, o fogo é sincronizado pelo centro, numa
  "batalha conduzida", e prevalece a cultura da ordem: obediência acima da iniciativa (Lind 2004).
  **No jogo:** a frente se reparte pela extensão, pela ameaça e pelo objetivo (`allocate`) e anda
  segmento a segmento pela razão de poder (`land_step`). Ganha-se por atrito e território.
- **3ª geração.** Também é resposta ao poder de fogo, mas movida "sobretudo" por ideias: a
  Alemanha sabia que perderia uma guerra de material (Lind 1989, p. 23). O ataque se infiltra para
  contornar e fazer ruir as forças inimigas, em vez de cerrar e destruir; a defesa em profundidade
  convida a penetração para contra-atacar. Os conceitos estavam prontos no fim de 1918, e o tanque
  levou isso ao nível operacional (p. 23). O ataque busca a retaguarda e faz o inimigo ruir "de
  trás para a frente". A ordem diz o resultado, nunca o método (Auftragstaktik), e a iniciativa
  vale mais que a obediência (Lind 2004). O esforço principal (Schwerpunkt) é uma unidade, não um
  ponto no mapa, e fica onde o comandante acha que pode decidir. As "superfícies e brechas"
  orientam onde pô-lo: a unidade que acha a brecha passa por ela e puxa as outras (Lind 1985,
  PT49 e trechos da busca). **No jogo:** o jogador escolhe o centro de gravidade do inimigo
  (capital, região industrial, nó de transporte, o grosso do exército). A frente põe
  `focus_share` dos seus elementos livres no **segmento de foco**, que é o de menor poder inimigo
  por quilômetro entre os que levam ao objetivo, e só cobre os demais pela extensão
  (`allocate_front`). Onde a razão de poder, com a reserva do defensor somada, passa
  `breakthrough_ratio`, o segmento rompe (`breakthrough`): a exploração anda
  `exploitation_speed` vezes a velocidade de base, e os segmentos vizinhos ficam **contornados**,
  com o trecho de suprimento cortado (`interdiction` = 1, logo o termo de suprimento cai ao piso).
  É o "contornar e fazer ruir" com a conta aberta.
- **Como um exército passa de uma para a outra, e por que alguns não passam.** Para Lind a causa
  são ideias e cultura, não a técnica. A 3ª exigiu romper com a cultura da ordem: jogos de guerra
  em que o jovem oficial só resolvia o problema desobedecendo (Lind 2004), educação, jogos de
  guerra livres e tolerância ao erro (Lind 1985, PT61–PT69). Os obstáculos são a "burocracia, a
  política de promoção e o pensamento por fórmula" (PT73); com mentalidade de defeito zero, não há
  ordem por missão (PT25). O rádio aparece só como meio de comando (PT45), e a confiança pesa mais
  que o equipamento (PT37). **No jogo:** `readiness` soma termos nomeados:
  - educação dos oficiais (o peso maior);
  - treino;
  - experiência de guerra (a Alemanha de 1918 aprendeu derrotada pelo material);
  - motorização e rádio, que pesam menos porque, nas fontes, são meios e não causas.

  A **cultura da ordem** desconta: obediência acima da iniciativa, defeito zero, expurgo e
  controle político do corpo de oficiais. Com histerese: sobe a 650, cai abaixo de 450
  (`next_generation`). Um exército com tudo de técnica e expurgo no corpo de oficiais não chega à
  3ª.
- **Contra o determinismo da sequência.** Echevarria critica sobretudo a 4ª geração, mas também a
  ideia de que as gerações se substituem. As formas coexistem: o poder de fogo pesou tanto quanto
  a manobra na 2ª Guerra, na Coreia e no Vietnã. A Blitzkrieg não era doutrina oficial, e sim a
  continuação da guerra de movimento alemã dos anos 1890; o sucesso veio de planejamento, treino e
  liderança descentralizada (pp. 9–10, 14–15). Ele não contesta a descentralização como qualidade
  real. **No jogo:** a geração é traço institucional reversível, não degrau da árvore de
  tecnologia. O poder de fogo continua no termo de equipamento das duas gerações, e a 3ª não ganha
  bônus de combate, só outra regra de repartição e de ruptura.
- **Como uma enfrenta a outra.** A 2ª, que tenta parar toda penetração, é exatamente o que a
  infiltração explora (Lind 1989, p. 23). A França de 1940, que exigia certeza para decidir, ficou
  paralisada pelas sondagens alemãs (Lind 1985, PT17). Mas profundidade e reserva derrotam o
  atacante que só procura a brecha: a defesa elástica contra-ataca antes que ele se consolide
  (PT21; Lind 2004), e a brecha pode ser armadilha, o "saco de fogo" soviético. **No jogo:** a
  2ª reparte pelo poder inimigo da semana anterior e reage com uma semana de atraso à
  concentração. A reserva do defensor entra na razão de ruptura, e frente contínua em terreno
  preparado (terreno e fortificação já estão no poder do defensor) com reserva não rompe.
- **Regras nossas, sem fonte nas leituras.** Nenhum texto lido trata o combustível como freio da
  Blitzkrieg. Mesmo assim, a degradação da 3ª para a 2ª sem rádio ou sem combustível
  (`effective_generation`, abaixo de `radio_min` ou `fuel_min`) é regra do desenho: sem
  comunicação a ação por intenção se desfaz, e sem combustível não há exploração.
- **4ª geração**, que o dono não pôs entre as principais: fica onde estava, no tipo `irregular`
  e no movimento com território.

Testes (`war::doctrine`):
- a prontidão soma as causas, e a cultura da ordem a segura;
- a 3ª sem rádio ou sem combustível luta como a 2ª e não rompe;
- uma reserva atrás de frente preparada impede a ruptura;
- infantaria a pé não explora;
- a 2ª com o mesmo poder só empurra a linha;
- a 3ª concentra na brecha rumo ao objetivo, e a 2ª, onde está o inimigo;
- a repartição fica igual de uma semana para a outra.

### Passo de combate em terra

`war::land_step` resolve uma semana de combate num segmento. Para cada lado:

> **poder** = homens × fogo por homem × treino × suprimento × terreno × ar

| Termo | Conta |
|---|---|
| Número | homens presentes (milhares) |
| Equipamento | fogo por homem do equipamento que a unidade de fato tem, relativo ao da infantaria de referência |
| Treino | `training_floor` + (1 − piso) × treino |
| Suprimento | `supply_floor` + (1 − piso) × preenchimento da munição: sem munição sobra a baioneta |
| Terreno | quem defende multiplica pelo fator de defesa do terreno e pela fortificação; quem ataca multiplica pelo fator da sua classe de mobilidade no terreno (o blindado no pântano perde) |
| Ar | 1 − `air_weight` × (1 − superioridade aérea do lado) |

As perdas de cada lado são uma taxa sobre o próprio efetivo:
`land_lethality` × 2 × (parcela do poder que é do inimigo) × (1 − proteção própria). Com poderes
iguais, cada lado perde `land_lethality` (1,5 % por semana em `rules.ron`); contra um inimigo que
tem todo o poder, o dobro; nunca mais que o efetivo. A taxa não cresce com o equipamento caro: dois
exércitos de 2025 se desgastam no mesmo ritmo que dois de 1925, e quem tem mais equipamento ganha
pela parcela do poder, não pelo número absoluto. O exército sem saída se desfaz pelo avanço da
frente (cerco, na fatia que ligar a guerra), não pela taxa. Se o poder de quem ataca passa o de quem defende, a frente anda
`velocidade × (1 − poder do defensor ÷ poder do atacante)` quilômetros na semana. O resultado volta
com a decomposição: cada termo dos dois lados e a razão entre eles, que multiplicadas dão a razão
de poder ("número 1,8×, equipamento 0,9×, suprimento 0,6×, terreno 0,7×: a ofensiva para").

### Passo de combate no mar

`war::naval_step` resolve um encontro numa zona marítima. Cada lado soma, por grupo de navios:
canhão e torpedo contra os navios de superfície do outro; antissubmarino contra os submarinos do
outro; aviões a bordo contra a superfície, menos a antiaérea do alvo. Como em terra, cada frota
perde `naval_lethality` da sua tonelagem de superfície por semana com golpes iguais e até o dobro
com todos do outro lado; os submarinos perdem pela parcela do antissubmarino do outro sobre os
próprios torpedos. O dano, em toneladas, se divide entre os grupos alvo pela parcela de tonelagem; afunda `dano ÷ tonelagem do navio` navios
inteiros, e o resto fica como avaria. Treino e combustível multiplicam como em terra. A
decomposição mostra canhão, torpedo, ar, antissubmarino, treino, suprimento e número.

Bloqueio e comboio usam o mesmo passo: o bloqueio é a perda extra do trecho marítimo no
suprimento e no comércio, proporcional ao poder de interdição (submarino, superfície, ar de
patrulha) sobre o de escolta. Desembarque é transporte com carga de tropa, que chega com o que
sobrou do passo e luta em terra com o fator de terreno `amphibious`.

### Passo de combate no ar

`war::air_step` disputa uma zona (a de uma frente ou a de um estado bombardeado): a superioridade
de um lado é o seu poder ar-ar sobre a soma dos dois. Cada lado perde `air_lethality` dos seus aviões
com superioridade igual e até o dobro sem ela, divididos entre os grupos pela quantidade, por maior
resto. O que sobra de ataque ao solo e de
bombardeio estratégico, vezes a superioridade, vira golpe: ataque ao solo soma ao fogo da frente;
bombardeio estratégico tira capacidade (fábricas, ferrovia, usina) do estado alvo; ataque naval
entra no passo do mar. A superioridade alimenta o termo "ar" dos outros dois.

### Perdas que voltam para o mundo

- **Demografia:** mortos saem da pirâmide na faixa de serviço; feridos voltam à força de trabalho
  com uma parte inválida (`invalid_share`). Civis morrem de bombardeio, fome por bloqueio e
  ocupação. Tudo por idade, quando a pirâmide existir (PR #54).
- **Economia:** equipamento perdido tem de ser fabricado de novo; o bombardeio e a frente que passa
  destroem níveis de edifício e capacidade de transporte do estado (devastação), que a fila de
  obras reconstrói.
- **Ocupação:** o estado ocupado produz para o ocupante com a eficiência da resistência; a
  resistência nasce como movimento (`politics.pt-BR.md`) e vira guerrilha com território
  (`irregular`).

### A conta da guerra

Decisão 7, pedida pelo dono com o capítulo II de *A Arte da Guerra* (Sun Tzu, trad. Lionel Giles,
1910, domínio público, "Waging War",
<https://en.wikisource.org/wiki/The_Art_of_War_(Giles)/Waging_War>). Dali vêm as ideias, não o
texto:
- o exército custa todo dia, em casa e na frente (II.1);
- a campanha longa esgota o Estado e convida terceiros a aproveitar a fraqueza (II.2–4);
- nenhum país ganhou com guerra prolongada (II.6);
- abastecer de longe empobrece o povo (II.10);
- o exército perto encarece tudo (II.11–12);
- uma parte grande da renda das famílias e da receita do Estado vai para repor o que a guerra
  gasta (II.13–14);
- o que se toma do inimigo vale muitas vezes o que se traz de casa (II.15).

O custo é personagem do desenho. `war::WarLedger` soma, semana a semana e desde o começo da guerra:

| Linha | O que soma | De onde vem |
|---|---|---|
| Gente | soldados mortos **por faixa de idade**, inválidos, civis mortos | passos de combate, bombardeio, bloqueio, ocupação |
| Bens | valor do que as forças consumiram (comida, combustível, munição, manutenção) e do equipamento perdido | `upkeep`, perdas dos passos |
| Dinheiro e dívida | gasto de guerra do Estado e a dívida nova que o pagou | orçamento e títulos (3c) |
| Produção que deixou de existir | trabalho dos mobilizados, fábricas destruídas ou ocupadas, comércio cortado | economia |
| Preços | índice de preços desde o começo da guerra | mercado |

A conta alimenta a exaustão por dois termos novos, **gasto** (gasto acumulado sobre um ano de
produção) e **preços** (alta desde o começo da guerra). Os dois crescem com as semanas mesmo sem
nenhuma derrota: a guerra longa dói também em quem está ganhando. Teste:
`ledger::a_long_war_exhausts_even_without_defeat` (seis anos sem perdas e sem ocupação; a exaustão
sobe todo ano).

## Arma nuclear: dissuasão medida

Decisão 4. O dono concordou e pediu que a dissuasão se apoie nos trabalhos sobre a diplomacia dos
mísseis. Foram lidos:
- Wohlstetter, "The Delicate Balance of Terror" (RAND P-1472, 1958), inteiro;
- Brodie, "Implications for Military Policy", capítulo dele em *The Absolute Weapon* (1946), na
  reimpressão autorizada da *Air Force Magazine*;
- Brodie, *Strategy in the Missile Age* (RAND R-335, 1959): o resumo e o capítulo 8, "The Anatomy
  of Deterrence";
- três trabalhos de Schelling na RAND: "The Reciprocal Fear of Surprise Attack" (P-1342, 1958),
  "Randomization of Threats and Promises" (P-1716, 1959) e "Nuclear Weapons and Limited War"
  (RM-2510, 1959);
- Kahn, *The Nature and Feasibility of War and Deterrence* (RAND P-1888-RC, 1960), e os degraus 1
  a 9 da escada de *On Escalation* (1965), na reprodução de Beal, *Phases of a Crisis* (ONR/ARPA,
  1975, DTIC AD-A005354);
- os tratados nas páginas do Departamento de Estado americano (arquivo 2009–2017).

Ficaram sem ler, porque só existem em empréstimo digital: *The Strategy of Conflict* (1960) e
*Arms and Influence* (1966) de Schelling, e o *On Escalation* inteiro. Por isso compelência,
brinkmanship nos termos dele e os 44 degraus de Kahn não estão citados aqui; entram quando houver
leitura.

O que o desenho tira deles:

- **Dissuasão é o que sobra depois do primeiro golpe, não o que se tem.** Ter a arma não basta.
  Wohlstetter lista os obstáculos que a retaliação precisa vencer, todos eles: sobreviver ao
  ataque, decidir e transmitir a ordem, alcançar o território inimigo, atravessar as defesas e
  destruir o alvo (P-1472, "The Quantitative Nature of the Problem"). Brodie diz o mesmo: o que
  conta é o tamanho a que o inimigo acha que pode reduzir a força por surpresa (R-335, pp.
  281–283). `war::second_strike` faz essa conta obstáculo a obstáculo, por base:
  - silo endurecido: sempre alvo, difícil de matar;
  - submarino: a salvo no mar, alvo mole no porto;
  - bombardeiro: a salvo só na fração em alerta, e ainda precisa atravessar a defesa aérea;
  - lançador móvel: a salvo quando disperso.

  Depois vêm o alcance, a penetração da defesa antimíssil e o comando que sobrevive para dar a
  ordem. Tudo em valor esperado, sem sorteio.
- **Diversificar a base** multiplica a incerteza do atacante (Brodie, R-335, pp. 285–286): com
  várias bases, os golpes do atacante se dividem pelos alvos expostos.
- **Estabilidade, não simetria.** O equilíbrio é estável quando nenhum lado, golpeando primeiro,
  destrói a capacidade do outro de responder (Brodie citando Schelling, R-335, pp. 302–304).
  `war::deterrence` mede o segundo golpe de cada lado contra o dano que o outro tem por
  inaceitável. Daí sai se a dissuasão é mútua, de um lado só ou de nenhum, e quem ganha por golpear
  primeiro (`first_strike_incentive`). A paridade em números não entra.
- **Defesa antimíssil desestabiliza.** Ela reduz o segundo golpe do outro lado. O Tratado ABM
  limitou a defesa justamente para deixar de pé a capacidade de penetração da retaliação de cada
  um (narrativa do Departamento de Estado). Teste:
  `missile_defense_can_break_the_other_sides_second_strike`.
- **Contraforça contra contravalor.** Golpear as bases inimigas perde valor no segundo golpe,
  porque elas já estarão vazias. Golpear cidades dissuade mais antes da guerra, mas executar essa
  ameaça queima o que resta para barganhar (Brodie, R-335, pp. 288–294). O primeiro golpe é
  contraforça (`second_strike`); o contravalor é `countervalue_share` sobre os alvos que o mapa
  conta. A reserva retida fica para a fatia de uso.
- **A ameaça que deixa algo ao acaso.** Schelling mostra que a ameaça pode carregar um risco real
  de se cumprir sem que ninguém queira (P-1716, pp. 8–10). Mostra também que a linha
  nuclear/não nuclear é arbitrária mas útil, que o primeiro uso abre precedente e que nenhum dos
  lados destrói essa linha facilmente (RM-2510, pp. 1–12). No jogo isso vira
  `escalation_risk_per_week`: o custo esperado que uma guerra direta entre duas potências com
  segundo golpe garantido carrega por semana. A decisão pesa esse custo; ninguém o sorteia. O
  primeiro uso na partida rebaixa de vez o limiar de todas as guerras seguintes (fatia de uso).
- **A escada.** Kahn separa a subcrise (gestos e declarações) da crise tradicional (endurecimento,
  demonstração de força, mobilização, assédio e confronto). Entre as duas fica o limiar "não
  balançar o barco"; acima da crise, o limiar "a guerra nuclear é impensável" (Beal 1975,
  reproduzindo Kahn 1965). Os nossos degraus de `history-rails` cabem aí: tensão e rivalidade na
  subcrise, corrida armamentista e crise na crise tradicional. Acima vêm `War`,
  `NuclearCoercion` e `NuclearUse`. `war::escalation_limits` diz até que degrau cada lado pode
  subir contra o outro:

| Situação | Quem tem segundo golpe | Teto de cada um | Risco por semana de guerra direta |
|---|---|---|---|
| Ninguém armado | — | guerra | 0 |
| Só A tem segundo golpe | só A | A: até o uso; B: crise | `escalation_risk_per_week` se B também tem a arma; 0 se não |
| Dissuasão mútua | A e B | guerra, com risco; a coerção não tem crédito | `escalation_risk_per_week` |
| Um armado sem segundo golpe, outro desarmado | ninguém | A: coerção; B: guerra | 0 |

  Kahn lembra o motivo de medir e não supor: quem acredita sozinho na destruição mútua automática
  se expõe a "Munique" (P-1888-RC, pp. 1–3).
- **A capacidade de mobilizar também dissuade:** a provocação dispara o rearmamento do outro lado
  (Kahn, P-1888-RC, pp. 35–38). Entra na hostilidade como corrida armamentista.
- **Controle de armas** é tratado da fase 4 com **modo de verificação**. Meios técnicos nacionais
  só veem o que é observável (silos, submarinos) e pedem regra de contagem. Inspeção no local vê
  ogivas e móveis, ao custo de se expor. Salvaguardas da AIEA servem para os não nucleares.
  Schelling defende que o acordo mire o desenho observável das forças, não regras de alerta que
  ninguém vê (P-1342, pp. 25–27). Pedido à fase 4: um campo de verificação no `Treaty`. Os tratados
  entram como fatos datados de referência, não como roteiro:

| Tratado | Assinado | Em vigor | Limita | Verificação |
|---|---|---|---|---|
| Proibição parcial de testes | 1963-08-05 | 1963-10-10 | testes na atmosfera, no espaço e debaixo d'água | nenhuma no tratado; só a detecção nacional |
| TNP | 1968-07-01 | 1970-03-05 | transferência e aquisição de armas por não nucleares | salvaguardas da AIEA (art. III) |
| SALT I (acordo interino) | 1972-05-26 | 1972-10-03 | congela lançadores de ICBM; teto de SLBM | meios técnicos nacionais; comissão consultiva |
| ABM | 1972-05-26 | 1972-10-03 | dois sítios de defesa antimíssil (um, pelo protocolo de 1974), 100 interceptadores cada | meios técnicos nacionais |
| SALT II | 1979-06-18 | nunca ratificado | 2.400 vetores, depois 2.250; tetos para MIRV | meios técnicos nacionais, regras de contagem |
| INF | 1987-12-08 | 1988-06-01 | elimina os mísseis de solo de 500 a 5.500 km | inspeção no local por 13 anos, monitoramento contínuo |
| START I | 1991-07-31 | 1994-12-05 (fonte secundária) | 1.600 vetores, 6.000 ogivas atribuídas | inspeções no local de vários tipos, telemetria |
| New START | 2010-04-08 | 2011-02-05 | 700 vetores, 1.550 ogivas, 800 lançadores | 18 inspeções no local por ano |

Fontes dos tratados: páginas do Departamento de Estado americano, arquivo 2009–2017. O texto e o
artigo de cada tratado estão em `https://2009-2017.state.gov/t/isn/4797.htm` (LTBT),
`/t/isn/trty/16281.htm` (TNP), `/t/isn/4795.htm` (SALT I), `/t/avc/trty/101888.htm` (ABM),
`/t/isn/5195.htm` (SALT II), `/t/avc/trty/102360.htm` (INF), `/documents/organization/27360.pdf`
(START I) e `/t/avc/newstart/index.htm` (New START). A entrada em vigor do START I vem da Arms
Control Association.

Teste da regra da casa: `deterrence_is_mutual_only_with_secure_second_strikes`. Bombardeiros sem
alerta não dão segundo golpe; com alerta, dão. Trocar os lados troca o resultado.

## Arma nuclear na escada de crise (fases 4 e 5): decidida, com os passos 1 e 2 feitos

O que segue foi combinado com o orquestrador em 2026-11-19. Só o passo 1 (a ponte e o painel,
leitura) está no código. O resto espera as respostas do dono (N1 a N9).

### O que o código tinha e o que falta
- **Árvore (`technologies.ron`), toda com `state_buyer`:** fissão (1938/1944), enriquecimento
  (1944/1945, o bem `nuclear_fuel` e a usina `nuclear_fuel_plant`), bomba atômica (1945),
  termonuclear (1952/1954), míssil balístico (1957/1959), submarino nuclear e defesa antimíssil.
- **Tipos:** `nuclear_warhead` (um núcleo de `weapon_cores` na fissão, 0,73 na termonuclear; ver "A
fatia do físsil"), `ballistic_missile`,
  `missile_submarine`, `missile_defense` e o `strategic_bomber`. As forças de 1925 não têm nenhum.
- **`war::nuclear`:** puro, testado e, até aqui, chamado por ninguém. Faz o segundo golpe por base,
  a dissuasão e o teto de cada lado na escada.
- **A crise do #157:** exigência, prazo, recusa, declaração. A resposta é pela W3, sem ler a arma.
- **Faltava ou falta:**
  - a ponte formações → `NuclearSide`: **feita** (`war::balance`);
  - o painel: **feito**;
  - o programa;
  - o teste e a doutrina;
  - o degrau na crise;
  - o uso e as consequências;
  - os comandos do jogador.
  A escada de hostilidade (tensão, rivalidade, corrida) só existe em `history-rails.pt-BR.md`.

### Passo 1, feito: a ponte e o painel "Balanço de dissuasão"
`war::balance::balance(mil, world, a, b, viewer)` monta os dois lados:
- As ogivas vão aos lançadores: submarinos primeiro (os que mais sobrevivem), depois mísseis,
  depois bombardeiros. Cada lançador recebe até o que carrega (`warheads_per_submarine` 16,
  `_missile` 1, `_bomber` 2). Sem lançador, a ogiva não conta.
- Os submarinos têm metade no mar (`submarine_alert` 500); os bombardeiros, nenhum em alerta até
  a doutrina.
- Defesa: `missile_defense` × 100 interceptores × 50% sobre o que chega.
- Comando: 90% sobrevive.
- Todo lançador alcança: ainda não há alcance no mapa.
- **Dano inaceitável:** 25% da população, a 185 mil mortos por ogiva de 1 Mt equivalente.
  - Fonte: Enthoven e Smith, *How Much Is Enough?* (1971), a destruição assegurada de McNamara,
    com 400 megatons equivalentes para cerca de um quarto a um terço dos soviéticos.
- **N3 (aberta, já no valor recomendado):** um arsenal não testado conta `opaque_weight` (500‰)
  para quem lê; o próprio conta inteiro. Como ainda não há teste no jogo, todo arsenal alheio é
  opaco.
- **N9 (aberta, já no valor recomendado):** com `umbrella`, os protetores que uma cláusula de
  defesa chamaria (`third::who_would_join`) somam os arsenais ao lado protegido.
- O painel aparece na tela da crise e na janela "Guerra". Mostra, para cada lado:
  - as ogivas como o jogador as lê;
  - o guarda-chuva;
  - quanto ainda entrega depois do primeiro golpe, contra o que o outro tem por inaceitável;
  - se o segundo golpe é garantido e se há incentivo ao primeiro golpe;
  - o teto na escada;
  - e, para o par, o risco por semana de guerra direta quando a dissuasão é mútua.
- É só leitura: o hash não muda.
- Testes (`war_nuclear_balance.rs`, `bf-client/tests/nuclear_panel.rs`):
  - sem ogiva, sem balanço;
  - dissuasão de um lado só, com os tetos uso e crise, e a leitura opaca do outro lado;
  - bombardeiros no chão não dão segundo golpe, e os submarinos no mar dão;
  - dissuasão mútua com o risco;
  - o guarda-chuva da França sobre a Tchecoslováquia pelos tratados de 1925.

### Passo 2, feito: o programa e o estoque (N1, N2)
`war::programme`, atrás de `war`; o estado é `WarState::nuclear` (save 37). O comando é
`GameCommand::NuclearProgramme`, e a tela é a seção "Programa nuclear" da aba Forças, com entrada
nas Agendas.

**Funcionamento:**
- **O programa:** o governo fixa uma parcela do PIB por ano, em passos de 0,1%, até 2%. O dinheiro
  é gasto toda semana, por fora do teto militar (`econ::week` soma as compras do programa às do
  Estado).
- **Pesquisa:** até a bomba, o dinheiro compra os bens de um laboratório. A cada mês, o que o
  mercado entregou é creditado à primeira tecnologia da cadeia ainda por entender ou prototipar
  (fissão, enriquecimento, bomba, termonuclear), a `dollars_per_cost` (8 milhões) por ponto de
  custo, com a mesma penalidade por estar à frente da história.
  - A calibração é relativa ao PIB: o Projeto Manhattan custou cerca de 2 bilhões de dólares de
    1945 em 1942–45 (Hewlett e Anderson, *The New World*, 1962), uns 0,4% do PIB americano por
    ano durante três anos, para cerca de 1.800 pontos da cadeia.
  - Do protótipo à indústria, o programa é o comprador da cadeia (`buyer` 1.500).
- **Usinas:** com o enriquecimento industrial, o programa pede um nível da usina de enriquecimento
  no estado metropolitano mais populoso. Pede outro quando o mercado entrega menos de 90% do
  material físsil pedido, um pedido a cada 26 semanas. A fila de obras a constrói, paga pelo
  Tesouro. O investimento aceita esse pedido do Estado.
- **Ogivas:** com a bomba construível, três quartos do dinheiro mantêm uma encomenda A2 de
  ogivas, uma a uma, comprando o equipamento fora do teto. O material físsil vem do mesmo
  mercado das usinas elétricas.
- **N2:** as armas (bomba e termonuclear) são imitadas à metade do ritmo usual. A partilha entre
  aliados por cláusula espera a cláusula dos tratados.

**Conferência do ritmo**, por rodada (`the_pace_from_1940`, `#[ignore]`), com a ogiva de 60 "kg
enriched", antes do #261 e em lotes de 10. EUA, Alemanha e Reino Unido começam o programa em 1940
com 0,4% do PIB. A conferência com a ogiva de 3.600 está em "O custo da ogiva em minério":

| País | Bomba industrial | Primeiro lote de ogivas | Estoque em 1955 |
|---|---|---|---|
| EUA | 1943 (3 anos) | 1954 | 250 |
| Reino Unido | 1946 | 1946 | 10 |
| Alemanha | 1947 | 1948 | 10 |

- **A pesquisa está no ritmo:** a primeira bomba sai em 3 a 7 anos com base industrial.
- **O estoque não cresce:** fica preso no material físsil. A usina pedida é construída e fecha sem
  insumo. O preço da eletricidade não muda (±0,5%), porque nenhuma usina de enriquecimento chega a
  rodar.
- O achado tem duas partes, registradas separadas e sem forçar:
  1. **Escala do insumo: a ogiva, não a usina.** O `buildings.ron` dá as quantidades por nível
     **por ano**.
     - Um nível de `nuclear_fuel_plant` por difusão gasosa pede 200 t de U3O8 por ano e produz
       34 t de "kg enriched", com a razão de cerca de 6 do urânio pouco enriquecido de reator.
     - Em 1925, a extração mundial é de uns 65 t (Shinkolobwe, Jáchymov, Urgeiriça), e nenhuma
       mina nova abre pela demanda: a usina fecha sem insumo.
     - O descompasso de dado está na ogiva. Ela pede 60 "kg enriched" (~360 kg de minério),
       enquanto um núcleo de Little Boy (64 kg a 80%) exigiu de 12 a 24 t de urânio natural.
     - Números de Oak Ridge em 1945, em Hewlett e Anderson, *The New World* (1962):
       - cerca de 6.000 t de óxido em estoque no fim de 1944 (pp. 291–292);
       - 250 t de óxido por ano para cada 1 kg de U-235 por dia (p. 65);
       - US$ 300,6 milhões em Y-12 e 458,3 milhões em K-25 (p. 723).
     - Também: cerca de 10 kg por mês de urânio altamente enriquecido em meados de 1945
       (Wellerstein, 2025, a partir do *Manhattan District History*).
     - A correção da ogiva vai num PR de dados separado.
  2. **Descoberta com data marcada.** Os depósitos de urânio do mapa só "existem" a partir do ano
     da descoberta histórica, quase todos depois de 1946. Isso é história forçada: na vida real, a
     prospecção respondeu em poucos anos à procura (Colorado, Eldorado, Congo ampliado).
  - As duas são lacunas de funcionalidade do thread da economia: mineração e descoberta que
    respondem ao preço e à compra do Estado.

### A fatia do físsil (U1 a U4, o dono, 2026-11-19)
**Decisões:**
- **U1:** o programa compra o seu minério.
  - O capital do Estado abre mina em depósito conhecido: em casa e na colônia.
  - No exterior, abre pelo `foreign_capital`, quando o capital entre países for ligado. O minério
    vem então pelo comércio, sujeito a embargo e bloqueio.
- **U2:** um bem só de material físsil de arma, contado em núcleos, com dois métodos de
  produção: alto enriquecimento, e reator de produção mais reprocessamento.
  - Não há bem `plutonium` nem escolha de rota na tela.
  - O `nuclear_fuel` volta a ser só de energia, com a razão dele (~6 kg de U3O8 por kg).
- **U3:** cada geração da arma gasta menos material e tem mais potência.
- **U4:** o rádio é o bem que puxa a mina antes da bomba, e o minério que o mercado não compra fica
  em estoque.
  - O dono observou que o enriquecimento para reator e o para arma são processos distintos, com
    razões e perdas diferentes; U2 os separa.

**Passo 1, feito: o núcleo, os dois métodos, as gerações e a potência.**
- **O bem `weapon_cores`:** um núcleo de fissão. Vem da usina `weapon_core_plant`, que tem dois
  métodos. Cada nível é uma usina da época:
  - **Alto enriquecimento (com o enriquecimento de urânio):** um nível é o Oak Ridge de 1945.
    - Cerca de 10 kg por mês de urânio a ~80% (Wellerstein, 2025, a partir do *Manhattan District
      History*): 1,875 núcleos de Little Boy, de 64,15 kg, por ano (Sublette, *Nuclear Weapons
      FAQ*, §8.1.3).
    - Cada kg a 80% pede 194 kg de urânio natural, com rejeito de 0,3% (**conta minha** pelo
      balanço de massa; a 93%, 230 kg e 200 SWU, IPFM 2006, p. 23). São 27,5 t de U3O8 por ano.
    - Cerca de 270 MW médios de eletricidade (Reed, APS 2015, a partir de Jones, p. 391).
  - **Reator de produção (tecnologia nova, 1942/1944):** um nível é o Hanford B.
    - 250 MW térmicos (OSTI), ~0,9 g de Pu por MWd (IPFM 2006, p. 27): 82 kg por ano, ou 13,35
      núcleos de Fat Man de 6,15 kg (Sublette, §8.1.1.1).
    - ~0,9 kg de Pu por t de urânio natural: 107,6 t de U3O8 por ano (**conta minha**).
    - A eletricidade e os químicos são **estimativa**.
  - **Minério por núcleo:** 14,7 t de U3O8 pelo enriquecimento e 8,1 t pelo reator.
    `tests/warhead_cost.rs` confere as duas faixas.
  - **A unidade (corrigida depois do #333):** os números acima são o que uma usina fez por ano
    nos EUA de 1945. As quantidades de um método valem na produtividade 1, e a produtividade
    americana de 1945 é 7,25, medida no tick. Por isso o dado é cada número ÷ 7,25, insumos e
    produto juntos: 0,2586 e 1,841 núcleos por nível-ano (**conta minha**).
    - Antes, um nível dos EUA de 1945 tinha capacidade de cerca de 13,6 e 97 núcleos por ano.
    - O minério por núcleo não muda. A semana arredonda o dado a 1/1000: o reator fica em 1,82
      núcleo por nível-ano, 1,1% abaixo.
    - Conferência de 1940, EUA a 0,4% do PIB, com a montagem do #333:

      | Fim do ano | 1944 | 1945 | 1946 | 1947 | 1950 | 1955 |
      |---|---|---|---|---|---|---|
      | antes desta correção | 4 | 8 | 16 | 26 | 60 | 136 |
      | depois | 2 | 6 | 13 | 21 | 49 | 114 |
      | real | 0 | 2 | 9 | 13 | ~300 | ~2.400 |

      - A usina de núcleos chega a 3 níveis em 1945, 5 em 1947, 8 em 1950 e 12 em 1955.
      - A Alemanha fica com 10 e 42 ogivas em 1950 e 1955; o Reino Unido, com 17 e 70.
      - Com `war` desligada, o hash de 10 anos é o do #333 e a série de 101 anos fica idêntica.
- **A usina do programa:** o programa pede `weapon_core_plant`. A obra escolhe o último método que
  o país conhece, o reator quando o conhece. Na prática é o mais barato; não há comparação
  explícita de custo (**leitura minha**).
- **O preço-base do núcleo:** 300 milhões (**estimativa**). Fica acima do que o Oak Ridge de 1945
  gasta num núcleo a preços-base, cerca de 254 milhões, quase tudo eletricidade. O mercado
  corrige.
- **A ogiva (U3):** a de fissão pede 1 núcleo e tem 21 kt (Fat Man; Little Boy 15 kt). A
  termonuclear pede 0,73 núcleo, um primário de ~4,5 kg de Pu (Sublette, §4.3.1), e tem 1,4 Mt.
  - O 1,4 Mt é o meio do Mk-15 (1,69 Mt), do Mk-28 (1,1 a 1,45 Mt) e do W-49 (1,44 Mt), pela lista
    de armas de Sublette.
  - As bombas reforçadas do começo dos anos 1950 (Mk-6) ficam de fora: não há massa publicada.
- **A potência (U3):** cada ogiva vale `Y^(2/3)` megatons equivalentes (`Spec::emt_mille`, em
  inteiro).
  - Fonte da escala: a distância de uma mesma sobrepressão cresce com `W^(1/3)` (Glasstone e
    Dolan, 1977, §3.61–3.62); a área com `W^(2/3)` é **conta minha**.
  - É a "potência equivalente em megatons" da OTA (1979, p. 45).
  - 21 kt valem 0,076 e 1,4 Mt valem 1,25.
  - A variante linear fica só como contraste. Ela erra Hiroshima e Nagasaki ainda mais que a
    `Y^(2/3)`; as duas erram, pela densidade, que é o passo seguinte.
- **Os mortos por megaton equivalente:** `killed_per_warhead` passa a 370.000, reancorado. Os 100
  primeiros megatons equivalentes matariam cerca de 37 milhões de soviéticos (Enthoven e Smith,
  1971, p. 207, citados no FRUS 1964–68, vol. X, doc. 102).
  - Os 100 seguintes rendem menos: 21% em 200 contra 15% em 100. A regra fixa não vê esse
    retorno decrescente.
  - A densidade do alvo, passo seguinte, é que o traz, junto com Hiroshima e Nagasaki.
- **O balanço:** o dano inaceitável é contado em ogivas da potência média do outro lado.
  - Os testes de dissuasão passaram a armar com ogivas termonucleares.
  - A conferência de 1962 arma com termonucleares e a de 1950 com fissão.

**Passo 2, feito: o programa abre as suas minas (U1).**
- **A regra:**
  - Com usinas de núcleos e o mercado entregando menos de `mine_shortage` (90%) do minério que elas pedem, o programa pede um nível de `uranium_mine`.
  - O pedido usa a fila de obras do Estado, como a usina, e é um de cada vez (`plant_weeks`).
  - O lugar é um depósito já descoberto e com teto livre: primeiro em casa, depois em colônia, com o maior espaço que sobra.
  - No exterior (pelo capital do Estado), espera o capital entre países.
- **A usina de núcleos não fecha por prejuízo:** é empresa do Estado, o prejuízo é do Tesouro, e o Estado a mantém enquanto a quer. Sem isso, usina e mina fechavam em ciclo, uma sem a outra.
- **Com `war` desligada** não há programa nem empresa do Estado: a série anual de 101 anos fica idêntica à do main.
- **Teste:** `the_programme_opens_a_mine`. Os EUA pedem a mina na própria carnotita e a constroem.

**Conferência de 1940, com o passo 2 e a ogiva consertada** (a ogiva de fissão pedia componentes eletrônicos, que só existem a partir de 1950; por isso o estoque ficava em zero até os anos 1950). Medida sobre o main com o estoque por mercado (#299, desligado) e o aceite geral das usinas do Estado (#300), sem ajuste; EUA, Alemanha e Reino Unido a 0,4% do PIB desde 1940 (`the_pace_from_1940`):

| Fim do ano | 1944 | 1945 | 1946 | 1947 | 1950 | 1955 |
|---|---|---|---|---|---|---|
| EUA, jogo | 9 | 20 | 37 | 60 | 132 | 189 |
| EUA, real | 0 | 2 | 9 | 13 | ~300 | ~2.400 |

- **O que limita os EUA:**
  - em 1943–44, o minério: o preço fica em 270 a 294 (100 sem programa), com uma e depois duas minas;
  - de 1945 em diante, a obra: usina e mina crescem um nível por ano cada (8 e 8 em 1950), e o minério volta a cerca de 80;
  - o estoque sobe de 17 a 24 ogivas por ano até 1950, o que esse ritmo de obra entrega.
- **Dependências anotadas, sem correção agora:**
  1. **A montagem e as mãos de Los Alamos.** O jogo não tem esse passo. Em 1945–47, o real ficou em 2 a 9 ogivas por ano com o físsil de que dispunha, limitado pela montagem e pelos técnicos. No jogo, o núcleo vira ogiva na semana em que chega. Isso explica o excesso de 4 a 5 vezes em 1946–47, junto com o tamanho de um nível de usina, que é uma usina inteira da época.
  2. **A bomba industrial em 1943, dois anos antes da real (1945).** O caminho é o crédito do programa. Com 0,4% do PIB desde 1940, ele empurra a cadeia (fissão, enriquecimento, reator, bomba) a `dollars_per_cost` por ponto. A penalidade de estar à frente da história (`ahead`) freia, mas não basta. Por isso há 9 ogivas já em 1944.
  3. **O orçamento fixo em 0,4% do PIB.** O gasto americano saltou depois de 1950 (Coreia, a corrida da bomba H); os ~2.400 de 1955 vêm daí. O jogo fica em 189. É decisão de quem joga, ou da IA da fase 6, e não regra.

### Passo 3 do físsil: o rádio (U4), atrás da chave `radium`, desligada
Antes da bomba, a mina de urânio trabalha pelo rádio, que os hospitais compram. O urânio sai como
subproduto e, com `market_stock`, fica em estoque.
- **Atrás da chave.** A chave é lida na carga (`GameData::load_with`, `--rule radium`).
  - Desligada, o bem `radium`, o seu lugar nos métodos e a tecnologia `cobalt_teletherapy` saem
    do catálogo e da árvore, que ficam iguais aos do main. O hash de 10 anos e a série de 101
    anos são os do main.
  - O motivo da chave: o rádio muda empregos e PIB do jogo de hoje, e o equilíbrio vem por
    último. Liga junto com a fatia nuclear e o `market_stock`.
- **A mina** (`uranium_mine`): o rádio é o primeiro produto, na razão física com o urânio, de
  3,40e-7 em massa (conta minha pelas meias-vidas), com 80% de recuperação (estimativa).
- **O hospital pede 3,09 mg por nível.** Na produtividade 1, são os ~24 g do mundo por volta de
  1925 (Minerals Yearbook 1930, p. 873; 1934, p. 503) sobre os 3.615 níveis de hospital de 1925,
  que pesam 7.768 vezes a produtividade (conta minha, medido no tick).
  - A primeira versão punha 7 mg, e o jogo pedia 54 g.
- **O rádio some com a teleterapia de cobalto-60.** O campo `fading: [(bem, tecnologia)]` vai no
  método, no dado, e é genérico: o uso do insumo cai com o estágio da tecnologia no país.
  - Metade no industrial e zero no difundido (`econ::week::FADE_*`). **Valores de desenho.**
  - Sem método usando o campo, o tick é o mesmo.
  - A troca de método da economia (10% a mais de lucro) nunca tiraria o rádio do hospital, que
    pesa ~0,03% do custo do nível.
  - A tecnologia `cobalt_teletherapy`, que requer a fissão:
    - demonstrada em 1951: London, Ontário, em 27 out., e Saskatoon, em 8 nov. (LHSC, 2011;
      Houston e Fedoruk, *Encyclopedia of Saskatchewan*; páginas web sem número);
    - industrial em 1953: estimativa, pela primeira unidade na Itália em 1953.
- **A mina segue o minério quando o rádio some.** Uma usina cujo produto principal é um bem que
  some (`Catalogue::fades`) segue o produto que mais vende.
  - Sem isso, a mina de rádio caía a 5% de uso sem comprador, e a falta de minério construía o
    dobro de níveis com pessoal cheio. O Canadá de 2025 ia a +10% do PIB.
  - A regra vale só para esses bens. Aplicada a todo prédio com vários produtos (carne e lã,
    petróleo e gás), mudava o jogo desde 1925.
- **Conferência com a chave ligada** (`tests/radium.rs`), medida e não ajustada:
  - Rádio pedido no mundo, em gramas: 24 em 1925, 31 em 1935, 41 em 1945, 47 em 1955, 38 em
    1965, 10 em 1975 e 0 de 1985 em diante.
    - A teleterapia fica difundida nos EUA em 1965; no Canadá, no Raj e na China em 1975; no
      Brasil em 1985.
    - A queda real do uso não tem série anual com página aqui. O Minerals Yearbook de 1969 diz
      que o rádio "is being replaced gradually", e a AECL parou de refinar rádio em 1950–53.
      Fica como lacuna.
  - Minério de urânio em estoque em 1940: 1.681 t de U3O8 em todos os mercados (913 t no Congo
    belga, 562 t nos EUA). O real é ~1.220 t (conta minha: Congo mais Staten Island; H&A, p.
    26; Jones, p. 79).
  - **101 anos contra a faixa do `17782c2`** (15 rodadas): de 2% a 8% dos territórios fora a 2σ
    por década, o ruído da faixa.
    - Em 1925 são 12%, mas ali o σ da faixa é quase zero.
    - PIB mundial: +0,8σ em 1965, −2,2σ em 1985 e +0,5σ em 2025.
    - Canadá: −0,5σ em 1985, +0,7σ em 2000 e −1,3σ em 2025.
- **Dependência anotada: o preço de um bem sem comprador não cai.** O rádio fica em ~650 por mg
  depois que os hospitais param de comprá-lo. O investimento vê as minas como lucrativas e
  constrói mais: a URSS chega a 195 níveis de mina de urânio em 2025, contra 26 sem rádio.
  - O valor adicionado fica pequeno, porque o rádio não vende.
  - É regra do mercado (economia), a rever quando a chave ligar.

### A montagem da ogiva (item 3 da lista de funções; opção (a) do orquestrador, 2026-11-23)
O passo que faltava entre o núcleo e a arma.
- **Fonte:** Hewlett e Duncan, *Atomic Shield 1947–1952*, AEC, 1969.
  - p. 47: o relatório da AEC a Truman, de 3 abr. 1947, diz que nenhuma das bombas estava
    montada. As equipes civis da guerra tinham se desfeito, e as militares ainda não estavam
    treinadas.
  - p. 134: Los Alamos não tinha linhas de produção nem procedimentos escritos.
- **A regra:**
  - A ogiva pede um núcleo (`weapon_cores`) e um conjunto não nuclear (`weapon_assemblies`): as
    lentes de explosivo, o iniciador, a espoleta e o invólucro.
  - O conjunto é feito pela `weapon_assembly_plant`, empresa do Estado como a usina de núcleos:
    o lucro e o prejuízo vão ao Tesouro, e ela não fecha por prejuízo.
  - O programa pede a usina quando as usinas de núcleos rodam e a bomba pode ser feita. Volta a
    pedir quando o mercado entrega menos de `assembly_shortage` (90%) dos conjuntos que a
    encomenda de ogivas pede.
  - O pedido usa a mesma espera entre pedidos (`plant_weeks`) da usina de núcleos e da mina.
- **Por que um bem, e não a usina entregando a ogiva direto:** a ogiva é uma formação montada
  por encomenda a partir de bens, não produto de prédio. Fazer um prédio entregar formações seria
  uma mudança maior que o bem.
- **Métodos:**
  - `hand_assembly` (bomba atômica, 1945): 2 t de explosivo e a espoleta como equipamento
    elétrico (50) por conjunto. São os insumos que a ogiva pedia antes.
  - `electronic_assembly` (termonuclear, 1950): 1 t de explosivo e 100 de componentes
    eletrônicos.
- **VALOR DE DESENHO, sem fonte:** um nível faz 12 conjuntos por ano nos EUA de 1945. Os
  empregos são os da usina de núcleos, com mais engenheiros.
  - As quantidades de um método valem na produtividade 1: um nível roda as suas quantidades
    vezes a produtividade do prédio onde está.
  - A produtividade americana de 1945 é 7,25, medida no tick. Por isso o dado é 12 ÷ 7,25 =
    1,655 conjunto por ano, com os insumos de um conjunto na mesma proporção.
  - Um nível é um quadro fixo de pessoal: em outro país faz 1,655 vez a produtividade dele.
  - Não há página para a produção de Sandia nem para a do Mark 4 (cerca de 550 feitos de 1949 a
    1951, também sem página). O estoque de 1945–47 é medida, não meta.
  - O preço-base do conjunto (1 milhão, estimativa) deixa o valor adicionado por trabalhador da
    usina na faixa da economia.
- **A unidade, achada pela interface no #334:** a primeira versão deste PR punha 12 no dado.
  Nos EUA de 1945 isso dava uns 87 conjuntos por nível-ano, e a montagem nunca amarrava.
- **Conferência de 1940** (`the_pace_from_1940`; EUA a 0,4% do PIB):

  | Fim do ano | 1944 | 1945 | 1946 | 1947 | 1950 | 1955 |
  |---|---|---|---|---|---|---|
  | EUA, sem a montagem (#313) | 9 | 20 | 37 | 60 | 132 | 189 |
  | EUA, com a montagem | 4 | 8 | 16 | 26 | 60 | 136 |
  | EUA, real | 0 | 2 | 9 | 13 | ~300 | ~2.400 |

  - A usina de montagem tem 1 nível em 1944–45, 2 em 1946–47, 4 em 1950 e 11 em 1955.
  - Há núcleos esperando montagem (0,1 a 0,4) de 1945 a 1950: a montagem amarra.
  - **Leitura minha:** o excesso de 1946–47 cai para 1,8 a 2 vezes (16 e 26, contra 9 e 13). O
    resto é o que travou os EUA naqueles anos: as equipes que se desfizeram com a desmobilização
    (p. 47). O jogo não tem desmobilização de técnicos: fica como lacuna anotada, sem número.
- **A encomenda de ogivas (trocada depois do #336):** antes, o programa encomendava uma ogiva
  por vez (`batch`), entregue com a encomenda inteira. Isso dava exatamente 26 ogivas por ano de
  1950 a 1955, qualquer que fosse o dinheiro ou a obra. Era um teto artificial, não calibração.
  - **A regra agora:**
    - A encomenda cresce, de ogiva em ogiva, até que o que ainda falta comprar valha pelo menos
      o dinheiro do estoque da semana. O dinheiro é 75% da parcela, depois que a bomba pode ser
      feita.
    - O dinheiro do programa e o que as usinas entregam são os únicos tetos.
    - A ogiva entra assim que o seu núcleo e o seu conjunto chegam, uma a uma, como os aviões.
  - **Quando o jogador encerra o programa ou zera a parcela:**
    - a encomenda fica parada com os núcleos e os conjuntos que já tem, visíveis na tela, e nada
      some;
    - o orçamento militar nunca compra a encomenda do programa;
    - um núcleo que já tinha o seu conjunto vira ogiva; o resto espera um programa novo.
    - Teste: `the_order_follows_the_money_and_waits_when_the_programme_ends`.
  - **O que amarra agora** (`war::programme::limit`), nas palavras do jogador:
    - as usinas de núcleos, ou as de montagem, quando o mercado entrega menos de nove décimos do
      que o programa comprou;
    - senão, o dinheiro do programa.
  - **Teto de estoque do jogador** ("parar em N ogivas"): fica para depois, com o save. Hoje o
    jogador freia pela parcela.
  - **Conferência de 1940, medida e não alvo** (`the_pace_from_1940`):

    | Fim do ano | 1944 | 1945 | 1946 | 1947 | 1950 | 1955 |
    |---|---|---|---|---|---|---|
    | EUA, encomenda antiga, 0,4% (#336) | 2 | 6 | 13 | 21 | 49 | 114 |
    | EUA, encomenda nova, 0,4% | 3 | 8 | 24 | 57 | 162 | 352 |
    | EUA, encomenda nova, 1,0% | 0 | 9 | 33 | 66 | 258 | 710 |
    | EUA, real | 0 | 2 | 9 | 13 | ~300 | ~2.400 |

    - A Alemanha fica em 0 / 21 / 61 (1947 / 1950 / 1955) a 0,4% e em 37 / 93 / 197 a 1,0%. O
      Reino Unido fica em 3 / 39 / 109 e em 45 / 133 / 299.
    - **A 0,4%, o dinheiro amarra.** A usina de núcleos para em 4 níveis em 1947, porque o
      núcleo não falta. O estoque sobe cerca de 35 por ano até 1950 e 38 por ano até 1955.
    - **A 1,0%, a montagem amarra até 1950.** São de 44 a 74 núcleos esperando conjunto de 1944 a
      1949. A usina de montagem cresce um pedido a cada 26 semanas, junto com a de núcleos e a
      mina.
    - **Dependências anotadas, sem ajuste:**
      1. O excesso de 1946–47 (24 e 57 contra 9 e 13 a 0,4%) tem duas causas que o jogo não tem:
         - a desmobilização das equipes (Atomic Shield, p. 47);
         - o **programa em ritmo de guerra depois de 1945**: a parcela não cai na paz, e a IA que
           a baixa é da fase 6.
         Com a encomenda nova, o dinheiro compra o que as usinas já podiam fazer: em 1947 os
         níveis de usina são quase os mesmos de antes (núcleos 4 contra 5, montagem 3 contra 2).
      2. O déficit de 1955 (352 ou 710 contra ~2.400) vem da parcela. O gasto americano saltou
         depois de 1950; a parcela fixa de 0,4% é decisão do jogador ou da IA da fase 6.
      3. Um pedido de obra a cada 26 semanas (`plant_weeks`), compartilhado entre núcleos, mina
         e montagem, é valor de desenho e amarra a 1,0%.
- **Com `war` desligada:**
  - o bem e o prédio novos, no fim das listas, mudam o hash de 10 anos para `71d55cda…`;
  - a série anual de 101 anos fica idêntica à do main;
  - o save não muda;
  - nenhuma usina de montagem existe sem programa (`no_core_plant_without_a_programme`).

### O custo da ogiva em minério (PR de dados, substituído pelo passo 1 acima)
A ogiva pedia 60 "kg enriched" de `nuclear_fuel` (30 na termonuclear), cerca de 350 kg de U3O8.
Agora pede 3.600 nas duas gerações, contadas pelo minério que um núcleo real consumiu.
- **Unidade do jogo:** o `nuclear_fuel` é medido em "kg enriched", combustível de reator. A
  difusão gasosa do `buildings.ron` faz 34.100 com 200.000 kg de U3O8, uma razão de 5,87 kg de
  minério por unidade.
- **Núcleo de referência:** o do Little Boy, 64 kg a cerca de 80% de U-235 (Wellerstein, 2025,
  a partir do *Manhattan District History*).
  - Com rejeito de 0,3 a 0,5%, isso pede de 12 a 24 t de urânio natural. **Conta minha** pelo
    balanço de massa do enriquecimento.
  - Em U3O8, são 14,1 a 28,3 t, ou 2.400 a 4.800 "kg enriched" (**derivado**).
  - O meio da faixa, **3.600**, é o valor do dado (**derivado**), igual a 21,1 t de U3O8 por ogiva.
- **Termonuclear:** fica igual à fissão até haver fonte para o físsil de um primário
  (**estimativa**, não medida).
- **Rota do plutônio:** falta. Não há reator de produção nem reprocessamento. Toda ogiva do jogo
  sai do enriquecimento.
  - **O plutônio poupa minério**, de duas a três vezes. O texto anterior dizia o contrário, a
    partir de uma estimativa minha sem fonte (0,2 a 0,25 kg de Pu por t), e estava errado.
  - O núcleo do Fat Man era de 6,15 kg de liga de Pu-Ga (Sublette, *Nuclear Weapons FAQ*, §8.1.1.1,
    a partir de LA-3067, p. 45).
  - Um reator de baixa queima dá cerca de 0,9 kg de Pu por t de urânio natural: 9,2 kg de 10,2 t
    de combustível no exemplo do CIRUS (IPFM, *Global Fissile Material Report 2006*, p. 27).
  - Isso dá cerca de 6,8 t de urânio natural por núcleo (**conta minha**), contra 12 a 24 t do
    Little Boy.
  - É a rota de quem não tem difusão, e também a mais barata em minério.
- **Teste:** `tests/warhead_cost.rs` lê a razão da usina e confere que cada geração custa de 14,0 a
  28,5 t de U3O8. Hoje dá 21,1 t.

**Conferência de 1940 com o custo novo** (medida, não meta; sobre o #257, com EUA, Alemanha e
Reino Unido a 0,4% do PIB desde 1940-01-01; o dinheiro está em dólares de 2011):
- **Estoque americano no fim do ano:**

  | | 1945 | 1946 | 1947 | 1954 | 1955 | 1960 |
  |---|---|---|---|---|---|---|
  | Jogo | 0 | 0 | 0 | 100 | 110 | 520 |
  | Real | 2 | 9 | 13 | — | ~2.400 | ~18.600 |

  - O real vem de Norris e Kristensen (2010) e, para 1945–47, do DOE (1993).
  - Reino Unido e Alemanha têm o primeiro lote de 10 em 1946 e 1947.
- **Pesquisa:** a bomba fica industrial nos EUA em 1943; a real ficou pronta em 1945.
- **Usinas:** os EUA chegam a 24 níveis em 1945 e a 34 em 1950. Sem minério, todos fecham no ano
  seguinte e são reconstruídos.
- **Minério:** a extração mundial do jogo é de 311 t de U3O8 em 1945. Os 24 níveis pediriam
  cerca de 250.000 t por ano.
- **Custo da primeira ogiva** (os EUA, até o primeiro lote em 1954):
  - Um nível custa cerca de 68 milhões de dólares de 2011, ou uns 5,4 milhões de 1945 pelo CPI-U
    (fator de cerca de 12,5, **conta minha**).
  - Os níveis vistos de um fim de ano ao outro somam pelo menos 84 construídos. São cerca de 5,7
    bilhões de 2011, ou uns 460 milhões de 1945 (**conta minha** a partir da medida).
  - A operação de um lote de 10 compra 36.000 "kg enriched", a 2.000–2.600 por unidade: 72 a 94
    milhões de 2011.
  - Na fonte, Y-12, K-25 e S-50 custaram cerca de 770 milhões de dólares de 1945 em construção
    até dezembro de 1945 (Hewlett e Anderson, *The New World*, 1962, p. 723).
  - O programa inteiro gastou 29 bilhões de 2011 até 1945 e 52 bilhões até 1954. O Projeto
    Manhattan custou cerca de 2 bilhões de 1945, uns 25 bilhões de 2011.
- **Diferenças anotadas, sem ajuste:**
  - **Tamanho do nível civil:** em minério, um nível vale cerca de três Oak Ridges. Uma usina
    sozinha bastaria se houvesse minério.
  - **Sem estoque de minério rico:** o Projeto tinha cerca de 6.000 t de óxido no fim de 1944
    (pp. 291–292). No jogo, as minas de 1925 extraem dezenas de toneladas.
  - **As minas não respondem à demanda:** o preço do minério sobe, mas nenhuma mina abre por isso.
  - **Sem a rota do plutônio:** depois de agosto de 1945, a produção real seguiu também pelo
    plutônio, a rota alternativa ao enriquecimento.

**Ordem de grandeza na década de 1960:**
- O jogo extrai cerca de 3.500 t de U3O8 por ano no mundo em 1960, o bastante para cerca de 165
  ogivas por ano a 21,1 t cada (medido).
- O mundo livre extraía cerca de 35.000 a 40.000 t por ano entre 1959 e 1961. A fonte é a tabela
  do *Minerals Yearbook* do US Bureau of Mines, em toneladas curtas, com o ano de cada coluna
  ainda por conferir. Isso daria umas 1.700 a 1.900 ogivas por ano.
- O estoque americano subiu cerca de 3.000 por ano entre 1955 e 1960. Dezenas de milhares de
  ogivas **não cabem** no minério nem do mundo real com núcleos só de urânio a 21 t cada.
- A diferença é o **núcleo menor das gerações seguintes** (reforço, núcleo composto).
  - O arsenal real usou da ordem de 7 a 10 t de U3O8 por ogiva, contra 21 do Little Boy
    (**estimativa sem fonte conferida**).
  - O plutônio também conta: poupa de duas a três vezes o minério por núcleo (ver acima, com a
    fonte).
  - Fica anotada para a fatia do físsil: o físsil por geração (U3), a rota do plutônio (U2) e as
    minas que o programa encomenda (U1).

### Passo 3, feito: o degrau nuclear na crise (N3, N4, N9)
Atrás de `war`, com estado em `NuclearState` (save 38).

- **N3, o teste:** `GameCommand::NuclearTest` (aba Forças, "Testar a arma") marca o país como
  testado na semana seguinte, se ele tiver ao menos uma ogiva. Daí em diante, o arsenal conta
  inteiro para todos; antes, os outros contam a metade (`opaque_weight`). O teste não se desfaz.
- **N4, a coerção:** quando quem exige tem segundo golpe garantido e o alvo não, segundo a leitura
  do alvo (a de um arsenal não testado conta a metade), a razão de força para ceder um pedido
  periférico cai para `nuclear_coercion` (500‰, em `crisis.ron`). O existencial nunca cede.
  - A tela da crise diz quando há coerção.
  - Com a N10 (abaixo), a metade fixa deu lugar ao termo nuclear somado à razão. O teste é
    `coercion_weighs_on_the_periphery_only`.
- **O teto bloqueia a declaração:** toda semana, o sistema de forças marca as crises em que o
  teto de quem exige contra o alvo, segundo a própria leitura, fica abaixo de `War`
  (`NuclearState::blocked`).
  - A declaração dessas crises não sai; a crise fica, e só resta recuar.
  - A tela mostra o motivo e desativa o botão de declarar.
  - A IA, que hoje não exige, recua como antes.
- **N9 na crise:** o guarda-chuva entra pelo balanço. A França, obrigada a defender a
  Tchecoslováquia e com segundo golpe garantido, bloqueia a declaração da Hungria
  (`the_umbrella_blocks_the_declaration`).
- **Fica para o passo 4:** o risco semanal de escalada entre dois lados com segundo golpe garantido
  pesando na decisão da IA de declarar, e o custo de reputação do teste com os não nucleares.

### Passo 4, em rascunho: o uso e a consequência (N5, N6, N7)
Está em `war::nuclear_use`, atrás de `war`, no save 39. Os parâmetros ficam em `rules.ron`
(`nuclear`) e são estimativas de desenho, a conferir contra as fontes.
- **O comando `NuclearUse`:** dá o alvo (um país em guerra com quem usa), o modo e o número de
  ogivas.
  - Sai na semana seguinte, com o que os lançadores carregam (as ogivas sem lançador ficam no
    depósito).
  - A defesa antimíssil do alvo para a sua parcela. Cada ogiva que chega é um efeito.
  - O uso conta como teste (N3).
- **Contravalor:**
  - mata nas cidades do alvo, pela densidade do alvo (abaixo, "Densidade do alvo"), e não mais um
    número fixo por ogiva;
  - derruba níveis de edifício no dobro da parcela da população morta (`wreck_factor`).
  - Fonte de escala (RERF, FAQ, Q1, mortos agudos nos dois a quatro primeiros meses):
    Hiroshima, 90 a 166 mil de 340 a 350 mil; Nagasaki, 60 a 80 mil de 250 a 270 mil. O texto
    anterior dava 70 a 140 mil e 40 a 70 mil, números que a RERF não confirma.
- **Contraforça:** mata um décimo disso (`counterforce_civilians`) e põe 20 mil homens das forças
  de terra do alvo fora da guerra por ogiva (`counterforce_men`), em proporção em cada formação.
- **Apoio (N6):**
  - Os mortos de cada país atingido ficam em `NuclearState::dead` e entram no termo de baixas do
    seu apoio à guerra, somados aos da frente.
  - O apoio de quem usa não lê nada do uso.
  - Medido no teste: 8 ogivas de contravalor sobre a Polônia levam o apoio dela de 779 a 742 em
    cinco semanas, e o da Alemanha fica igual (288).
- **Reputação:** quem usa leva uma mancha na reputação de segurança de dois anos do custo das suas
  forças (`use_stain`). Para comparar: uma chamada de defesa não atendida vale um ano.
  - Fica de fora o chamado aos aliados do atingido. Os aliados com cláusula de defesa já entraram
    na declaração (W4), e o uso não chama de novo.
- **Retaliação proporcional (N5):**
  - A IA atingida, com ogivas prontas, responde na semana seguinte no mesmo degrau e com o mesmo
    número.
  - Sem forças de terra, responde com tudo, em contravalor.
  - A resposta não é respondida de novo.
  - O risco semanal de subir entre dois lados seguros fica para a IA da fase 6.
- **O tabu (N7):**
  - É um estoque mundial de 0 a 1.000.
  - Cresce um vigésimo (`taboo_years` 20) a cada janeiro em que há ogiva no mundo e o ano anterior
    não teve uso.
  - Cada uso o corta à metade (`taboo_cut`).
  - A IA atacada usa primeiro só com a população metropolitana sob o inimigo acima de uma linha.
    A linha vai de 30% sem tabu (`first_use_occupied`) a 100% com o tabu inteiro (Tannenwald,
    *The Nuclear Taboo*, 2007).
- **A tela:** a janela "Guerra", sob o painel de dissuasão, mostra:
  - o modo e as ogivas;
  - os mortos estimados e os homens postos fora da guerra;
  - a mancha, a resposta esperada e o tabu, com os anos desde o último uso.
  "Lançar…" pede a confirmação, que diz a frase do que muda, com ponto de restauração antes.
- **Testes** (`tests/war_nuclear_use.rs`, Alemanha jogada contra a Polônia):
  - contravalor com mortos, edifícios e mancha;
  - contraforça com menos civis e homens fora;
  - sem lançador, nada sai;
  - a resposta no mesmo degrau;
  - os mortos pesam só no atingido;
  - o tabu cresce e cai;
  - a IA só usa primeiro depois da linha.
- **A precipitação (N8)** está no passo 4b, logo abaixo.

### Densidade do alvo (item 5 da lista de funções; opção (b) do orquestrador, 2026-11-19)
Está em `war::target`, atrás de `war`. Os três leitores usam a mesma função: o golpe (`strike`), o
dano inaceitável da dissuasão (`balance`: quantas ogivas até um quarto da população) e a tela
(`nuclear_view`). O `killed_per_warhead` saiu.
- **A regra:**
  - A população urbana de cada região metropolitana é uma cidade cuja densidade cai do centro,
    `D(r) = D0·e^(−r/r0)`, com `r0 = √(U ÷ 2πD0)`.
  - As áreas letais das ogivas que caem nela se somam num disco de raio `R`, e morrem os de
    dentro: `U·[1 − (1 + x)·e^(−x)]`, com `x = R/r0 = √(2·A·D0/U)`.
  - Cada ogiva vai para a cidade onde mata mais. A primeira mata na densidade do núcleo; as
    seguintes chegam aos subúrbios. O retorno cai pela forma, sem número-alvo.
  - Nunca passa da população urbana da região: há teste com 500 ogivas numa cidade só.
  - A conta é toda inteira (partes por bilhão), igual em qualquer máquina.
- **`core_density` = 9.900 por km².** É a densidade a menos de 0,6 milha do marco zero em
  Hiroshima (25.800/mi²) e Nagasaki (25.500/mi²), da tabela 12.09 de Glasstone e Dolan, *The
  Effects of Nuclear Weapons*, 1977, p. 544. São cidades japonesas de 1945, de madeira e sem
  aviso: o valor serve de teto para outras épocas. A variação por país é lacuna.
- **`lethal_km2_per_emt` = 132.** Conta minha, sobre a mesma tabela: os mortos de cada anel
  divididos pela densidade do anel.
  - Hiroshima dá 8,46 km² para 12,5 kt; Nagasaki, 9,03 km² para 22 kt (p. 36).
  - Juntas, são 17,5 km² para 0,132 EMT.
  - Separadas, dão 157 e 115 km² por EMT. Nagasaki teve morros que abrigaram parte da cidade.
- **A população rural fica fora do alvo** por construção. A precipitação (N8) é que a alcança.
  Se as cidades de um país não somam um quarto da população, o dano inaceitável fica fora de
  alcance (`balance::UNREACHABLE`).
- **Ponto em aberto, levado ao dono (2026-11-23): o critério de McNamara tinha duas pernas.**
  Era 20 a 25% da população e cerca de metade da capacidade industrial (Enthoven e Smith, 1971).
  - O jogo só lê a primeira perna, e a população rural fica fora do alvo.
  - Por isso, contra um país pouco urbano, "um quarto do povo" fica difícil ou inalcançável. Foi
    o que virou a decisão das 3.000 ogivas de fissão no #322.
  - A segunda perna, a indústria, entraria pelo `wreck_factor` sobre os níveis de edifício das
    cidades atingidas.
  - A regra não muda agora.
- **Conferência, medida e não alvo** (`tests/war_target.rs`; os dois mundos vêm de uma rodada
  desde 1925 com a guerra desligada, a série do main):
  - **Japão de 1945:** 65,1 M de pessoas, 52,6 M nas cidades (81%), 19 cidades. A maior é
    JPN-TOKYO, com 4,33 M.
    - Uma ogiva de 12,5 kt mata 62.620 na maior cidade. Hiroshima real: 68.000 mortos de 256.300
      em risco.
    - Uma de 22 kt mata 89.367 (Nagasaki real: 38.000 de 173.800).
    - É leitura, não conferência: a densidade do núcleo saiu da mesma tabela, e o jogo atira na
      maior cidade, não em Hiroshima.
  - **União Soviética de 1964:** 340,5 M de pessoas, 217,3 M nas cidades (64%), 110 cidades. Os
    36% rurais ficam intocados.

    | EMT | ogivas | mortos | parcela da população | real (FRUS) |
    |---|---|---|---|---|
    | 100 (de 1 EMT) | 100 | 61,8 M | 18,2% | 15% |
    | 200 (de 1 EMT) | 200 | 99,0 M | 29,1% | 21% |
    | 100 (de 0,076) | 1.315 | 65,1 M | 19,1% | — |
    | 400 (de 1 EMT) | 400 | 139,6 M | 41,0% | — |

    O real vem de Enthoven e Smith, 1971, *How Much Is Enough?*, p. 207, como o FRUS 1964–68,
    vol. X, doc. 102, cita.
    - O jogo passa o FRUS em 1,2× com 100 EMT e em 1,4× com 200. O retorno cai, mas menos que no
      real.
    - **Leitura minha:** o núcleo japonês de 1945 é mais denso e mais frágil que a cidade
      soviética de 1964. Além disso, a URSS do jogo tem 340 M de pessoas e 64% urbanas, contra
      cerca de 228 M e cerca de metade urbana na real. Esses dois números estão sem página aqui:
      é dependência da demografia, anotada e não corrigida.
    - Para um quarto da população soviética de 1964 são precisas 155 ogivas de 1 EMT, 127 de
      1,25 e 2.003 de 0,076.
- **O que muda nos testes N1–N10** (todos com o mundo de 1925):
  - **Uma decisão vira:** "3.000 ogivas de fissão em 1.500 bombardeiros" deixa de dissuadir a
    URSS. Matam 32,3 M, abaixo do quarto (35,2 M); antes eram 37,4 M.
  - Nenhuma outra decisão muda: os EUA de 1962 seguem com segundo golpe, Cuba cede, Berlim não
    cede e a Coreia não muda.
  - O dano inaceitável muda em número de ogivas:
    - o da URSS, em ogivas americanas: de 77 para 117;
    - o dos EUA, em ogivas soviéticas: de 70 para 49;
    - o da China em 1950, em ogivas americanas: de 4.245 para 8.645.
  - Os EUA de 1962 matam 47,3 M, e não mais 141 M: as cidades da URSS de 1925 saturam.
  - Os termos da N10 caem de ±20.641 para ±13.584.

### Passo 4b, em rascunho: a precipitação (N8)
O dono pediu uma perda de produtividade gradual, que cai com os anos e fica à vista como estoque.
Está em `war::nuclear_use::fallout`, atrás de `war`, no save 40.
- **A perda:** cada ogiva que chega tira `fallout_loss` (10%) da produtividade da região onde cai.
  - As regiões dos estados vizinhos perdem `fallout_downwind` disso (três décimos, 3%). Sem
    vento no mapa, todos os vizinhos fazem as vezes do lado de sotavento.
  - A perda se compõe sobre o que resta, até `fallout_cap` (90%).
  - Os três números são **estimativa**, a conferir. Escala de referência: depois de Chernobyl,
    cerca de 784 mil hectares de terra agrícola saíram de uso (IAEA, *Chernobyl's Legacy*,
    2005).
- **O estoque:** `NuclearState::fallout` guarda, por região, a parcela perdida. A produtividade da
  região (`tfp`) fica em `1 − parcela` do que seria sem ela.
- **O tempo:** a cada janeiro, a parcela cai a `fallout_keep` (977‰), a meia-vida de 30 anos do
  césio-137, e a produtividade volta junto. Abaixo de 1‰, some.
  - O crescimento anual da produtividade (`econ::yearly`) segue por cima. Por isso o
    alcance do líder pode fechar parte da perda mais cedo (**leitura minha**, a medir numa
    rodada longa).
- **A tela:** a janela "Guerra" lista, estado por estado, quanto da produtividade cada lado perdeu,
  com a meia-vida.
- **Fica de fora:** a saúde (os mortos tardios por câncer, Semipalatinsk) e o vento.
- **A perda chega à produção:** a `tfp` da região é a que todas as usinas leem (`econ::plant_tfp`).
  A região produz menos com os mesmos trabalhadores. A população que sobra continua consumindo, e
  o mercado fecha pelo preço. Nada é criado nem destruído pela regra; só a produtividade muda.
- **Conferência de sanidade, sem ajuste:**
  - Hiroshima e Nagasaki se reconstruíram em cerca de uma década. As duas foram explosões no ar,
    com pouca precipitação local.
  - A premissa do césio-137 segura a perda por 30 anos.
  - A regra de hoje não distingue explosão no ar de explosão no solo, e isso pesa para
    superestimar a perda num ataque a cidade.
  - **Passo futuro:** a parcela de precipitação local por tipo de explosão, com a fonte de
    Glasstone e Dolan, cap. IX ("Residual Nuclear Radiation and Fallout"). A página está a
    conferir.
- **Teste** (`the_fallout_takes_productivity_and_decays`, Alemanha contra a Polônia):
  - uma ogiva leva a região atingida a 0,900 da produtividade do mundo sem o ataque;
  - os vizinhos perdem 30‰, inclusive um estado alemão;
  - um ano depois, o estoque é de 97‰ e a produtividade volta a 0,906.

### Decisões do jogador, cada uma com tela e `player_commands`
- `NuclearProgram` (Agendas → Forças): parcela, anos estimados, custo.
- `NuclearTest` (Agendas, com confirmação): torna a posse pública (N3); custa reputação com os não
  nucleares.
- `NuclearDoctrine` (aba Forças): sem primeiro uso ou primeiro uso, mais o nível de alerta.
- `NuclearUse`: decisão rara, na janela "Guerra".
  - O jogador escolhe o alvo (contraforça numa frente, ou contravalor em estados) e o número de
    ogivas.
  - Antes de confirmar, a tela mostra os mortos estimados, a reputação, a retaliação esperada e o
    tabu.
  - Confirmação dupla e ponto de restauração automático.

### Conferências (passo 5), medidas
`tests/war_nuclear_conferences.rs` põe as forças de cada época no mapa de 1925, com a população de
1925, e lê o que as regras dão. É medida, não meta.

**Fontes das forças:**
- Norris, "The Cuban Missile Crisis: A Nuclear Order of Battle", Wilson Center, 24/10/2012 (versão
  aberta do Nuclear Notebook, *Bulletin* 68(6), 2012):
  - EUA: cerca de 3.500 ogivas prontas (p. 15); 182 ICBMs (p. 15); 7 submarinos Polaris com 112
    mísseis em patrulha (p. 17); 1.479 bombardeiros prontos em 4/11 (p. 22).
  - URSS: cerca de 300 ogivas prontas (p. 15); 42 ICBMs (p. 25, segundo Karlov); cerca de 160
    bombardeiros pesados (p. 27). Os submarinos de mísseis não tiveram papel (p. 25).
- 1950: 369 ogivas americanas e 5 soviéticas (Norris e Kristensen, 2010, a tabela anual). Sem
  mísseis.
  - Os bombardeiros capazes não têm fonte conferida. O caso dá um bombardeiro por duas ogivas
    (**estimativa**).
- Coreia: Truman disse em 30/11/1950 que o uso esteve "em consideração ativa", e a Casa Branca
  disse no mesmo dia que nenhuma autorização foi dada (FRUS 1950, vol. VII, doc. 909, p. 1262).

**Cuba, outubro de 1962 (medido):**
- **Segundo golpe dos EUA garantido:** 1.589 ogivas entregues depois de um primeiro golpe
  soviético, contra 191 que a URSS tem por inaceitáveis.
  - A URSS não tem segundo golpe: entrega 0 contra 174.
  - Teto: EUA no uso nuclear, URSS na crise. A dissuasão não é mútua.
- **A exigência periférica dos EUA à URSS** ("retirar os mísseis", posta como um pagamento
  pequeno): há coerção, e a razão pedida cai à metade, para 1,5×. Ainda assim a URSS **não
  cede**, porque a força de terra americana é 0,25 da soviética.
- **Os EUA podem declarar guerra:** nada os bloqueia.
- **Diferenças com a história, sem ajuste:**
  1. **O ceder lê a força de terra.** A coerção nuclear só divide a razão. Na crise real, a
     assimetria nuclear (17:1, Gribkov, p. 8) pesou sozinha. **Leitura minha:** a coerção
     precisaria somar a assimetria nuclear à razão de forças, e não só reduzir o limiar.
     **Feito pela N10 (#271):** o termo nuclear entra somado à razão, e a URSS cede em Cuba (ver
     "Regras decididas depois das medidas").
  2. **Os EUA não são contidos.** Falta a "Europa como refém": a URSS ameaçando os aliados
     europeus dos EUA com força de terra e mísseis de alcance médio, ou seja, o guarda-chuva ao
     contrário. Falta também o peso do risco de escalada na decisão, que é da IA da fase 6.
  3. **Com um oitavo dos bombardeiros em alerta**, como na crise, nada muda: o segundo golpe vem
     dos submarinos e dos bombardeiros que sobrevivem.
  4. **A população é a de 1925** (URSS ~147 milhões), não a de 1962.

**Coreia, 1950–53 (medido):**
- Com 369 ogivas, os EUA têm segundo golpe garantido contra a URSS, que tem 5 ogivas e nenhum.
  Contra a China, o teto é coerção nuclear.
- **Em guerra com a China, a IA americana não usa primeiro** com tabu 0, 200 ou 1.000.
- **Leitura minha:**
  - O resultado bate com a história, mas pelo motivo errado. A regra de primeiro uso (N7) só olha
    a ocupação da metrópole, e os EUA não são ocupados.
  - A ponderação de três custos que explica o não uso (a reputação com os aliados, o tabu e a
    reserva contra a URSS; Dingman, Tannenwald) não existe na IA. Fica para a IA da fase 6.
  - Um jogador americano pode usar a arma. A tela mostra a mancha, a resposta esperada e o tabu.

### Regras decididas depois das medidas: G9 e N10
- **G9 (o dono, 2026-11-19: opção (b)):** com `occupation_by_advance`, a população de um estado
  conta como sob o inimigo na proporção do avanço inimigo sobre a profundidade do estado
  (`war::support::under_enemy`). O estado só troca de dono no fim do avanço.
  - O limiar continua: 500‰ com a capital tomada, 900‰ sem ela.
  - A tela de ocupação (#266) lê a mesma conta.
  - Medida (`tests/war_g9.rs`, Alemanha jogada contra a Polônia, sem tratados): a população sob o
    inimigo aparece desde a primeira semana. O governo cai na semana 9, contra a 12 de antes. A
    Hungria contra a Tchecoslováquia cai em uma a duas semanas nas duas regras.
  - A Argélia (G6) não muda: 5,9 anos, porque a metrópole não é ocupada.
  - Alemanha 1914–18 e Vietnã não leem a ocupação.
- **N10 (o dono, 2026-11-19: opção (a), simétrica):** na exigência periférica, `weigh_crisis` soma
  à razão de força de terra a vantagem nuclear de um lado só de quem exige. Essa vantagem é o que
  o seu segundo golpe entrega sobre o dano inaceitável do alvo, na leitura do alvo. A vantagem do
  alvo, contada do mesmo jeito, é descontada (`Weighing::nuclear`).
  - O existencial continua sem ceder.
  - A tela da crise mostra o termo e quem coage ou dissuade.
  - Testes (`n10_coercion_and_theatre`):
    - Cuba 1962: a URSS cede, com termo de +8,3×.
    - Berlim 1961: os EUA não cedem, com termo de −8,3×, e a URSS não pode declarar.
    - Coreia 1950: a China não cede.
  - **Ressalva das entradas:**
    - os exércitos e as marinhas são os de 1925 do jogo;
    - Berlim usa os arsenais de 1962 (estimativa, porque 1961 não tem fonte aqui);
    - os bombardeiros de 1950 são estimativa.
  - **Fica para uma fatia futura:** a variante (b), a razão no teatro, pelo que cada lado leva até
    lá com o controle do mar e do ar da zona. Ela pede o controle de zona lido em tempo de crise.

### Decisões do dono (2026-11-19, todas na recomendação levada pelo orquestrador)
- **N1:** o programa é decisão explícita, com orçamento e anos (feito no passo 2).
- **N2:** imitação mais lenta das armas e partilha por cláusula entre aliados (a imitação está no
  passo 2; a cláusula espera os tratados).
- **N3:** a posse só é conhecida depois do teste; sem teste, o arsenal conta a metade (no dado
  desde o passo 1).
- **N4:** contra quem coage de um lado só, o alvo cede mais fácil no periférico; no existencial,
  nunca.
- **N5, retaliação proporcional:** a resposta fica no mesmo degrau (contraforça por contraforça,
  cidade por cidade), com o risco semanal de subir. O contravalor total fica para quem recebeu
  contravalor ou perdeu a capacidade de responder de outro jeito.
- **N6:** nada direto no apoio interno de quem usa.
- **N7, tabu como estoque:** o limiar sobe devagar com os anos sem uso no mundo e cai a cada uso,
  para todos (Tannenwald, *The Nuclear Taboo*, 2007). A tela mostra os anos desde o último uso.
- **N8, precipitação: fora desta fatia, mas decidida como fatia seguinte ao uso.** Nos estados
  atingidos e nos que ficam a sotavento, a terra e a saúde perdem produtividade por anos.
  - Fonte: depois de Chernobyl, cerca de 784 mil hectares de terra agrícola saíram de uso na
    Bielorrússia, na Rússia e na Ucrânia (IAEA, *Chernobyl's Legacy*, The Chernobyl Forum,
    2005).
  - Fonte: a precipitação dos testes de Semipalatinsk sobre as regiões vizinhas (Simon e Bouville,
    *Health Physics* 82(5), 2002).
  - Vem depois do passo 4.
- **N9:** o guarda-chuva é dado pelo aliado com cláusula de defesa (no dado desde o passo 1).

### Para a IA da fase 6
- proliferação: quem começa o programa e quem fica sob guarda-chuva;
- corrida armamentista;
- doutrina e alerta;
- brinkmanship;
- dissuasão estendida como cláusula de aliança;
- os tratados como cláusulas (LTBT, TNP, SALT/ABM/INF/START, com o modo de verificação), que são do
  thread dos tratados.

## Como começa e como acaba

1. **Escada de crise** (`history-rails.pt-BR.md`): tensão, rivalidade, corrida armamentista, crise
   com exigência e prazo apoiada num casus belli, guerra. Cada degrau tem causa visível; a
   declaração seca não existe.
2. **Mobilização:** na crise, o país passa homens para o serviço pela lei de conscrição e põe a
   indústria em pedido de equipamento; a mobilização custa trabalho e confiança antes do primeiro
   tiro.
3. **Coalizão:** entra quem as cláusulas do tratado obrigam (defesa, ofensiva; "Crise e
   declaração", "Terceiros"); honrar ou trair é escolha, com preço.
4. **Exaustão medida:** `war::exhaustion` soma termos nomeados, cada um com peso em `rules.ron`:
   perdas sobre os homens em idade, mobilizados sobre os homens em idade (trabalho drenado), falta
   de bens de consumo, população ocupada, semanas de guerra, indústria destruída, gasto de guerra
   e alta de preços (a conta da guerra, acima). Vai de 0 a 1.000
   por mil, e a tela mostra cada termo.
5. **Paz negociada:** o lado que pede paz oferece parte das exigências; o outro aceita quando o
   valor esperado de continuar (o que ainda pode tomar menos o custo de mais semanas de exaustão)
   fica abaixo do que lhe é oferecido. Exigência é o que está na mesa desde a crise: território,
   reparações (dívida em moeda estrangeira, fase 4), desarmamento (limite de forças, como
   Versalhes), mudança de lei. Não há pontuação de guerra: quem ocupa o que pede e quem está mais
   exausto, com os números à vista.

## O que muda em relação ao Victoria 3

| | Victoria 3 | Between Frontiers |
|---|---|---|
| Unidade | batalhão de 1.000 homens em formação com general | elemento de 1.000 homens, navio ou avião, em formação de comando |
| Tipos | por grupo (infantaria, artilharia, cavalaria; navios leves, de linha, de apoio), com melhoria por tecnologia | 33 tipos por papel em quatro domínios, gerações por tecnologia |
| Equipamento | bens consumidos pelas casernas, sem estoque por unidade | estoque por unidade; o fogo é o equipamento que ela tem |
| Suprimento | percentual por frente, sem caminho | toneladas pelo caminho, trecho a trecho, com gargalo nomeado |
| Ar | modificador de tecnologia | domínio próprio com superioridade disputada e bombardeio estratégico |
| Mar | frotas em zonas, bloqueio de comércio | zonas, comboio, bloqueio como perda do trecho, desembarque com carga |
| Batalha | sorteio de rolagem por batalha e de modificadores do general | sem sorteio: poder com seis termos nomeados |
| Início | jogada diplomática com escalada | escada de crise com casus belli e exigência (`history-rails.pt-BR.md`) |
| Fim | pontuação de guerra e apoio à guerra | exaustão medida por termos; paz negociada pelas exigências |
| Perdas | população perde homens | perdas por idade na pirâmide, inválidos, devastação de edifícios e de transporte |

## Comparação com o Cold War Era

Contagem nas definições de `common/` do CWE no commit `8316f96` (2026-11-16). Só contagens e
sistemas; nada copiado.

| Sistema | CWE | Nós |
|---|---|---|
| Tipos de unidade | 34 em 4 grupos: 11 de infantaria, 10 de artilharia, 10 aéreos (no grupo de cavalaria), 3 de fuzileiros | 33 em 4 domínios, com gerações |
| Opções de mobilização | 24 | lei de conscrição e o que a mobilização tira do trabalho; opções viram leis e ordens de indústria |
| Guerra nuclear | decisões, ultimato nuclear, ataque nuclear e estacionamento de armas | ogiva como estoque com dissuasão na hostilidade; uso com consequência (decisão 4) |
| Guerra química e biológica | decisões próprias | não planejadas |
| Guerra limitada, guerrilha, guerra civil, paramilitares | entradas de diário | guerrilha como tipo `irregular` e movimento com território; guerra civil quando um movimento toma o Estado (fase 3) |
| Bases militares e acesso | ações diplomáticas | tratado de base e de acesso (fase 4) |
| Corrida espacial | entrada de diário e decisões | satélite como tipo estratégico; prestígio depois |

## Decisões do dono

Decididas pelo dono em 2026-11-16. A 2, a 3, a 5 e a 6 seguiram a recomendação. A 1, a 4 e a 7
vieram com leitura, que mudou o desenho.

| # | Decisão | O que vale | Fonte | Como se testa | Quanto custa manter |
|---|---|---|---|---|---|
| 1 | Controle tático | **Objetivo por frente**, sendo a frente toda a fronteira contínua com um inimigo; o segmento só luta. A **geração do exército** (2ª ou 3ª) é estado dele, com causa, e muda como a frente se reparte e anda (seção "Doutrina") | Lind et al. 1989; Lind 1985; Lind 2004; Echevarria 2005 | repartição estável de uma semana para a outra; a 3ª concentra no ponto fraco rumo ao centro de gravidade; sem rádio ou sem combustível degrada para a 2ª; reserva atrás de frente preparada impede a ruptura | baixo: um comando por frente, regra de repartição única |
| 2 | Granularidade | **Elemento como conta, divisão como formação** (cerca de 10 elementos), frota e ala no mar e no ar | recomendação | `forces.ron` de 1925 dá cerca de 580 divisões no mundo; o passo semanal cabe no orçamento do tick | baixo: formação é só agrupamento |
| 3 | Mar com zonas próprias | **Zonas grossas** (cerca de 50, das bacias e estreitos), com portos ligando estados a zonas. Feito: 63 zonas e 24 estreitos e canais (`map-pipeline.pt-BR.md`, "Zonas marítimas") | recomendação | todo porto alcança todo porto por zonas; comércio e suprimento passam pelas mesmas zonas | médio: um passo novo no gerador do mapa |
| 4 | Arma nuclear | **Estoque e dissuasão na fase 5, uso numa fatia própria no fim dela.** A dissuasão é o segundo golpe medido, obstáculo a obstáculo, e entra na escada de crise (que degrau cada lado pode subir), não no combate (seção "Arma nuclear") | Wohlstetter 1958; Brodie 1946 e 1959; Schelling (RAND, 1958–59); Kahn 1960 e a escada de 1965 via Beal 1975; tratados do Departamento de Estado | segundo golpe com e sem alerta; defesa antimíssil quebra o segundo golpe do outro; simetria | médio: o uso pede tela e regra de resposta |
| 5 | Tipos e gerações | **33 tipos com gerações**, primeira fatia com os 22 de 1925–1945 | recomendação | nunca mais de 20 tipos em uso numa década | baixo: geração nova é linha de dado |
| 6 | Passo e segmento | **Uma semana por passo, rodando por segmento** (par de estados vizinhos em guerra); o comando fica na frente | recomendação | custo por sistema no `bf-bench`: a guerra mundial de 1939 com todos os segmentos abaixo de 15 % do tick | médio: segmentos ativos por semana |
| 7 | Perdas e custo | **Mortos e inválidos por idade na pirâmide**, civis também, e a **conta da guerra** sempre à vista: gente, bens, dinheiro e dívida, produção perdida e preços (seção "A conta da guerra") | Sun Tzu, cap. II, trad. Giles 1910 | a exaustão cresce com a duração mesmo sem derrota; contrafactual de 1914–18 na pirâmide francesa quando o PR #54 entrar | baixo: uma escrita por semana de guerra |

## Telas que isso vai pedir

Cada uma nasce como painel registrado no HUD modular, com atalho no registro de comandos.

- **Forças do país:** efetivo por domínio e tipo, geração, equipamento contra o necessário,
  treino, custo semanal; quanto da indústria vai para a guerra.
- **Mobilização:** em idade, chamáveis, reservados, disponíveis; efeito no trabalho por profissão.
- **Frentes no mapa:** linha da frente, postura, objetivo e centro de gravidade, poder dos dois
  lados, o segmento de foco e os contornados.
- **Doutrina:** a geração de cada exército, a prontidão termo a termo (educação dos oficiais,
  treino, rádio, motorização, experiência, menos a cultura da ordem) e o que a degrada nesta
  semana (falta de rádio ou de combustível).
- **Repartição da frente:** por segmento, extensão, poder inimigo, puxada do objetivo, peso e
  elementos recebidos.
- **Batalha do segmento:** a decomposição do passo (número, equipamento, treino, suprimento,
  terreno, ar) dos dois lados e as perdas da semana.
- **Suprimento:** o caminho até a frente trecho a trecho, o gargalo e o preenchimento.
- **Mar:** zonas, frotas, comboios, bloqueio e perda por zona.
- **Ar:** superioridade por zona, bombardeio e o que ele destruiu.
- **Conta da guerra:** por semana e acumulada, sempre à vista no painel do país em guerra: mortos
  e inválidos por idade, civis, bens consumidos, equipamento perdido, gasto e dívida, produção
  perdida e índice de preços; a mesma conta do inimigo, até onde a inteligência alcança.
- **Exaustão e paz:** os termos da exaustão dos dois lados, as exigências na mesa e a proposta.
- **Dissuasão:** por par de potências, o segundo golpe de cada lado obstáculo a obstáculo (por
  base: silo, submarino, bombardeiro, móvel), o dano inaceitável, quem é seguro, o teto na escada
  e o risco semanal de escalada; tratados de controle de armas em vigor e sua verificação.
- **Crise:** a escada com o degrau atual, o casus belli, a exigência, o prazo e quem tomou lado.
- **Lente de guerra no mapa:** ocupação, devastação, terreno de combate.

## Forças de 1º de janeiro de 1925

`data/scenario/1925/forces.ron`, gerado por `tools/war/forces_1925.py` (fontes, licenças e regras
de leitura em `tools/war/SOURCES.md`). Contagem: 44 países, 187 linhas, das quais 31 com
`estimate` e 156 com o número impresso na fonte; 40 países com exército (5,84 milhões de homens),
27 com marinha e 20 com aviação.

- **Fonte principal:** *Armaments Year-Book* da Liga das Nações, 1º ano (1924), domínio público;
  números de 1923-1924, os mais próximos que as fontes abertas imprimem. O Hansard (respostas do
  War Office e do Air Ministry) cobre exércitos e aviações que o Year-Book não totaliza; o
  *Brassey's* de 1925 e a Wikipedia, os navios das marinhas menores e a Reichsmarine.
- **Exército inteiro como `infantry`** em 1925: a fonte dá o efetivo, não a divisão por arma; a
  artilharia e a cavalaria orgânicas estão no equipamento do elemento. Tropas coloniais entram como
  `garrison` nas colônias.
- **Estimativas:** tetos de tratado tomados como efetivo (Alemanha, Hungria, Bulgária), números de
  outro ano (Pérsia 1926, México 1920, Irlanda pela meta de 1924) e a divisão por tipo das
  aviações que só têm total (França, Estados Unidos, Itália e outras cinco), pelas parcelas das
  aviações que a fonte divide (caça 25 %, bombardeio 14 %, reconhecimento 61 %).
- **Conferido no teste:** os navios capitais retidos pelo Tratado de Washington (18 americanos,
  22 britânicos, 10 japoneses) e Versalhes (Alemanha sem aviação nem submarino).

### Lacunas

- **Reserva e milícia:** entraram Suíça, Estados Unidos, Reino Unido, Canadá e Austrália (5
  linhas, a australiana estimada pelo estabelecimento). Falta a África do Sul, sem número no
  Year-Book, e a reserva dos exércitos de conscrição do continente: o Year-Book de 1924 não a
  totaliza.
- Sem número em fonte aberta: exército de Egito, Peru, Colômbia, Venezuela, Uruguai, Bolívia,
  Paraguai, Equador, Cuba, Sião, Afeganistão e Etiópia; aviação de URSS, Polônia, Romênia,
  Bélgica, Turquia, Brasil e Argentina; destróieres alemães.
- O *Armaments Year-Book* de 1925-26 daria números de 1924-25 e a divisão por arma; não foi achado
  em scan aberto.

## Dependências de outros donos

Funções que faltam e que a guerra lê, mas que pertencem a outro sistema (anotadas em 2026-11-19, na ordem do orquestrador):
- **Apoio ao movimento por comunidade** (independência): `support[unidade][comunidade]`, que sai da causa medida por comunidade, da distância de cultura e religião e da organização. Lido pela mobilização, pelos irregulares e pelas forças locais (GC6). A medida de 2026-11-19 mostrou que trocar isso por regra de religião ou de cultura erra na Argélia (berberes), na Malásia (chineses) e com os harkis.
- **Mineração e descoberta de urânio que respondem ao preço e à compra do Estado** (economia), no lugar da descoberta com data marcada.
- **IA estratégica, fase 6:**
  - a "Europa como refém";
  - a razão de forças no teatro (variante b da N10);
  - terceiros chamados à guerra colonial pelos acordos;
  - o orçamento militar que acompanha a ameaça;
  - o risco semanal de escalada e o custo de reputação do teste.
- **Expurgos** (eventos, fase 7).
- **Reassentamento** (GC7): espera a decisão do dono.

## Pedidos a outros arquivos

Nada disto foi acrescentado aqui: muda o hash da economia ou da árvore e colide com outros
trabalhos.

- **Bens (`goods.ron`):** `missiles` (míssil guiado e de cruzeiro, hoje coberto por `artillery` e
  `electronic_components`), `nuclear_warheads` (hoje `nuclear_fuel` mais `explosives`),
  `military_trucks` (hoje `automobiles`), `military_electronics` (radar, sonar, guerra eletrônica;
  hoje `electronic_components`), `satellites` (hoje `electronic_components` e `machinery`).
- **Tecnologias (`technologies.ron`):** `helicopter` (o tipo `helicopter` usa `gas_turbine` como
  substituto), `sonar` (as gerações antissubmarino usam `radar`), `amphibious_warfare` (o
  `transport` não tem geração de desembarque), `airborne_forces` (o `airborne` usa
  `stressed_skin_aircraft`), `aerial_refueling`.
- **Mapa:** feito. Terreno por estado (`map-pipeline.pt-BR.md`, "Terreno") e zonas marítimas
  (decisão 3).
- **Política:** grupo de leis de serviço militar com idade (hoje a idade fica em `rules.ron`);
  movimento de resistência à ocupação.

## Calibração prevista

### Antes de ligar: o roteiro da rodada de calibração

A ordem do dono é funcionalidade, otimização, calibração. As funções da guerra entram atrás das
suas chaves, desligadas; nenhuma calibração começa antes da rodada própria. Ela segue esta ordem,
e nenhuma chave liga antes de a anterior estar medida contra a faixa.

**O que já está medido.** A rodada de 101 anos com `war` e `crisis` ligadas (main `abd15c5`):
- nenhuma guerra começa: a IA não exige nem declara (fase 6);
- o mundo fica dentro do ruído da faixa (população, PIB, alfabetização);
- o gasto militar sobre o PIB sobe a 8 a 13 % em 2000, pelo teto do orçamento (item 1 abaixo).

**A ordem:**
1. **O orçamento militar em paz** (detalhe abaixo): o teto contra 1 a 3 % do PIB nos anos 1930.
   Depois dele, a `share` da encomenda da IA (item 11) sai de 0.
2. **A terceira geração alemã** (detalhe abaixo): rádio e `armored_warfare`; reabre
   `front_pace::france_1940_in_weeks`.
3. **Os dados das forças de 1925:** a reserva treinada dos países de conscrição (França, Itália,
   Polônia, URSS, Japão sem linha de reserva; seção "Reserva por coorte") e a cavalaria, que não
   existe nas forças de 1925 (o teste da Polônia contra a Tchecoslováquia a põe à mão).
4. **`mobile_share`** (item 12), **valor de desenho**: medir a concentração contra 1939–1940 depois
   da terceira geração.
5. **A reserva por coorte** (`reserve_cohorts`): `training_loss` e `reserve_years` são valores de
   desenho; a classe é o exército de terra sobre o prazo e dá a França em 7,07 milhões em 1939
   contra 4,56 (falta a parcela de conscritos, com página); a conferência mede, não fixa.
6. **O ritmo da frente:** a Polônia de 1939 a 210 km por semana, fora da faixa de 60 a 200 (teste
   ignorado com o motivo).

**As duas que seguram o PR de ligar a guerra** (o orquestrador, 2026-11-23):
1. **O tamanho do orçamento militar em paz.** O teto dá 5 a 7% do PIB, contra 1 a 3% reais nos
   anos 1930 (Eloranta, 2017, p. 122, tabela 2). A encomenda da IA (item 11) fica em `share` 0
   até ele ser recalibrado (seção "A IA encomenda navios e aviões").
2. **A Alemanha não chega à terceira geração até 1940.** A prontidão fica em ~450 a 480, contra
   650 (seção "Geração do exército").
   - O rádio é lento: a troca de geração do equipamento leva a Alemanha a 0,13 em 1941.
   - `armored_warfare` ainda está em Ciência em 1939; na história era industrial em 1936.
   - Sem isso, a França de 1940 não rompe (`front_pace::france_1940_in_weeks` ignorado com esse
     motivo).

- Manter as forças de 1925 custa, por semana, uma parcela do produto da indústria bélica de 1925 e
  do orçamento militar da ordem dos anos 1920 (2 a 5 % do PIB nas potências): o teste confere a
  ordem de grandeza quando o orçamento militar existir.
- A Primeira Guerra Mundial como teste de regressão: com as forças e a indústria de 1914 (não
  temos o cenário), as frentes do oeste param pela razão de poder perto de 1. Fica como referência
  qualitativa: a frente estática nasce da defesa em terreno preparado e do suprimento por
  ferrovia, não de uma regra de "trincheira".

## Forças em paz (no tick, fatia A)

O passo 1 da lista abaixo, e parte dos passos 2 e 3, já rodam. Código em `war/state.rs` e
`war/system.rs`; regras trocáveis em `data/military/peace.ron`.

**Estado salvo.** `GameWorld::war` guarda as formações de cada país: tipo, geração, elementos,
equipamento que tem, treino, colonial ou não, e se é de reservistas chamados. Guarda também a
reserva, a parcela do exército de paz e os pedidos da semana. O estado sai de `forces.ron` na
partida. Um save de antes das forças carrega: o `world.war` é o último campo, e o snapshot mais
curto é completado com listas vazias (`game::decode_world`). Sem forças, o estado não entra no hash.

**Sistema `forces`** (semanal, segunda-feira, antes da economia):
1. credita o que o Estado comprou na semana anterior ao equipamento que falta, a cada formação
   pelo quanto lhe falta;
2. sobe o treino e desgasta o equipamento;
3. chama a reserva ou a manda para casa;
4. uma vez por ano, ajusta o exército de paz;
5. grava os pedidos da semana;
6. no começo do mês, reparte os soldados pelas regiões.

| Peça | Como é na vida real | No jogo | Muda a simulação? |
|---|---|---|---|
| Soldados nos pops | o exército tira homens do trabalho; quem sai do serviço volta à profissão | os soldados de cada região são os homens das formações (decisão 1), repartidos pela população: os coloniais nas colônias, o resto na metrópole. O `Labour` contrata e devolve um décimo da diferença por mês, como já fazia com os servidores | sim: em 1926 o mundo tem 6,3 milhões de soldados trabalhando, contra 8,0 milhões pela regra antiga de 1 % da população |
| Tamanho em paz | muda devagar, por lei e por orçamento; o exército colonial (KNIL, exército da Índia) ficou de tamanho quase igual nas décadas de 1920 e 1930, fixado pela metrópole | cada janeiro, os homens em serviço em terra **na metrópole** seguem a parcela de 1925 sobre os homens em idade de serviço da metrópole, vezes a razão do efeito da lei de conscrição sobre o de 1925 (decisão 2); as formações coloniais ficam fora da regra e mantêm o tamanho; trocável em `peace.ron` | sim, devagar: a população cresce, o exército da metrópole acompanha |
| Treino | por coorte de recrutas; o treino se acumula durante o serviço | sobe 1/semanas-de-treino do tipo por semana até o completo; recruta novo entra sem treino e dilui a média da formação | só no combate (fatia C); a tela mostra o estoque |
| Mobilização | a reserva se apresenta em dias ou semanas, por classe, e sai do trabalho | comando (`Mobilize`, decisão de Agenda com dois passos): a cada semana se apresenta um quarto do que resta, no mínimo mil homens; os reservistas chegam com 70 % de treino e sem equipamento, que é comprado aos poucos. Desmobilizar devolve todos de uma vez | sim: tira homens do trabalho enquanto dura |
| Reposição de equipamento | gradual, pela produção e pelo orçamento | a falta de cada formação é comprada em 26 semanas (`replace_weeks`) | sim: é demanda à indústria bélica |
| Reserva de guerra | estoque em depósito (fuzis, munição, peças) do qual a mobilização tira o equipamento dos reservistas | o que sobra do teto vai ao depósito; quem falta equipamento saca dele primeiro | sim: a mobilização equipa os reservistas em semanas, não em meses |
| Troca de geração | unidade a unidade, ao longo de uma década ou mais, quando a indústria (ou o fornecedor estrangeiro) entrega | uma parcela do efetivo por semana, em `modernize_years` anos, enquanto a sobra do teto paga; a aba mostra cada geração em sua linha | sim: o fogo vem do equipamento |
| Desgaste | contínuo | 2 por mil do valor por semana, cerca de um décimo por ano | sim: é o que a compra repõe |

**Compras do Estado.** As cinco linhas militares da cesta do Estado (40 % dos bens) dão lugar ao
pedido das formações, e o teto do orçamento militar fica o de antes: a parte da cesta que era
militar, sobre o orçamento de bens que a lei dá. A parte civil (60 %) recebe o mesmo dinheiro de
antes. Dentro do teto, em ordem:
1. **Necessidade:** o consumo do catálogo vezes `upkeep_scale`, mais a reposição do que falta a
   cada formação (em `replace_weeks`). Passando do teto, tudo é cortado igual.
2. **Modernização:** com sobra, uma parcela de cada formação passa por semana à geração seguinte
   do tipo, em `modernize_years` anos para o efetivo inteiro (10, valor de desenho). Isso só vale
   quando o país faz a tecnologia da geração (estágio industrial) ou ela já se difundiu em algum
   lugar e pode ser comprada fora. Os homens que passam levam o equipamento e o treino. O que falta
   da geração nova entra como reposição nas semanas seguintes, e a passagem para quando a sobra não
   paga essa reposição.
3. **Reserva de guerra:** o que ainda sobra compra bens militares na proporção da cesta militar
   de antes (armas leves, munição, artilharia, navios, aviões) e vai para o depósito do país.
   As formações a que falta equipamento sacam dele antes de qualquer compra: os reservistas
   chamados, a parte do efetivo que passou à geração nova e o desgaste da semana. O depósito se
   gasta como o resto (2 por mil por semana) e aparece na aba Forças. As formações ficam no máximo
   com a dotação; antes do depósito, a infantaria britânica chegava a 12 vezes a dotação em um ano.

   **Por que na proporção da cesta, e não do equipamento das formações.** A indústria bélica de
   1925 está dimensionada para a cesta. Pelo valor do equipamento, a sobra iria quase toda para
   navios, porque as frotas valem mais que os exércitos, e pediria aos estaleiros o dobro do que
   fazem. Só 44 % do pedido de navios seria comprado, e o resto do dinheiro ficaria parado: na
   primeira rodada assim, os bens militares ficaram 26 % abaixo da `main`, e a dívida das grandes
   economias saiu da faixa.

O teto inteiro é gasto, como antes das forças: a demanda à indústria bélica e o orçamento ficam no
nível da `main`, agora com o que as forças de fato usam.

Dois acertos de unidade e de calibração:
- **Dólares de 1925.** O catálogo conta o equipamento em lotes de dólares de 1925, e a economia
  dá preço em dólares de 2011. As quantidades dos bens vendidos por lote são multiplicadas na carga
  pelo fator do CPI-U (12,85, o mesmo que `economy.pt-BR.md` usa para o automóvel); o peso e o fogo
  por lote são divididos pelo mesmo fator, então o combate e a logística não mudam.
- **`upkeep_scale` = 1.** Multiplica só o consumo em paz (munição de treino, combustível,
  peças), não o desgaste do equipamento, e fica no catálogo como escrito. O consumo de 1926 é cerca
  de 0,4 % do PIB do mundo, e o resto do teto vai para equipamento. Uma escala maior só trocaria
  equipamento por munição de treino, que a indústria de 1925 não faz em quantidade bastante.

**Rodada de 101 anos sem guerra** (`90fba4f`, sem guerra, contra a mesma partida com a chave
neutra). Os bens militares do mundo ficam no nível de antes em 1935 e 1975 (+0,5 % e +1,7 %), e
11,6 % acima em 2024. O PIB mundial fica dentro da faixa de reembaralhamento em 1935 e 2024 e sai
dela em 1975 (+3,2 %, contra 2σ de 2,5 %). A dívida de 5 a 6 das dez maiores economias sai da
faixa a cada data, nos dois sentidos, como já saía antes do depósito. A faixa por país tem só três
amostras, então é estreita demais para julgar um país sozinho. O nível alto de 1975 em diante é o
da cesta do Estado, não destas forças. Na vida real o gasto militar em paz das potências fica de
2 a 5 % do PIB, pago e equipamento juntos; recalibrar o teto é do equilíbrio fino, depois da
fase 6.

**Dependência anotada: o tamanho do orçamento militar.** O teto é a parte militar da cesta do
Estado, uma parcela fixa do orçamento de bens, e por isso cresce com o Estado ao longo do século
(7,5 % do PIB mundial em bens militares em 2024). Na vida real, o gasto militar em paz acompanha a
ameaça percebida e a lei de cada país: subiu na corrida armamentista dos anos 1930 e na Guerra
Fria e caiu depois de 1991. A ameaça percebida é da IA estratégica (fase 6), e a lei é o orçamento
de cada país (`politics`). Até lá o teto fica o herdado, sem calibrar. A série real para o placar
é a do SIPRI, via Banco Mundial (`MS.MIL.XPND.GD.ZS`, de 1960). Ela está em `fetch.py --only
military`, só local, porque os termos do SIPRI ainda não foram lidos (`data/history/SOURCES.md`,
"Left out").

**Por que o exército colonial fica fora da regra de janeiro.** Escalado pelos homens da
metrópole, o exército colonial crescia com a população da metrópole e tirava gente das colônias.
Com as forças ligadas, isso levou a Holanda a −64 % e a Índia britânica a −48 % do PIB em 2024,
contra a faixa de oito amostras da `main` (b2acc61). Com o exército colonial fixo, as duas ficam
em −1 % e −3 %, e o PIB mundial volta à faixa (+1,6 %, 2σ ±2,2 %). Experimentos E1 a E4 do PR de
ligar as forças.

**Chave: as forças ligadas.** As forças entraram desligadas (decisão de 2026-11-17: regra que
muda o hash de todos os países entra desligada) e foram ligadas por um PR próprio, julgado pelos
agregados contra a faixa de oito amostras da `main` (`03b00cb`, `tools/bench/band.sh`). A chave é
`forces` em `data/rules.ron`, agora `true` (`Switches::SHIPPED_ON`); desligada, os soldados seguem
a regra antiga do `peace.ron` (`soldiers: PopulationShare(10)`).

Rodada de 101 anos com as forças ligadas contra a faixa (base mais oito empurrões):

| Ano | PIB mundial | 2σ | População mundial | 2σ |
|---|---|---|---|---|
| 1935 | +0,78 % | ±0,83 | −0,01 % | ±0,01 |
| 1975 | −0,36 % | ±3,99 | −0,01 % | ±0,19 |
| 2015 | +2,32 % | ±3,74 | −0,39 % | ±0,66 |
| 2024 | +3,14 % | ±3,86 | −0,51 % | ±0,74 |

- **Dentro da faixa** em todas as décadas, no PIB e na população do mundo. Das doze maiores
  economias, só o Japão de 2024 sai (+6,3 % contra ±3,8).
- **Por território:** em 1925 metade sai, porque a faixa quase não abre no começo e as forças
  trocam os soldados dos pops no primeiro dia; de 1945 em diante, cerca de 9 % dos territórios
  saem a 2σ, contra 5 % por acaso.
- **O que muda de propósito:** os soldados são os homens das formações. São 6,4 milhões em 1926 e
  13 milhões em 2024, contra 8,0 e 35 milhões da regra antiga (1 % da população); a ordem de
  grandeza mundial real é de cerca de 20 milhões em armas.
- **Dependência anotada:** os bens militares ficam acima da regra antiga em 2024 (9,3 % contra
  8,2 % do PIB): o teto inteiro é gasto e o depósito compra na proporção da cesta. Os dois níveis
  vêm da cesta do Estado, alta demais diante dos 2 a 5 % reais; recalibrar o teto é da alavanca
  D3 e do equilíbrio fino, depois da fase 6.
- **O PIB a mais de 1975** que a faixa de três amostras acusava some com a faixa de oito (−0,4 %
  contra ±4,0). A queda do fim do século (Holanda, Índia britânica) vinha do exército colonial
  escalado pelos homens da metrópole, corrigido no #195.

## Reserva por coorte (item 9, atrás de `reserve_cohorts`, desligada)

Desenho do orquestrador (2026-11-23), com save 46. Código em `war/reserve.rs` e no passo 3 e 4 do
sistema `forces`; regra em `data/military/peace.ron`, `reserve_cohorts`. Desligada, a reserva é a
de 1925, parada, como antes.

| Peça | Como é na vida real | No jogo | Muda a simulação? |
|---|---|---|---|
| A classe que sai do serviço | todo ano a classe que cumpriu o prazo vai à reserva com o treino que tem, e outra entra | cada janeiro, os homens em serviço em terra na metrópole sobre os meses de serviço, vezes 12, entram na reserva como a coorte do ano anterior, com o treino médio das formações | sim: a reserva cresce onde há conscrição |
| O treino do reservista | cai fora do serviço; os cursos de reciclagem das leis dos anos 1920 mostram a perda, não o tamanho (Tchecoslováquia: 14 semanas no total, Armaments Year-Book 1924, p. 346) | cada coorte perde `training_loss` = 50 por mil por ano. **Valor de desenho**, sem fonte | só na mobilização: o reservista chega com o treino da sua coorte, não com 70 % |
| Quanto tempo na reserva | pela lei de cada país: França 28 anos, Polônia e URSS até os 40 | `countries` com página no Year-Book; sem número, `reserve_years` = 15 (**valor de desenho**); a coorte mais velha sai em janeiro | sim |
| Quem a mobilização chama | as classes mais novas primeiro | a coorte mais nova primeiro, cada uma com o seu treino; a formação chamada fica com a média ponderada | sim, na guerra |
| Desmobilizar | os homens voltam à reserva | voltam como a coorte do ano anterior, com o treino que têm | sim |
| A reserva e a economia | o reservista trabalha até ser chamado | igual: a reserva é uma conta, não sai da força de trabalho; quem é chamado sai, como antes | não, até a mobilização |
| O prazo do serviço | pela lei; a França passou de 1 para 2 anos em 1935 | `service_months` por país: o de 1925 com página, ou pelo efeito da lei (`law_months`: voluntário 0, seletivo 12, universal 24, **valores de desenho**). O jogador muda por `SetServiceMonths`, de 6 a 36 meses; a IA não muda (fase 6) | sim: o exército de paz cresce pela razão dos meses sobre os de 1925 |
| Prazo maior, gradual | a classe que ia sair fica mais um ano; o exército cresce ao longo do prazo novo | os meses em efeito sobem 12 por ano até o novo prazo, e a parte da classe que fica não vai à reserva naquele janeiro; encurtar vale no janeiro seguinte | sim |

**A reserva de 1925.** A reserva de `forces.ron` (onde há linha `reserve: true`) se divide em
coortes iguais pelos anos de reserva do país, a mais nova de 1924, o treino de cada uma caindo pela
perda por ano. A soma é a mesma (teste `the_reserve_is_the_sum_of_its_cohorts`). **Dependência:**
a maioria dos países de conscrição (França, Itália, Polônia, URSS, Japão) não tem linha de reserva
em 1925 e começa com a reserva vazia, que se enche classe a classe; o dado é da rodada de
calibração (o Year-Book dá as classes, não o total treinado).

**Conservação.** O que entra (a classe, a desmobilização) menos o que sai (a idade, a chamada) é a
variação da reserva, país a país (`men_in_and_out_balance`); a reserva do país é a soma das
coortes toda semana.

**O comando do jogador.** `SetServiceMonths { tag, months }`: o sistema o mantém entre 6 e 36.
A tela (Agendas, "Forças armadas em paz", "Prolongar o serviço…") vem do thread da interface no
mesmo trem, em dois passos e com ponto de restauração, e diz antes de confirmar quanto o exército
de paz cresce, em quantos anos (12 meses por ano) e quanto custa por ano. Até ela entrar, o comando
fica na lista de exceções de `player_commands`.

**Conferência de 1939 (medida, não meta).**
- França: 4.564.000 mobilizados em setembro de 1939 (Souvenir Français, *Les 100 de 1939*,
  pp. 2–3).
- Alemanha: cerca de 4,5 milhões (Vogel e Lieb, "Die Wehrmacht: Struktur, Entwicklung, Einsatz",
  bpb; **artigo sem página**).
- Classes alemãs de 1935: Reserve I e II abaixo dos 35 anos, Landwehr de 35 a 45 (TM-E 30-451,
  cap. I, seção VI, §2d: o manual americano, não a lei).

**Medido com a chave ligada** (`war_reserve::france_by_cohort_1925_to_1939`, fevereiro de cada ano,
sem a IA mudar o prazo):

| | 1925 | 1935 | 1939 |
|---|---|---|---|
| França, reserva (coortes) | 0 (0) | 4.453.198 (10) | 6.348.228 (14) |
| França, em terra na metrópole | 637.399 | 696.422 | 718.975 |
| França, mobilizável | 637.399 | 5.149.620 | 7.067.203 |
| Itália, mobilizável | 248.000 | 2.074.850 | 2.885.033 |
| Polônia, mobilizável | 276.375 | 1.860.822 | 2.590.636 |
| URSS, mobilizável | 530.000 | 3.561.674 | 4.972.114 |
| Alemanha, mobilizável | 100.000 | 96.843 | 94.539 |

França por coorte em 1939: de 479.316 homens (classe de 1938, treino 0,992) a 424.932 (classe de
1925, treino 0,514), cerca de 3,3 % a menos de treino por ano a mais fora.

**O que a medida mostra (para a calibração, não para corrigir aqui):**
- **A França passa da conferência:** 7,07 milhões mobilizáveis contra 4,56 milhões mobilizados em
  1939. A classe do jogo é todo o exército de terra sobre o prazo (465 mil por ano), e o exército
  de 1925 tem oficiais e engajados que não saem a cada 18 meses; a classe francesa real era da
  ordem da metade. Medida, não meta: entra no roteiro de calibração como a parcela de conscritos do
  exército de paz (dado com página a buscar).
- **A Alemanha fica em 95 mil:** a conscrição de 1935 é decisão de governo, e a IA não muda a lei
  nem o prazo (fase 6). Contra os 4,5 milhões de 1939, é dependência da IA, não erro da reserva.
- **A reserva de 1925 vazia** nos países de conscrição (dependência acima) faz 1925 subcontar.

**Neutralidade.** Desligada: o hash de 10 anos é o da main
(`71d55cda4ad2617e73f39398eeae582a5743f3a90ab85188ab7372f78657737b`) e a série de 101 anos é
idêntica à da main `ca718bf`. Ligada: o hash muda (a reserva entra no estado), e a série de 101 anos
continua idêntica à da main: em paz a reserva é uma conta e não tira ninguém do trabalho; só a
mobilização e o comando do jogador mexem na economia. CPU igual (442 s contra 438 s, ruído).

## Encomenda de forças (fatia A2, rascunho)

O jogador encomenda tropas, navios e aviões; o Estado paga com o teto militar, logo depois das
necessidades. Código em `war/system.rs` (`place_orders`, `deliver`), estado em
`CountryWar::queue`; prazos em `data/military/peace.ron`.

**Ordem dentro do teto:**
1. necessidades (consumo e reposição);
2. **encomendas**, na ordem em que foram feitas;
3. modernização;
4. reserva de guerra (depósito).

A encomenda consome a sobra que antes ia para o depósito. Com o teto curto, ela espera: a
encomenda não corta a reposição das formações que já existem.

| Peça | Como é na vida real | No jogo | Muda a simulação? |
|---|---|---|---|
| Divisão nova | chamam-se classes de recrutas ao longo de meses; o treino básico vem depois, e a divisão fica pronta em cerca de um ano | os homens se apresentam por classe: a cada semana 1/12 do pedido (`raise_weeks` = 12), sem treino e sem equipamento. O treino sobe pelas semanas do tipo, e o equipamento sai do depósito e da reposição | sim: tira homens do trabalho e pede equipamento à indústria |
| Exército encomendado e lei | o tamanho em paz segue a lei e o orçamento; uma expansão decidida fica | as formações encomendadas (`raised`) ficam fora da regra de janeiro, que só ajusta o exército de paz | sim: as tropas encomendadas não somem no ano seguinte |
| Navio | lançado no estaleiro, fica de 1 a 3 anos na carreira e entra em serviço inteiro (contratorpedeiro 1 ano, cruzador 2, encouraçado e porta-aviões 3) | o equipamento é comprado a cada semana, na parte do pedido inteiro sobre as semanas de construção (`build_weeks`), e fica "em construção". Os navios do pedido são construídos lado a lado e entram juntos, inteiros, ao fim das semanas de construção, com a tripulação sem treino | sim: é demanda aos estaleiros por anos, antes de haver navio |
| Avião | encomendado em lote, entregue em meses | comprado como o navio, em 26 semanas, mas sai da linha avião a avião | sim |
| Depósito | o estoque paga a encomenda, mas não apressa o estaleiro | o depósito paga primeiro a parte da semana; a construção leva o mesmo tempo | sim: gasta o estoque antes de comprar |
| Cancelar | o casco inacabado vira sucata ou estoque | o que foi construído vai ao depósito; os homens já chamados ficam | sim |
| Dissolver e dar baixa | a divisão desmobilizada manda os homens para a reserva e as armas para o depósito; o navio sai de serviço e vai para a frota de reserva | `DisbandForces` tira elementos inteiros de uma formação: o equipamento deles vai ao depósito, e os homens de uma formação encomendada vão à reserva. A formação do exército de paz volta ao tamanho de paz no janeiro seguinte; os reservistas chamados voltam por `Mobilize` | sim: menos soldados no trabalho e menos consumo; o depósito guarda o equipamento |
| Geração | o navio segue o projeto de quando foi lançado | a geração é a mais nova que o país sabe equipar na primeira semana paga, e fica | não |

**Gradual ou por coorte.** O soldado é gradual (classe por classe), e o navio e o avião são por
coorte: um navio pela metade não luta. A diferença importa, porque metade do dinheiro de um
encouraçado não dá meio encouraçado no mar. A aba mostra o estoque intermediário: as encomendas,
quantos elementos já foram entregues e quanto do equipamento está em construção.

**Hash.** Sem encomendas, o hash é o da fatia A: a fila vazia e o `raised` falso não entram no
hash.

**Dependência: o teto da alavanca D3.** Hoje o teto é a parte militar da cesta (fatia A). Quando a
alavanca das compras do Estado entrar (thread da demografia e do ensino), as encomendas passam a
ler o teto de `public::military_room(m)`, que o jogador decide, e mais nada muda aqui.

**Falta no rascunho:**
- a tela (lista de encomendas na aba Forças, com a encomenda e o cancelamento) e o teste de
  comandos do jogador;
- a rodada de 101 anos com encomendas de teste e a CPU.

## Composição das forças (atrás de `war`)

Os exércitos mudaram de composição devagar, divisão a divisão, e mesmo no auge a parcela
motorizada ou blindada ficou pequena:

| | 1939–40 | 1943–44 |
|---|---|---|
| Alemanha | ~14 % das divisões (6 panzer e 8 motorizadas ou ligeiras de 98 a 103) | 52 de 322 em nov/1943, ~16 % |
| França, mai/1940 | ~13–16 % (7 DIM, 3 DLM e 3 DCR, mais 5 DLC meio a cavalo, de ~83–94) | — |
| EUA | fim de 1942: 5 motorizadas e 14 blindadas de 73, ~26 % | 16 blindadas de 89, ~18 % |
| URSS | — | 6 exércitos de tanques; o cavalo ainda fazia cerca de metade do transporte |

Fontes:
- "Horses in World War II" (Wikipedia);
- ww2-weapons.com, ordens de batalha de 1º de setembro de 1939;
- "Cavalry Corps (France)" (Wikipedia) e littlewars.se;
- GlobalSecurity, US Army divisions WWII;
- hgw-davie, motor vehicles in the Red Army.

**A regra.** Uma vez por ano, em janeiro, a infantaria troca de equipamento rumo a uma parcela-alvo
dos homens de terra sobre rodas e em lagarta (`peace.ron`, `composition`). Os alvos por geração
de equipamento são valores de desenho; a calibração ajusta os números, e a estrutura fica:

| Geração | Sobre rodas | Em lagarta | De onde vem |
|---|---|---|---|
| 0, a de 1925 | 150 por mil | 50 por mil | dentro da faixa de 13 a 26 % das fontes acima |
| 1, armas em série | 300 por mil | 100 por mil | meio caminho |
| 2, mísseis guiados (de ~1960) | 650 por mil | 350 por mil | os exércitos grandes todos sobre rodas ou lagartas (as divisões ROAD de 1961 do exército americano, com a infantaria mecanizada) |

O alvo é o da geração mais nova do equipamento da infantaria que o país faz.

- **Ritmo.** No máximo 20 por mil dos homens de terra por ano, uma turma de equipamento, e só para
  os tipos que o país sabe equipar.
- **O exército não cresce.** Os homens saem da infantaria com o treino que tinham. O equipamento
  que o tipo novo usa vai junto, e o resto vai para o depósito.
- **Pagamento.** A conversão é paga com a sobra do teto militar, depois das necessidades e das
  encomendas, na mesma ordem da A2. Em guerra ela continua, se o teto paga, como a Alemanha e os
  EUA fizeram.
- **Quando falta.** Sem dinheiro ou sem veículo no mercado, a formação fica em conversão com
  equipamento parcial. A tela conta as semanas paradas e diz o motivo.
- **Decisão do jogador** (`GameCommand::SetComposition`, aba Forças): "Regra histórica" ou "Fixado
  pelo governo", com as duas parcelas. A mesma mecânica converte até o alvo fixado. A encomenda
  da A2 continua sendo o caminho para formação nova.
- **Duas contas.** O alvo conta os homens pelo tipo da formação, para não pedir duas vezes. A parcela
  que vale para a geração do exército e para a tela conta cada formação pelo equipamento que
  tem (`composition::equipped`): convertida sem caminhão, ela anda a pé.

**Fica para a fase 6:** a parcela-alvo responder à ameaça (corrida armamentista).

## A IA encomenda navios e aviões (item 11; desenho do orquestrador, 2026-11-23)

Em paz, só o exército de terra seguia os homens em idade. A frota e a aviação ficavam com os
elementos de 1925 e só trocavam de geração, e a sobra do teto militar comprava bens para o
depósito.

**A regra** (`war::ai_orders`, atrás de `war`):
- Cada janeiro, um país sem jogador transforma `share` de um ano da sobra do teto sobre as
  necessidades (o que iria ao depósito) em encomendas de navios e aviões. A fila é a A2, a
  mesma do jogador.
- **Mar e ar** dividem o dinheiro por uma regra só para todos. O mar pesa mais com a parcela
  dos estados da metrópole com porto e com a gente nas colônias (`sea_base`, `sea_coast`,
  `sea_colonies`, `air_base`). A tabela por país é calibração e fica fora.
- **Dentro de cada serviço,** a encomenda mantém a forma que a força tem: cada tipo pela sua
  parcela do valor do serviço, na geração mais nova que o país sabe equipar, sem encomenda nova
  de um tipo que já tem uma aberta.
- **Os tetos dos tratados em vigor** (`ai_orders::caps`) limitam o que limitam: navios capitais e
  porta-aviões por tonelagem (Washington, 1922) e homens (Versalhes, art. 160). Honrar é o padrão.
- **VALORES DE DESENHO, sem fonte.** Não há página aqui para a divisão do gasto militar por
  serviço.
- **`share` = 0 no dado:** o mecanismo, os tetos e a tela entram, e nada muda, até o teto militar
  ser recalibrado.
- **Tela:** as encomendas da IA aparecem na mesma lista das encomendas (`CountryWar::queue`), sem
  campo novo no save.

**Medido com `share` = 0,5, `war` ligada, 1939, sem a regra → com a regra** (`tests/war_ai_orders.rs`):

| País | Aviões | Navios (mil t) | Depósito (bi) | Gasto militar/PIB |
|---|---|---|---|---|
| Reino Unido | 577 → 144.581 | 1.376 → 2.230 | 169 → 28 | 6,5% → 6,0% |
| EUA | 750 → 324.264 | 1.499 → 5.192 | 607 → 56 | 6,8% → 4,6% |
| França | 1.280 → 134.542 | 551 → 1.296 | 124 → 23 | 5,7% → 5,3% |
| Japão | 548 → 59.910 | 652 → 1.070 | 79 → 12 | 6,0% → 5,5% |
| Alemanha | 0 → 0 | 304 → 1.652 | 140 → 122 | 7,1% → 7,5% |
| URSS | 0 → 0 | 132 → 822 | 86 → 63 | 5,5% → 5,3% |

- **Leitura: a regra está certa, o dinheiro não.** O teto militar em paz dá 5 a 7% do PIB. O real
  (Eloranta, *Essays in Economic & Business History* XXXV, 2017, p. 122, tabela 2):

  | Gasto militar/PIB, % | 1925 | 1930 | 1935 | 1938 |
  |---|---|---|---|---|
  | Reino Unido | 2,5 | 2,3 | 2,8 | 6,5 |
  | EUA | 0,8 | 0,9 | 1,3 | 1,5 |
  | França | 2,5 | 4,6 | 6,0 | 6,8 |
  | Japão | 2,8 | 3,0 | 5,7 | 22,7 |

  - Hoje essa sobra vira depósito parado. Transformada em forças, faz frotas e aviações de dezenas
    a centenas de vezes as reais.
  - A RAF tinha 1.911 aviões de primeira linha em 3 set. 1939 (Richards, *Royal Air Force
    1939–1945*, vol. I, 1953, ap. V, p. 410); o jogo dá ao Reino Unido 144.581, 76 vezes mais.
  - A Luftwaffe tinha 2.916 aviões de combate em set. 1939 (Murray, *Strategy for Defeat*, 1983,
    tabela XXI, p. 101).
  - A Royal Navy tinha 12 couraçados e cruzadores de batalha efetivos e 6 porta-aviões (Roskill,
    *The War at Sea*, vol. I, 1954, p. 50).
  - Para a tonelagem da Marinha dos EUA em 1939 não achei página.
- É a dependência "o tamanho do orçamento militar" (acima): a regra sobe quando o teto for
  recalibrado.
- **Dois defeitos achados e consertados antes do PR, só no código novo.** A encomenda media a sobra
  de uma semana de janeiro, e o produto de dois valores em bilhões estourava o `Fixed`. O caminho
  que já roda no main (necessidades, modernização, composição, depósito) não tem nenhum dos dois;
  o hash do main não muda.

**Dependências, com dono, para a calibração:**
1. **Versalhes, no dado:** só tem o teto do exército (100 mil homens). Os navios por contagem
   (art. 181) e a proibição de aviação e submarinos (arts. 191 e 198) estão só na nota da
   cláusula; a Alemanha chegou a 1.148 mil t de navios capitais. É do thread dos tratados.
2. **Couraçados britânicos em 1925:** o dado de forças dá 22 couraçados a 30 mil t, 660 mil t.
   O tratado manteve 22 navios capitais britânicos com 580.450 t (Five-Power Treaty, cap. II,
   parte 1, em *FRUS 1922*, vol. I, doc. 77, p. 254).
   - O teto de 525 mil t (art. IV, p. 250) é o da tonelagem de substituição. Lido como teto do
     total, ele dá o que o tratado deu: nenhum navio capital novo do Reino Unido até 1936.
   - A tonelagem por navio do dado (30 mil t, contra ~26,4 mil t em média) fica para o dado de
     forças conferir.
3. **URSS sem aviação em 1925 no dado:** ela tinha força aérea.
   - A Administração da Frota Aérea Vermelha é de maio de 1918, e em 1924 "one hundred combat
     aircraft were built and delivered to the air force" (Yakovlev, *Fifty Years of Soviet
     Aircraft Construction*, 1970; o scan não traz número de página).
   - O total de 1925 ficou sem página.
   - Com a regra mantendo a forma, a URSS nunca cria uma aviação. É do dado de forças.
   - A Alemanha em zero está certa, por Versalhes.

As três mudam o jogo de hoje, porque `forces` está ligada no main: ficam para a calibração.

**O motor da fase 6** (a opção (c), não feita): a IA repõe o que envelhece, por vida útil de cada
tipo, e só cresce por ameaça. Os anos de 1925 a 1935 foram de frotas estáveis sob os tratados. Pede
a idade das formações, que é campo novo no save.

## Crise e declaração (fatia B, desenho)

A guerra nasce de uma exigência recusada; não há declaração seca. A fatia B põe no estado a crise
e a guerra declarada, sem combate (o combate é a fatia C, e até lá a guerra só mobiliza, compra e
pesa no orçamento).

**Estado** (no fim de `WarState`, então o save anterior lê pelo mesmo degrau de bytes vazios):
- `Crisis`: quem exige, de quem, a exigência como cláusulas do catálogo de tratados (`treaty.rs`,
  as mesmas que a paz da fatia D executa por `treaty_exec`), prazo em semanas, degrau atual e
  quem já tomou lado;
- `War`: os dois lados (líder e quem entrou), data de início, a crise de origem e a exigência em
  jogo.

**Degraus.** Exigência → prazo correndo → resposta: aceitar (as cláusulas executam como tratado e
a crise fecha), recusar, ou deixar vencer, que conta como recusa → quem exigiu decide: declarar
guerra ou recuar (a crise fecha sem nada). Declarada a guerra, entra quem as cláusulas obrigam (abaixo,
"Terceiros"), e cada um decide na semana seguinte.

**IA mínima: só responde.** Ela nunca exige nem declara.
- Como alvo, pesa a força contra o que se pede (W3, dono, 2026-11-17). Força é a força em terra de
  quem exige sobre a sua, contando os terceiros que viriam pelas cláusulas. O que se pede é o peso
  das cláusulas exigidas sobre o total do país, medido pela avaliação de proposta dos tratados
  (população, PIB e território que a exigência leva, e se leva a capital). A razão exigida para
  ceder sobe com essa parcela, numa curva em dados. Uma exigência periférica cede a partir de 3×
  (valor de desenho). Uma exigência existencial (anexação total, fim do Estado, a capital, a
  maior parte da gente ou do território) nunca cede, qualquer que seja a razão. Casos:
  - Crimeia, 2014: parcela pequena e periférica diante de força muito maior, cedida sem guerra;
  - Israel, 1948: parcela existencial, recusada diante de força maior.
  Os dois viram testes de jogo: `a_peripheral_demand_by_a_threefold_force_is_yielded` e
  `an_annexation_is_refused_whatever_the_odds`.
- Quem exigiu e ouviu recusa recua.
- Cada termo aparece na tela da crise.

**Exigência sem motivo** (W1): permitida. Fica registrada como exigência sem casus belli, para a
reputação e para o canal de ameaça do thread de blocos (dormente hoje).

### Terceiros: quem entra, por qual cláusula (W4)

Decidido pelo dono em 2026-11-17: o aliado entra somente quando as cláusulas preveem o caso, e
aliança é mais do que defesa mútua. O vocabulário é o da literatura. **Aliança** é o gênero: um
acordo formal com alguma obrigação militar. As espécies são as do Correlates of War (Gibler 2009,
Formal Alliances v4.1):
- tipo I, **pacto de defesa**: ajudar com força se o outro for atacado;
- tipo IIa, **pacto de neutralidade**: não ajudar quem ataca o outro;
- tipo IIb, **pacto de não agressão**: não atacar o outro;
- tipo III, **entente**: consultar na crise.

As cláusulas são as obrigações do ATOP (Leeds e outros 2002, v5.1), cada uma com sua condição:
- **defesa**: entrar se o parceiro for atacado;
- **ofensiva**: entrar também na guerra que o parceiro começa;
- **neutralidade**: não ajudar quem ataca o parceiro (não entrar contra ele);
- **não agressão**: não atacar o parceiro;
- **consulta**: ser ouvido na crise.

Um tratado é um feixe dessas cláusulas, que pode ser unilateral: a garantia britânica à Polônia,
de 1939, é uma cláusula de defesa num sentido só. O tipo do tratado sai do feixe pela regra do
COW, e o thread de blocos rotula o bloco militar pelo mesmo feixe. Em `clauses.ron` (do thread
dos tratados) são `defence`, `offence`, `neutrality`, `non_aggression` e `consultation`.

Na declaração, para cada terceiro, a regra lê o feixe que ele tem com cada lado:
- tem **defesa** com o lado atacado (o que recebeu a declaração): está obrigado a entrar;
- tem **ofensiva** com o lado que declarou: está obrigado a entrar;
- tem **neutralidade** com um lado: não pode entrar contra ele;
- a **consulta** dá aviso na crise, sem obrigação de entrar.

A tela da crise mostra, antes de declarar, quem entraria e por qual cláusula.

**No tick (W4, com `treaties_in_force`, `forces` e `crisis` ligadas).** Na declaração,
`war::third::on_declaration` lê `TreatyState::duties(quem declarou, quem recebeu)` (contrato do
#181, por sigla):
- **defesa ou garantia de fronteira** chama o terceiro para o lado atacado; **ofensiva** com as
  partes nomeadas chama para o lado de quem declarou. Chamado para os dois lados, o terceiro
  atende só à defesa: a defesa vem antes da ofensiva;
- **neutralidade** com um lado impede de entrar contra ele (a chamada nem é feita);
- **não agressão** do próprio agressor é quebrada no ato: `record_breach(..., Attacked { war })`;
- a cláusula **suspensa** (o parceiro quebrou o acordo antes, `suspended`) não chama nem custa;
- a **IA** chamada entra na hora e chama a reserva (a escolha dela é da fase 6);
- o **jogador** recebe uma chamada (`Conflicts::calls`) com `call_weeks` semanas (2) para
  `AnswerCall { honour }`. Honrar entra na guerra; não honrar grava a quebra
  (`record_breach(..., CallUnanswered { war })`), cujo valor é o custo de um ano das forças dele
  aos preços da semana: a ajuda que ele deixou de dar. O prazo vencido honra
  (P2, decidido pelo dono; `lapsed_call_honours` em `crisis.ron`).

A tela lê `war::third::who_would_join(world, quem declara, quem recebe)`, a mesma conta, linha a
linha: quem, por qual acordo e cláusula, de que lado, e se está suspenso ou barrado.

O motivo da crise (`Crisis::casus_belli`) passou a ser a causa registrada pelos tratados contra o
alvo (`TreatyState::cause`), a do registro de quebras.

| Caso | Como foi | Como o jogo lê | Teste |
|---|---|---|---|
| Itália, 1914 | A Tríplice Aliança (1882) era defensiva. A Itália declarou neutralidade e tratou a Áustria-Hungria como agressora: não houve *casus foederis* | só defesa com a Áustria, e a Áustria declarou: a Itália não é obrigada | `a_defence_pact_does_not_bind_when_the_ally_attacks` |
| Pacto de Aço, 1939 | o art. III obriga a ficar ao lado do parceiro em qualquer "complicação militar", sem perguntar quem começou | ofensiva mais defesa: a Itália é obrigada quando a Alemanha declara | `the_pact_of_steel_binds_in_any_war` |

### Honrar ou trair (dono, 2026-11-17)

Quebrar o tratado é possível, e o preço é muito alto. Quando a cláusula chama o país, o jogador
recebe uma entrada no diário com prazo e duas saídas, honrar ou não honrar. É uma decisão rara, com
dois passos e ponto de restauração. Na fatia B, a IA honra sempre. A escolha da IA é da fase 6 e vai
pesar os mesmos termos do preço, mais a força relativa e o estado de preparo (Itália em 1939).

#### O preço de trair: desenho para o dono

Não honrar é uma **quebra da cláusula de defesa** na regra que os tratados já têm
(`treaties.pt-BR.md`, seção 9). A guerra não cria um sistema de punição próprio: ela chama a regra
de quebra com o valor da cláusula. O que entra:

| Peça | O que faz | De onde vem | Medida |
|---|---|---|---|
| Registro | data, cláusula, guerra que a chamou | tratados, 9.1 | — |
| Suspensão recíproca | o traído pode suspender o que dava em troca (sua defesa, base, acesso) sem quebrar | tratados, 9.2 (Viena, art. 60) | as cláusulas do mesmo acordo |
| Causa | o traído ganha causa contra quem traiu | tratados, 9.3 | o valor da cláusula: a força que ele deixou de receber sobre a ameaça (termo "Segurança") |
| Reputação | sobe pelo valor da quebra e cai 10 % ao ano; desconta as promessas de quem traiu em toda mesa futura | tratados, 9.4 e termo "Confiança" | o mesmo valor |
| Boa vontade | perda com o traído e, pela metade, com cada país que tem defesa com o traído | influência, canal "Cumprimento" | o mesmo valor, meia-vida de 8 anos |
| Legitimidade em casa | cada grupo de interesse soma ou tira conforme a sua posição sobre a guerra | política | peso do grupo × posição |

**Atenuante sem regra nova.** Se o parceiro quebrou antes uma cláusula do mesmo acordo (a
consulta, na Itália de 1939), a defesa já está suspensa pela 9.2: não honrar não é quebra e não
custa reputação. Fica só a boa vontade com o parceiro, que perde pela metade.

**Fora do caso não é traição.** Se a cláusula não cobre a guerra (só defesa, e o parceiro atacou:
Itália, 1914), não há chamada nem preço.

**Recomendação: reputação de segurança separada da de dinheiro.** Hoje os tratados têm uma
reputação por país, e a dívida tem a sua (`default_history`). Recomendo que a quebra de cláusula
militar (defesa, ofensiva, neutralidade, não agressão) suba uma **reputação de segurança** própria,
lida pela mesa só quando a proposta tem cláusula militar. A quebra de pagamento, tarifa ou capital
fica na reputação geral. Dois motivos:
- é o que a literatura mede: Gibler (2008) acha que o Estado que abandona aliado forma menos
  alianças depois; Tomz (2007) acha a reputação do devedor ligada ao pagamento, não às alianças;
- no jogo, a França de 1938 não deve pagar juros mais altos por Munique, mas deve achar menos
  quem prometa defendê-la.

**Alternativa: uma reputação só**, como os tratados já desenham. É mais simples (um número, um
termo na mesa), mas mistura a dívida com a guerra, e o país que trai um aliado passa a pagar mais
pelo crédito.

**Combinado com o thread dos tratados (2026-11-18).**
- `Reputation { general, security }` por país, as duas subindo pelo valor da quebra e caindo 10 %
  ao ano (`Weights::reputation_decay`).
- Sobem `security`: `defence`, `non_aggression`, `consultation`, `border_guarantee`,
  `troops_quota`, e as duas cláusulas novas, `offence` e `neutrality`. As outras cláusulas sobem
  `general`, e a dívida segue em `default_history`.
- Na mesa, o termo "Confiança" de cada cláusula lê a reputação da sua espécie: uma proposta mista
  desconta cada cláusula pela sua, e não a proposta inteira por uma só.
- A guerra grava a quebra por `TreatyState::record_breach(agreement, clause, by, value, date,
  BreachCause::CallUnanswered { war })`, que faz 9.1, 9.3 e 9.4. Antes ela pergunta
  `TreatyState::suspended(agreement, clause, by)`: se der verdadeiro, a defesa já estava suspensa
  e não há quebra.
- A boa vontade fica com a guerra, pela influência.
- O contrato entra na fatia dos tratados depois do #158. A fatia B desenha contra essa
  assinatura e a chama quando ela estiver no main.

**Decidido pelo dono (P1–P3, 2026-11-18), com as recomendações:**
1. **P1, reputação:** duas, `security` e `general`. A reputação é dos tratados; a guerra só grava
   a quebra.
2. **P2, prazo da chamada vencido:** conta como honrar. "Trair é decisão, honrar é o natural":
   não honrar é um comando explícito do jogador na entrada da chamada, decisão rara com
   confirmação em dois passos e ponto de restauração; sem resposta, o país entra na guerra.
3. **P3, troca de regime:** corta a reputação de segurança pela metade (tratados).

**Medida de controle.** Na literatura, as alianças são honradas em cerca de 75 % dos casos em que
a cláusula as chama (Leeds, Long e Mitchell, 2000). Quando a IA decidir (fase 6), a taxa de
honrar nas rodadas longas deve ficar perto disso; até lá a IA honra sempre.

| Caso | O que houve | O custo, pelas fontes | Para o jogo |
|---|---|---|---|
| Itália, 1914 | neutralidade apesar da Tríplice, e em 1915 guerra do outro lado (Tratado de Londres) | pouco imediato, porque o texto a cobria; a fama de parceiro incerto ficou | não é traição: a cláusula não cobria o caso |
| Itália, 1939 | não beligerância apesar do Pacto de Aço; Roma alegou a quebra alemã da consulta e o despreparo; entrou em junho de 1940 | o pacto sobreviveu; a Itália perdeu peso junto à Alemanha | traição atenuada quando o outro quebrou a consulta antes |
| França, 1938 | Munique: a França não honrou o tratado com a Tchecoslováquia | Moscou concluiu que não podia contar com o Ocidente, o que preparou o pacto de 1939; os pequenos aliados viram que o tratado não protegia | o caso de referência do preço: reputação e valor dos pactos dos outros |

Fontes:
- Gibler, D. (2008). The Costs of Reneging: Reputation and Alliance Formation. *Journal of Conflict
  Resolution* 52(3).
- Tomz, M. (2007). *Reputation and International Cooperation: Sovereign Debt across Three
  Centuries*. Princeton.
- Leeds, B. A., Long, A. G. e Mitchell, S. M. (2000). Reevaluating Alliance Reliability. *Journal
  of Conflict Resolution* 44(5).
- [COW Formal Alliances v4.1](https://correlatesofwar.org/data-sets/formal-alliances)
- [ATOP, via QoG](https://datafinder.qog.gu.se/dataset/atop)
- [Tríplice Aliança](https://en.wikipedia.org/wiki/Triple_Alliance_(1882))
- [1914-1918 Online, Itália](https://encyclopedia.1914-1918-online.net/article/london_treaty_of_1915)
- [Pacto de Aço](https://en.wikipedia.org/wiki/Pact_of_Steel)
- [Munique](https://en.wikipedia.com/wiki/Munich_Pact)

O valor de cada termo vem da regra de quebra dos tratados; a guerra só fornece o valor da cláusula.

**Telas.** Exigir e declarar são decisões raras em Agendas, com confirmação em dois passos e
ponto de restauração (`Reason::War`) antes de cada uma. A crise vira uma entrada do diário com:
- o prazo em barra;
- a exigência e a parcela do país que ela leva;
- a resposta esperada da IA, com seus termos;
- quem entraria e por qual cláusula.
Quem recebe a exigência tem "aceitar" e "recusar" na mesma entrada. Quem é chamado por cláusula
tem "honrar" e "não honrar", com o preço à vista.

**Decisões do dono (2026-11-17):**
- W1: exigência sem motivo é permitida, com registro;
- W2: prazo padrão de 4 semanas, em dados;
- W3: a força pesa contra a parcela pedida; o existencial nunca cede;
- W4: entra quem a cláusula obriga; trair é possível e caro.

## A guerra no tick (fatia C, desenho)

A fatia C põe no tick a guerra declarada pela crise (fatia B) e a guerra colonial que nasce da
revolta (decisão I3 do dono, opção A). Só terra nesta fatia: o termo "ar" do combate em terra fica
neutro, e mar e ar entram na fatia C2. Chave `war` em `data/rules.ron`, desligada; precisa de
`forces`. Sem guerra declarada, nada muda e o hash é o da `main`.

### Estado

Guardado e no hash só quando há guerra:
- **controle por estado:** quem ocupa cada estado. A ocupação muda o controle, não o dono; o dono
  muda só pela paz (`treaty_exec`);
- **por frente:** objetivo (um estado inimigo ou nenhum) e postura (atacar, manter, recuar),
  dados pelo jogador ou pela IA;
- **por segmento:** avanço acumulado em km;
- **por guerra:** a conta (`WarLedger`) e a exaustão de cada lado;
- a data do último registro de revolta lido (guerra colonial, abaixo).

A frente e o segmento não são estado: saem toda semana da vizinhança dos estados, pelo controle.

### Passo semanal

No sistema das forças, depois do passo de paz, para cada guerra:
1. **frentes:** toda fronteira contínua de terra entre estados controlados por lados opostos;
   cada par de estados vizinhos é um segmento;
2. **repartição:** os elementos de terra de cada país vão às frentes pelo peso de cada frente, e
   dentro da frente aos segmentos por `allocate` (regra aberta, decisão 1);
3. **combate:** `land_step` por segmento, com o terreno de `state_terrain`, o suprimento de
   `supply` desde o depósito e a fortificação do defensor; a decomposição fica para a tela;
4. **perdas:** os elementos perdidos saem das formações; os mortos e os inválidos saem da
   pirâmide, na faixa de serviço (decisão 7); o equipamento perdido volta como reposição;
5. **avanço:** o avanço de cada segmento soma; passada a profundidade do estado (raio da área),
   o controle muda para o atacante e a frente se refaz na semana seguinte;
6. **ocupação:** o ocupado paga ao ocupante, com guarnição e devastação (G3, abaixo);
7. **conta, exaustão e apoio:** `WarLedger` soma gente, bens, dinheiro, produção perdida e preços;
   `exhaustion` dá a exaustão material; o apoio à guerra (G4) decide a paz.

### Fim

Quem decide a paz e a rendição é o **apoio à guerra** (G4, abaixo), não a exaustão.
- **Paz negociada:** o lado cujo apoio cai abaixo do limiar de paz oferece parte da exigência da
  crise; o outro aceita quando o que ainda pode tomar vale menos que a oferta. O jogador propõe e
  responde em Agendas.
- **Rendição:** com o apoio no fundo, o lado aceita a exigência inteira.
- Nos dois casos a paz é um `treaty::Proposal` executado pela mesa de tratados (`settle`, `queue`),
  e o acordo fica em `agreements` com as cláusulas contínuas (ocupação, limite de armas), testadas
  todo mês. A guerra termina com o acordo gravado.
- **A exaustão material fica** como insumo do apoio (privação, gasto) e como limite físico: sem
  homens, munição ou combustível, a frente não ataca, por mais apoio que haja.
- **Terceiros no meio da guerra:** quando um país entra ou é atacado durante a guerra, as chamadas
  dos acordos dele seguem a regra W4 (P2: sem resposta, entra; não honrar é comando explícito).

### Guerra colonial (I3, opção A; G6)

Combinado com o thread das fronteiras (seção 11 de `borders.pt-BR.md`, "A revolta e a guerra"):
a revolta é uma **proclamação seguida de guerra**, como a da Indonésia de 1945 a 1949. Está
atrás de `war`. Com `war` desligada, a revolta segue como antes, uma separação sem acordo, e o
hash não muda. O código está em `crates/bf-sim/src/war/colonial.rs`, os números em `colonial`
de `data/military/war.ron`.

**Partição na revolta.** A independência continua decidindo a revolta com a regra do thread de
fronteiras: recusa e a mobilização passando a marca. O que muda é o `release(How::Revolt)`: ele
passa ao `Secede` só os estados que a guarnição não segura.
- A guarnição são os soldados da metrópole nas regiões da unidade (`war.soldiers`).
- Cada estado custa 2 homens por mil habitantes, o mínimo de O3 para população calma (Quinlivan,
  1995). O dono fica primeiro com os estados com porto e depois com os mais populosos, enquanto a
  guarnição pagar.
- **O primeiro porto fica sempre com a metrópole,** mesmo sem guarnição que pague: é o mais
  populoso, onde estão a administração e a frota (`first_port`). Sem ele, a Malásia de 1925
  proclamava inteira, e o Reino Unido não tinha por onde desembarcar. A tela da recusa avisa: "você
  mantém ao menos o porto de X".
- O movimento leva sempre ao menos um estado: o último nessa ordem.
- Na Argélia de 1925, a guarnição segura Argel, Constantina e Orã, e o Saara (`DZA-SUD`) proclama.
  A granularidade por estado é grossa: a guerrilha que nasce no interior de um estado segurado
  pela costa (o Aurès, em Constantina) depende da fatia seguinte, a guerrilha por guarnição
  faltante.

**Gatilho.** Toda segunda-feira o sistema de forças lê os `Record` de revolta dos sete dias
anteriores e abre a guerra pelo `open_war`, extraído de `crisis_decide`.
- Lado A é a metrópole, com a exigência `Union`. Lado B é o país novo.
- A abertura não chama terceiros pelos acordos. Eles só leem a guerra; a entrada deles fica para
  a fase 6.
- A guerra fica registrada em `WarState::colonial`: a unidade, os estados mantidos, os homens
  desembarcados e se a metrópole chega por terra. Save 34.
- O `Record` da independência não mudou de formato.

**Teto da expedição.** Nos segmentos contra a colônia, a metrópole luta só com a guarnição que
ainda tem na unidade mais os homens desembarcados. O resto do exército fica nas outras frentes.
- A cada semana desembarcam `men_per_port_week` (1.000) homens por porto que ela segura e
  controla na unidade, até `expedition_cap` (500‰) dos seus homens de terra.
- Fonte (Horne, 1977): na Argélia, cerca de 80 mil homens em novembro de 1954 e cerca de
  400 mil em 1956–57, por Argel, Orã e Bône. São 320 mil em dois anos, uns 3.000 por semana, cerca
  de 1.000 por porto.
- 500‰ é a metade do exército. Os 400 mil da Argélia exigiram a convocação dos reservistas e o
  serviço de 27 meses (1956). No jogo, a França de 1925 já mobilizada tem 742 mil homens de terra,
  e o teto fica em 371 mil, abaixo dos 400 mil. A diferença é **mobilização que falta** (a lei de
  serviço em guerra, o prazo de serviço maior), não um teto a inflar.
- Colônia alcançável por terra (busca sobre `neighbours` a partir da metrópole) não tem teto.
- Sem porto, não chega reforço.
- **Quanto mandar.**
  - A IA da metrópole dosa: manda até cobrir 20 por mil da população que segura na unidade (O3,
    população hostil; `cover_target` 1000), nunca acima do teto.
  - O jogador decide com `GameCommand::SetExpedition`, na janela "Guerra", seção "Expedição a X".
    Ele dá a ordem em passos de 10 mil ou volta à regra.
  - Os homens desembarcam, ou voltam, pelos portos, no ritmo de cada semana.
  - Sem a dosagem, a IA desembarcava sem critério até o teto: o Reino Unido punha 217 mil na
    Malásia.

**Irregulares.** O país novo luta com a unidade `irregular`.
- Combatentes = apoio à guerra do país × `armed_share` (20‰) × homens em idade de serviço de
  **toda a unidade**, inclusive dos estados que a metrópole segura: o movimento recruta lá também.
- **Mobilização e apoio são grandezas diferentes.**
  - A mobilização é a parte do povo que apoia o movimento antes da proclamação. É ela que decide
    se há exigência e revolta (independência).
  - Depois da proclamação, o movimento é um Estado em guerra. A parte que luta e recruta passa a
    ser o apoio à guerra dele (G4), que cai com as baixas e com a duração como o de qualquer país.
    Por isso o apoio substitui a mobilização no alvo de combatentes.
  - A marca da revolta (200‰) dava 8 mil na Argélia, abaixo da fonte. O apoio, que começa em 800,
    dá o número da fonte.
- Começam com `initial_share` (100‰) dos combatentes e metade das armas (GC3).
- A cada semana recrutam `recruit_week` (25‰) da distância até os combatentes, freados pela
  parte armada: sem armas, os recrutas esperam. As armas chegam pelas compras e pelo depósito da
  fatia A2, ou seja, pelo orçamento.
- Por GC1, as formações coloniais da metrópole passam para o país novo na proporção da
  população colonial que a unidade tem vezes a mobilização.

| Conferência (absoluta) | Jogo | Fonte |
|---|---|---|
| Argélia, combatentes com apoio 800 | 21.198 com a gente de 1925; ×1,6 até 1954 dá cerca de 34 mil | ALN, 15 a 40 mil (NWC, DTIC ADA422710) |
| Malásia, idem | 13.403 com a gente de 1925 | MNLA, 5 a 10 mil, pico de 7 a 8 mil (IWM) |

A Malásia passa porque o jogo não separa a base étnica. O MNLA recrutava quase só na comunidade
chinesa, cerca de 38%. Com essa parte, ficaria por volta de 5 mil. Fica anotado para a
calibração; não é meta.

**Guerrilha.** Perder a frente não extingue o movimento. Nos estados da unidade que a metrópole
segura ou controla, toda semana:
- Os armados dali, isto é, os armados vezes a parte da população da unidade que vive lá,
  emboscam. A metrópole perde `ambush` (5‰) deles por semana em baixas.
- Os irregulares perdem as baixas da metrópole vezes a razão de troca, que é 2 + 1,6 × a
  cobertura.
  - Cobertura: os homens da metrópole na unidade sobre 20 por mil da população que ela segura lá
    (O3, população hostil).
  - A reta passa pelos dois casos: Malásia, troca ≈ 3,6 com cobertura ≈ 1; Argélia, troca ≈ 5,5
    com cobertura ≈ 2,2.
- A taxa de emboscada é um valor de desenho entre a Argélia (≈ 10‰) e a Malásia (≈ 2‰). A
  dispersão é de seis vezes, e fica anotada.
- As baixas entram nas perdas que o apoio lê (G4) e são enterradas.
- A guarnição e os desembarcados ficam **amarrados** na unidade (`Colonial::tied`), e a série
  mostra quanto do exército isso é.
- Enquanto tiver armados, o país novo não é "overrun", e o território perdido não pesa no apoio
  dele: a guerrilha não se mede por terra segurada.

**Apoio.** A metrópole começa em `support.colonial` (550), o valor do arquivo que até agora não
era lido. O país novo começa em 800 (GC2, o mesmo de um país atacado). O resto do apoio segue a
regra G4.

**Fins.** São dois, e cada um tem o seu teste em `tests/war_colonial.rs`:
- **A metrópole desiste** (apoio < 300, o caminho `withdrawn`). Os estados que ela ainda tem na
  unidade passam ao país novo por `Transfer`, sem deixar enclave, e ela reconhece o país novo.
- **A colônia perde** quando o apoio dela cai abaixo de 300 **ou** quando os armados chegam a
  zero. Ela não é mais tomada por território. A `Union` sai pela ponte dos tratados (`treaty_exec`), sem
  passar pela mesa. A IA da metrópole avaliava a união do Saara como negativa e recusava a paz
  pela qual lutava.
  - Se a metrópole é do jogador, a proposta vai à mesa como numa guerra comum e precisa de
    `negotiation`.
  - Uma semana depois, as regiões da unidade voltam a ser colônia. O movimento recomeça em
    `movement_after_reconquest` (500‰) da marca da revolta, ou seja, 100‰ (GC5, em dado).
- Nos dois testes, no dia em que a fronteira muda, a população e o dinheiro do mundo fecham
  iguais e nenhuma região fica sem dono vivo.

**Conferências até o fim (G6), série por ano.** A revolta é posta à mão em fevereiro de 1925
(`war_colonial.rs`, testes `#[ignore]`, rodados no slot). A IA dosa a expedição à cobertura O3.
As colunas são: apoio dos dois lados, regiões controladas (metrópole/país novo), armados,
desembarcados, homens amarrados (e a parcela do exército, em ‰) e baixas acumuladas.

*Argélia (FRA × DZA)*:
- Sem G8, a França desiste em set/1929, depois de 4,6 anos: perto da faixa de 5 a 8 anos.
- **Com G8 (fator 3), desiste em jan/1931, depois de 5,9 anos: dentro da faixa.**

| data | apoio FRA/DZA | regiões | armados | desembarcados | amarrados (‰) | baixas FRA/DZA |
|---|---|---|---|---|---|---|
| 1926-02 | 587 / 492 | 4/0 | 7.455 | 135.222 | 143.616 (193) | 2.547 / 7.248 |
| 1927-02 | 458 / 516 | 4/0 | 4.220 | 137.312 | 145.724 (196) | 4.102 / 12.821 |
| 1928-02 | 405 / 476 | 4/0 | 5.664 | 139.447 | 147.875 (199) | 5.426 / 17.564 |
| 1929-02 | 362 / 439 | 4/0 | 5.689 | 141.620 | 150.064 (202) | 6.939 / 22.988 |
| 1930-02 | 328 / 401 | 4/0 | 5.494 | 143.836 | 152.297 (205) | 8.382 / 28.156 |
| 1931-01 | desiste | 0/4 | 4.777 | — | — | 9.570 / 32.417 |

*Malásia (GBR × MYS)*, com o porto mantido em MYS-FMS:
- Sem G8, o Reino Unido desiste em nov/1929 (4,8 anos); com G8 (fator 3), em out/1930
  (5,7 anos).
- **O critério de mais de 10 anos não passa:** fica como dependência anotada, não como meta.

| data | apoio GBR/MYS | regiões | armados | desembarcados | amarrados (‰) | baixas GBR/MYS |
|---|---|---|---|---|---|---|
| 1926-02 | 562 / 570 | 5/0 | 5.655 | 53.000 | 53.073 (121) | 1.178 / 3.244 |
| 1927-02 | 463 / 535 | 5/0 | 5.562 | 93.849 | 93.922 (215) | 2.666 / 8.186 |
| 1928-02 | 411 / 493 | 5/0 | 5.175 | 95.661 | 95.735 (219) | 4.054 / 13.161 |
| 1929-02 | 359 / 457 | 5/0 | 4.864 | 97.565 | 97.639 (224) | 5.352 / 17.824 |
| 1930-02 | 321 / 421 | 5/0 | 4.582 | 99.515 | 99.589 (228) | 6.574 / 22.192 |
| 1930-10 | desiste | 0/5 | 4.467 | — | — | 7.333 / 24.911 |

O que as séries mostram:
- A frente acaba no primeiro ano: o país novo perde o território. A guerra continua como guerrilha,
  com os armados entre 4 e 7 mil.
- A metrópole perde primeiro porque começa mais perto do limiar: 550 contra 800. Os dois lados
  caem por baixas e por duração em ritmos parecidos.
- Os armados nunca chegam a zero. O recrutamento (25‰ da distância por semana) repõe as perdas,
  e o equilíbrio fica em uns 60% do alvo.
- Na Malásia, a cobertura O3 só se faz com homens da metrópole, cerca de 100 mil (22% do exército
  de terra). Na história eram uns 40 mil britânicos mais as forças locais. Por isso a guerra pesa
  e custa baixas de mais.

**Forças locais e a política contra a base do movimento (desenho para a fatia depois do C2).**
É uma fatia só, com a guerrilha por falta de guarnição fora da guerra colonial (a ocupação comum).

*O que conta como força local:*
- polícia e guarda doméstica recrutadas na própria unidade, ou no território ocupado;
- tropa nativa regular sob comando da metrópole.

Casos, com fonte:
- **Malásia** (Komer, *The Malayan Emergency in Retrospect*, RAND R-957-ARPA, 1972):
  - polícia regular: 10 mil em 1948, cerca de 28 mil no fim de 1952, dos quais 2.488 chineses, "mostly Malay" (p. 38);
  - policiais especiais: 29.700 no fim de 1948 e 40 mil no fim de 1951; polícia auxiliar: 47 mil no fim de 1949 (p. 40);
  - guarda doméstica: 99 mil no fim de 1951. Em 1953, cerca de 50 mil chineses e 100 mil malaios e aborígines, com **um terço armado**; 152 mil no fim de 1955 (pp. 40–41);
  - o partido comunista vinha "almost exclusively" da minoria chinesa, cerca de 38% da população (pp. 1 e 6).
- **Argélia** (Faivre, *Les combattants musulmans de la guerre d'Algérie*, pp. 251–252, reproduzido no rapport Ceaux, *Aux harkis, la France reconnaissante*, 2018, p. 11), máximos:
  - 63 mil harkis, 8.500 GMS, 19.700 moghaznis, 62 mil nos grupos de autodefesa e 7.600 nas unidades territoriais e Aassès.
- **Quênia:**
  - guarda doméstica de cerca de 25 mil homens com cerca de 5 mil armas de fogo, ou **um quinto armado** (HC Deb 22 jul. 1954, vol. 530, c. 1590);
  - 10 mil kikuyu alistados em fevereiro de 1953 (HC Deb 25 fev. 1953, vol. 511, c. 2067);
  - era recrutada entre kikuyu, embu e meru, a mesma base do movimento. A fonte é o working paper de Branch (2005, p. 3), que pede para não ser citado; fica como pista até achar a página no livro dele.
- **Outros:** os auxiliares das ocupações (a Schutzmannschaft no Leste), sem número aqui.

*De onde vêm:*
- da parte da população da unidade que não apoia o movimento: 1 − apoio do proclamado, ou a
  resistência baixa da ocupação;
- **a fonte certa é o apoio por comunidade**: leal = Σ população da comunidade × (1 − apoio dela ao
  movimento). Isso dá os harkis e a guarda kikuyu, que vinham da própria base do movimento. É
  dependência do modelo de independência, que hoje guarda um só apoio por unidade (ver
  "Dependências de outros donos"). Até lá, vale 1 − apoio da unidade;
- **teto: 40 por mil da população controlada.** É a estimativa de antes, agora com âncora: cerca de
  220 mil entre polícia, policiais especiais e guarda, para cerca de 5,5 milhões na Malásia de 1953
  (Komer, pp. 38–41). A conta é minha, e a população ainda está sem página;
- recrutadas aos poucos, uma parcela por semana, com teto em uma fração dos homens em idade de
  serviço dessa parte.

*Quanto custam:*
- soldo e equipamento leve (a unidade `garrison`), pagos pelo orçamento militar da metrópole;
- contam na cobertura com um peso menor que o da tropa da metrópole. Há dois candidatos, à espera do dono:
  - **meio soldado:** valor de desenho, sem fonte;
  - **a fração armada:** um terço na Malásia (Komer, p. 41) e um quinto no Quênia (Hansard, 1954). Tem fonte, e é a candidata melhor;
- não entram no exército de terra para a G8: não amarram a metrópole nem pesam na duração da
  guerra;
- o jogo também os torna alvo: a guerrilha mata mais deles, e o movimento ganha apoio quando a
  milícia é dura (a resistência sobe pela extração, como em O1).

*A política contra a base:* o reassentamento (plano Briggs, 1950–52, cerca de 500 mil
moradores de áreas rurais nas "novas aldeias") e o reagrupamento na Argélia (≈ 2 milhões).
- O efeito: corta o recrutamento do movimento numa parte da população.
- O custo: dinheiro e o apoio do proclamado, que sobe entre os reassentados.
- É uma decisão de Agendas, com prazo e custo.

*O que o jogador decide:* quantas forças locais recrutar (ordem com teto, como a expedição) e se
aplica o reassentamento.

*Conferência que a fatia deve passar:* a Malásia em reconquista, com o apoio britânico acima de
300 por mais de 10 anos, sem forçar o número.

**G8, duração pelo empenho (decidida pelo dono em 2026-11-19, opção A).** O peso da duração no
apoio passa a ser a parcela do exército de terra empenhada na guerra × `commitment_factor` (3, em
dados), até 1. A regra é geral, atrás de `support.duration_by_commitment`, ligada dentro de
`war`.
- Hoje ela só reduz o peso da metrópole cujas guerras são todas coloniais. Qualquer outra guerra
  empenha o exército inteiro e pesa 1.
- A Alemanha de 1914–18 não muda.
- **Vietnã:** 540 mil de cerca de 1,5 milhão é uma parcela de 0,36, que com fator 3 dá peso 1. O
  alvo fica em 308 no ano 5 e em 273 no ano 6: a paz vem entre o quinto e o sexto ano, como no
  caso do dono.
- Com fator 2 o peso era 0,72, e a paz só vinha no oitavo ano.
- O teste `duration_by_commitment_keeps_the_owners_cases` imprime os dois fatores e assere o
  caso do dono com o fator dos dados.

**IA da metrópole.** Luta até o apoio cair abaixo de 300 e então desiste. Não abandona cedo nem
dosa a expedição; isso fica anotado para a fase 6.

**Jogador.**
- Na tela de exigência, a recusa avisa que a revolta vira guerra colonial e que há ponto de
  restauração antes da resposta (o `Reason::Demand` que já existia). Também diz quais estados as
  tropas atuais seguram e qual proclama (`DemandView::held`).
- O diário (Agendas, grupo Forças) mostra "Um país nasceu em revolta", com os combatentes, os
  armados e os desembarcados. Também lembra que a troca de país fica no ⋯ da tela de Campanhas
  (C4).
- A expedição é decisão nova: `SetExpedition`, com tela na janela "Guerra" e linha em
  `player_commands` e na wiki ("O que o jogador decide"). A postura nos segmentos usa o comando de
  sempre.

**Decididas pelo dono em 2026-11-19, todas na recomendação:**
- GC1: as formações coloniais desertam pela mobilização (`defect: true`).
- GC2: o país novo começa em 800 de apoio.
- GC3: metade das armas no nascimento.
- GC5: o movimento após a reconquista é dado, 500‰ da marca.

**Código das fronteiras tocado** (o thread está em pausa): `independence.rs` ganhou
`IndependenceSetup::partition` (com `war`), o corte em `release(How::Revolt)` e
`DemandView::held`.

### Tela (esboço, para o thread de interface)

Lida do estado que o tick grava, sem refazer a semana:
- **Guerra:** os lados (`world.war.conflicts.wars`: `side_a`, `side_b`, o líder primeiro), as
  semanas desde `since`, a exigência em jogo.
- **Apoio à guerra, primeiro na tela** (`fighting.support_of(país)`, G4):
  - o valor, de 0 a 1.000 por mil, com os dois limiares: paz (300) e colapso (100);
  - os termos da semana (`terms`): partida, baixas, duração, privação, território, curso da frente
    e o alvo, que é a soma deles;
  - o apoio de cada grupo de interesse (`groups`, na ordem do catálogo) e o seu peso na média;
  - a contagem (`countdown`: semanas até o governo cair), visível enquanto o apoio estiver abaixo
    do colapso. Nessas semanas o jogador ainda pode fechar a paz nos próprios termos;
  - depois da queda (`fallen`), o que caiu. `Regime`: uma forma de governo nova, por exemplo
    a monarquia que vira república; a reputação de segurança cai pela metade (P3). `Cabinet`:
    numa república cai o gabinete e a forma fica; não há corte de reputação. Nos dois casos, o
    governo que vem pede a paz: com o jogador, a proposta sai sozinha na mesa, e o texto diz
    "o governo caiu; o novo governo pede a paz".
- **Ocupação, por região ocupada** (`fighting.occupied`, G3): as semanas, a resistência, a
  guarnição mantida sobre a necessária e o que o ocupante levou na semana (`taken`).
- **Devastação** (`fighting.wrecks`): a parte dos níveis de edifício destruída e os civis mortos.
- **Frente, uma por inimigo:** a postura (`fighting.posture(país, inimigo)`, padrão manter) com o
  comando `SetPosture { tag, enemy, attack }`; as perdas acumuladas (`fighting.losses`, homens);
  os segmentos da última semana (`fighting.last_week`: estados `from` e `into`, países `a` e `b`,
  e o `outcome` com `engaged`, os `terms` dos dois lados, número, equipamento, treino,
  suprimento, terreno e ar, o `power`, as `losses` e o `advance_km`).
- **Avanço:** `fighting.advance` por segmento, como barra até `depth_km` do estado (metade do lado
  de um quadrado da sua área), quando o controle muda.
- **Lente no mapa:** o controle (`fighting.control`: região e ocupante) e a linha da frente (os
  segmentos da última semana).
- **Terceiros:** as chamadas dos acordos (fatia B, W4) seguem na tela da crise e no diário.
- **Paz:** a proposta vai pela mesa de tratados (`ProposeTreaty`); assinada entre os líderes dos
  dois lados, a guerra acaba, a ocupação entre eles termina e quem fica sem guerra desmobiliza.

### Ritmo da frente

O avanço semanal de um segmento é a velocidade-base da mobilidade × o terreno × o fator da
densidade × (1 − defesa/ataque). O que decide o ritmo:
- **Densidade do defensor por km de frente.** A largura do segmento é a fronteira real entre os
  dois estados, em `assets/map/borders.ron` (`bf-mapgen borders`, lido uma vez na carga).
  - Uma linha cheia tem 600 homens por km, cerca de uma divisão a cada 25 km, o mínimo de uma
    defesa coerente (Mearsheimer, 1989). Nela o atacante anda na velocidade-base.
  - O fator é o quadrado da razão entre a linha cheia e a densidade, entre 0,1 e 1,75. Uma
    linha densa como a do Somme (3.300 homens por km) desce ao piso; uma rala deixa a frente
    correr até 1,75 vez, a perseguição a pé de um exército quebrado.
- **Entrincheiramento.** Um segmento parado (menos de 2 km na semana) cava semana a semana
  (`Fighting::still`) e fica cheio em 10 semanas. Um segmento que anda recomeça do zero. A
  fortificação cheia dobra o terreno do defensor: trincheira com arame e metralhadora.
- **Postura da IA por segmento.** A IA ataca onde o seu fogo no segmento é pelo menos 1,5 vez
  o do inimigo ali (G5): é superioridade local, e dois países parelhos ainda atacam onde o outro
  está mais ralo.
- **Mobilidade (passo 3).** O exército anda no ritmo da maioria dos seus homens de terra
  (`war::fight::majority_mobility`): a classe de mobilidade que soma mais homens; as guarnições
  ficam fora, porque não saem do posto, e no empate vale a mais lenta. Na vida real, a maior
  parte do exército alemão de 1940 ia a pé atrás dos blindados, e a infantaria segura o conjunto.
- **Perseguição (passo 6).** Um lado que avançou num segmento na semana anterior e o ataca nesta
  com 1,5 vez o poder do defensor (`front.pursuit_ratio`, a mesma razão com que a IA ataca, valor
  de desenho) não empurra mais uma linha: o defensor está se desfazendo. O atacante anda na
  velocidade de marcha da sua mobilidade no terreno, sem o freio da densidade
  (`calc::pursuit_km`). Na vida real é a perseguição que transforma uma vitória em centenas de
  quilômetros: a Polônia de 1920 do Berezina ao Vístula, a França de 1940 depois de Sedan. A
  primeira semana ainda é a linha empurrada; a perseguição vem da segunda em diante. Não há
  estado novo: o avanço da semana anterior é o que `Fighting::last_week` já guarda.
- **Concentração das tropas móveis (item 12, feito).** Um exército de segunda geração põe a sua
  classe móvel (cavalo, rodas ou lagarta, a que tem mais homens) no segmento onde o inimigo é mais
  fraco por km, o mesmo que a terceira geração escolhe como foco, sem concentrar os outros homens.
  - O segmento anda no ritmo dessa classe quando os homens dela ali são pelo menos `mobile_share`
    dos homens do segmento (`fight::concentrated_mobility`). **Valor de desenho:** um terço; a
    fonte fica para a calibração.
  - A ruptura e o aproveitamento em profundidade continuam só da terceira geração: a segunda
    concentra para empurrar e perseguir.
  - A terceira geração segue como antes: o foco recebe a classe móvel.
  - **Tela:** cada segmento da semana (`Fighting::last_week`) traz a mobilidade de cada lado ali e
    a parcela dos homens dele na classe móvel. Não entram no save: depois de um load, voltam na
    semana seguinte.
  - **Casos de `front_pace`, medidos e não ajustados:** 
    - Polônia de 1920, a cavalo: 45,9 km por semana (faixa de 40 a 150). Passa; antes, a pé, eram 26.
    - Polônia de 1939: 210 km por semana sobre rodas e em lagartas (faixa de 60 a 200). Fica
      acima, ignorado com o motivo: o caso põe todos os atacantes do segmento sobre rodas, e em
      1939 as divisões móveis eram minoria dos homens de cada eixo, com a infantaria atrás.
      Se um terço é a linha certa (`mobile_share`) fica para a calibração.
    - França de 1940: segue ignorada. Precisa da ruptura da terceira geração, que a Alemanha do
      jogo não alcança até 1940 (rádio lento e `armored_warfare`; ver "Calibração prevista").

### Geração do exército (passo 4, rascunho atrás de `war`)

Cada exército é de segunda geração (frente linear, fogo e desgaste, comando do centro) ou de
terceira (unidades que agem pela intenção do comandante, concentração num ponto, ruptura e
aproveitamento). A geração sai da **prontidão**, somada termo a termo (`rules.ron`, `doctrine`).
Ela sobe a 650 e só cai abaixo de 450, para não oscilar. É lida no começo de cada mês, só com a
chave `war` ligada; com ela desligada nada disso existe e o hash não muda.

| Termo | Peso | De onde sai no jogo | O que o jogador faz em paz |
|---|---|---|---|
| Oficiais formados para agir pela intenção | 350 | com tradição registrada (G7, decidida pelo dono em 2026-11-19): a parcela do corpo de oficiais formada na doutrina, que muda uma turma por ano; sem tradição: metade da alfabetização da metrópole | pesquisar `armored_warfare` (a escola passa a ensinar a doutrina) e treinar (a turma se espalha na medida do treino); alfabetizar, para quem não tem tradição |
| Treino | 200 | treino médio das formações de terra | manter o exército treinado (o recruta novo dilui o treino) |
| Rádio | 100 | parcela dos homens em geração de equipamento posterior à de 1925 | modernizar (a sobra do orçamento militar troca a geração, fatia A) |
| Motorização | 150 | parcela dos homens em roda ou lagarta | encomendar formações mecanizadas e blindadas (A2) |
| Experiência | 200 | semanas de guerra, cheia em 52, meia-vida de 5 anos | — (vem da guerra) |
| Controle político (subtrai) | 400 | troca da forma de governo nos últimos 5 anos: cheio na troca, some no fim | não trocar de regime às vésperas da guerra |

O regime em si não entra: o exército alemão de 1939–41, sob ditadura, comandava por missão; o
francês de 1940, sob democracia, lutava a batalha metódica do centro.

**Na guerra.** Um exército de terceira geração:
- põe seis décimos dos homens livres no segmento de foco, o mais fraco por km
  (`doctrine::allocate_front`);
- manda para lá as tropas móveis (a classe de cavalo, roda ou lagarta com mais homens);
- rompe o segmento quando o seu poder passa de 3 vezes o do defensor e avança além da linha a 2
  vezes a velocidade-base (`doctrine::breakthrough`).

Sem combustível (abaixo da metade do pedido) ele luta como de segunda.

**Cultura de comando (G7: o dono escolheu (a) em 2026-11-19).** O dado é o de 1925, a
instituição daquele ano, com fonte em `rules.ron`:
- Alemanha 700: a Reichswehr de Seeckt;
- Reino Unido 400;
- União Soviética 350: antes das operações em profundidade;
- França 200: a instrução de 1921 e a batalha metódica.

O que cada exército vira depois sai do jogo:
- uma turma por ano, 1/25 do corpo, sai formada no que a escola ensina, na medida do treino;
- a escola ensina a doutrina nova quando o país tem `armored_warfare` em estágio industrial;
- a derrota (governo caído) dobra a velocidade da revisão por 10 anos; a vitória não.

Quem não tem dado lê metade da alfabetização do povo (`untraditioned_share`, valor de desenho):
um corpo escolarizado, sem a doutrina de uma escola de estado-maior. Sem esse desconto, a Polônia
de 1939 (alfabetização 0,79) passaria a Reichswehr (0,7). Na tela, aparece como "sem tradição
registrada".
A formação dos oficiais é automática nesta fatia. Como decisão, ela seria uma linha do orçamento
militar ("Regra histórica" ou "Fixado pelo governo"), desenho para depois.

**Leitura para a tela:** `war::army::army_reading(world, mil, politics, país, dia)` devolve a
geração, a prontidão termo a termo, as entradas, se há tradição, a experiência e os limiares.

**Conferência de 1939–41** (`war_land::generation_check_1939_to_1941`, ignorado, rodado no
slot: o jogo de 1925 com `war` ligada e nenhuma guerra lutada):

| | oficiais | treino | rádio | motorização | total | geração | `armored_warfare` |
|---|---|---|---|---|---|---|---|
| Alemanha, set/1939 | 0,70 | 1 | 0 | 0 | 445 | 2ª | Ciência |
| Alemanha, mai/1940 | 0,70 | 1 | 0,02 | 0 | 447 | 2ª | Ciência |
| Alemanha, jun/1941 | 0,70 | 1 | 0,13 | 0 | 457 | 2ª | Protótipo |
| França, mai/1940 | 0,20 | 1 | 0 | 0 | 270 | 2ª | Desconhecida |
| URSS, jun/1941 | 0,35 | 1 | 0,04 | 0 | 326 | 2ª | Ciência |
| Polônia, set/1939 (sem tradição) | 0,39 | 1 | 0 | 0 | 337 | 2ª | Desconhecida |

A ordem sai certa: Alemanha, Polônia, URSS, França. Ninguém chega à terceira geração (650), e o
limiar não baixa. O que falta não é do mesmo tipo:
- **Motorização: feita depois desta tabela, pela regra histórica de composição (#235).** Esta
  tabela é de antes. Medido sobre o main `803bf72` (`composition_check_1939_and_1944`), os homens
  de terra sobre rodas e em lagarta são:
  - em 1939: 15 a 20% e 5 a 10% (Alemanha 16,5% e 5,5%; EUA 19,7% e 10%), contra 0% sem a regra;
  - em 1944: 19 a 30% sobre rodas.
  - A fonte da seção "Composição das forças" dá 13 a 26%.
- **A Alemanha de 1939 tem 94.541 homens de terra**, o teto de Versalhes. A lei de conscrição de
  1935 não muda no jogo. É dependência da política (a lei sob pressão ou decisão) e dos tratados
  (a denúncia de Versalhes), não da encomenda.
- **Ritmo da pesquisa de guerra blindada (equilíbrio, para a fila de calibração).** No jogo a
  Alemanha está em Ciência em 1939; na história a técnica estava industrial em 1936. A pesquisa
  militar não responde a verba nem a ameaça: sem foco do jogador, a ciência nova vai para a
  tecnologia aberta mais barata, e os protótipos dividem o esforço por igual. É entrada para a
  IA estratégica da fase 6 (pesquisar o que a ameaça pede).
- **Rádio lento (equilíbrio, para a fila de calibração).** A troca de geração do equipamento leva
  a Alemanha a 0,13 em 1941.

Os casos de `front_pace` seguem ignorados com esse motivo.

**O que a tela mostra.** `army_reading` diz, por termo, o que o segura (`war::army::Hold`):
- a escola sem a doutrina, com o estágio da pesquisa;
- sem tradição registrada;
- treino incompleto;
- nenhuma formação em geração nova;
- nenhuma formação motorizada;
- sem experiência de guerra;
- troca de regime, com os anos que faltam.

**Lacuna anotada.** Os expurgos soviéticos de 1937–38 e os comissários não entram: não houve
troca de forma de governo, e o jogo não tem evento de expurgo. Se a União Soviética chegar a 1941
melhor que a real, é dependência anotada, não meta.

**Casos de conferência** (`tests/front_pace.rs`, com a fonte em cada teste):

| caso | história | faixa | modelo |
|---|---|---|---|
| frente ocidental, 1916 (Somme) | ~10 km em 20 semanas | abaixo de 2 km/semana | 1,3 |
| a mesma frente antes das trincheiras | guerra de movimento de 1914 | ao menos o dobro | ok |
| Polônia, 1920 | ~600 km em 6 semanas | 40 a 150 km/semana | 1ª semana: 26 (a maioria a pé); falta 4 (cavalaria no segmento: 46) |
| Polônia, 1920, da 2ª semana em diante (perseguição) | o mesmo | 40 a 150 km/semana | 40, a marcha a pé |
| Polônia, 1939 | Varsóvia em ~8 dias | 60 a 200 km/semana | 56 (a maioria a pé); falta 4 |
| França, 1940 | Sedan ao Canal em ~7 dias | 100 a 300 km/semana | 35 (a maioria a pé); falta 4 |
| Hungria, 1925, contra a Tchecoslováquia com ~4× o fogo | 1919: meses contra um exército de massa; a perseguição final, ~100 km em 10 dias | 9/10 do povo tomados em 3 a 8 semanas | 4 semanas |

Os casos ignorados ficam marcados com o que lhes falta; a faixa não foi afrouxada.

### Custo de CPU

Decisão 6: a guerra mundial de 1939, com todos os segmentos ativos, abaixo de 15 % do tick. A
fatia mede com uma guerra de teste de cinco grandes países, porque a IA não declara
(`war_support::a_great_war_costs_under_a_seventh_of_the_tick`, ignorado, rodado no slot).

Na medida de 2026-11-18, um ano de guerra opôs Alemanha, Itália, Hungria e União Soviética a
França, Reino Unido, Polônia, Tchecoslováquia e Iugoslávia. Ninguém pediu paz: os limiares
foram zerados para a guerra durar o ano. O sistema de forças gastou 92,6 ms de 6.225 ms nos
sistemas, **1,5 % do tick**, com 53 regiões ocupadas no fim e 25 segmentos na última semana.
Isso inclui o combate, o apoio, a ocupação e a devastação.

Uma primeira medida deu 0,8 %, mas a Alemanha desistiu da guerra nas primeiras semanas, abaixo
da paz, e o ano correu quase todo em paz.

Na medida de 2026-11-18, com a densidade e o entrincheiramento, a guerra de nove países durou o
ano inteiro (`front_pace::a_great_war_costs_under_a_seventh_of_the_tick`, ignorado, rodado no
slot). O sistema de forças gastou 82,3 ms de 5.854 ms, **1,4 % do tick**. No fim havia 56
regiões ocupadas e uma frente de 18 segmentos, toda entrincheirada.

### Decidido pelo dono (G1–G6, 2026-11-18)

| # | Decisão | O que o dono disse |
|---|---|---|
| G1 | só terra nesta fatia; mar e ar na C2 | "A terrestre é a mais importante, mar e ar podem vir depois." |
| G2 | objetivo e postura por frente; formações pela regra aberta | "Em segundo momento podemos ir pensando em táticas melhores": o desenho fica aberto a mais comando (formações por frente, eixo, reserva) |
| G3 | o território ocupado produz e paga ao ocupante, com três acréscimos: a parcela calibrada pela vida real, a devastação e a guarnição | abaixo, "Ocupação" |
| G4 | **mudou:** a rendição e a paz são decididas pelo apoio à guerra, não pela exaustão | "um país pode estar exausto e ainda batalhar, ou pode nem ter cansado e desistir; a medida é apoio de guerra, como a Alemanha na Primeira Guerra e os Estados Unidos no Vietnã" |
| G5 | a IA defende e contra-ataca com 1,5 vez o poder; **pede paz quando não tem mais apoio à guerra** | — |
| G6 | proclamação e guerra entre dois países; **se a colônia perder, a metrópole reconquista** | — |

### Apoio à guerra (G4; o dono decidiu A1–A3 na recomendada, 2026-11-18)

Um número por país em guerra, de 0 a 1.000 por mil, feito do apoio de cada **grupo de interesse**
(`politics`), pesado pela influência do grupo (`clout`). É o que a tela da guerra mostra em
primeiro lugar, termo a termo e grupo a grupo.

**Ponto de partida, na declaração** (dados, valores de desenho):
- atacado: 800 (o efeito de união em torno da bandeira; Mueller, 1973);
- atacante com causa registrada (`casus_belli`): 600; sem causa: 400;
- guerra colonial: metrópole 550; o país novo começa com a mobilização do movimento.
- Cada grupo parte daí com o seu viés: o militar acima; os trabalhadores e os camponeses, que dão
  os soldados, abaixo; os negócios conforme a sua parte na indústria de guerra.

**O que puxa para baixo, toda semana:**
- **baixas:** pelo logaritmo dos mortos acumulados sobre a população; o apoio cai com o log das
  baixas acumuladas (Mueller, 1973; Coreia e Vietnã), e mais quando a taxa de baixas sobe
  (Gartner e Segura, 1998);
- **duração:** um tanto por ano de guerra;
- **privação civil:** a parte das necessidades básicas dos pops que não é atendida (comida,
  aquecimento) e a alta de preços; o inverno do nabo de 1916–17 e as greves de 1918 na Alemanha
  (Offer, 1989);
- **território:** a parcela da própria população sob ocupação (depois do primeiro efeito de união);
- **resultado:** a frente que recua tira apoio, a que avança devolve; a expectativa de vitória pesa
  mais que o número de mortos (Gelpi, Feaver e Reifler, 2009).

**O que puxa para cima:** vitórias (a frente avança, o inimigo perde território), o fim da
privação, e uma exigência menor na mesa.

**Regime e censura:** o efeito `censorship` da lei de imprensa e o `franchise` mudam a
sensibilidade. Com censura, o apoio cai mais devagar, mas a queda abaixo do limiar de colapso faz
crescer o movimento `regime_change` (fase 3d); é a queda súbita da Rússia de 1917 e da Alemanha de
1918. Com eleições livres, cai mais depressa e o governo sente antes (Reiter e Stam, 2002).

**Decisão:**
- abaixo de 300 por mil (limiar de paz), o governo da IA pede paz, oferecendo parte da exigência;
- abaixo de 100 (colapso), a rendição: aceita a exigência inteira;
- o jogador vê o apoio e decide. Com o apoio abaixo do limiar de paz, continuar custa
  legitimidade e faz crescer o movimento contra a guerra. **Recomendado:** no colapso, a
  rendição é forçada também para o jogador, como a Alemanha em novembro de 1918.
  **Alternativa:** o jogador nunca é forçado; só paga legitimidade e movimento.

**A rendição passa pela política** (orquestrador, 2026-11-18), como em novembro de 1918:
- Abaixo do colapso corre uma contagem (`revolution_weeks`: 6, o tempo entre o pedido de
  armistício de Ludendorff e a abdicação).
- Esgotada a contagem, o governo cai. A forma de governo seguinte vem de `revolution` em
  `data/military/war.ron`:
  - monarquias → `parliamentary_republic`;
  - `one_party_state` → `military_dictatorship`;
  - repúblicas parlamentares e presidenciais, e a monarquia parlamentar: só o gabinete cai
    (os EUA em 1968, a França em 1940).
- Uma forma nova entra em vigor como obra do movimento `regime_change`, no registro de trocas
  de lei. O tratado lê ali uma troca de regime e corta a reputação de segurança (P3): o
  governo novo pode não honrar o que o anterior honrava.
- O governo que vem pede a paz. O país atacado propõe na mesa a exigência inteira. O que
  declarou desiste da guerra, que acaba como começou.
- Quando só o gabinete cai, não há corte de reputação.
- A3 é dado: com `player_forced: false`, a contagem não corre para o país do jogador.

**O país tomado** (o caso da tela, #218): sem esta regra, uma guerra ficava aberta para sempre
com o país inteiro ocupado e sem frente.
- O governo cai na hora, pelo mesmo caminho do colapso, quando o inimigo controla 90 % da
  população metropolitana (`overrun`), ou metade dela com a capital tomada
  (`capital_overrun`).
- A capital é o estado metropolitano de centro mais próximo de `capital_lon_lat`.
- Exceções: um aliado que ainda luta no mesmo lado, ou uma colônia que o inimigo não tomou,
  mantém a guerra.
- Referência: a Hungria de Kun, em agosto de 1919, com o exército romeno em Budapeste.

**O que a fase 3d ainda não dá:** o movimento `regime_change` anda só uma vez por ano e não
impõe lei ao jogador. Por isso a revolução de guerra roda no passo semanal da guerra. Um
indicador `war_support` para as causas dos movimentos fica para quando um movimento contra a
guerra entrar.

**Casos de conferência** (testes de `war::support`, com os valores de `war.ron`):
1. **Alemanha, 1914–1918.** Cerca de 2 milhões de mortos em 67 milhões, quatro anos de guerra,
   a fome do bloqueio (35 % das necessidades básicas perdidas) e a frente recuando 5 km por
   semana depois de Amiens. Nenhum território alemão ocupado. O alvo fica abaixo do colapso:
   o país cede por dentro. Sem a fome e o recuo, os mesmos mortos e anos o deixam acima da paz.
2. **Estados Unidos no Vietnã.** 58 mil mortos em 200 milhões, nenhuma derrota e nenhum
   território perdido. O apoio fica acima de 300 aos três anos e abaixo aos seis, sem chegar
   ao colapso: o país pede paz sem derrota militar.

### Ocupação: o que se extrai, a devastação e a guarnição (G3; o dono decidiu O1–O3 na recomendada, 2026-11-18)

**(a) O que o ocupante extrai.** A França ocupada pagou "custos de ocupação" de 400 milhões de
francos por dia a partir de agosto de 1940, 300 milhões a partir de maio de 1941 e 500 milhões a
partir de novembro de 1942, além de requisições e do trabalho forçado (o STO mandou cerca de 650
mil trabalhadores à Alemanha). Somado, o que a Alemanha tirou chegou perto de um terço da renda
nacional francesa em 1943, e a produção industrial francesa caiu a cerca de metade da de 1938 em
1944 (Milward, 1970; Occhino, Oosterlinck e White, 2008).
- **Recomendado:** o ocupante leva uma parcela do PIB do território ocupado, que sobe com o tempo
  de ocupação de 20 % no primeiro ano até um teto de 33 % (dados), vezes a eficiência da guarnição
  (c); a produção do ocupado cai pela devastação (b), pelos homens tirados e pela resistência. O
  "50 %" do desenho anterior fica acima do caso francês.
- **Alternativa:** parcela fixa (simples, sem curva no tempo).

**(b) Devastação.** O combate que passa por um estado destrói capital, infraestrutura e gente, e o
estado produz menos depois. A União Soviética perdeu cerca de um quarto do capital reprodutível
na Segunda Guerra, e a Polônia mais de um terço da riqueza nacional (Harrison, 1998).
- **Recomendado:** cada semana de combate num segmento destrói uma parcela dos níveis de edifício
  e do transporte dos dois estados do segmento, proporcional à intensidade (o fogo dos dois lados),
  com teto, e mata civis por idade, mais nas cidades; a fila de obras reconstrói. É gradual, como
  a frente que demora.
- **Alternativa:** um golpe único quando o controle muda.
- **A terra arrasada** (destruir ao recuar, como a União Soviética em 1941) fica para depois,
  como ordem de postura.

**(c) Guarnição e polícia militar.** O ocupante precisa de tropa no território para manter o
controle. A referência é Quinlivan (1995): cerca de 20 homens de segurança por mil habitantes para
uma população hostil, de 2 a 4 para uma calma. A Alemanha mantinha cerca de 100 mil homens na
França em 1941, para uns 40 milhões, algo como 2,5 por mil.
- **Recomendado:** a exigência de guarnição de cada estado ocupado é de 2 por mil da população,
  subindo até 20 por mil com a resistência (o movimento no estado). O ocupante destaca
  formações de terra (o tipo `garrison` existe no catálogo) e paga homens e manutenção. Com
  guarnição abaixo da exigência, a parcela extraída cai na mesma proporção e a resistência cresce;
  com falta grande, nascem guerrilhas (`irregular`), numa fatia depois.
- **Alternativa:** um custo fixo de guarnição, sem exigência nem consequência.

**No rascunho (código):**
- **Guarnição (item 16, feito):** as formações `garrison` do ocupante, as coloniais à parte, ficam
  de guarnição primeiro, com o seu próprio fogo (`occupation::garrisons`, `fight::posts`). Só o
  que elas não cobrem sai do exército de campo, até metade dele (`garrison_cap`) como antes, e o
  fogo desse exército sai na mesma proporção. Os homens de guarnição não lutam na frente.
  - **A IA encomenda guarnições** (`occupation::garrison_order`). Um ocupante sem jogador, com
    regiões ocupadas há `garrison_order_weeks` (4) e a guarnição da semana anterior abaixo da
    necessidade, encomenda pela fila A2 um elemento de `garrison` (mil homens) para cada mil que
    faltam. Não encomenda de novo enquanto houver uma aberta, e o pagamento sai do teto militar.
    **Valores de desenho:** as 4 semanas e o um por um.
  - **Quando o teto não cobre a encomenda,** ela fica parada (com as semanas paradas contadas,
    como toda encomenda), e a falta continua saindo do exército de campo até `garrison_cap`, como
    antes. Exemplo (`war_support::garrison_formations_and_the_ais_order`):
    - A Hungria declara à Tchecoslováquia e perde terra para ela e os aliados.
    - Com o teto de campo cortado a 1% dos homens (`garrison_cap` 10), a Tchecoslováquia guarda 2,5%
      da guarnição necessária (63.426 homens em 4 regiões). A IA encomendou 15 elementos, nenhum
      entregue até julho de 1925.
    - Com 60 mil homens em formações de guarnição desde o começo, ela mantém a guarnição inteira
      (10.569 homens necessários: a resistência sobe menos com a guarnição cheia) e não encomenda.
  - **Conferência, medida e não alvo:** a regra pede 2 por mil numa população calma, 80 mil homens
    para os ~40 milhões da França. A Alemanha tinha cerca de 100 mil homens em toda a zona alemã da
    França em dezembro de 1941. A fonte é a Wikipédia ("German military administration in
    occupied France"), que cita uma monografia do Connecticut College que não abriu: **sem página
    verificada.**
  - **Guarnição colonial de 1925 à parte:** as formações `garrison` coloniais seguem na colônia e
    não servem às ocupações. A ligação com a guerra colonial (forças locais, guarnição faltante:
    GC6 a GC8, abertas com o dono) fica para lá.
  - **Por frente, com comando do jogador (G2):** as guarnições servem a todas as ocupações do
    país, pela necessidade de cada uma. Mandar formações a uma frente é dependência das frentes
    como objeto de comando.
  - **Tela:** por ocupante, a guarnição necessária, a mantida por formações de guarnição e a tirada
    do exército de campo, e a encomenda de guarnição aberta. Vem do que o tick já calcula
    (`occupation::garrison_need`, `fight::posts`, a fila de encomendas), sem campo novo no save.
- **Extração:** vai do tesouro do dono para o do ocupante. É a parcela do PIB do ano anterior
  pela parte da renda do país na região, de 20 % a 33 % em três anos, vezes a guarnição mantida.
- **Resistência:** cresce com o que se leva e com a guarnição que falta, e cai devagar com a
  guarnição cheia.
- **Devastação:** cada semana de combate tira até 0,4 % dos níveis de edifício dos dois estados
  do segmento, pelo peso das baixas, com teto de 40 %, e mata civis. O transporte entra quando
  a infraestrutura tiver estado por região.

**Dado novo:** nenhum arquivo de fonte além destas referências; as parcelas, as taxas e as
exigências são valores de desenho em `data/military/war.ron`, com as fontes no comentário.

Fontes: Mueller, J. (1973), *War, Presidents and Public Opinion*; Gartner, S. e Segura, G. (1998),
"War, Casualties, and Public Opinion", *Journal of Conflict Resolution* 42(3); Gelpi, C., Feaver,
P. e Reifler, J. (2009), *Paying the Human Costs of War*; Reiter, D. e Stam, A. (2002), *Democracies
at War*; Offer, A. (1989), *The First World War: An Agrarian Interpretation*; Milward, A. (1970),
*The New Order and the French Economy*; Occhino, F., Oosterlinck, K. e White, E. (2008), "How Much
Can a Victor Force the Vanquished to Pay?", *Journal of Economic History* 68(1); Harrison, M.
(1998), *The Economics of World War II*; Quinlivan, J. (1995), "Force Requirements in Stability
Operations", *Parameters* 25(4).

#### Obras em terra ocupada (atrás de `occupied_works`, desligada)

Achado da terra arrasada (#360): a construção não conhecia o controle. O dono seguia investindo e
reconstruindo na região que o inimigo ocupa, e isso valia também para a devastação do combate. O
orquestrador decidiu OC1 a OC4 nas recomendadas (2026-11-24), como padrão de correção; o dono foi
informado. Código em `econ::construction` (`held`, `stopped`), no investimento mensal
(`econ::monthly::Investment`), nas obras públicas (`econ::public::public_works`) e na entrega da
fila (`econ::week`). Sem estado novo e sem save novo: tudo sai do controle da semana
(`Fighting::controller`).

| Peça | Como é na vida real | No jogo | Muda a simulação? |
|---|---|---|---|
| O dono (OC1) | não constrói na terra que perdeu; a obra fica embargada | nenhuma obra nova numa região que o dono não controla: privada, pública, firma do Estado, pedido do governo (D8) ou fundo de fora. As obras já na fila ficam paradas, sem pedir nem receber pontos, com o que já tinham, e voltam a andar quando a região volta | só com a regra |
| O dinheiro (OC2) | o capital procura onde pode | o fundo escolhe entre as regiões que o dono controla, pela regra de sempre; o que não acha onde pôr fica no fundo | sim |
| O ocupante (OC3) | constrói o que decide (obras de guerra) | nada nesta fatia; construir em terra ocupada fica como decisão futura do jogador (Agendas) e da IA (fase 6) | — |
| Fechar (OC4) | a fábrica que perde fecha | pela regra de sempre, também na região ocupada | — |

**Medido** (`war_occupied_works::the_owner_does_not_build_on_land_it_does_not_hold`, com a
devastação do combate do `main`; a terra arrasada fica para quando o #360 entrar): a Polônia,
jogada, declara à Tchecoslováquia e perde uma das 16 regiões (602) na primeira semana de combate.

| | jun/1925 | set | dez | mar/1926 |
|---|---|---|---|---|
| níveis na região perdida, sem a regra | 239 | 247 | 247 | 251 |
| níveis na região perdida, com a regra | 239 | 240 | 241 | 242 |
| níveis nas regiões mantidas, sem / com | 9.362 / 9.362 | 9.470 / 9.471 | 9.489 / 9.491 | 9.521 / 9.523 |
| obras na fila da região perdida, sem / com | 16 / 16 | 11 / 16 | 13 / 16 | 14 / 16 |
| obras na fila das regiões mantidas, sem / com | 266 / 266 | 193 / 198 | 238 / 243 | 265 / 269 |
| fundo de investimento (bi), sem / com | 9,99 / 10,03 | 9,67 / 9,78 | 9,67 / 9,79 | 9,69 / 9,82 |

- Com a regra, as 16 obras da região perdida esperam inteiras. Os 3 níveis a mais na região são
  autônomos urbanos (`informal_trade`, da mão de obra, não do investimento); uma fábrica de armas
  com prejuízo fechou (OC4).
- O total investido **não** é igual: com a regra, o fundo põe um pouco mais nas regiões mantidas
  (2 níveis e 4 obras a mais) e guarda o resto. Em março de 1926 o fundo tem 1,3 % a mais: o
  dinheiro não se perde, mas o fundo não acha nas outras regiões, dentro do teto de obras por mês e
  do retorno mínimo, onde pôr tudo o que iria à região perdida.
- Na Hungria ocupada inteira (HUN×CZE), não sobra região: o fundo só acumula (3,98 contra 2,89 bi
  em dezembro de 1925).

**Tela:** a obra da fila numa região que o dono não controla mostra "parada: território ocupado"
(`econ::construction::held(world, região)` falso, com a regra ligada).

## Mar e ar (fatia C2, desenho)

A fatia C2 traz para a guerra da fatia C o mar e o ar: `naval_step` e `air_step` (já escritos e
testados) no tick, sobre as 63 zonas marítimas do `SeaGraph` (`map-pipeline.pt-BR.md`, "Zonas
marítimas") e sobre as zonas aéreas das frentes. Mesma chave `war`; vem depois da C.

### Mar

- **Frotas por zona:** as formações de mar de cada país ficam numa zona marítima, com uma
  **missão**: controlar a zona, escoltar comboios, bloquear, ou atacar o comércio (submarinos). O
  jogador dá a missão por zona; a IA mantém as frotas nas zonas dos seus portos (C5).
- **Encontro:** numa zona com frotas dos dois lados, `naval_step` resolve a semana: canhão e
  torpedo contra a superfície, antissubmarino contra os submarinos, aviões a bordo; perdas por
  navio inteiro e avaria. A decomposição vai para a tela, como em terra.
- **Controle da zona:** quem tem mais poder de superfície na zona no fim da semana controla. O
  controle decide quem passa.
- **Bloqueio e comboio:** o trecho de uma rota (`SeaGraph::route`) que passa por zona
  controlada ou disputada pelo inimigo perde uma parcela do que leva, proporcional ao poder de
  interdição (submarino, superfície, ar de patrulha) sobre o de escolta. A perda vale para o
  suprimento da frente (`supply`, pernas marítimas) e para o comércio (por par, quando
  `trade_by_pair` estiver ligada; senão, a importação do país bloqueado cai pela mesma parcela).
- **Desembarque:** transporte com tropa sai de um porto, atravessa as zonas e desembarca num
  estado costeiro inimigo; chega com o que sobrou do passo de mar e luta em terra com o fator de
  terreno `amphibious`.
- **Casos:** o bloqueio da Alemanha de 1914–19 (importação de comida e de nitrato cortada); a
  Batalha do Atlântico (comboio contra submarino, a escolta e a patrulha aérea que viraram o jogo
  em 1943); a Normandia (desembarque sobre controle do mar e do ar).

### Ar

- **Zonas aéreas:** a zona de cada frente, e a de cada estado bombardeado.
- **Superioridade:** `air_step` disputa a zona; a superioridade de um lado é o seu poder ar-ar
  sobre a soma dos dois. Ela entra no termo "ar" do combate em terra (neutro na fatia C) e no
  passo de mar.
- **Ataque ao solo:** soma ao fogo da frente, vezes a superioridade.
- **Bombardeio estratégico:** tira níveis de edifício (fábrica, usina, ferrovia) do estado alvo,
  vezes a superioridade, e a fila de obras reconstrói (devastação, "Perdas que voltam para o
  mundo"). Mata civis por idade na pirâmide.
- **Repartição:** os aviões de cada país vão às zonas pelo peso da frente, pela mesma regra aberta
  da terra; o jogador escolhe o alvo do bombardeio estratégico (C2-S4).
- **Casos:** a Batalha da Inglaterra (superioridade sem invasão); o bombardeio da Alemanha
  (USSBS 1945: a produção alemã de armas subiu até o meio de 1944, e o golpe que contou foi no
  combustível e no transporte).

### Estado

Guardado só quando há guerra: a zona e a missão de cada frota, o controle de cada zona marítima,
e o alvo de bombardeio escolhido. A superioridade não é estado: sai da semana.

### Custo de CPU

63 zonas marítimas, poucas com dois lados ao mesmo tempo, e as zonas aéreas das frentes: o custo
fica abaixo do da terra. A meta segue a decisão 6 (a guerra mundial de 1939 abaixo de 15 % do
tick, terra, mar e ar juntos).

### Decisões para o dono

| # | Escolha | Recomendado | Alternativa |
|---|---|---|---|
| S1 | Comando no mar | missão por zona (controlar, escoltar, bloquear, atacar o comércio); as frotas se repartem pela regra | o jogador move cada frota |
| S2 | Bloqueio no comércio | perda proporcional à interdição sobre a escolta, no comércio por par e no suprimento | só no suprimento da frente |
| S3 | Guerra submarina irrestrita | o jogador escolhe; atacar navios neutros dá causa aos neutros (registro de quebras) e custa boa vontade, como em 1917 | os submarinos só atacam o inimigo |
| S4 | Alvo do bombardeio | o jogador escolhe a classe de alvo (indústria, transporte e combustível, cidades) | alvo automático pela indústria de guerra do inimigo |
| S5 | Ar | aviões repartidos pela regra entre as frentes; o jogador só escolhe o alvo estratégico | o jogador designa cada ala |

## Mar e ar mínimos (fatia C2, feito, atrás de `war`)

O recorte combinado com o orquestrador (2026-11-19). O código está em `war/sea.rs`, no ar de
`fight_week` e na rota de `colonial::route_cut`; o save é o 35 (`WarState::sea`). Tudo roda só
com uma guerra aberta: com `war` ligada e em paz, o custo é zero.

**Frotas.**
- Cada país em guerra põe a marinha numa zona, com a missão fixa "controlar".
- A **zona de saída** de um país é a zona mais presente nas rotas mais curtas dele até os portos
  dos outros países. A da Alemanha é o mar do Norte.
- Se a força de superfície do país é maior que a do inimigo principal, a frota vai à zona de
  saída do inimigo e bloqueia. Se é menor, fica na própria zona de saída, como frota em potência,
  sem buscar batalha.
- O comando do jogador (S1, missão por zona) vem na fatia seguinte. Até lá, a janela "Guerra"
  diz onde está a frota, por quê e quem ela bloqueia (`FleetLine`).

**Batalha e controle.**
- Numa zona com frotas de países em guerra entre si, o mais forte presente e os aliados dele
  enfrentam os inimigos presentes com `naval_step`.
- Saem das formações os navios afundados e a avaria dos que ficam à tona, como fração de navio.
  O navio avariado fica fora de ação, e a reposição é paga pelas compras do estaleiro.
- Controla a zona quem fica com mais poder de superfície no fim da semana.
- O combustível ainda não é seguido: o suprimento é cheio.

**Bloqueio, por par.**
- O par (B, X) fica cortado quando o caminho mais barato entre os dois é por mar
  (`Geography::by_sea`) e nenhuma rota marítima dos portos de B chega aos de X com as zonas que
  os inimigos de B controlam fechadas (`SeaGraph::reach`).
- Par por terra não fecha. Par com rota livre também não.
- A leitura é `war::fight::blockaded_pairs(world)`; o comércio fecha e reabre o par
  (`feat/embargo-war`, do thread do comércio). Para a tela, `blockaded(world)` dá (A, B).
- Os pares de quem está em guerra fecham pelo CT7 (`war_pairs`), e por isso não entram aqui.

**Desembarque colonial.** Os homens só desembarcam se houver rota marítima aberta dos portos
metropolitanos até o porto mantido, sem passar por zona que o inimigo controla. Com a rota
cortada, nada desembarca, e a tela da expedição diz onde: "a rota Marselha–Argel está cortada em
Mediterrâneo ocidental".
- A capacidade continua a dos portos.
- As forças de 1925 não têm navio `transport`: a tropa ia em mercantes requisitados, e a marinha
  mercante não está modelada.
- A capacidade dos transportes entra quando houver navio de transporte ou marinha mercante.

**Ar em terra.**
- Os aviões de cada país se repartem igualmente pelos segmentos dele.
- Em cada segmento, `air_step` dá a superioridade, que entra no termo `air` do `LandSide`, no
  lugar do 0,5 neutro (peso `air_weight`, 25%). As perdas saem das formações.
- O ataque ao solo somado ao fogo fica de fora: o poder aéreo está em valor de equipamento, o
  fogo de terra em `fire_per_unit`, e não há conversão calibrada entre os dois.

**Neutros.** O bloqueio corta o comércio do bloqueado com terceiros. Nesta fatia não acontece nada
entre o bloqueador e o neutro prejudicado: nem relação, nem reputação, nem causa de guerra (os
Estados Unidos em 1917). Fica como entrada da fase 6.

**Conferências** (`tests/war_sea.rs`, mundo de 1925):

| Caso | O que deu |
|---|---|
| Bloqueio da Alemanha (guerra GBR × GER) | O Reino Unido, mais forte, fica no mar do Norte, a saída da Alemanha, e o controla. Corta 51 dos 61 pares que o `tests/blockade.rs` fecha à mão (todo parceiro cujo caminho mais barato até a Alemanha é por mar). Ficam abertos, pelo Báltico e pelo Kattegat: Danzig, Polônia, Lituânia, Letônia, Estônia, Finlândia, Suécia, Noruega, Dinamarca e URSS. Alemanha–Suécia aberta é o caso do teste. O tamanho do erro do bloqueio binário seriam esses 10 pares. A Noruega aberta pelo Kattegat é um limite: na história, o comércio norueguês com a Alemanha passava pelas águas territoriais. |
| Jutlândia | O mar do Norte fica britânico nas 12 semanas. A frota alemã de Versalhes vai de 16 a 8 navios, e a britânica de 303 a 273. |
| Ar em terra | A Alemanha de 1925 não tem força aérea (Versalhes). Contra a aviação francesa, o termo de ar alemão fica em 0,819; com a francesa no chão, em 0,840. O avanço alemão muda 0,12 km por segmento-semana. A França de 1940 com a Luftwaffe só se mede quando a Alemanha tiver aviação (ordens A2, ou um mundo de 1939). |
| Rota colonial | Com a Itália em guerra com a França e controlando o Mediterrâneo ocidental, a rota Marselha–Argel fica cortada nessa zona, e a expedição para. |
| Paz | Sem guerra, o mar fica vazio, e `blockaded_pairs` também. |

**Custo de CPU.** Na grande guerra de um ano (`a_great_war_costs_under_a_seventh_of_the_tick`:
Alemanha, Itália, Hungria e URSS contra França, Reino Unido, Polônia, Tchecoslováquia e
Iugoslávia):
- forças com mar e ar: 64,6 ms de 2.154 ms dos sistemas, ou 3,0%;
- sem mar e ar (o ramo da guerra colonial): 57,7 ms, ou 2,2%;
- ficaram 7 frotas, 5 zonas controladas e 89 pares de comércio cortados;
- a meta da decisão 6 é ficar abaixo de 15%.

## O que falta para ligar ao tick

O contrato, os dados e as funções estão prontos e testados. Nada roda ainda. Quando a fase 5
abrir, a ordem proposta é esta, cada passo com o hash e o custo medidos contra a `main`:

| # | Passo | O que entra no estado e no save | Depende de |
|---|---|---|---|
| 1 | **Forças no `GameWorld`** | formações por país: tipo, geração, elementos, equipamento que têm, treino, estado onde estão; carga de `forces.ron` na partida | nada novo |
| 2 | **Custo semanal** | as forças compram o consumo (`upkeep`) e a manutenção no mercado, pelo orçamento do Estado, e o transporte do estado onde estão; o equipamento novo entra como compra do Estado à indústria bélica | orçamento e compras do Estado (3c); transporte local (3b) |
| 3 | **Recrutamento e reserva** | homens tirados da força de trabalho pela lei `conscription` (`mobilizable`) e a reserva chamada (`call_up`) | efeitos da política ligados (fase 3); pirâmide por idade (demografia nova, PR #54) para tirar homens por idade |
| 4 | **Doutrina** | geração de cada exército, recalculada uma vez por ano (`readiness`, `next_generation`) | indicadores de educação dos oficiais e de controle político (fase 3) |
| 5 | **Guerra pelo comando do jogador** | estado de guerra entre países, frentes (fronteira contínua no grafo de vizinhança dos estados) e segmentos; passo semanal por segmento (`allocate_front`, `land_step`, `breakthrough`, `supply`); ocupação muda o controle do estado, não o dono | 1–3; terreno (`terrain.ron`) |
| 6 | **Ar e mar** | superioridade por zona (`air_step`); frotas em zonas marítimas, comboio, bloqueio como perda do trecho e do comércio, desembarque (`naval_step`, `SeaGraph::route`) | 5; o comércio da fase 4 roteando pelo mesmo grafo |
| 7 | **Custo e fim** | `WarLedger` por país; perdas por idade na pirâmide e devastação de edifícios e transporte (fila de obras); `exhaustion`; paz pelas exigências | 3–6; tratados da fase 4 (reparações como dívida, desarmamento, cessão de território) |
| 8 | **Escada de crise** | guerra que nasce de crise com exigência e prazo; terceiros entram por aliança e garantia | hostilidade e IA estratégica (fase 6); casus belli e pressão histórica (fase 7) |
| 9 | **Arma nuclear** | estoque e dissuasão na escada (`deterrence`, `escalation_limits`); a fatia de uso por último | tecnologia `atomic_bomb` difundida; tratados com verificação (fase 4) |

Até o passo 8, a guerra começa só por comando do jogador ou de teste; a IA não declara. Orçamento:
a guerra mundial de 1939, com todos os segmentos ativos, abaixo de 15 % do tick (decisão 6).

**A paz passa pelos tratados (passo 7, pedido do thread dos tratados depois do #145).** A paz é
uma proposta de `treaty_exec::settle` e `queue`, e valem as regras do reconhecimento:
- toda cláusula contínua (defesa, base, desmilitarização, limite de armas, ocupação) pede que quem
  dá e quem recebe se reconheçam (`treaty::recognition_conflicts`);
- pagamento, cessão e dívida passam sempre;
- em 1925, só os EUA não reconhecem a URSS, a Mongólia é reconhecida só pela URSS e o Tibete por
  ninguém. Uma paz com ocupação entre EUA e URSS seria recusada sem reconhecimento antes;
- o país criado pela paz (`ReleaseCountry`) é reconhecido só pelos signatários, e já pode estar
  em `side_b`;
- na fatia seguinte deles, `queue` devolve o id do acordo e o grava em
  `GameWorld::treaties.agreements` com as cláusulas contínuas. A ocupação e o desarmamento da paz
  moram ali, e o teste mensal de cumprimento lê deles; a guerra não guarda cópia própria.
