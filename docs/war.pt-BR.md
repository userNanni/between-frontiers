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

### Guerra colonial (I3, opção A)

Combinado com o thread das fronteiras (seção 11 de `borders.pt-BR.md`, "A revolta e a guerra"):
a revolta é uma **proclamação seguida de guerra**, como a da Indonésia de 1945 a 1949.
- **Gatilho, sem mudar a regra deles:** a metrópole recusa a exigência e a mobilização da unidade
  passa a marca da revolta. A independência faz o `Secede` sem acordo e grava o `Record` de
  revolta com o índice do país novo (`country`, no #173). A independência só age no 2 de
  janeiro, e a fila de fronteiras roda no mesmo dia.
- **Com `war` ligada,** a guerra lê os `Record` de revolta mais novos que a data guardada e abre
  a guerra: de um lado o antigo dono, com a exigência `Union` da unidade de volta; do outro o país
  novo, sem reconhecimento.
- **Tropas:** o país novo forma unidades irregulares (`irregular`) do tamanho da mobilização vezes
  os homens em idade de serviço da unidade; as formações coloniais do dono que estão ali lutam
  pelo dono. O tipo do vínculo (colônia, protetorado) vem de `IndependenceSetup::bond_of`.
- **Custo e desgaste:** os mesmos da guerra comum. A guerra longa desgasta a metrópole mesmo
  quando ela ganha batalhas: semanas, gasto e preços entram na exaustão.
- **Fim:** o dono sem apoio à guerra aceita a paz por acordo de independência (reconhecimento e
  as cláusulas de independência dos tratados); se a colônia perde, **a metrópole reconquista** o
  território pela `Union`, executada como tratado (G6).
- **Com `war` desligada,** a revolta segue como hoje (separação sem acordo), sem mudar o hash.

### Tela (esboço, para o thread de interface)

Lida do estado que o tick grava, sem refazer a semana:
- **Guerra:** os lados (`world.war.conflicts.wars`: `side_a`, `side_b`, o líder primeiro), as
  semanas desde `since`, a exigência em jogo; o apoio à guerra e a conta entram com a parte G4.
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

### Custo de CPU

Decisão 6: a guerra mundial de 1939, com todos os segmentos ativos, abaixo de 15 % do tick. A
fatia mede com uma guerra de teste de cinco grandes países, porque a IA não declara.

### Decidido pelo dono (G1–G6, 2026-11-18)

| # | Decisão | O que o dono disse |
|---|---|---|
| G1 | só terra nesta fatia; mar e ar na C2 | "A terrestre é a mais importante, mar e ar podem vir depois." |
| G2 | objetivo e postura por frente; formações pela regra aberta | "Em segundo momento podemos ir pensando em táticas melhores": o desenho fica aberto a mais comando (formações por frente, eixo, reserva) |
| G3 | o território ocupado produz e paga ao ocupante, com três acréscimos: a parcela calibrada pela vida real, a devastação e a guarnição | abaixo, "Ocupação" |
| G4 | **mudou:** a rendição e a paz são decididas pelo apoio à guerra, não pela exaustão | "um país pode estar exausto e ainda batalhar, ou pode nem ter cansado e desistir; a medida é apoio de guerra, como a Alemanha na Primeira Guerra e os Estados Unidos no Vietnã" |
| G5 | a IA defende e contra-ataca com 1,5 vez o poder; **pede paz quando não tem mais apoio à guerra** | — |
| G6 | proclamação e guerra entre dois países; **se a colônia perder, a metrópole reconquista** | — |

### Apoio à guerra (G4, desenho para confirmar antes de codar)

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

### Ocupação: o que se extrai, a devastação e a guarnição (G3, desenho para confirmar)

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
