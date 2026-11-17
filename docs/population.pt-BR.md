# População: comunidades de 1925 e demografia

A demografia (nascimentos, mortes, idade e migração entre países) roda na simulação: seção
"Demografia" no fim. As comunidades ainda não.

## Comunidades de 1925

Estado: contrato e dados, **sem ligação com a simulação**. Os parâmetros abaixo são **proposta
para o dono decidir** (decisões em aberto 1 a 3 de `architecture.pt-BR.md`).

- Contrato: `crates/bf-sim/src/communities.rs` (listas, composição por estado, regra ABC).
- Dados: `data/culture/cultures.ron`, `data/culture/religions.ron`,
  `data/scenario/1925/communities.ron` (gerado).
- Fonte de cada número: `tools/communities/spec/*.txt`, uma linha por estado com a citação e a base
  (censo, misto ou estimativa); `python3 -I tools/communities/generate.py` refaz o `.ron` e a
  conferência `tools/communities/check.md`.
- Nomes: `assets/i18n/*/communities.ftl`.
- Teste: `crates/bf-sim/tests/communities_1925.rs`.

## O que existe

| Item | Quantidade |
|---|---:|
| Culturas | 552 (34 famílias linguísticas) |
| Religiões | 24 (8 famílias) |
| Comunidades (pares cultura + religião que existem em algum estado) | 852 |
| Linhas de relevância (país × comunidade) | 287 |
| Estados com composição | 870 de 870 |

Base dos números, por estado: 91 de censo, 503 mistos, 276 estimativa. Por população (com a
aproximação de população por estado da economia): 8 % censo, 52 % misto, 39 % estimativa. O
peso da estimativa vem da China (sem censo até 1953), da África tropical, da Pérsia, da Arábia e
do Sudeste Asiático continental.

O que é estimativa, em resumo: China (parcelas de minorias do censo de 1953 projetadas para
trás), Indochina, Sião, Pérsia, Afeganistão, Arábia, Golfo, Nepal, Butão, Mongólia, Tuva,
Etiópia, Sudão (censo de 1955/56 para trás), Congo Belga, África Equatorial Francesa, Libéria,
Pacífico menor, as línguas regionais da França e da Espanha, e o Brasil (cor e religião do censo
de 1940 aplicadas a 1925). Na URSS a nacionalidade é do censo de 1926 e a religião é inferida.
Cada linha de `tools/communities/spec/` diz qual é o caso.

## Método

**Comunidade = cultura + religião**, como pede "O desenho" da arquitetura. Cultura é grupo
etnolinguístico (a língua primeiro), na granularidade que o censo da época usava, agregada quando o
estado não comporta mais de dez entradas. Religião é uma lista curta e fixa (24 chaves) em que a
diferença importa para a política do século: católicos e protestantes ficam separados; uniatas
(greco-católicos, maronitas) são `eastern_catholic`; armênios, coptas e etíopes são
`oriental_orthodox`; xiitas, alauítas, drusos e ibaditas são chaves próprias.

**Uma chave por grupo no mundo inteiro.** A diáspora usa a chave da pátria: chineses na Malásia
são `han`, indianos em Fiji são `hindustani` ou `tamil`, italianos em Buenos Aires são `italian`.
Quem já mudou de língua em 1925 conta na cultura de chegada.

**Raça só onde era fronteira legal ou social nítida em 1925**: afro-americanos (Jim Crow),
`cape_coloured` na África do Sul, `afro_brazilian` e `afro_cuban` com a categoria "preto/negro" dos
censos (pardos e mestiços ficam na cultura nacional). É escolha de modelagem, registrada aqui para
ser revista.

**Uma linha por estado, com fonte.** Cada estado tem a composição em parcelas que somam 1. Entra
toda comunidade com 0,5 % ou mais; abaixo disso ela é somada à mais próxima (mesma religião
primeiro), salvo se tiver fator de relevância (os parses entram com 0,3 % em Bombaim e 0,1 % no
Sind). Esse corte é limite de precisão dos dados, não do modelo: durante a partida a contagem de
"outras minorias" guarda qualquer comunidade, por menor que seja.

**Base.** Cada estado declara se os números vêm de censo de 1915–1935 (`Census`), de censo para uma
dimensão com a outra inferida ou de censo de ano próximo ajustado (`Mixed`), ou de estimativa
(`Estimate`, com o método na fonte). Onde o censo não perguntou religião (URSS de 1926), a religião
é a tradicional de cada nacionalidade.

**Relevância** vem dos dados, só pela situação em 1925 (sem retrospecto): `Elite`,
`Organization`, `LegalStatus` (privilegiado ou perseguido), `Patron`. A concentração é calculada:
maioria em algum estado sem ser a maior comunidade do país. A relevância vale para a metrópole
(`!TAG` na tabela de fontes) ou para as colônias (`!TAG:C`): colono inglês é elite no Quênia, não
na Inglaterra.

## Fontes

Todas compatíveis com CC BY-SA 4.0: censos e anuários oficiais da época (fato; domínio público na
maioria das jurisdições), publicações da Liga das Nações, Wikidata (CC0) e artigos acadêmicos como
fato. A Wikipédia serviu de ponteiro para os números dos censos; a citação é o censo. Nada de
Victoria 3, Hearts of Iron, Europa Universalis ou mods (todos os direitos reservados), nem fonte
NC ou GPL. A lista completa, por região, está no bloco `# SOURCES` de cada arquivo em
`tools/communities/spec/`.

**Grau de conferência.** A base (`Census`, `Mixed`) diz de onde o número vem, não que cada
decimal foi lido na tabela original. Foram conferidos na publicação ou numa transcrição dela: o
censo soviético de 1926 (tabela VI inteira), a língua materna do censo dos EUA de 1920, o
Religious Bodies de 1926, o Canada Year Book 1922-23, a cor e a religião do censo brasileiro de
1940, as Filipinas de 1918, a Polônia de 1921 em cinco voivodias, a Turquia de 1927, o Ceilão, o
Líbano, a Síria, a Comissão de Mosul, a Tunísia de 1926 e Maurício. O resto foi montado de
memória a partir dos mesmos censos e precisa de conferência: os decimais alemães e austríacos, as
províncias sul-africanas fora dos europeus, os estados mexicanos (a parte mais fraca), Cuba,
Argentina, as Antilhas, as residências das Índias Holandesas e a Coreia por província. Os
comentários de cada arquivo em `tools/communities/spec/` dizem o que foi conferido.

Principais, por região (a citação completa e o que cada uma deu estão nos arquivos):

| Região | Fontes |
|---|---|
| Europa ocidental | Censos de 1920–1926: Reich alemão 1925, Áustria 1923, Irlanda do Norte 1926, Estado Livre 1926, Inglaterra e Escócia 1921, Bélgica 1920, Países Baixos 1920, Suíça 1920, Itália 1921, Finlândia 1920; estrangeiros do recenseamento francês de 1926 |
| Europa oriental | Polônia 1921 (e 1931 onde 1921 não chegou), Danzig 1923, Tchecoslováquia 1921, Hungria 1920, Romênia 1930 projetada, Iugoslávia 1921, Bulgária 1926, Grécia 1928, Turquia 1927, Estônia 1922, Letônia 1925, Lituânia 1923 |
| URSS e Mongólia | Censo da União de 1926, tabela VI (nacionalidade por guberniya, okrug e ASSR); 1939 para repartir unidades |
| Sul da Ásia e Oriente Médio | Census of India 1921 (1931 para conferir), Ceilão 1921, Chipre 1921, Palestina 1922, Líbano e Síria 1921–22, Egito 1927, Comissão de Mosul da Liga das Nações (1925), Lorimer para o Golfo |
| África | África do Sul 1921, censos coloniais britânicos de 1921–1931, recenseamentos da AOF de 1926, Tunísia 1926, Marrocos 1926, Argélia 1926, Maurício 1921, relatórios de mandato à Liga das Nações |
| Leste da Ásia e Oceania | Coreia, Japão e Taiwan 1925, Volkstelling 1930 das Índias Holandesas, Malásia Britânica 1921, Hong Kong 1921, Austrália 1921, Nova Zelândia 1926, Fiji 1921, Filipinas 1918 |
| América do Norte | Censo dos EUA de 1920 (língua materna por estado), Religious Bodies 1926 (católicos, judeus e SUD por estado), Census Working Paper 56 (raça), Canadá 1921 (origem e religião por província), Terra Nova 1921 |
| América Latina | México 1921 e 1930, Guatemala 1921, Cuba 1919, Argentina 1914, Chile 1920, Brasil 1920 (estrangeiros) e 1940 (cor e religião), Peru 1940 projetado, censos coloniais do Caribe de 1921 |

## Quais comunidades viram linha (proposta: decisão 1)

Implementado em `Communities::rows` com `RowRule::PROPOSED`.

**Importância = parcela × relevância**, relevância = 1 + soma dos pesos dos fatores.

| Fator | Peso proposto | Por quê |
|---|---:|---|
| Elite | 3 | Uma elite de 5 % (brancos na Rodésia, europeus nas Índias Holandesas) tem de pesar como um grupo de 20 %: é quem tem terra, capital e Estado |
| Status legal | 2 | Perseguido ou privilegiado é alvo de lei; a doutrina pede que conte como mandar |
| Organização | 1 | Partido, igreja ou milícia própria; soma com os outros, não basta sozinho para grupo minúsculo |
| Patrono | 1 | Pátria no exterior que reivindica; idem |
| Concentração | 1 | Maioria em algum estado (catalães, curdos): já costuma ter linha pelo tamanho, o peso só a protege no ranking nacional |

Regras:

- **Corte A+B = 95 %** da importância acumulada, ordenada da maior para a menor, dentro do estado.
  Entra a comunidade cuja importância acumulada *antes* dela ainda está abaixo do corte. Num estado
  homogêneo (99 % de uma comunidade) sai uma linha; num estado partido sai o que for preciso.
- **Ranking nacional**: o mesmo corte sobre o país (metrópole e colônias à parte, como os blocos da
  economia). Comunidade A+B do país ganha linha em todo estado onde tem **2 %** ou mais: os judeus
  da Polônia ou os alemães da Tchecoslováquia aparecem onde vivem, mesmo onde o ranking do estado
  sozinho os deixaria de fora. A população do estado, para esse ranking, é a aproximação da
  economia enquanto `population.ron` não existir.
- **Teto: 8 linhas por estado**, mais "outras minorias". O teto corta pela importância.
- **Histerese: 97 %** para continuar com linha, contra 95 % para ganhar: perder a linha pede
  importância menor que ganhá-la. Só vale quando a relevância se mover durante a partida.

Resultado com os dados de 1925 (`cargo test -p bf-sim --test communities_1925 -- --nocapture`):

| Variante | Linhas por estado (média) | Estados no teto | Linhas da tabela | Pessoas em "outras minorias" |
|---|---:|---:|---:|---:|
| **Proposta** (corte 95 %, teto 8, piso nacional 2 %) | **3,23** | 37 | **45 630** | 2,3 % |
| Corte 90 % | 2,73 | 12 | 40 326 | 4,2 % |
| Corte 98 % | 3,84 | 81 | 51 636 | 1,1 % |
| Teto 6 | 3,12 | 124 | 44 395 | 2,7 % |
| Teto 10 | 3,26 | 5 | 45 812 | 2,2 % |
| Sem ranking nacional | 3,04 | 30 | 44 005 | 2,8 % |
| Sem relevância | 3,25 | 36 | 45 578 | 1,9 % |

"Linhas da tabela" é o teto: estado × (comunidades com linha + "outras minorias") × 13 profissões,
como se toda comunidade ocupasse toda profissão. Máximo por estado: 9 × 13 = 117.

- **Orçamento.** A ilustração de "Tamanho" (900 × 6 × 12 ≈ 65 mil) vira o limite do teste: 1925
  usa 70 % dele. A folga é para o século: migração e relevância que nasce só acrescentam linhas,
  e o limiar proporcional à população do estado segura o resto.
- **Por que 95 %.** A 90 %, 4,2 % da humanidade fica sem estado próprio, inclusive grupos inteiros
  de dezenas de milhares numa província; a 98 % as linhas sobem 13 % para tirar mais 1,2 %.
- **Por que teto 8.** Só morde em estados de muita mistura (Daguestão, Iunã, colônias africanas,
  as Pradarias canadenses, Transvaal). De 8 para 10 o custo é desprezível (182 linhas), mas com 6
  o teto corta 124 estados, inclusive grupos com relevância. Fica 8, e pode subir sem medo.
- **Por que os pesos.** A relevância quase não muda o total; muda *quem* tem linha: 165 linhas
  existem só por ela, como os ingleses e os indianos do Quênia, os chineses das Índias Holandesas,
  os japoneses nas províncias rurais da Coreia, os américo-liberianos e os jainistas de Gujarat.
  Sem ela, essas elites somem na média das minorias e com elas a política colonial. Elite pesa 3
  porque uma elite de 5 % precisa pesar como um grupo de 20 % para passar o corte num estado onde
  a maioria tem 80 %.

## Assimilação (proposta: decisão 3, sem código)

Assimilação é fluxo explícito entre comunidades, calculado uma vez por ano, nunca efeito do
agrupamento. Duas regras separadas, porque mudar de língua e mudar de religião têm ritmos
diferentes:

**Cultura.** Por ano, a parcela de uma comunidade (cultura *c*, religião *r*) num estado que passa
para (cultura primária do país, *r*):

    a = a₀ × L × (1 − D) × (1 − s)² × F

| Termo | Proposta | Leitura |
|---|---|---|
| a₀ (taxa base) | 1,5 % ao ano | Em uma geração (30 anos) e sem resistência, 36 % mudam de língua; em 75 anos, 68 %, a ordem do que se viu com imigrantes europeus nos EUA e na Argentina entre 1900 e 1970 |
| L (lei) | exclusão 0; pluralismo 0,5; neutra 1; assimilação forçada 1,5 | Segregação impede (Jim Crow não assimila afro-americanos); escola só na língua oficial acelera (bretões na França) e, na fase 3, gera radicalismo |
| D (discriminação) | 0 a 1, da comunidade no país | Quem é discriminado assimila menos: a porta está fechada dos dois lados |
| s (concentração) | parcela da comunidade no estado | A 80 % do estado, o fator é 0,04: catalães e flamengos não assimilam; a 5 %, 0,9 |
| F (família) | mesma família linguística 1; outra 0,5 | Italiano vira argentino mais rápido que japonês vira brasileiro |

- A cultura primária, até a fase 3 ter leis de cidadania, é a maior cultura do país (metrópole e
  colônias à parte). Colono não assimila no nativo; nativo de colônia só assimila na cultura do
  colonizador por lei explícita (assimilação forçada).
- Cidade acelera e o campo isolado freia; fica para quando a urbanização por linha existir.

**Religião.** Mesma forma, com a₀ = 0,2 % ao ano, L pela lei de religião do Estado (religião
oficial imposta 1,5; tolerância 1; liberdade com separação 0,5) e o alvo na religião majoritária do
país, mantida a cultura. Conversão por missão (África, Coreia) entra como evento ou movimento da
fase 3, não pela taxa base.

**Dentro de "outras minorias"** a mesma regra corre sobre a contagem de cada comunidade, com a
discriminação e a concentração dela. Quem assimila sai da contagem e entra na linha de destino.

**Calibragem esperada** (para o benchmark de 100 anos quando ligar): falantes de bretão de ~1,1
milhão em 1925 para ~0,2 milhão em 2000; imigrantes italianos na Argentina majoritariamente
`argentine` em 1960; catalães e flamengos estáveis; afro-americanos não assimilam até a lei mudar.

## O que fica para quando ligar na simulação

- **A linha demográfica**: riqueza, alfabetização, pirâmide etária e atitudes por estado ×
  comunidade × profissão; a repartição das profissões do estado entre as comunidades (a elite
  ocupa mais `landowner`, `capitalist`, `manager`, `civil_servant`).
- **Discriminação por país** (D da assimilação e fator da renda), que depende das leis da fase 3.
- **A relevância que muda**: lei nova que mira um grupo, movimento que nasce, promoção a linha
  herdando a média de "outras minorias" e rebaixamento com a histerese.
- **Migração por comunidade**: hoje a migração interna move gente sem identidade.
- **`population.ron`** com a população real de cada estado: o ranking nacional usa a aproximação
  da economia, que subestima as cidades grandes.
- **Relevância por estado ou região**: hoje ela vale para o país (metrópole ou colônias). Russos
  como elite de colonos na Ásia Central e no Cazaquistão não cabem nisso, porque são a maioria da
  União; ficaram sem marca.
- **Revisar** a estimativa religiosa da URSS, as culturas agregadas da África e da China, e as
  regras de raça como cultura.

## Demografia

Código: `crates/bf-sim/src/demography.rs` (sistema `demography`, todo dia 15). Estado salvo em
`GameWorld::demography`: a pirâmide de cada estado, a escolaridade das mães e as taxas do mês.
Nenhuma taxa é escrita por país nem por data: os parâmetros são mundiais e a diferença entre
países sai da economia de cada estado.

### Pirâmide

Cada estado guarda quantas pessoas tem em 17 grupos de cinco anos (0–4 … 75–79, 80+). Todo mês:
nascimentos entram em 0–4, cada grupo perde os seus mortos e um sessenta avos de cada grupo passa
ao seguinte. A pirâmide é reescalada à população do estado antes do mês (a migração interna mexe
nos pops) e o saldo do mês vai para as profissões na proporção do tamanho de cada uma.

**1925.** Não há pirâmide nos dados. Cada estado começa estável nas taxas que teria com 65 % do
desenvolvimento de hoje (conforto, escolaridade, trabalho fora do campo, hospitais, remédio) e
cresce 60 anos com o desenvolvimento subindo até o de 1925. Assim a Europa de 1925 já é mais
velha que a Índia e mais jovem do que seria se tivesse sempre tido a fecundidade de 1925.

### Mortalidade

Uma tabela mundial de mortes por idade em cinco níveis de saúde `h` (0, ¼, ½, ¾, 1), com
esperança de vida ao nascer de 27,7, 38,9, 52,5, 68,0 e 82,0 anos; entre os níveis, linear. A
tabela é um modelo de Gompertz-Makeham com mortalidade infantil (mortes de 0–4, 5–14, fundo
adulto e alta exponencial com a idade), construída para isso, não copiada de tábua de país.

    antes   = 0,38 × conforto + 0,35 × escolaridade − 0,05 − 0,25 × (1 − comida)
    teto    = 0,70 + 0,25 × conforto + 0,13 × hospitais
    alcance = 0,35 + 0,65 × max(hospitais, escolaridade)
    remédio = clamp((efeito "mortality" da pesquisa − 0,20) / (0,80 − 0,20)) × alcance
    h       = antes + (teto − antes) × remédio

- **Conforto** é o atendimento médio das necessidades de nível 3; **comida**, as de nível 1;
  **hospitais**, a cobertura hospitalar do estado (`public::coverage`).
- O remédio é o efeito `mortality` das tecnologias que o país usa (`research_system`). O que os
  países ricos já tinham em 1925 (vacina, saneamento) está no "antes"; só o que vem depois conta
  como remédio, e ele chega inteiro com 80 % da árvore (antibióticos, vacinas e inseticidas são a
  maior parte).
- **Alcance:** a campanha de massa leva um terço do remédio a quem não tem hospital nem escola
  (DDT no Ceilão de 1946); o resto vem com hospitais ou com mães que leem.
- Substitui a fórmula da fase 1 (natalidade e mortalidade em função linear da alfabetização,
  pisos de 7 e 6 por mil, remédio por calendário de 1935 a 1970 e mortalidade × hospitais). Sem
  dados de pesquisa (ferramentas e testes) o remédio ainda segue esse calendário.

### Fecundidade

Filhos por mulher (TFR), distribuídos pelas idades de 15 a 49 num perfil fixo (8, 23, 25, 20,
14, 8 e 2 %):

    nota  = 0,25 × sobrevivência + 0,55 × escolaridade + 0,10 × fora do campo
          + 0,10 × conforto + 0,17 × contracepção
    T     = suave((nota − 0,45) / (0,90 − 0,45))           (suave(t) = 3t² − 2t³)
    piso  = 1,75 + 0,5 × (1 − contracepção)
    TFR   = piso + (6,5 − piso) × (1 − T)

- **Sobrevivência:** a chance de passar dos 5 anos na mortalidade do mês, de 50 % (0) a 98 % (1).
  Os filhos morrem menos, as famílias querem menos filhos; o atraso entre as duas quedas sai da
  escolaridade.
- **Escolaridade das mães:** a alfabetização do estado seguida com atraso de 20 anos (240 meses):
  a menina na escola hoje é mãe daqui a uma geração.
- **Fora do campo:** parcela da população fora das profissões rurais (`urban_share`).
- **Contracepção:** o efeito `fertility` das tecnologias (a pílula) vezes o alcance,
  0,25 + 0,75 × max(escolaridade, cobertura hospitalar): o planejamento familiar chegou pela
  escola e pela rede de saúde (médicos descalços na China, postos na Indonésia e em Bangladesh).
- **Fome de terra (freio preventivo), desligado:** a regra existe (`LAND_BRAKE`, a fecundidade
  cai quando a terra por lavrador do estado fica abaixo da de 1925), mas com peso zero. O teto de
  terra da economia é a lavoura de 1925, não a terra arável; sem fronteira agrícola todo país que
  cresce "fica sem terra", o Brasil e a Argentina também, e o freio derrubou o mundo de 2025 para
  5,5 bilhões. Volta quando o teto for a terra potencial. Sem ela a transição para em 2,25 filhos (abstinência,
  coito interrompido, casamento tardio); com ela, em 1,75.
- **Decomposição:** `DemographyState::fertility_breakdown(econ, mercado, colônia)` dá, por
  território, cada termo com valor, peso e contribuição (`FertilityTerm`), para a tela mostrar
  "por que nascem tantos aqui". Os termos são a lista `FERTILITY_TERMS`: termo novo entra como
  mais uma linha ali.

### Migração entre países

Código: `crates/bf-sim/src/demography/migration.rs`. Unidade: território (metrópole ou colônias de
um país). Tudo roda no mês da demografia, em três camadas: o fluxo contínuo, a corrente da
diáspora e as ondas.

**Atração** de um destino `j` sobre a gente de `i`:

    atração = brecha × distância × vagas × corrente × família × lei de imigração de j
    brecha  = min(1, max(renda, terra) + rede)

- **Renda:** PIB por habitante do país a preços-base, repartido entre metrópole e colônias pela
  renda monetária por habitante. Começa a puxar a 1,5 vez a renda da origem; cheia a 4,5 vezes.
- **Terra:** teto de terra do território (níveis de fazenda) por mil lavradores (camponeses e
  assalariados rurais). Puxa a partir de 2 vezes a terra por lavrador da origem, cheia a 6 vezes,
  se o destino não for mais pobre que 0,3 da renda da origem: as fronteiras da Argentina, dos EUA,
  do Canadá, da Austrália.
- **Rede:** a diáspora puxa sozinha (cartas, passagem paga, emprego esperando): até 0,5 de
  brecha, para destino com pelo menos 0,20 da renda da origem. É o que leva italiano e português
  a um Brasil mais pobre que a Itália e que Portugal.
- **Distância:** 1 / (1 + (d / 8 000 km)²) entre os centros de população.
- **Vagas:** parcela dos operários do destino com emprego, à quarta potência.
- **Corrente:** 0,2 numa estrada sem ninguém do outro lado; sobe até ×4 quando a diáspora chega
  a 5 % da população da **origem** (quantas famílias têm alguém naquela estrada). Sem rede, pouca
  gente vai (teoria das redes de migração); com ela, a estrada se mantém. Medida contra a origem,
  a rede não nasce sozinha do fluxo contínuo de um país enorme (a China não "funda" uma corrente
  para a Alemanha com algumas dezenas de milhares).
- **Minorias com estrada própria** (`DemographyState::homes`): uma cultura que vive fora da pátria
  e não é a maior do seu território de origem (levantinos nas colônias francesas, armênios e
  ucranianos na URSS, galegos e catalães na Espanha, guzerates e punjabis na Índia) tem o próprio
  fluxo contínuo, tirado da sua gente na origem (a de 1925, escalada pela população do
  território), com a corrente medida contra ela. Os 41 mil sírio-libaneses do Brasil em 1925 são
  0,06 % das colônias francesas, mas 1,8 % dos levantinos delas.
- **Família:** × 2 na mesma família linguística (a maior do território). Metrópole e colônias
  não têm bônus próprio (`SAME_EMPIRE` = 1): com ele, Bélgica, Holanda e Portugal ficavam no teto
  de imigração por 60 anos.

**Fluxo contínuo.** Sai por ano até 3 por mil da população, vezes a melhor atração (até 1), a lei
de emigração (até 1000) e o empurrão do desemprego no campo (1 + parcela ociosa). Em cada
estrada fica quem pode pagar a passagem: 1 − (1 − própria) × (1 − pré-paga) × (1 − paga pela lei),
com a própria = 0,05 + 0,95 × alfabetização², a pré-paga = diáspora da estrada contra 5 % da
origem (parentes e contratadores mandavam a passagem) e a paga pela lei abaixo. Os emigrantes se repartem pelos destinos pela atração
vezes a população; nenhum destino recebe mais de 4 por mil ao ano.

**Ondas.** A pressão de saída de uma origem soma a parcela ociosa do campo, a fome (1 − necessidades
básicas atendidas) e metade da melhor atração. Quando passa de 0,6 e a origem não tem onda, abre
uma para o destino de maior atração × tamanho (relativo à origem), entre os que têm atração de
0,25 ou mais, porta aberta e vagas (operários empregados ≥ 0,5). Por mês a onda leva até 3 por mil
ao ano da origem, vezes a força da pressão, quem pode pagar e a atração; o destino recebe de ondas
no máximo 8 por mil ao ano. A onda fecha, com motivo registrado, quando:

| Motivo (`WaveEnd`) | Condição |
|---|---|
| `DoorClosed` | a lei de imigração do destino ou a de emigração da origem cai abaixo de 500 ‰ |
| `PressureEased` | a pressão cai abaixo de 0,35 |
| `Full` | as vagas do destino caem abaixo de 0,5 |
| `NoPull` | a atração cai abaixo de 0,125 |

A onda só abre numa estrada já aberta: com pioneiros (diáspora de pelo menos 0,1 da corrente
cheia) ou com passagem paga por lei. O motivo da abertura (`Push`) é o maior termo da pressão: terra (`Land`), fome (`Hunger`) ou
renda lá fora (`Poverty`). `DemographyState::waves` guarda as abertas e `waves_ended` as
fechadas; o placar lista as maiores ("Ondas de migração").

**Passagem paga pela lei.** Valor da lei acima de 1000 é a parcela da passagem que o Estado paga
(`subsidy`): `subsidized_immigration` (2000, destino) e `sponsored_emigration` (2000, origem).
O subsídio do destino só paga quem vem de origem com emigração livre ou patrocinada
(`regulated_emigration`, 700, a Itália de 1925: o rei proibiu a emigração subsidiada para o
Brasil). O patrocínio da origem só vale para destino que também subsidia, e só para a metrópole
(um programa de colonização dos dois governos: o Japão e o Brasil a partir de 1925); ele recruta
como 0,3 de uma corrente cheia. A passagem paga encurta a distância na mesma proporção. Em 1925:
o Brasil subsidia (lavoura paulista), o Japão patrocina, a Itália regula (fontes em
`data/scenario/1925/laws.ron`); a referência dá a cota brasileira de 1934.

**Porta.** Cotas (300 ‰) fecham a porta para ondas, não para o fluxo contínuo, que passa reduzido.
Quando a porta de um destino fecha, a onda termina com `DoorClosed` e a próxima abre para o melhor
destino aberto: é o desvio de destino (EUA fecham → Brasil e Argentina). As leis vêm do estado
político (`Demography::migration_laws`, efeitos `immigration` e `emigration`); quando a política
mudar as leis no meio da partida, as ondas reagem no mês seguinte.

**Diáspora.** `DemographyState::diaspora`: pessoas por origem, destino e cultura (`Diaspora`), com
`people` (chegados e descendentes, crescendo com nascimentos e mortes do destino) e `arrived`
(chegadas desde 1925, sem filhos). Quem chega por uma estrada leva as culturas que já estão nela,
na proporção; numa estrada nova, a cultura principal da origem. Quando os pops carregarem cultura,
cada entrada vira a comunidade `culture` no destino. `immigrant_share(econ, país)` e
`immigrants_by_origin(país)` dão a parcela de origem estrangeira para a política.

**1925.** O estoque inicial sai das comunidades de 1925 (`initial_diaspora`): uma cultura fora da
pátria (o território com mais gente dela), onde não é nativa (a maior em algum estado) e em estado
que não encosta na pátria por terra, conta como gente da pátria naquele território. Resultado
(`cargo test -p bf-sim --test demography_1925 -- --nocapture`): 270 estradas, 28 milhões de
pessoas. No Brasil: italianos 896 mil, alemães 519 mil, portugueses 415 mil, espanhóis 249 mil,
poloneses 89 mil, sírio-libaneses 41 mil (vindos das colônias francesas, onde ficava o mandato
da Síria e do Líbano), japoneses 27 mil, ucranianos 24 mil. Ficam de fora:

- culturas sem pátria clara (a maior parte em nenhum território): asquenazes (32 % no maior),
  sefarditas, curdos, melanésios, amazônicos, inuítes;
- minorias de fronteira (alemães da Boêmia, húngaros da Transilvânia, quíchuas da Bolívia), que
  não migraram;
- o que os dados de 1925 já fundiram na cultura do país (o "brasileiro" de origem italiana).

**Quem se move:** sai dos camponeses, assalariados rurais e operários; chega como operário, nas
cidades (na proporção da população urbana dos estados). Na pirâmide, idades de 20 a 34.

### O que falta

- **Guerra, fome e regime.** Não há guerra (1914–18 já passou; 1939–45 não acontece), nem fome
  (China 1928–30 e 1959–61, URSS 1932–33, Bengala 1943), nem política de natalidade (filho único,
  proibição do aborto na Romênia em 1966), nem a alfabetização forçada e a saúde pública dos
  regimes socialistas, que adiantaram a transição no Leste Europeu e na URSS.
- **Comunidade nos pops.** O imigrante chega como gente, não como comunidade: o `Pop` é estado ×
  profissão. A diáspora guarda origem e cultura até as comunidades entrarem nos pops.
- **Exclusão racial e refugiados.** A exclusão racial é lei dirigida e não pesa enquanto o
  migrante não carrega comunidade; não há refugiado (sem guerra).
- **Fronteira agrícola.** O freio por fome de terra espera o teto de terra arável (thread da
  economia).
- **Planejamento familiar como lei.** Hoje a causa é a rede de saúde; quando a política tiver
  quem peça, vira lei (programa voluntário, programa de Estado, limite coercitivo como o filho
  único).
- **Datas das leis de migração.** As mudanças da referência (o fim do subsídio paulista por volta
  de 1930, a cota brasileira de 1934, a restrição fascista de 1927, a guerra civil espanhola) só
  acontecem se um movimento da política as fizer; na rodada, o Brasil subsidia até 1994.
- **Ondas de minorias.** As minorias têm fluxo contínuo próprio, não ondas.
- **Natalidade por classe (próximo PR).** Hoje os nascimentos do estado vão às profissões na
  proporção do tamanho (`distribute`). Desenho: cada pop com o seu TFR, pelos mesmos termos
  regionais (sobrevivência, escolaridade, contracepção) mais o conforto da própria pop
  (`satisfaction[2]`) e "fora do campo" igual a 0 ou 1 pela profissão; nascimentos do estado =
  mulheres férteis da pirâmide × parcela de cada pop × TFR dela, e cada pop recebe os próprios
  filhos. Sem pirâmide por pop: todas as pops do estado têm a mesma estrutura de idade. Custa 13
  avaliações por estado por mês (~1 s de CPU em 100 anos). Fica para depois porque o TFR da
  região passa a ser a média dos TFR das pops, não o TFR da média (a curva é não linear), o que
  desloca a calibração, e porque muda a composição de classes e o êxodo rural.
- **Cultura e religião na natalidade (próximo PR, com as comunidades).** Cultura como difusão da
  transição dentro da família linguística (a conferir no Princeton European Fertility Project),
  não como "cultura X tem mais filhos"; religião só com efeito medido em fonte pública, líquido
  de escolaridade e renda. Entram como termos novos de `FERTILITY_TERMS`.
- **Pirâmide por comunidade**; idade de trabalho fixa em 40 % de cada pop.
- **Baby boom.** Nada no modelo faz a fecundidade dos ricos subir de 1946 a 1964.
