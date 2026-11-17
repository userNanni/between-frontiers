# Cenário de 1925: população, força de trabalho e indústria

`population.ron` (contrato `bf_sim::initial::PopulationFile`) dá a população de cada um dos 870
estados em 1925-01-01 com a parcela urbana, e a força de trabalho de cada país (metrópole e colônias
à parte). `industry.ron` (`bf_sim::initial::IndustryFile`) dá a produção de 1925 de 15 bens e três
estoques de infraestrutura por país, colônias somadas ao dono. Licença dos dados: `industry.ron`
CC BY-SA 4.0; `population.ron` **CC BY-NC-SA 4.0** (uso não comercial), porque deriva do HYDE (ver
`LAND.md`, "Licença do HYDE"). Atribuições abaixo. Invariantes em `crates/bf-sim/tests/initial_1925.rs`.

## Fontes e licenças

| Fonte | O que dá | Licença |
|---|---|---|
| Klein Goldewijk, K. (2023). *History Database of the Global Environment 3.3* (HYDE 3.3), Universidade de Utrecht, doi:10.24416/UU01-AEZZIT. Cenário de base (`hyde33_c7_base_mrt2023`), `1920AD_pop.zip` e `1930AD_pop.zip` | População total (`popc`) e urbana (`urbc`) por célula de 5' em 1920 e 1930 | CC BY 4.0 no DataCite e na página pública do Yoda; o projeto segue o texto mais restritivo do pacote, **CC BY-NC-SA 4.0** (`LAND.md`, "Licença do HYDE") |
| `countries.ron` deste cenário (Maddison 2023, OWID, censos; ver `SOURCES.md`) | Totais de população e urbanização de cada bloco | CC BY-SA 4.0 |
| Censos nacionais dos anos 1920 (lista em "Conferência com censos") | Só os totais publicados, como fatos, para conferir a distribuição | Estatística oficial |
| Séries históricas de produção (anuários estatísticos nacionais, *Statistical Abstract of the United States*, *Mineral Resources*, Lloyd's Register, AT&T *Telephone and Telegraph Statistics of the World*, Liga das Nações), na forma compilada por B. R. Mitchell, *International Historical Statistics* | Produção industrial e estoques de 1925 | Fatos citados; nenhuma tabela copiada |

Nenhuma fonte com cláusula não comercial ou GPL entrou nos dados.

Para reproduzir, da raiz do repositório:

```bash
mkdir -p data/raw/hyde33/asc
# HYDE 3.3: https://geo.public.data.uu.nl/vault-hyde/HYDE%203.3%5B1701183392%5D/original/
#   hyde33_c7_base_mrt2023/zip/{1920AD_pop,1930AD_pop}.zip
#   (o servidor pede uma prova de trabalho, o Anubis; baixe pelo navegador)
unzip -o -j data/raw/hyde33/1920AD_pop.zip popc_1920AD.asc urbc_1920AD.asc -d data/raw/hyde33/asc
unzip -o -j data/raw/hyde33/1930AD_pop.zip popc_1930AD.asc urbc_1930AD.asc -d data/raw/hyde33/asc
python3 -I tools/initial/aggregate.py    # grades → tools/initial/state_hyde.csv (~10 s)
python3 -I tools/initial/population.py   # → population.ron e tools/initial/population_check.md
python3 -I tools/initial/industry.py     # → industry.ron e tools/initial/industry_check.md
```

`state_hyde.csv` é versionado: `population.py` e `industry.py` rodam sem baixar nada. As tabelas
de entrada são `tools/initial/workforce.csv`, `tools/initial/industry.csv` e
`tools/initial/census/*.csv`, cada linha com a sua fonte.

## População dos estados

1. **HYDE por estado.** 1925 é a média das grades de 1920 e 1930 (interpolação linear). As pessoas
   de cada célula de 5' se repartem entre os pixels de terra do mapa dentro dela, pela área
   verdadeira; célula com gente e sem pixel de terra no nosso mapa (litoral diferente) vai inteira
   para o estado do pixel de terra mais próximo (5,3 M de 2 017 M). Mesma geometria de
   `tools/land/aggregate.py`.
2. **Repartição.** Cada bloco (metrópole de um país; colônias de um dono; cada súdito com tag
   conta como país) tem o total de `countries.ron` repartido entre os seus estados na proporção do
   HYDE, calibrada pelos censos onde há (seção seguinte), com arredondamento pelo maior resto. As somas batem exatamente com `population` e
   `colonies.population`.
3. **Urbano.** Parcela do estado = urbano / total do HYDE, multiplicada por um fator único por
   bloco (com teto 1 e o piso das cidades-estado), achado por bisseção para que a média ponderada
   pela população seja a `urbanization` do bloco. Bloco sem urbano no HYDE (Mônaco, San Marino)
   recebe a parcela do país em todos os estados; estado sem gente no HYDE (Guernsey), a do bloco.

O HYDE usa fronteiras atuais nos seus totais nacionais, por isso a soma do HYDE nos estados de um
bloco difere do total de `countries.ron`; só a proporção entre estados é usada. A razão por bloco
está em `tools/initial/population_check.md`. Razões longe de 1: Alemanha 0,91 (o HYDE
põe mais gente nas províncias perdidas em 1919–22 que ficaram nos estados alemães do mapa), Irlanda
0,82, URSS 0,84, Pérsia 0,73, Lituânia 1,34 (Memel e a questão de Vilna), Butão 1,91, Hejaz 0,70,
Transjordânia 0,69, Tuva 0,56, Mônaco 4,5 e colônias da Austrália 0,39 (o HYDE dá a Papua e Nova
Guiné 1,4 M; a estimativa colonial é 0,56 M). Fatores urbanos longe de 1: Sarre 2,0, Hejaz 1,7,
Kuwait 1,4, Terra Nova 0,47, Tuva 0,49, Tibete 0,61: aí o alvo de `countries.ron` e o HYDE
discordam, e prevalece `countries.ron`.

## Conferência com censos

Censos de 1914–1926 por subdivisão, de 14 países, em `tools/initial/census/*.csv`. Cada linha é
um grupo de estados do jogo que cobre aproximadamente o território de uma ou mais unidades do
censo, com o total publicado, a fonte (URL lida) e as aproximações em `# note:`. Os totais vieram
de fonte baixada (IBGE SIDRA, Census of India 1921 e *Statesman's Year-Book* 1923 no
archive.org, demoscope.ru, SPLAF, Wikipedia de cada língua com os números oficiais), nunca de
memória.

**Uso.** Primeiro o HYDE sozinho reparte o país; esse resultado é comparado com o censo, grupo a
grupo (tabela completa em `tools/initial/population_check.md`). Depois o censo calibra: os estados
fora de todo grupo ficam com a parcela do HYDE, os grupos dividem o resto pela parcela do censo, e
o HYDE só reparte cada grupo entre os seus estados. Assim os grupos do arquivo final seguem o censo
(a menos da diferença de data, de 1914 na Argentina a 1926 na URSS; os EUA usam 1920 e 1930
interpolados para 1925). Só a metrópole é calibrada.

| País | Censo | Grupos | Desvio do HYDE > 20 % (parcela HYDE / parcela censo − 1) |
|---|---|---|---|
| EUA | 1920 e 1930, interpolados | 51 | nenhum |
| França | 1926 | 23 | nenhum (Córsega fora, sem número de 1926) |
| Brasil | 1920 | 21 | Goiás +45 % |
| Japão | 1925 | 19 | Hokkaidō −36 %; Hokuriku +93 % |
| Índia (RAJ) | 1921 | 20 | Tripura +49 %; Délhi +29 %; NWFP −44 % |
| Itália | 1921 | 16 | Venezia Giulia −37 %; Lazio + Úmbria + Campânia +36 %; Abruzzi e Molise −23 %; Sardenha +27 % |
| URSS | 1926 | 14 | Ucrânia −28 %; Nakhichevan −40 %; Cazaquistão −23 %; Quirguízia +32 % (Bielorrússia fora: fronteira diferente em 1926) |
| Alemanha | 1925 | 11 | Berlim −44 %; Norte +21 %; Silésia −29 %; Prússia Oriental −40 % |
| Reino Unido | 1921 (Irlanda do Norte 1926) | 7 | Jersey +81 %; Guernsey −100 % (o HYDE não tem gente na ilha) |
| México | 1921 | 31 | Baja California +75 %; Colima −24 %; Estado de México +25 % |
| Canadá | 1921 | 11 | Territórios do Noroeste +118 % |
| Austrália | 1921 | 8 | Território do Norte +39 %; Território da Capital +221 % |
| Argentina | 1914 | 7 | Chaco + Formosa +187 % (fronteira agrícola que cresceu depois de 1914) |
| Polônia | 1921 | 1 | só o total (voivodias de 1921 não casam com os estados do jogo; Alta Silésia e Wilno fora) |

Os desvios grandes vêm de três causas: a grade de 5' do HYDE espalha as grandes cidades pelas
células vizinhas (Berlim, Délhi, e a capital australiana recebe gente de Queanbeyan); o HYDE
reparte a população nacional por padrões modernos (Hokkaidō e Hokuriku trocados, Ucrânia baixa e
Rússia alta, Lazio e Campânia altos); e territórios quase vazios recebem gente demais. Os grupos
alemães e italianos marcados têm fronteira aproximada (ver notas dos CSVs), então parte do desvio é
do agrupamento. Espanha ficou sem conferência (sem tabela provincial de 1920 legível).

**Urbano das cidades-estado.** Pela mesma razão da grade, o HYDE dá a Berlim 0,46 de urbano e a
Washington 0,54. Sete estados que são uma cidade têm um piso (`CITY_FLOORS` em `population.py`):
Berlim 0,95, Hamburgo 0,90, Bremen 0,85, Viena 0,95, Distrito de Colúmbia 1,0, Distrito Federal
mexicano 0,85, Délhi 0,6. O fator do bloco compensa nos demais estados.

## Força de trabalho

`tools/initial/workforce.csv`, uma linha por bloco (83 metrópoles e 13 conjuntos coloniais), com a
fonte e a marca `census` ou `est`.

- **Participação** = população economicamente ativa (ocupados e desocupados) / população total.
  Os censos da época contam mal o trabalho feminino no campo (Brasil 0,32, Espanha 0,36, México
  0,33) e bem onde o recenseador perguntava pela família camponesa inteira (URSS 0,55, Bulgária
  0,55, França 0,53): a diferença é de método censitário, e foi mantida.
- **Setores:** agricultura inclui silvicultura, caça e pesca; indústria inclui mineração,
  manufatura, construção e utilidades; serviços é o resto (comércio, transporte, administração,
  serviço doméstico, profissões, forças armadas). Ocupação "não declarada" foi repartida na
  proporção.
- **Censos usados** (`census`): Reino Unido 1921 (e Irlanda do Norte 1926), Estado Livre Irlandês
  1926, França 1926, Bélgica 1920, Países Baixos 1920, Índias Holandesas 1930, Suíça 1920, Espanha
  1920, Portugal 1920, Itália 1921, Alemanha 1925, Áustria 1923, Hungria 1920, Tchecoslováquia
  1921, Polônia 1921/1931, Lituânia 1923, Letônia 1925, Estônia 1922, Finlândia 1920, Suécia 1920,
  Noruega 1920, Dinamarca 1921, Islândia 1920, URSS 1926, Romênia 1930, Iugoslávia 1921, Bulgária
  1926, Grécia 1928, Turquia 1927, EUA 1920/1930, Canadá 1921, México 1921, Costa Rica 1927, Cuba
  1919, Chile 1920, Brasil 1920, Japão 1920/1930, Coreia e Taiwan 1930, Sião 1929/1937, Índia 1921,
  Egito 1927, África do Sul 1921, Austrália 1921, Nova Zelândia 1926.
- **Estimativas** (`est`): todas as colônias salvo as Índias Holandesas e as do Japão (análogos de
  censos coloniais de 1921–1931), a China (Liu e Yeh, força de trabalho de 1933, e levantamentos
  de Buck), a América Latina sem censo próximo (repartição de censos de 1938–1950 projetada para
  trás), o Oriente Médio, a Etiópia, a Libéria, o Himalaia, a Mongólia e os microestados.

## Indústria

`tools/initial/industry.csv`, uma linha por país (as 83 tags), colônias somadas ao dono, com a
nota de fonte e a coluna `est` das grandezas estimadas (`all` = a linha toda). Unidades em
`units`. Chaves: os bens `steel`, `cement`, `textiles`, `paper`, `electricity`, `fuel`,
`chemicals`, `fertilizer`, `explosives`, `aluminium`, `copper`, `base_metals`, `automobiles`,
`ships` de `goods.ron`, mais `pig_iron` (insumo intermediário, para semear altos-fornos à parte das
aciarias) e os estoques `railways_km`, `merchant_fleet_grt`, `telephones`. Atenção: em
`goods.ron`, `automobiles` e `ships` são lotes de US$ 1 000; aqui são milhares de veículos e
milhares de TAB lançadas, como diz `units`; a conversão é da economia.

Definições:

- **Aço:** aço bruto (lingotes e peças fundidas). **Gusa** à parte.
- **Têxteis:** fibra consumida por fiações e tecelagens mecânicas (algodão, lã lavada, juta,
  linho e cânhamo, seda, raiom), proxy de tecido. A tecelagem manual (China, Índia) fica de fora;
  a juta de Calcutá (≈ 1 Mt) entra, por isso a Índia aparece em segundo.
- **Combustível:** derivados refinados (vazão das refinarias). As refinarias de Curaçao e das
  Índias Holandesas contam para os Países Baixos; Abadã para a Pérsia; Trinidad e Sarawak para o
  Reino Unido; o petróleo venezuelano refinado em Curaçao não conta para a Venezuela.
- **Químicos:** ácido sulfúrico em H2SO4, proxy grosseiro de toda a química pesada.
- **Fertilizante:** nutrientes N + P2O5 + K2O: nitrogênio sintético e de coqueria, nitrato do
  Chile (como N), superfosfato e escória Thomas (como P2O5), potássio alemão e alsaciano (K2O).
- **Explosivos:** explosivos comerciais (mineração e obras); munição militar fica fora.
- **Automóveis:** carros e caminhões fabricados; montagem de kits (Ford na Argentina, Brasil,
  Japão, Dinamarca…) fica fora.
- **Navios:** TAB lançada ao mar (Lloyd's Register). **Frota mercante:** navios a vapor e a motor
  de 100 TAB ou mais em meados de 1925, com a frota dos Grandes Lagos nos EUA e no Canadá.
- **Ferrovias:** km de linha pública em operação; linhas de usina e de mina ficam fora (Cuba).
- **Telefones:** aparelhos em 1/1/1926 (estatística da AT&T).

### Totais mundiais contra referências

| Grandeza | Este arquivo | Referência de 1925 |
|---|---|---|
| Aço bruto | 90,5 Mt | ≈ 90 Mt |
| Gusa | 77,2 Mt | ≈ 76 Mt |
| Cimento | 62,0 Mt | 60–70 Mt |
| Eletricidade | 195 TWh | 180–200 TWh |
| Alumínio primário | 182 kt | 180–200 kt |
| Automóveis | 4,89 M (EUA 4,27 M) | ≈ 4,7 M (EUA ≈ 4,3 M) |
| Navios lançados | 2,31 M TAB | ≈ 2,2 M TAB (Lloyd's) |
| Frota mercante | 64,2 M TAB | 62–64 M TAB (Lloyd's) |
| Ferrovias | 1,19 M km | ≈ 1,2 M km |
| Telefones | 27,7 M | 26–27 M (AT&T, 1/1/1926) |
| Fertilizantes | 6,2 Mt de nutrientes | 5–6 Mt |
| Cobre refinado | 1,50 Mt | 1,4–1,6 Mt |
| Chumbo + zinco + estanho | 3,07 Mt | 2,9–3,1 Mt |
| Derivados de petróleo | 128 Mt | abaixo do bruto, ≈ 146 Mt |

Tabela completa com os dez maiores de cada grandeza: `tools/initial/industry_check.md`. O teste
cobra faixas largas em torno das cinco primeiras referências.

### Maiores produtores

| # | Aço (kt) | Eletricidade (GWh) |
|---|---|---|
| 1 | EUA 46 120 | EUA 85 000 |
| 2 | Alemanha 12 195 | Alemanha 20 300 |
| 3 | Reino Unido 7 504 | França 11 400 |
| 4 | França 7 446 | Reino Unido 10 100 |
| 5 | Bélgica 2 549 | Canadá 10 100 |
| 6 | Luxemburgo 2 087 | Japão 9 300 |
| 7 | URSS 1 868 | Itália 7 300 |
| 8 | Itália 1 786 | Noruega 5 400 |
| 9 | Sarre 1 570 | Suíça 4 100 |
| 10 | Tchecoslováquia 1 467 | Suécia 3 300 |

### O que é estimativa

Todas as linhas fora das ~30 maiores economias são estimativas regionais (`est: all`), em geral
zero onde não havia a indústria. Nas grandes economias, são estimativas: químicos e explosivos
(sempre), fertilizantes fora da Alemanha e do Chile, têxteis de quase todos (consumo de fibra é
proxy), combustível da Europa (refino pequeno, mal registrado), cimento e papel dos países médios,
metais básicos e cobre refinado (o refino de concentrado importado é difícil de atribuir). Os
números de aço, gusa, alumínio, automóveis, navios, frota, ferrovias e telefones das grandes
economias são os das séries publicadas, arredondados.

## Dúvidas

1. **Origem dos números de indústria.** As séries (Mitchell, anuários nacionais, Lloyd's, AT&T)
   foram transcritas da memória do modelo que preparou este arquivo, não conferidas página a página
   numa cópia dos livros. Os totais mundiais batem com as referências conhecidas, mas um valor
   isolado pode estar 10–30 % fora, e mais nos marcados como estimativa. Conferir os 30 maiores
   contra uma cópia de Mitchell é o próximo passo.
2. **Localização de fábricas japonesas na China.** Anshan e Penhsihu (Manchúria) contam para a China
   pelo território, embora fossem da Mantetsu; o mesmo para as fiações japonesas de Xangai.
3. **Gusa não é bem do catálogo.** `pig_iron` está fora de `goods.ron`; o teste o aceita como chave
   declarada fora dos bens. Se a economia não quiser altos-fornos à parte, a chave pode sair.
4. **URSS em ano civil.** As estatísticas soviéticas eram por ano econômico (out.–set.); usei o ano
   civil de 1925 quando conhecido, abaixo de 1925/26.
5. **Licença do HYDE.** Decidido em 2026-11-17: `population.ron` e `tools/initial/state_hyde.csv`
   ficam sob CC BY-NC-SA 4.0, como diz `LAND.md` ("Licença do HYDE").
6. **Urbano de `countries.ron`.** Onde o alvo de `countries.ron` e o HYDE discordam muito (Sarre,
   Hejaz, Kuwait, Terra Nova, Tuva, Tibete), o fator único por bloco desloca todos os estados do
   bloco; vale rever o alvo nesses países.
7. **Calibração com grupos aproximados.** Onde o grupo do censo e os estados do jogo não cobrem o
   mesmo território (Norte e Renânia–Vestfália–Hessen na Alemanha, Lazio–Úmbria–Campânia na Itália,
   Ucrânia sem o sul da Bessarábia), a calibração transfere esse erro de fronteira para os estados.
   Os censos de 1914 (Argentina) e 1921 também não veem o crescimento desigual até 1925.
