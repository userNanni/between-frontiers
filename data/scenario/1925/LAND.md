# Cenário de 1925: terra e clima por estado

`land.ron` (formato `bf_sim::resources::LandFile`) dá, para cada um dos 870 estados do mapa, as
parcelas de clima, a lavoura, o pasto e a floresta por volta de 1925, se o estado tem costa e a
aptidão (0 a 1) de cada tipo de fazenda. Licença dos dados: CC BY-SA 4.0, com as atribuições
abaixo.

## Fontes e licenças

| Fonte | O que dá | Licença conferida |
|---|---|---|
| Beck, H. E., Zimmermann, N. E., McVicar, T. R., Vergopolan, N., Berg, A., Wood, E. F. (2018). "Present and future Köppen-Geiger climate classification maps at 1-km resolution". *Scientific Data* 5, 180214. doi:10.1038/sdata.2018.214. Arquivo `Beck_KG_V1.zip`, figshare, artigo 6396959 (versão 2) | Classe de Köppen–Geiger em 1 km e 10 km (`Beck_KG_V1_present_0p0083.tif`, `_0p083.tif`: clima de 1980–2016) | **CC BY 4.0** na página do figshare (API do figshare, campo `license`, consultada em 2026-11-15) |
| Klein Goldewijk, K. (2023). *History Database of the Global Environment 3.3* (HYDE 3.3), Universidade de Utrecht, doi:10.24416/UU01-AEZZIT. Cenário de base (`hyde33_c7_base_mrt2023`), arquivos `1920AD_lu.zip`, `1930AD_lu.zip` e `general_files.zip` | Lavoura, pasto, pastagem natural, irrigação e arroz em 1920 e 1930 (grade de 5'), área máxima de terra por célula, máscara de floresta potencial (`forest_wwf_cr.asc`) | **CC BY 4.0** no DataCite e na página pública do Yoda; **CC BY 3.0** no `readme_HYDE_3.3.txt` do pacote; **CC BY-NC-SA 4.0** no `yoda-metadata.json` do pacote. Ver "Dúvidas" |
| `assets/map/provinces.png` e `provinces.ron` do jogo | Ids de província em projeção de Miller, estados, áreas | CC BY-SA 4.0 (Natural Earth, domínio público, na origem) |

Os arquivos grandes ficam em `data/raw/` (ignorado pelo git). Para reproduzir, da raiz do
repositório:

```bash
mkdir -p data/raw/koppen data/raw/hyde33/asc
curl -L -o data/raw/koppen/Beck_KG_V1.zip https://ndownloader.figshare.com/files/12407516
unzip -o data/raw/koppen/Beck_KG_V1.zip 'Beck_KG_V1_present_0p0083.tif' 'Beck_KG_V1_present_0p083.tif' -d data/raw/koppen
# HYDE 3.3: https://geo.public.data.uu.nl/vault-hyde/HYDE%203.3%5B1701183392%5D/original/
#   general_files.zip e hyde33_c7_base_mrt2023/zip/{1920AD_lu,1930AD_lu}.zip
#   (o servidor pede ao navegador uma prova de trabalho, o Anubis; baixe pelo navegador)
unzip -o -j data/raw/hyde33/general_files.zip 'general_files/*.asc' -d data/raw/hyde33/asc
unzip -o -j data/raw/hyde33/1920AD_lu.zip -d data/raw/hyde33/asc
unzip -o -j data/raw/hyde33/1930AD_lu.zip -d data/raw/hyde33/asc
python3 -I tools/land/aggregate.py   # rasters → tools/land/state_stats.csv (~25 s, ~3 GB de RAM)
python3 -I tools/land/farms.py       # state_stats.csv → land.ron e tools/land/check.md
```

Só numpy e Pillow; `tools/land/state_stats.csv` (versionado) guarda o resultado da agregação, de
modo que as regras de fazenda se refazem sem baixar nada.

## Método

**Do raster ao estado.** Cada pixel de terra de `provinces.png` (8192 × 4103, Miller, 84° N a
58° S) volta a lon/lat pela mesma fórmula de `crates/bf-mapgen/src/projection.rs` e pesa a sua área
verdadeira na esfera (raio 6371 km). O id da província dá o estado.

**Clima.** A classe de Köppen de 1 km no centro do pixel; onde o mapa de Beck é mar e o nosso é
terra (diferença de litoral), vale a classe de terra mais próxima no mapa de 10 km. As 30 classes
de Köppen viram as 14 chaves de `CLIMATES` (As não existe no mapa de Beck; Cwb e Cwc contam como
`oceanic`, Cwa como `humid_subtropical`, Ds* como continental ou subártico pelo verão). Quatro
estados sem pixel (Ashmore e Cartier, Coral Sea, Gibraltar, Spratly) usam a classe do centro das
suas províncias. As parcelas são arredondadas a 3 casas, as menores que 0,0005 caem e o resto é
reescalado para somar 1.

**Lavoura e pasto (1925 = média de 1920 e 1930 do HYDE).** Os km² de cada célula de 5' do HYDE se
repartem entre os pixels de terra do nosso mapa dentro da célula, pela área; assim o total do mundo
se conserva. Célula do HYDE sem nenhum pixel de terra no nosso mapa (litoral diferente) vai inteira
para o estado do pixel de terra mais próximo (0,01 Mkm² de lavoura). `cropland_km2` é a lavoura
(`cropland`); `pasture_km2` é o pastoreio todo do HYDE (`grazing` = pasto manejado + pastagem
natural convertida + pastagem natural), como pede o contrato ("pasture and rangeland"). Se lavoura
+ pasto passam da área do estado (só em estados minúsculos), os dois são reduzidos na proporção.

**Floresta (método próprio, a parte mais fraca).** O HYDE não tem floresta; ele traz a máscara de
floresta potencial do WWF (`forest_wwf_cr.asc`, 0/1 por célula), usada nos anthromes. Por célula:

1. potencial = área de terra (`maxln_cr`) se a célula é de floresta potencial;
2. desmatado = (lavoura + pasto manejado) × fração potencial da célula + pastagem natural
   convertida (que o HYDE define como antiga floresta). Pastagem natural não convertida é campo,
   cerrado ou estepe e não desconta;
3. floresta = (potencial − desmatado) × (1 − u), com u = (lavoura + pasto + pastagem convertida) /
   área de terra. O fator (1 − u) existe porque, numa paisagem agrícola, o que sobra fora de roça
   e pasto é sobretudo aldeia, estrada, pousio, charneca e terra baldia, não mata. Sem ele, a
   Inglaterra ficava com 27 % de floresta e Berlim com 78 %.

Fraquezas: não vê exploração madeireira, degradação, lavoura itinerante nem florestas manejadas
da Europa Central (a Alemanha sai com 16 % contra uns 27 % na época); a máscara do WWF é 0/1 por
célula de ~9 km, então savana arborizada e cerrado ficam inteiros dentro ou fora; e onde o HYDE
subestima a lavoura de 1925 (Brasil, Cuba) a floresta fica alta demais (Camagüey, em plena
expansão do açúcar, sai quase toda coberta). Conferência por país (fronteiras atuais, grade de
países do HYDE), em % da terra:

| País | Este método | Sem o fator (1 − u) | Ordem de grandeza na década de 1920 (aprox.) |
|---|---|---|---|
| Reino Unido | 10 | 27 | ~5 |
| França | 19 | 31 | ~19 |
| Alemanha | 16 | 27 | ~27 |
| Polônia | 8 | 14 | ~22 |
| Japão | 79 | 88 | ~60 |
| Índia | 28 | 42 | ~20 |
| EUA | 28 | 34 | ~32 |
| Finlândia | 84 | 90 | ~73 |
| Brasil | 60 | 62 | ~60 |

A última coluna é aproximada e de memória (estatísticas florestais nacionais da época e Zon &
Sparhawk, *Forest Resources of the World*, 1923); serve para ver o erro de ordem de grandeza, não
para calibrar.

**Costa.** Um estado tem costa se algum pixel de terra seu tem um vizinho (4-vizinhança, com a
volta na longitude 180°) que é mar. Mar é a água conectada a um corpo de água de 250 000 km² ou
mais (o oceano, e o Cáspio, com 371 000 km², que tem portos e pesca), ou que chega a 3 pixels
(~15 km) de um deles, para reabrir estreitos que o raster fecha (Bósforo, Kerch). Os lagos ficam de
fora: Grandes Lagos, Vitória, Titicaca, Ladoga, Aral. Província sem pixel conta como costeira se o
seu centro cai no mar. Resultado: 501 estados com costa. Hamburgo não tem costa (é porto de rio).

## Regras de aptidão das fazendas

Tudo em `tools/land/farms.py`; os números abaixo são os do código.

1. **Zonas.** Às 14 chaves de clima o código soma três zonas que elas escondem: `cont_hot`
   (continental de verão quente, Dfa/Dwa/Dsa, tirada de `humid_continental`), e, nos estados com
   centro a menos de 25° do equador, `highland_cool` (oceânico e mediterrâneo nos trópicos: os
   planaltos Cwb/Cfb/Csb) e `highland_warm` (subtropical úmido nos trópicos: Cwa/Cfa de planalto
   baixo), que saem de `oceanic`, `mediterranean` e `humid_subtropical`.
2. **Ajuste climático** = Σ peso × parcela (tabela).
3. **Irrigação**: + peso × (área irrigada / lavoura, do HYDE) × parcela seca (BWh + BWk + BSh +
   BSk). É o que põe algodão no Egito, no Turquestão e no Punjab, e trigo no Nilo.
4. **Escala**: cultura de exportação tem o ajuste climático multiplicado pela escala (0,35 a 0,7).
   O clima dá o potencial; quem plantava de fato em 1925 recebe um piso (passo 8). Sem isso, chá
   se espalhava por todo Cwa da Índia e café por todo Aw, porque Köppen não vê chuva anual, solo
   ácido nem altitude.
5. **Arroz**: onde o HYDE tem arroz (> 2 % da lavoura), aptidão = máx(0,8 × ajuste, 0,3 + 2 ×
   parcela de arroz); estado tropical seco (Aw + BSh > 50 %) sem arroz no HYDE fica com metade.
6. **Pecuária** = 0,55 × ajuste + 0,6 × mín(1, pastoreio / área). **Leite** = 0,7 × ajuste + 0,3 ×
   mín(1, 2 × pasto manejado / área): o pasto manejado do HYDE marca as bacias leiteiras perto das
   cidades. **Silvicultura** = 1,4 × floresta / área. **Pesca**: 0 sem costa; com costa, ajuste ×
   (0,4 + comprimento da costa / (2 × √área)), limitado a 1; costa do Japão no mínimo 0,85 (maior
   captura do mundo na década de 1920).
7. **Latitude** do centro do estado (geada e estação): fora da faixa, 0. A videira acima de
   50,5° fica com 30 % (Inglaterra, Bélgica, Holanda).
8. **Pisos** (`FLOOR`): estado que produzia a cultura em escala comercial por volta de 1925 recebe
   no mínimo o valor listado. **Tetos** (`CAP`): chá fora da Ásia, África e Cáucaso (Américas,
   Europa, Oceania) até 0,3; cacau fora do Atlântico (a leste de 40° E ou a oeste de 100° W) até
   0,03; leite na China agrícola, Coreia, Japão e Sudeste Asiático até 0,1 (sem tradição leiteira;
   o oeste pastoril da China fica livre); pesca em costa congelada a maior parte do ano (baía de
   Hudson, Ártico siberiano) até 0,3.
9. Estado sem lavoura nem pasto no HYDE (rocha, recife, Gibraltar) só tem silvicultura e pesca. A
   lista em `land.ron` traz só aptidões ≥ 0,05, com 2 casas.

Pesos (vazio = 0; "Cs" = `mediterranean`, "Dfb…" = `humid_continental` sem o verão quente,
"Dfa…" = `cont_hot`, "Dfc…" = `subarctic`, "plan. fr." = `highland_cool`, "plan. q." =
`highland_warm`; Escala, Irrig. e Latitude são os passos 4, 3 e 7):

| Fazenda | Af | Am | Aw | BWh | BWk | BSh | BSk | Cs | Cfa/Cwa | Cfb/Cwb | Dfb… | Dfa… | Dfc… | ET | EF | plan. fr. | plan. q. | Escala | Irrig. | Latitude |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `grain_farm` |  |  | 0.1 | 0.05 | 0.15 | 0.35 | 0.9 | 0.8 | 0.6 | 0.9 | 1 | 0.9 | 0.3 |  |  | 0.5 | 0.3 | 1 | 0.6 |  |
| `rice_paddy` | 0.7 | 0.9 | 0.6 |  |  |  |  |  | 0.6 |  |  |  |  |  |  |  | 0.5 | 1 | 0.5 | ≤ 50° |
| `maize_farm` | 0.3 | 0.5 | 0.7 |  |  | 0.4 | 0.3 | 0.45 | 1 | 0.3 | 0.45 | 1 |  |  |  | 0.8 | 0.9 | 1 | 0.3 |  |
| `livestock_ranch` | 0.1 | 0.3 | 0.7 | 0.25 | 0.5 | 0.9 | 1 | 0.5 | 0.5 | 0.5 | 0.45 | 0.5 | 0.15 | 0.15 |  | 0.6 | 0.6 | 1 |  |  |
| `dairy_farm` |  |  |  |  |  |  | 0.25 | 0.25 | 0.3 | 1 | 0.8 | 0.6 | 0.25 |  |  | 0.4 |  | 1 |  |  |
| `cotton_plantation` | 0.1 | 0.35 | 0.7 |  |  | 0.55 |  | 0.25 | 0.75 |  |  | 0.25 |  |  |  |  | 0.5 | 0.6 | 1.0 | ≤ 42° |
| `sugarcane_plantation` | 0.8 | 1 | 0.9 |  |  | 0.3 |  |  | 0.55 |  |  |  |  |  |  | 0.2 | 0.7 | 0.6 | 0.6 | ≤ 33° |
| `sugar_beet_farm` |  |  |  |  |  |  | 0.4 | 0.2 | 0.2 | 0.9 | 1 | 0.5 | 0.15 |  |  |  |  | 0.6 | 0.5 | ≥ 30° |
| `coffee_plantation` | 0.2 | 0.25 | 0.25 |  |  |  |  |  |  |  |  |  |  |  |  | 1 | 0.85 | 0.4 |  | ≤ 25° |
| `tea_plantation` | 0.45 | 0.6 | 0.15 |  |  |  |  |  | 0.7 | 0.2 |  |  |  |  |  | 0.9 | 0.8 | 0.4 |  | ≤ 43° |
| `cocoa_plantation` | 1 | 0.8 | 0.3 |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.2 | 0.4 |  | ≤ 20° |
| `rubber_plantation` | 1 | 0.75 | 0.2 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 0.4 |  | ≤ 20° |
| `tobacco_plantation` | 0.4 | 0.6 | 0.6 |  |  | 0.35 | 0.3 | 0.7 | 1 | 0.25 | 0.25 | 0.5 |  |  |  | 0.4 | 0.7 | 0.7 | 0.2 |  |
| `fruit_plantation` | 0.8 | 0.9 | 0.6 |  |  |  |  | 1 | 0.55 |  |  |  |  |  |  |  | 0.6 | 0.35 | 0.5 | ≤ 42° |
| `fiber_plantation` | 0.35 | 0.6 | 0.45 |  |  | 0.35 |  |  | 0.45 | 0.25 | 0.4 | 0.3 |  |  |  | 0.3 | 0.4 | 0.6 |  |  |
| `vineyard` |  |  |  |  |  | 0.25 | 0.25 | 1 | 0.2 | 0.4 | 0.1 | 0.15 |  |  |  |  |  | 0.6 | 0.5 | ≥ 28° e ≤ 52° |
| `fishery` | 0.4 | 0.4 | 0.4 | 0.45 | 0.5 | 0.45 | 0.5 | 0.5 | 0.6 | 0.9 | 0.7 | 0.6 | 0.9 | 0.7 | 0.3 | 0.6 | 0.5 | 1 |  |  |

Pisos por fazenda (lista completa em `FLOOR`, `tools/land/farms.py`; os estados são os produtores
de 1925 das histórias agrícolas usuais, sem tonelagem conferida estado a estado):

| Fazenda | Pisos | Regiões |
|---|---|---|
| `rice_paddy` | 76 | Bengala, Bihar, Orissa, Madras, Assam; Baixa Birmânia; Sião; Cochinchina, Tonquim, Camboja; China do Yangtzé e do sul; Coreia; Taiwan; Java; Luzon; Madagascar; Pó, Valência, delta do Nilo; Luisiana, Arkansas, Califórnia; Rio Grande do Sul; Gilan |
| `maize_farm` | 30 | Pampa (Litoral, Buenos Aires, Córdoba); Romênia, Bessarábia, Iugoslávia, Hungria, Bulgária; vale do Pó; sul da Ucrânia e Kuban; planalto mexicano; Transvaal e Orange |
| `livestock_ranch` | 11 | Buenos Aires, Patagônia, Uruguai, Rio Grande do Sul, Paraguai, NSW, Queensland, Nova Zelândia, Malvinas, Mongólia |
| `dairy_farm` | 21 | Dinamarca, Frísia e Holanda, Nova Zelândia, Wisconsin, Minnesota, Nova York, Vermont, Suíça, Irlanda, Ontário, Québec, Victoria, Götaland |
| `cotton_plantation` | 60 | Cinturão do algodão dos EUA; Egito; Gezira e Kassala; Decão, Berar, Hyderabad, Gujarat, Punjab, Sind; Turquestão soviético; China do norte e do Yangtzé; Nordeste do Brasil; costa do Peru; La Laguna e Mexicali; Uganda; Çukurova; Chaco |
| `sugarcane_plantation` | 49 | Cuba; Java; Províncias Unidas, Bihar, Punjab; Havaí, Luisiana; Porto Rico, Dominicana; Negros e Pampanga; Taiwan; Maurício, Reunião; Pernambuco, Alagoas, Campos; costa norte do Peru; Natal; Queensland; Tucumán; Antilhas e Guianas; Fiji; Alto Egito |
| `sugar_beet_farm` | 39 | Saxônia, Baixa Saxônia, Silésia; Boêmia e Morávia; Nord, Picardia, Champagne; Bélgica, Holanda; Poznań, Pomorze; Ucrânia da margem direita; Kursk e Voronezh; Emília e Vêneto; Baixa Áustria; Escânia; Colorado, Michigan, Utah, Idaho, Nebraska |
| `coffee_plantation` | 50 | São Paulo, Minas, Espírito Santo, Rio, norte do Paraná; Colômbia; Venezuela; América Central; Veracruz, Chiapas, Oaxaca; Haiti, Porto Rico; Java, Sumatra do Sul; Mysore, Malabar; Quênia, Uganda, Tanganica; Sidamo e Harar; Iêmen; Angola |
| `tea_plantation` | 43 | Assam, Bengala (Darjeeling e Dooars), Travancore, Nilgiris; Ceilão; Java, Sumatra; Fujian, Zhejiang, Anhui, Jiangxi, Hunan, Hubei; Taiwan; Shizuoka; Ajária; Niassalândia; Quênia |
| `cocoa_plantation` | 18 | Costa do Ouro, Nigéria ocidental, Costa do Marfim, Camarões, Fernando Pó, São Tomé, Togo; Bahia; Equador; Venezuela; Trinidad; Granada; Dominicana |
| `rubber_plantation` | 30 | Malaia; Sumatra; Java; Bornéu; Ceilão; Cochinchina e Camboja; sul do Sião; Libéria (Firestone, 1926); Travancore; Amazônia (borracha silvestre) |
| `tobacco_plantation` | 28 | Kentucky, Carolina do Norte, Virgínia, Tennessee, Connecticut; Pinar del Río; Deli (Sumatra); Java oriental; Luzon norte; Macedônia e Trácia; Esmirna; sul da Bulgária; Bahia e Rio Grande do Sul |
| `fruit_plantation` | 20 | Bananas: Honduras, Costa Rica, Guatemala, Panamá, Jamaica, Santa Marta, Canárias, Antilhas francesas; cítricos: Califórnia, Flórida, Valência, Sicília, Calábria, Palestina |
| `fiber_plantation` | 30 | Juta (Bengala Oriental e Ocidental, Bihar, Assam); sisal e henequém (Tanganica, Quênia, Iucatã, Moçambique); abacá (Visayas, Luzon sul, Mindanao); linho e cânhamo (Rússia ocidental, Bálticos, Flandres, Campânia, Emília, Voivodina); fórmio na Nova Zelândia |
| `vineyard` | 56 | França (Languedoc, Gironda, Borgonha, Champagne, Provença, Charentes…); Itália; Espanha (La Mancha, Catalunha, Jerez…); Portugal (Douro); Argélia (Oran, Argel); Romênia, Bessarábia, Hungria (Tokaj), Grécia (Peloponeso), Dalmácia; Reno; Crimeia, Geórgia; Califórnia; Mendoza; Chile central; Cabo; Austrália do Sul |
| `fishery` | 68 | Noruega, Islândia, Feroé; Terra Nova, Nova Escócia, Saint-Pierre; Colúmbia Britânica, Alasca, Pacífico e Nova Inglaterra dos EUA; Hokkaido, Karafuto, Kamchatka, Primorye; Murmansk; Astracã (Cáspio); Mar do Norte britânico, holandês, alemão e dinamarquês; Galiza, País Basco, Portugal, Bretanha, Boulogne; ressurgências do Peru, Chile, Namíbia, Cabo e Marrocos; costa chinesa; Coreia |

## Conferência

Totais do mundo (870 estados) contra referências para 1925:

| Item | Este arquivo | Referência |
|---|---|---|
| Lavoura | 10,73 Mkm² | HYDE 3.3: 10,34 (1920) e 11,12 (1930). Ramankutty e Foley (1999) e Houghton dão a mesma ordem, ~10–12 Mkm² na década de 1920 (a conferir) |
| Pasto e pastagem natural | 19,34 Mkm² | HYDE 3.3: 18,48 (1920) e 20,20 (1930); dos quais 3,52 de pasto manejado e 13,8 de pastagem natural |
| Floresta | 42,9 Mkm² | Floresta potencial do WWF no HYDE: 58,8. FAO FRA 2020: 40,6 Mkm² em 2020, depois de décadas de desmatamento tropical, então 1925 deve ficar algo acima. Zon e Sparhawk (1923) contaram ~30 Mkm² de floresta "comercial", definição mais estreita e trópicos mal levantados |
| Área irrigada | 0,88 Mkm² | HYDE 3.3 (só serve às regras) |

Por fazenda, os 10 estados de maior aptidão × terra que a fazenda usa (lavoura, salvo indicação),
como gerados em `tools/land/check.md`, e se eram produtores em 1925:

| Fazenda | Top 10 | Confere? |
|---|---|---|
| `grain_farm` | KAZ-NORTH, USA-KS, USA-ND, USA-IA, USA-TX, IND-UNITED-PROVINCES, RUS-ALT, USA-IL, USA-NE, CHN-HA | 10/10 (trigo das Planícies, aveia do Iowa e Illinois, trigo de Akmolinsk e do Altai, das Províncias Unidas e de Henan). Pradarias canadenses, Ucrânia e Pampa vêm logo abaixo |
| `rice_paddy` | IND-UNITED-PROVINCES, IND-ANDHRA, CHN-JS, CHN-SC, IND-ORISSA, IND-BENGAL, CHN-HA, IND-MADRAS, CHN-HB, CHN-AH | 9/10 (Henan planta pouco arroz). Birmânia, Sião, Indochina, Java e Japão têm aptidão ≥ 0,9, mas estados de lavoura menor |
| `maize_farm` | IND-UNITED-PROVINCES, IND-CENTRAL-INDIA, CHN-HA, USA-IA, USA-KS, USA-IL, IND-BOMBAY, USA-TX, USA-MO, CHN-SD | 8/10 (o Decão de Bombaim e a Índia Central plantavam sorgo e milheto, não milho). Pampa, Romênia e Iugoslávia têm aptidão ≥ 0,85 |
| `livestock_ranch` (× pastoreio) | MNG, AUS-QLD, KAZ-WEST, ARG-PATAGONIA, KAZ-EAST, AUS-NT, CHN-XZ, KAZ-SOUTH, AUS-WA, AUS-NSW | 10/10 |
| `dairy_farm` (× pasto manejado) | DEU-BY, IRL-MUNSTER, DEU-NI, CHN-NM, IRL-LEINSTER, FRA-PAYS-DE-LA-LOIRE, GEO, CZE-CECHY, GBR-WALES, FRA-NORMANDIE | 8/10 (Mongólia Interior e Geórgia são pasto do HYDE com aptidão baixa) |
| `cotton_plantation` | IND-BOMBAY, USA-TX, IND-CENTRAL-INDIA, CHN-HA, IND-GUJARAT, PAK-PUNJAB, IND-UNITED-PROVINCES, USA-OK, CHN-SD, IND-HYDERABAD | 10/10. Egito e Fergana têm 0,95, com lavoura pequena |
| `sugarcane_plantation` | IND-UNITED-PROVINCES, IND-CENTRAL-INDIA, IND-BOMBAY, PAK-PUNJAB, IND-BIHAR, IND-MYSORE, IND-PUNJAB, IND-ANDHRA, IND-ORISSA, IND-MADRAS | 9/10 (a Índia tinha a maior área de cana do mundo, para gur; Índia Central pouco). Cuba e Java (0,95) somem do topo porque o HYDE dá a Cuba ~1 500 km² de lavoura por província |
| `sugar_beet_farm` | USA-NE, USA-ND, RUS-ALT, KAZ-NORTH, RUS-TA, USA-MN, USA-MI, RUS-VOR, RUS-BA, USA-KS | 4/10 (Nebraska, Minnesota, Michigan, Voronezh). O resto é potencial do cinturão continental que a URSS ocupou depois (Tatarstão, Bascortostão, Altai). Ucrânia, Boêmia, Saxônia e Picardia têm 0,85–0,95 e lavoura menor por estado |
| `coffee_plantation` | IND-MYSORE, IDN-ZUID-SUMATRA, IND-MADRAS, IND-CENTRAL-INDIA, TZA, IDN-OOST-JAVA, KEN, MEX-VER, CHN-GD, CHN-YN | 7/10 (Índia Central, Guangdong e Yunnan são só potencial). São Paulo e Minas têm 0,95 e 0,9, mas o HYDE dá a São Paulo só 4 500 km² de lavoura |
| `tea_plantation` | CHN-AH, CHN-SC, IND-MADRAS, IND-ASSAM, CHN-HB, IND-BENGAL, IND-UNITED-PROVINCES, IND-MYSORE, CHN-HN, IND-CENTRAL-INDIA | 8/10 (Províncias Unidas e Índia Central com 0,26 e 0,17, potencial). Ceilão tem 0,95 com lavoura pequena |
| `cocoa_plantation` | CMR, NGA-WESTERN, CIV, VEN, ECU, GHA-COLONY, TGO, GHA-ASHANTI, COD-ORIENTALE, NGA-BORNU-ADAMAWA | 8/10 (Congo Oriental e Bornu são potencial baixo) |
| `rubber_plantation` | IDN-ZUID-SUMATRA, IDN-MIDDEN-SUMATRA, IDN-OOST-BORNEO, IDN-OOSTKUST, MMR-LOWER, VNM, IDN-WEST-BORNEO, IND-BOMBAY, IDN-OOST-JAVA, IDN-WEST-JAVA | 9/10 (Bombaim, 0,07). A Malaia tem 0,9–0,95 e lavoura menor |
| `tobacco_plantation` | IND-UNITED-PROVINCES, IND-CENTRAL-INDIA, USA-KY, USA-TX, CHN-HA, IND-BOMBAY, IND-ANDHRA, USA-OK, USA-TN, CHN-JS | 7/10 (Texas, Oklahoma e Índia Central são potencial; Henan, Guntur e Nipani plantavam) |
| `fruit_plantation` | USA-CA, IND-CENTRAL-INDIA, IND-UNITED-PROVINCES, IND-BOMBAY, ITA-SICILIA, IDN-MIDDEN-SUMATRA, IDN-ZUID-SUMATRA, USA-TX, USA-OK, IND-MYSORE | 3/10 para fruta de exportação (Califórnia, Sicília, Texas do Rio Grande). A Índia era a maior produtora de banana, mas para consumo interno; as repúblicas bananeiras têm 0,7–0,9 com lavoura pequena |
| `fiber_plantation` | IND-BENGAL, IND-BOMBAY, IND-CENTRAL-INDIA, IND-UNITED-PROVINCES, IND-BIHAR, IND-ASSAM, IND-ORISSA, BLR, TZA, CHN-HA | 7/10 (Bombaim, Índia Central e Henan com 0,23–0,26, potencial de cânhamo-sunn) |
| `vineyard` | USA-CA, ESP-CASTILLA-LA-NUEVA, ESP-CASTILLA-LA-VIEJA, USA-TX, ITA-SICILIA, PAK-PUNJAB, ESP-ANDALUCIA-OCCIDENTAL, ROU-MUNTENIA, ITA-PUGLIE, IND-PUNJAB | 7/10 (Texas e os dois Punjabs são potencial baixo). França tem 0,85–0,95 com lavoura menor por região |
| `forestry` (× floresta) | RUS-SA, BRA-AM, BRA-PA, CAN-QC, RUS-KRASNOYARSK, CAN-ON, CAN-BC, RUS-TYUMEN, RUS-KHABAROVSK, RUS-IRK | 10/10 como reserva; em corte de 1925, Escandinávia, Finlândia e o sul e o noroeste dos EUA pesavam mais |
| `fishery` (só aptidão) | NOR-NORD-NORGE, CAN-NL, JPN-HOKKAIDO, ISL, NOR-VESTLANDET, FRO, USA-AK, RUS-KAM, CAN-NS, FRA-BRETAGNE | 10/10 |

A conferência por aptidão × lavoura herda a distribuição do HYDE: onde o HYDE dá pouca lavoura em
1925 (Brasil, Cuba, América Central) ou muita (Sumatra e Bornéu, com roça itinerante), o ranking
desloca, mesmo com a aptidão certa.

## Dúvidas

1. **Licença do HYDE 3.3.** Três textos dizem coisas diferentes: o DataCite e a página pública do
   Yoda (`https://public.yoda.uu.nl/geo/UU01/AEZZIT.html`) dão CC BY 4.0; o
   `readme_HYDE_3.3.txt` dentro do pacote dá CC BY 3.0; e o `yoda-metadata.json` do próprio pacote
   no cofre dá **CC BY-NC-SA 4.0**. No HYDE 3.5 (doi:10.24416/UU01-F45D44) é igual: página e
   DataCite CC BY 4.0, `yoda-metadata.json` CC BY-NC 4.0 (o que a OWID repete). Este arquivo segue
   o registro publicado (DataCite e página), como o `SOURCES.md` já faz para a população; se o
   mantenedor quiser certeza, a pergunta vai a Kees Klein Goldewijk (contato na página). Se valer o
   NC, lavoura, pasto e floresta precisam de outra fonte e só o clima fica.
2. **Floresta** é estimativa própria, não dado: potencial do WWF menos o uso do HYDE, com um fator
   de paisagem calibrado a olho contra ordens de grandeza nacionais. Sub-estima a Europa Central
   manejada e a Polônia, super-estima Japão, Índia e as fronteiras agrícolas que o HYDE subestima.
3. **Pisos** vêm de conhecimento geral de história agrícola, não de uma tabela de produção por
   estado; o próximo passo seria conferi-los contra o *International Yearbook of Agricultural
   Statistics* (Instituto Internacional de Agricultura, Roma, 1925–26).
4. **Latitude pelo centro do estado** decide planalto tropical e limites de geada; estados grandes
   que cruzam a faixa (Minas Gerais, Queensland, Índia Central) recebem um valor só.
5. O mapa de Köppen é de 1980–2016, não de 1925. O aquecimento desde então desloca bordas de
   classe (Dfb → Dfa, ET → Dfc) em poucas dezenas de quilômetros; para estados inteiros a diferença
   é pequena.
