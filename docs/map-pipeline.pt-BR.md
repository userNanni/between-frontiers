# Pipeline do mapa

`bf-mapgen` gera a textura de províncias e a lista de províncias e estados a partir do Natural
Earth, do arquivo de donos e das regras de estados de 1925. Roda offline, uma vez por mudança de dado, e o resultado vai para o
repositório.

```bash
cargo run -p bf-mapgen --release
```

## Entradas

| Arquivo | Origem | Licença |
|---|---|---|
| `data/raw/ne_10m_admin_1_states_provinces_lakes.zip` | Natural Earth 5.1.1, Admin 1 com grandes lagos | Domínio público |
| `data/raw/ne_10m_admin_0_countries_lakes.zip` | Natural Earth 5.1.1, Admin 0 com grandes lagos | Domínio público |
| `data/scenario/1925/province_owners.ron` | Escrito à mão para o jogo | CC BY-SA 4.0 |
| `data/scenario/1925/states.ron` | Escrito à mão para o jogo | CC BY-SA 4.0 |

- Os zips são baixados com `curl` na primeira execução e ficam em `data/raw/` (fora do git).
- Cada zip tem o BLAKE3 fixado em `crates/bf-mapgen/src/sources.rs`. Se o Natural Earth publicar
  outra versão no mesmo endereço, a ferramenta para com erro: leia as notas da versão, troque o
  hash e gere de novo.
- O admin-0 só entra para as unidades sem nenhum polígono no admin-1 (bancos de areia, o campo de
  gelo da Patagônia). As que já estão cobertas pelo admin-1 (Bir Tawil, zona-tampão de Chipre) são
  descartadas; das outras fica só a parte que nenhum polígono do admin-1 cobre. A chave delas é
  `ADM0-<A3>`.
- A Antártida (`ATA`) fica fora.

## Arquivo de donos

`province_owners.ron` tem cinco partes:

1. `countries`: dono padrão por `adm0_a3` do Natural Earth. Toda unidade usada precisa de entrada,
   e entrada que não casa com nenhuma unidade é erro.
2. `provinces`: exceções por `adm1_code`. Chave inexistente é erro.
3. `carves`: recortes. Uma máscara em lon/lat corta um pedaço de uma província e cria outra com a
   chave `<adm1_code>~<sufixo>`. Serve para fronteira que é paralelo, meridiano ou mar (Sacalina no
   paralelo 50, Dodecaneso, Curilas) e para enclave pequeno com contorno conhecido (Weihaiwei e
   Guangzhouwan, do OpenHistoricalMap). O de Danzig é desenhado à mão e grosseiro.
4. `excluded_countries`: unidades fora do mapa.
5. `metropoles`: a metrópole de cada dono que tem colônias (ver abaixo). Gera o campo `colony` de
   `provinces.ron`.

Todo dono precisa estar na lista de tags de 1925 (`TAGS_1925` em
`crates/bf-mapgen/src/owners.rs`; vai para `countries.ron` quando ele existir). Colônia de
administração direta pertence à tag da metrópole.

## Metrópole e colônias

Como no Victoria 3, domínios, o Raj, mandatos, protetorados e satélites com tag própria são
súditos e donos das próprias províncias. Colônia de administração direta é província do dono com
`colony: true` (o estado não incorporado do Victoria 3): a população dela entra em `colonies` de
`countries.ron`, nunca nos números nacionais.

O critério é o estatuto da população em 1º de janeiro de 1925, não o mapa jurídico: metrópole é o
território cujo povo tinha os direitos dos cidadãos (ou súditos) do centro do Estado; o resto que o
dono governa direto é colônia, inclusive mandato e protetorado sem tag. É a regra que põe a Argélia,
francesa na lei, entre as colônias, e as dependências da Coroa, fora do Reino Unido na lei, na
metrópole. Cada entrada de `metropoles` tem:

- `countries`: códigos `adm0_a3` cujas províncias, quando são daquele dono, são metrópole. Província
  do dono em outro `adm0_a3` é colônia (recorte herda o `adm0_a3` da província de origem).
- `provinces`: chaves de fora desses países que também são metrópole (as Curilas, recortadas de
  Sacalina).
- `colonies`: chaves dentro desses países que são colônia mesmo assim (os departamentos
  ultramarinos atuais da França).

Dono que não aparece não tem colônia. O `bf-mapgen` recusa país ou chave que não seja do dono,
entrada redundante e dono que fique sem metrópole ou sem colônia. Testes:
`cargo test -p bf-mapgen` (o mapa gravado bate com a lista de donos) e
`crates/bf-sim/tests/countries_1925.rs` (`colonies` em `countries.ron` existe exatamente para os
donos com província colonial).

| Dono | Metrópole | Colônias | Decisão discutível |
|---|---|---|---|
| GBR | Grã-Bretanha e Irlanda do Norte, com a Ilha de Man, Jersey e Guernsey (com Alderney e Sark) | Todo o resto, inclusive Weihaiwei | **Dependências da Coroa:** não fazem parte do Reino Unido nem votam em Westminster, mas a população era de súditos britânicos com os mesmos direitos, sem estatuto colonial; pela regra da população (a mesma da Argélia), são metrópole. 141 mil pessoas, com os números do Reino Unido |
| FRA | França continental e Córsega | Argélia, Tunísia, Marrocos, Síria e Líbano (mandatos), Indochina com Guangzhouwan, AOF, AEF, Madagascar, Índia, Antilhas, Guiana, Reunião, Pacífico | **Argélia:** três departamentos franceses desde 1848, mas a maioria muçulmana era súdita sob o *indigénat*, sem cidadania, com governador-geral, orçamento próprio (1900) e as Délégations financières. A lei a chamava de França; o Estado a governava como colônia. Fica colônia, como no Victoria 3, que exige incorporar. As "velhas colônias" só viram departamentos em 1946; Mayotte era parte da colônia de Madagascar |
| BEL | Bélgica | Congo, Ruanda-Urundi (mandato) | — |
| NLD | Países Baixos europeus | Índias Orientais, Suriname, Curaçao com Aruba, Bonaire, Saba, Santo Eustáquio, São Martinho | A revisão constitucional de 1922 chamou as colônias de partes do Reino, sem mudar o governo delas |
| ESP | Península, Baleares, Canárias, Ceuta e Melilla | Zona do protetorado no Marrocos (com Tânger), Saara, Guiné | Ceuta e Melilla eram praças de soberania ligadas a Cádis e Málaga |
| POR | Portugal com Açores e Madeira (ilhas adjacentes) | Angola, Moçambique, Guiné, Cabo Verde, São Tomé, Timor, Índia, Macau | — |
| ITA | Itália com a Marcha Juliana, Fiume, Zara e o Vaticano | Líbia, Eritreia, Somália, Dodecaneso | O Dodecaneso era possessão, nunca anexado |
| USA | 48 estados, Alasca e Havaí | Filipinas, Porto Rico, Guam, Samoa, Ilhas Virgens, Guantánamo, ilhas menores | **Alasca e Havaí** eram territórios *incorporados* (Organic Act de 1900; Rassmussen v. US, 1905): a Constituição valia por inteiro e o destino era virar estado. Os *não incorporados* dos Insular Cases são colônia |
| JAP | *Naichi*: ilhas principais, Okinawa e Curilas (Hokkaido) | *Gaichi*: Coreia, Taiwan, Karafuto, Kwantung, Mandato dos Mares do Sul | **Coreia** era anexada (1910), mas governada pelo Governo-Geral, fora da Constituição Meiji e da Dieta. **Karafuto** só vira *naichi* em 1943 |
| DEN | Dinamarca e Faroé | Groenlândia | **Faroé** era um *amt* do reino, com cadeiras no Folketing e no Landsting: metrópole. A Groenlândia foi colônia até 1953 |
| AUS | Austrália (estados, territórios federais, Lord Howe, Macquarie) | Papua, Nova Guiné (mandato), Nauru, Norfolk, Ilhas do Mar de Coral | — |
| NZL | Nova Zelândia com Chatham, Kermadec e as subantárticas | Samoa Ocidental (mandato), Ilhas Cook, Niue | As Cook e Niue foram anexadas aos limites da Nova Zelândia em 1901, mas com governo à parte |
| SAF | União Sul-Africana | Sudoeste Africano (mandato), com Walvis Bay | Walvis Bay era da Província do Cabo, mas administrada com o mandato desde 1922 |

Sem colônia: GER (perdeu as suas em 1919), SOV, CHN e os demais. Os três arrendamentos na China
(Kwantung, Weihaiwei, Guangzhouwan) são recortados da província chinesa e são colônia do
arrendatário; a população deles sai de CHN e entra em `colonies` do dono. RAJ conta a Birmânia, província da Índia
Britânica até 1937, como parte dele.

## Saídas

| Arquivo | Conteúdo |
|---|---|
| `assets/map/provinces.png` | PNG cinza de 16 bits, 8192 × 4103. Pixel = id da província (1..N); 0 = água |
| `assets/map/provinces.ron` | `MapFile` de `bf_sim::scenario`: meta, uma linha por província e uma por estado |
| `assets/map/ATTRIBUTION.md` | Crédito ao Natural Earth |
| `target-preview/*.png` | Prévia por dono (`owners-*`) e por estado (`states-*`), com bordas e recortes de conferência. Não vai para o git |

- **Projeção:** Miller cilíndrica, longitude -180..180, latitude 84 N no topo. A altura é a do
  paralelo 58 S arredondada para pixel inteiro; o `lat_bottom` exato (-57,9957) está no `meta`.
- **Rasterização:** um pixel pertence à província cujo polígono contém o centro dele (par-ímpar,
  com buracos). Sem anti-aliasing. Centro em cima de aresta compartilhada fica com um lado só, então
  não há buraco nem sobreposição entre vizinhos.
- **Ids:** províncias ordenadas pela chave e numeradas de 1 a N, sem buracos. Mudar o conjunto de
  províncias renumera.
- **`center`:** polo de inacessibilidade (algoritmo *polylabel*) da maior parte da província, em
  pixels. Fica dentro do polígono mesmo em província em forma de C, onde o centroide cairia fora.
- **`area_km2`:** área geodésica no elipsoide WGS 84, da geometria inteira.
- **`pixels`:** 0 quando a província é pequena demais para ter um centro de pixel. A província
  continua na lista (Vaticano, ilhotas, recifes). Toda tag tem pelo menos um pixel; Mônaco tem 2.
- **Determinismo:** mesmas entradas, saídas idênticas byte a byte. Nada depende de ordem de
  `HashMap`; os testes conferem que o PNG bate com a contagem de pixels do RON.

Testes: `cargo test -p bf-mapgen` (projeção ida e volta, rasterização com buraco, tags, ids
contíguos, PNG coerente com o RON e as regras de estado, ver abaixo).

## Estados

O estado é a unidade da simulação (população, economia); a província fica como unidade de desenho
e, mais tarde, de frente de guerra. Decisão de `docs/architecture.pt-BR.md` ("Quantos estados"):
cerca de 900, com os estados do Brasil, dos EUA e do México inteiros. O `bf-mapgen` agrupa as
províncias segundo `data/scenario/1925/states.ron` e grava em `provinces.ron`:

- `ProvinceDef.state`: id do estado (1..N, contíguo, na ordem das chaves).
- `MapFile.states`: um `StateDef` por estado, com `key` estável (`BRA-SP`, `FRA-BRETAGNE`), `name`
  (nome próprio, na língua da fonte ou da época), `owner`, `colony`, `center` (polo de
  inacessibilidade da maior parte contínua do estado, calculado na textura), `area_km2` e
  `provinces`.

`provinces.png` não muda: o estado é só metadado. Hoje são **870 estados** (220 coloniais), a
partir de 4.606 províncias; o `bf-mapgen` imprime a distribuição por dono e os maiores e menores.

### Regras

Aplicadas nesta ordem:

1. `states`: estados explícitos, com a lista de províncias. Cada item é uma chave de província
   (`adm1_code`, chave de recorte `<origem>~<sufixo>`, `ADM0-<A3>`) ou um código ISO 3166-2, que
   pega todas as províncias com aquele código (o Natural Earth parte algumas unidades ISO, como
   Dublin e Manila). Uma província só pode estar num estado explícito.
2. `attach`: ilhotas e fragmentos que entram no estado de outra província (Vaticano em Roma,
   Scarborough em Zambales, a ilha Macquarie na Tasmânia, os fragmentos `+99?` do Natural Earth).
   Origem e destino precisam ter o mesmo dono e o mesmo estatuto colonial.
3. `countries`: regra por `adm0_a3` para o que sobrou.
   - `Admin1`: cada unidade de primeiro nível vira um estado, com chave `<A3>-<sufixo ISO>`.
   - `Whole(name, key)`: a unidade inteira vira um estado. Unidades com a mesma `key` formam um
     estado só (Ilhas de Sotavento, Palestina, Chipre), e uma `key` igual à de um estado explícito
     junta-se a ele: é assim que o "resto" de um país vira um estado (Nord-Norge, Centro do Peru).
   - `Field(field, names)`: um estado por valor de `region` ou `region_sub` do Natural Earth;
     `names` renomeia valores, e valores com o mesmo nome se juntam.
4. Recorte (`~`) fora de estado explícito vira estado próprio (`RUS-KARAFUTO`, `POL-DANZIG`,
   `CHN-KWANTUNG`, `CHN-WEIHAIWEI`, `CHN-GUANGZHOUWAN`, `GRC-DODECANESE`). Unidade sem regra vira
   um estado por província.
5. Um grupo que misturaria donos, ou colônia e metrópole em 1º de janeiro de 1925, é partido: a
   maior parte fica com a chave e o nome, as outras recebem chave própria e o `bf-mapgen` imprime
   uma nota. Hoje nenhum grupo é partido; as regras já separam esses casos.
6. `tiny`: estados sem nenhum pixel que podem existir assim (Gibraltar, Spratly, Ashmore, Coral
   Sea). Todo outro estado precisa de pelo menos um pixel, e entrada de `tiny` que tenha pixel é
   erro.

O `bf-mapgen` recusa seletor que não casa com província, província em dois estados explícitos,
regra para país inexistente, nome de `Field` que não aparece e chave repetida. Testes
(`cargo test -p bf-mapgen`): toda província num estado só, ids contíguos e chaves ordenadas,
nenhum estado mistura donos nem estatuto colonial, contagem entre 850 e 950, pixel ou `tiny`, e
`center` dentro do próprio estado.

### Famílias de regra

| Família | Onde | Como |
|---|---|---|
| Primeiro nível inteiro | EUA (48 estados, Alasca, Havaí, Distrito de Colúmbia), México (31 estados e DF), Canadá, Áustria, Bélgica, Países Baixos, Polônia, Ucrânia e RSFSR (oblasts e repúblicas, perto das *guberniyas* e autonomias de 1925), China, Austrália | `Admin1`, com fusões explícitas onde a unidade moderna não existia |
| Estados de 1925 refeitos com unidades modernas | Brasil, China, Índia britânica, Alemanha, Polônia, Romênia, Coreia, Itália, Argélia, Congo Belga, Nigéria, África do Sul | Listas explícitas por código ISO |
| Regiões históricas | França (regiões de 1956–2015, perto das províncias do Antigo Regime), Reino Unido (regiões estatísticas, Escócia em três), Irlanda (quatro províncias), Espanha (regiões históricas), Japão (regiões tradicionais), Suécia (*landsdelar*), Noruega (*landsdeler*), Finlândia (*läänit*), Tchecoslováquia (terras), Suíça (*Grossregionen*), Turquia (NUTS-1), Pérsia, Iraque (os três vilaiete otomanos), Síria (os Estados do mandato) | Explícitas ou `Field` com `names` |
| Agrupamento por região do Natural Earth | Hungria, Portugal, Sião, Filipinas, Ceilão (as nove províncias), Uganda (as quatro províncias), Sudão anglo-egípcio, Azerbaijão | `Field` |
| País ou colônia inteiro | Países pequenos (Letônia, Estônia, Bulgária em duas, América Central, Uruguai, Paraguai…) e quase todas as colônias (AOF e AEF colônia a colônia, Quênia em três, Tanganica, Rodésias…) | `Whole`, com o nome da época (Haute-Volta, Dahomey, Gold Coast, Nan'yō) |

### Aproximações principais

- **Brasil:** os 26 estados e o DF modernos viram os 20 estados e o território do Acre de 1925:
  Tocantins e o DF atual voltam a Goiás, Mato Grosso do Sul e Rondônia a Mato Grosso, Roraima ao
  Amazonas, Amapá ao Pará. O Distrito Federal de 1925 (a cidade do Rio) não tem unidade própria no
  Natural Earth e fica dentro do estado do Rio de Janeiro. São 21 estados.
- **EUA e México:** unidades modernas; as fronteiras são as de 1925.
- **Alemanha:** Länder modernos (Baden e Württemberg juntos, Hannover e Oldenburg dentro da Baixa
  Saxônia, a Renânia e a Vestfália juntas, Anhalt na Saxônia-Anhalt). Ostpreußen, Pommern,
  Nieder- e Oberschlesien saem das voivodias polonesas; a Lubúsquia entra em Brandemburgo.
- **Itália:** regiões modernas como *compartimenti*: Vale de Aosta no Piemonte, Molise com o
  Abruzzo, Friul no Vêneto; a Venezia Giulia junta Gorizia, Trieste, o Litoral esloveno e a Ístria.
- **França:** Normandia numa peça só; Aquitânia, Centre e Ródano-Alpes partidos em Guyenne,
  Gascogne, Orléanais, Berry-Touraine e Savoie.
- **Índia britânica:** só o primeiro nível moderno. Bengala sai em quatro (oeste com Khulna, Daca,
  Chittagong, Rajshahi), mas as Províncias Unidas, Bihar e os grandes estados principescos não se
  dividem: Hyderabad é Telangana, Mysore é Karnataka, a Rajputana é o Rajastão, a Índia Central é
  Madhya Pradesh. Sylhet vai para Assam, como em 1925.
- **China:** províncias modernas com Chongqing em Sichuan, Hainan em Guangdong, Ningxia em Gansu,
  Pequim e Tianjin em Zhili, Xangai em Jiangsu. Rehe, Chahar e Suiyuan ficam dentro da Mongólia
  Interior.
- **URSS:** oblasts modernos, com fusões onde a divisão é posterior (Moscou e Leningrado com as
  cidades, okrugs autônomos dentro das oblasts, Adiguéia no Kuban). As repúblicas da Ásia Central
  saem por região histórica (Bucara, Samarcanda, Fergana, Khorezm); o Cazaquistão em quatro.
- **Japão:** regiões tradicionais partidas onde passavam de 4 milhões de pessoas (Kantō, Kinki,
  Kyūshū, Tōhoku).
- **Etiópia:** regiões modernas no lugar das províncias de 1925, com Adis Abeba em Shewa.
- **Coreia:** as treze províncias do Governo-Geral, refeitas com as províncias e cidades das duas
  Coreias.

Faltam para chegar a 900, e voltam quando houver desenho abaixo do primeiro nível
(OpenHistoricalMap): Bengala ocidental, Províncias Unidas e Sichuan divididas, os maiores estados
principescos separados das províncias britânicas, e o Distrito Federal brasileiro.

## Aproximações

O Natural Earth tem fronteiras atuais. Onde o primeiro nível administrativo não exprime a
fronteira de 1925, a unidade inteira vai para o dono da maior parte. Todas serão redesenhadas a
partir do OpenHistoricalMap.

### Recortes (fronteira certa, ou quase)

| Recorte | Como | Qualidade |
|---|---|---|
| Dodecaneso (ITA), de Notio Aigaio | Ilhas a leste de 26,2° E | Exato |
| Karafuto (JAP), de Sacalina | Ilha ao sul de 50° N | Exato |
| Curilas (JAP), de Sacalina | Ilhas a leste de 145,2° E | Exato |
| Kwantung (JAP), de Liaoning | Península ao sul de 39,45° N | Retângulo; área próxima da real (3.530 × 3.462 km²) |
| Weihaiwei (GBR), de Shandong | Contorno do OpenHistoricalMap (relação 2848211, CC0), simplificado a 0,004° | Bom: 772 km² de terra contra 746 km² reais (288 milhas², com Liugong). A ilha de Liugong não existe no Natural Earth 1:10m; 45 pixels |
| Guangzhouwan (FRA), de Guangdong | Contorno do OpenHistoricalMap (relação 2889933, CC0), simplificado a 0,004° | Razoável: 986 km² de terra contra 842 km² de terra firme reais (84.244 ha; 1.300 km² com a água da baía). Entram Donghai, Naozhou e Nansan; a sobra vem da costa generalizada do Natural Earth 1:10m e do contorno largo no interior; 48 pixels |
| Danzig (DNZ), de Pomerânia | Polígono desenhado à mão | Grosseiro: 1.593 km² contra 1.966 km² reais |

### Unidade inteira para o dono da maior parte

**Polônia e Alemanha**

- Silésia (Śląskie) → POL. Gliwice, Bytom e Zabrze ficaram com a Alemanha em 1922.
- Opole → GER. Partes do leste foram para a Polônia.
- Pomerânia (Pomorskie), fora Danzig → POL. Lębork, Bytów, Słupsk, Malbork e Sztum eram alemães.
- Grande Polônia (Wielkopolskie) → POL. A franja oeste era alemã.
- Vármia-Masúria → GER. Działdowo (Soldau) era polonês.
- Lubúsquia → GER. Algumas áreas do leste eram polonesas.
- Sarre: o Sarre atual (2.570 km²) é maior que o Território do Sarre de 1925 (1.912 km²).

**Linha polonesa-soviética e Lituânia**

- Minsk e Vitebsk → SOV. Os distritos do oeste (Maladzyechna, Vileyka, Myadzel, Valozhyn,
  Stowbtsy, Nesvizh; Braslaw, Pastavy, Hlybokaye, Miory) eram poloneses.
- Vilnius (condado) → POL. Ukmergė e Širvintos eram lituanos.
- Utena → LTU. Ignalina e Visaginas eram poloneses.
- Alytus → LTU. O sul de Varėna era polonês.

**Fronteiras soviéticas a oeste**

- Odessa → SOV. O Budjak (sul da Bessarábia: Izmail, Bilhorod-Dnistrovskyi, Bolhrad) era romeno.
- Oblast de Leningrado → SOV. O norte do istmo da Carélia (Viipuri) era finlandês; Ivangorod,
  estoniano.
- Carélia (república) → SOV. A Carélia do Ladoga (Sortavala, Suojärvi) era finlandesa.
- Murmansk → SOV. Petsamo era finlandês.
- Pskov → SOV. Petseri (Pechory) era estoniano; Abrene (Pytalovo), letão.
- Sacalina do norte: ocupada pelo Japão até maio de 1925; fica SOV, que tinha a soberania.

**Adriático e Bálcãs**

- Primorje-Gorski Kotar → YUG. Fiume (Rijeka), Abbazia (Opatija) e Cres-Lošinj eram italianos.
- Zadar → YUG. A cidade de Zara era italiana.
- Dubrovnik-Neretva → YUG. Lagosta (Lastovo) era italiana.
- Eslovênia, por município: o Litoral esloveno vai para ITA. Logatec, Žiri e Gorenja vas-Poljane
  ficam YUG, embora uma faixa do oeste (Hotedršica) fosse italiana.
- Vlorë → ALB. A ilha de Saseno era italiana.

**Marrocos**

- Tânger-Tetuão → ESP. A zona internacional de Tânger fica dentro.
- Taza-Al Hoceima-Taounate → FRA. Al Hoceima era da zona espanhola.
- Oriental → FRA. Nador era da zona espanhola.
- Guelmim-Es-Semara → FRA. O enclave de Ifni era espanhol (ocupado só em 1934).
- A República do Rif (Abd el-Krim) controlava em 1925 a maior parte da zona espanhola. Não há tag;
  fica ESP.

**Oriente Médio**

- Meca (região) → HEJ. Ibn Saud tomou Meca e Taif em outubro de 1924; Jidá resistia.
- Al Bahah → HEJ, com a mesma ressalva.
- Aqaba e Ma'an → HEJ. A Grã-Bretanha só anexou a região à Transjordânia em meados de 1925.
- Hodeidah e Hajjah → YEM. A Tihama, com Hodeidah, era do Idrisi de Asir até abril de 1925.
- Al Dali' → YEM. O Imã ocupava a parte do Protetorado de Áden.
- Lahij e o resto do Protetorado de Áden → GBR, embora o Imã ocupasse trechos até 1928.
- Golã: segue o recorte do Natural Earth (Israel ou Síria). Em 1925 era do mandato francês.
- Zona neutra Kuwait-Arábia: segue o recorte atual.

**Ásia**

- Bengala Ocidental → RAJ. Chandernagore era francesa.
- Balochistão → RAJ. Gwadar era de Omã.
- Puducherry: todos os enclaves (Pondicherry, Karaikal, Mahé, Yanam) → FRA.
- Dadra e Nagar Haveli e Damão e Diu → POR.

**Américas**

- Panamá e Colón → PAN. A Zona do Canal era dos EUA.
- Amazonas (Colômbia) → COL. O trapézio de Letícia e a margem norte do Putumayo estavam sob o Peru
  até 1930.
- Chaco paraguaio → PRY. A Bolívia tinha fortins no oeste.
- Tacna → CHL. Tarata voltou ao Peru em setembro de 1925.

**África**

- Al Butnan → ITA. Jaghbub era egípcio até dezembro de 1925.
- As fronteiras do Togo e dos Camarões saem certas no primeiro nível: o Togo britânico está em
  Gana, os Camarões do Sul são Noroeste e Sudoeste, e os do Norte estão na Nigéria.

### Donos sem tag própria

A lista de tags não tem estes casos. Ficam com o dono mais próximo:

| Lugar | Dono | Situação em 1925 |
|---|---|---|
| República do Rif | ESP | Independente de fato |
| Asir e Jizan (Idrisi) | NEJ | Emirado sob proteção saudita a partir de 1926 |
| Novas Hébridas (Vanuatu) | GBR | Condomínio anglo-francês |
| Tonga | GBR | Estado protegido britânico |
| Spratly | GBR | Reivindicação britânica de 1877; França anexa em 1933 |
| Scarborough | USA | Junto das Filipinas |
| Bouvet | GBR | Reivindicação britânica; Noruega anexa em 1927–1928 |
| Bajo Nuevo e Serranilla | COL | Reivindicados pela Colômbia e pelos EUA |
| Campo de gelo da Patagônia | CHL | Sem demarcação |
| Tokelau | GBR | Ilhas Gilbert e Ellice até 1926 |
| Vaticano | ITA | Tratado de Latrão só em 1929 |
| Tânger | ESP | Zona internacional (convênio em vigor em junho de 1925) |
| Sudão | GBR | Condomínio anglo-egípcio |

### Seguindo a lista de tags, contra a data

- Jubalândia → ITA. A transferência do Quênia para a Itália foi em 29 de junho de 1925.
- Svalbard → NOR. A soberania norueguesa entrou em vigor em 14 de agosto de 1925.

## Zonas marítimas

`bf-mapgen seas` corta a água do mapa em zonas marítimas e grava o grafo que a guerra (comboio,
bloqueio, desembarque) e o comércio da fase 4 vão usar. Ele lê a `provinces.png` já gravada, então
as províncias não mudam:

```bash
cargo run -p bf-mapgen --release -- seas            # gera seas.png e seas.ron
cargo run -p bf-mapgen --release -- seas --survey   # lista os polígonos marinhos, para escrever as regras
```

| Arquivo | Conteúdo |
|---|---|
| `data/raw/ne_10m_geography_marine_polys.zip` | Natural Earth 5.1.0, polígonos marinhos com nome (306: oceanos, mares, golfos, baías, estreitos). Domínio público, hash fixo em `sources.rs` |
| `data/map/sea_zones.ron` | Regras escritas à mão: zonas, gelo, estreitos e canais (CC BY-SA 4.0) |
| `assets/map/seas.png` | PNG cinza de 8 bits do tamanho da `provinces.png`. Pixel = id da zona (1..N); 0 = terra ou lago. É para o thread de interface; o cliente ainda não lê |
| `assets/map/seas.ron` | `SeasFile` de `bf_sim::sea`: zonas, contatos, portos, estreitos e canais |

Em uso: **63 zonas**, 98 contatos, 24 estreitos e canais, 713 portos (estado × zona) em 497 estados
costeiros. O `bf_sim::sea::SeaGraph` carrega o grafo (`GameData::seas`) e responde
`route(de, para, opções)`. Nada disso está no tick.

### Recorte

- Cada zona é uma lista de polígonos do Natural Earth, pelo nome. Um polígono pode vir cortado por
  uma caixa lon/lat: assim o Atlântico Norte se divide em oeste, centro, nordeste e sudeste, e o
  Barents, em sul (livre de gelo) e norte. O pixel fica com a primeira zona do arquivo que o
  reivindica, então os mares vêm antes dos oceanos em volta.
- O pixel de água que nenhum polígono listado cobre vai para a zona mais próxima pela água: busca
  em largura na ordem dos pixels, determinística. Isso cobre golfos e baías não listados (179
  polígonos) e a faixa ao sul do Cabo Horn. Água que não toca nenhuma zona (lagos) fica fora.
- São 63 zonas, mais que as cerca de 50 do desenho. As que passaram são os mares pequenos que um
  estreito fecha (Mármara, Azov, Kattegat, Golfo de Finlândia e de Bótnia, Mar Branco) e os
  oceanos partidos para que o comboio tenha onde estar.
- O Cáspio é zona isolada (`isolated`): o único fora da regra de que todo porto alcança todo porto.

### Portos, contatos e distância

- **Porto:** o estado com pixel de terra encostado (vizinhança de 4, com a volta em ±180°) numa
  zona tem porto nela. O peso é a extensão do contato em arestas de pixel, **só geometria**. A
  capacidade do porto vem depois, com a infraestrutura portuária da economia.
- **Contato:** duas zonas com pixels encostados.
- **Distância:** geodésica entre os polos de inacessibilidade das zonas (o mesmo cálculo do
  `center` dos estados) e do centro do estado ao centro da zona. É grosseira: serve para comparar
  rotas, não para medir viagem.

### Estreitos e canais

Critério (pedido do dono): entra o que **pode ser fechado por quem tem as margens e cujo
fechamento muda a rota ou prende uma frota**. O estreito é uma aresta declarada que substitui o
contato natural entre as duas zonas: fechar tira a passagem. Ele também cria a aresta quando a
textura, a cerca de 5 km por pixel, não mostra água (o Bósforo tem 3 pixels).

| Entram | Tipo |
|---|---|
| Gibraltar, Dover, Øresund, os Belts, Messina, Otranto, Dardanelos, Bósforo, Kerch, Bab-el-Mandeb, Ormuz, Malaca (com Singapura), Sunda, Lombok, Tsushima, Tsugaru, La Pérouse, Tartária, Bering, Magalhães | estreito |
| Suez (1869), Panamá (1914), Kiel (1895), Corinto (1893) | canal, fechado antes da data de abertura |

Ficam como contato comum, porque ninguém fecha das margens ou porque há outro caminho ao lado:
Groenlândia–Islândia–Reino Unido, o Canal de Moçambique, o Estreito da Sicília (largo), o
Estreito de Taiwan (passa-se pelo leste da ilha), o Estreito da Flórida, o Canal de Iucatã, Belle
Isle e Cabot (há dois caminhos para o Golfo de São Lourenço) e a Passagem de Drake. Magalhães entra
porque o Cabo Horn fica como contato aberto: fechar o estreito alonga a rota, não a corta.

**Quem controla.** O arquivo grava os estados das margens, nunca o dono. O controlador sai do dono
desses estados no momento (`SeaGraph::controllers`), então muda quando a margem muda de mão.
Margens de donos diferentes (Øresund: Dinamarca e Suécia; Bab-el-Mandeb: Iêmen, Reino Unido,
França e Itália): **basta um fechar**. Uma bateria, um campo minado ou uma frota de um lado só
fecham a passagem, como na prática. Teste: a Suécia fecha o Øresund mas não os Belts, e o Báltico
fica aberto; a Dinamarca fecha os dois, e sobra o canal de Kiel; Dinamarca e Alemanha juntas
fecham o Báltico. **Detentor por tratado** (pedido do orquestrador): o controle por tratado ou ocupação vem antes
do dono da margem. O campo opcional `holders` da passagem guarda país, início, fim de referência e
fonte. O detentor em vigor na data de início é o estado inicial (`Holders::on`), e quem detém fecha
e abre mesmo sem ser dono da margem. As datas de fim são história, não roteiro: quem tira o
detentor é a diplomacia da fase 4 (denúncia do tratado, nacionalização, guerra). Em 1925 há dois:

| Passagem | Detentor | Desde | Fim de referência | Fonte |
|---|---|---|---|---|
| Suez | GBR | 1922-02-28 | 1956-06 | ponto reservado (a) da declaração de independência egípcia de 1922; tratado anglo-egípcio de 1936; retirada concluída em junho de 1956 (Wikipedia) |
| Panamá | USA | 1904-02-26 | 1999-12-31 | Tratado Hay–Bunau-Varilla, arts. III e XXIII (Avalon Project); Tratados Torrijos–Carter (Wikipedia) |

Ficaram de fora, pela fonte:
- **Estreitos turcos:** a comissão internacional de Lausanne (1923–1936) supervisiona, mas não
  fecha. Pela convenção, a Turquia, dona das margens, só pode deter navios inimigos quando
  beligerante, e isso está no `regime`.
- **Gibraltar e Áden:** já são estados próprios do Reino Unido (`GIB` e `YEM-ADEN`), então entram
  como margem.
- **Kiel:** segue alemão. Versalhes só obriga a abrir o canal a quem está em paz com a Alemanha.

Teste: em 1925 o Egito sozinho não fecha Suez e o Panamá não fecha o canal; sem os detentores,
fecham.

**Regime jurídico.** O campo `regime` cita o tratado que governa a passagem, com a fonte, para a
fase 4 ligar "fechar é romper tratado":
- Convenção de Constantinopla de 1888 (Suez);
- Convenção dos Estreitos de Lausanne (1923), em vigor em 1925, depois Montreux (1936);
- Hay–Bunau-Varilla (1903) e Torrijos–Carter (1977), para o Panamá;
- Versalhes, arts. 380–386 (Kiel);
- Convenção de Copenhague de 1857 (estreitos dinamarqueses);
- tratado de limites Chile–Argentina de 1881 (Magalhães);
- passagem em trânsito da CNUDM de 1982, para os estreitos sem tratado próprio.

Sem comportamento agora.

### Gelo

Atributo por zona: livre, sazonal (fechado no inverno do norte sem quebra-gelo) ou permanente
(fechado nas duas estações do jogo sem quebra-gelo). Toda zona com gelo cita a fonte em
`sea_zones.ron` (Wikipedia, NOAA, só fatos). `route` recebe a estação e se há quebra-gelo.

- **Livres:** o Barents sul, onde está Murmansk (a Corrente do Atlântico Norte), e o Mar da
  Noruega. Teste: Murmansk sai para Londres no inverno.
- **Sazonais:**
  - o Mar Branco (Arkhangelsk, de outubro/novembro a abril/maio; teste: fechado no inverno, aberto
    com quebra-gelo);
  - Bótnia e Finlândia (Leningrado);
  - Azov, Okhotsk, Bering, Hudson, Labrador–Baffin, Groenlândia, Golfo de São Lourenço (Montreal
    só passa a ter navegação no ano todo em 1964);
  - o Cáspio.
- **Permanentes:** o Oceano Ártico, o arquipélago canadense e o Beaufort, e também Kara,
  Laptev–Sibéria Oriental e Chukchi. Esses mares abrem só 2 a 4 meses por ano e precisam de
  quebra-gelo na Rota do Mar do Norte (a primeira travessia numa temporada é de 1932). Abertos no
  "verão" do jogo, fariam a rota do Ártico mais curta que Suez já em 1925.

Limite conhecido: o gelo é da zona inteira. O Golfo de Pedro, o Grande (Vladivostok) congela de
dezembro a abril, mas o Mar do Japão é livre; o Bo Hai tem gelo na Baía de Liaodong, mas fica com o
Mar Amarelo. Quando o porto contar (fase 4), o gelo pode ir para o porto.

### Testes

- `cargo test -p bf-mapgen seas`:
  - o gerador é determinístico num mundo sintético e preenche a água aberta sem tocar no lago;
  - regras quebradas são recusadas;
  - o `seas.ron` gravado bate com as regras e com a contagem de pixels do `seas.png`.
- `crates/bf-sim/tests/sea_zones.rs`:
  - todo porto alcança todo porto, menos o Cáspio;
  - todo estreito liga duas zonas e tem margem;
  - fechar Suez leva Bombaim–Londres para o Cabo;
  - fechar Malaca desvia por Sunda ou Lombok;
  - fechar o Bósforo ou os Dardanelos fecha o Mar Negro;
  - Murmansk segue aberto no inverno e Arkhangelsk não;
  - o controle segue a margem;
  - o Panamá abre em 1914, e Magalhães tem o Horn atrás.
- Duas rodadas do gerador dão os mesmos bytes.
