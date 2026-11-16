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

O mapa já tem, por estado, a mistura de climas de Köppen e a área de floresta (`land.ron`). A
regra de `rules.ron` leva cada clima a uma classe de terreno (aberto, floresta, selva, deserto,
ártico) e a floresta soma à classe floresta; montanha, pântano e cidade pedem dado novo (abaixo). O
combate usa a mistura do estado: o fator de defesa é a média dos fatores das classes, pesada pela
parcela de cada uma. Inverno multiplica o consumo e corta a velocidade (`winter`).

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

## Como começa e como acaba

1. **Escada de crise** (`history-rails.pt-BR.md`): tensão, rivalidade, corrida armamentista, crise
   com exigência e prazo apoiada num casus belli, guerra. Cada degrau tem causa visível; a
   declaração seca não existe.
2. **Mobilização:** na crise, o país passa homens para o serviço pela lei de conscrição e põe a
   indústria em pedido de equipamento; a mobilização custa trabalho e confiança antes do primeiro
   tiro.
3. **Coalizão:** aliança chama aliado e garantia chama garantidor (`diplomacy.ron`); cada terceiro
   decide entrar pelo mesmo cálculo de valor contra custo da IA.
4. **Exaustão medida:** `war::exhaustion` soma termos nomeados, cada um com peso em `rules.ron`:
   perdas sobre os homens em idade, mobilizados sobre os homens em idade (trabalho drenado), falta
   de bens de consumo, população ocupada, semanas de guerra e indústria destruída. Vai de 0 a 1.000
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

## Decisões para o dono

| # | Decisão | Recomendação | Como se testa | Quanto custa manter |
|---|---|---|---|---|
| 1 | Controle tático | **Objetivo por frente**, sendo a frente toda a fronteira contínua com um inimigo. O jogador define objetivo, postura (atacar, segurar, recuar), formações e prioridade de suprimento; `allocate` reparte as formações entre os segmentos e o passo semanal anda sozinho. A IA usa os mesmos comandos | guerra de teste com os mesmos comandos dos dois lados dá resultado simétrico; a repartição fica igual de uma semana para a outra quando nada muda | baixo: um comando por frente, sem caminho de unidade nem comando por segmento |
| 2 | Granularidade | **Elemento como conta, divisão como formação** (cerca de 10 elementos), frota e ala no mar e no ar | `forces.ron` de 1925 dá cerca de 700 divisões no mundo; o passo semanal por frente cabe no orçamento do tick | baixo: formação é só agrupamento |
| 3 | Mar com zonas próprias | **Sim, zonas grossas** (cerca de 50, das bacias e estreitos), com portos ligando estados a zonas | teste de conexidade: todo porto alcança todo porto por zonas; rota de comércio e de suprimento passam pelas mesmas zonas | médio: um passo novo no gerador do mapa |
| 4 | Arma nuclear na fase 5 | **Estoque e dissuasão na fase 5; uso numa fatia própria no fim dela**, com consequência: devastação, mortos por idade, precipitação nos vizinhos, hostilidade de todos | teste de 100 anos: com dissuasão dos dois lados, nenhuma guerra direta entre potências nucleares sem crise extrema | médio: o uso pede tela e regra de resposta |
| 5 | Tipos e gerações | **33 tipos com gerações**, primeira fatia com os 22 de 1925–1945 | contagem: nunca mais de 20 tipos em uso numa década | baixo: geração nova é linha de dado |
| 6 | Passo e segmento | **Uma semana por passo, rodando por segmento** (par de estados vizinhos em guerra), alinhada ao tick; o comando fica na frente (decisão 1) | custo por sistema no `bf-bench`: a guerra mundial de 1939 com todos os segmentos abaixo de 15 % do tick | médio: segmentos ativos por semana |
| 7 | Perdas na demografia | **Mortos e inválidos por idade na pirâmide**, civis também, quando a demografia nova (PR #54) entrar | teste: 1914–18 contrafactual na pirâmide francesa mostra o buraco da geração | baixo: uma escrita por semana de guerra |

## Telas que isso vai pedir

Cada uma nasce como painel registrado no HUD modular, com atalho no registro de comandos.

- **Forças do país:** efetivo por domínio e tipo, geração, equipamento contra o necessário,
  treino, custo semanal; quanto da indústria vai para a guerra.
- **Mobilização:** em idade, chamáveis, reservados, disponíveis; efeito no trabalho por profissão.
- **Frentes no mapa:** linha da frente, postura, objetivo, poder dos dois lados.
- **Repartição da frente:** por segmento, extensão, poder inimigo, puxada do objetivo, peso e
  elementos recebidos.
- **Batalha do segmento:** a decomposição do passo (número, equipamento, treino, suprimento,
  terreno, ar) dos dois lados e as perdas da semana.
- **Suprimento:** o caminho até a frente trecho a trecho, o gargalo e o preenchimento.
- **Mar:** zonas, frotas, comboios, bloqueio e perda por zona.
- **Ar:** superioridade por zona, bombardeio e o que ele destruiu.
- **Exaustão e paz:** os termos da exaustão dos dois lados, as exigências na mesa e a proposta.
- **Crise:** a escada com o degrau atual, o casus belli, a exigência, o prazo e quem tomou lado.
- **Lente de guerra no mapa:** ocupação, devastação, terreno de combate.

## Forças de 1º de janeiro de 1925

`data/scenario/1925/forces.ron`, gerado por `tools/war/forces_1925.py` (fontes, licenças e regras
de leitura em `tools/war/SOURCES.md`). Contagem: 44 países, 182 linhas, das quais 30 com
`estimate` e 152 com o número impresso na fonte; 40 países com exército (5,84 milhões de homens),
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

- **Reserva e milícia:** Suíça (500.000 mobilizáveis, nenhum efetivo permanente), Austrália e
  África do Sul (forças de tempo parcial), Guarda Nacional americana, Exército Territorial
  britânico. Pedem uma regra de reserva (homens treinados fora do serviço, chamados na
  mobilização), que entra com a mobilização.
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
- **Mapa:** relevo por estado (parcela de montanha, do `relief.png` ou do Natural Earth), pântano e
  área urbana, para as classes de terreno que faltam; zonas marítimas (decisão 3).
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
