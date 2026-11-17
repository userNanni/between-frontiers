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
    alcance = 0,55 + 0,45 × max(hospitais, escolaridade)
    remédio = clamp((efeito "mortality" da pesquisa − 0,20) / (0,80 − 0,20)) × alcance
    h       = antes + (teto − antes) × remédio

- **Conforto** é o atendimento médio das necessidades de nível 3; **comida**, as de nível 1;
  **hospitais**, a cobertura hospitalar do estado (`public::coverage`).
- O remédio é o efeito `mortality` das tecnologias que o país usa (`research_system`). O que os
  países ricos já tinham em 1925 (vacina, saneamento) está no "antes"; só o que vem depois conta
  como remédio, e ele chega inteiro com 80 % da árvore (antibióticos, vacinas e inseticidas são a
  maior parte).
- **Alcance:** a campanha de massa leva mais da metade do remédio a quem não tem hospital nem
  escola, a centavos por pessoa: vacina contra varíola a 1–2 centavos a dose, penicilina contra
  a bouba em mais de 50 milhões de pessoas entre 1952 e 1964, DDT no Ceilão de 1946. O resto vem
  com hospitais ou com mães que leem. Preston (1975): a renda explica de 10 % a 25 % do ganho de
  expectativa de vida entre os anos 1930 e 1960; a Índia foi de 32 para 51 anos entre 1941 e 1975
  com um sexto das mulheres alfabetizadas e 0,5 leito por mil. Antes desta regra o alcance sem rede era
  um terço, e com a escola das mães na medida real a Índia ficava em 44 anos em 1975.
- **A ordem histórica:** a mortalidade cai primeiro (campanha, sem escola) e a fecundidade décadas
  depois (escola das mães e contracepção). Defasagem real: 15 a 20 anos no Ceilão, 30 a 35 no
  México e na África subsaariana, cerca de 40 na Índia; a fecundidade subsaariana chegou a subir
  (6,6 em 1960, 6,8 em 1975–80) enquanto a mortalidade caía. É isso que faz a população explodir.
- Substitui a fórmula da fase 1 (natalidade e mortalidade em função linear da alfabetização,
  pisos de 7 e 6 por mil, remédio por calendário de 1935 a 1970 e mortalidade × hospitais). Sem
  dados de pesquisa (ferramentas e testes) o remédio ainda segue esse calendário.

### Fecundidade

Filhos por mulher (TFR), distribuídos pelas idades de 15 a 49 num perfil fixo (8, 23, 25, 20,
14, 8 e 2 %):

    nota  = 0,30 × sobrevivência + 0,57 × escolaridade + 0,03 × fora do campo
          + 0,10 × conforto + 0,17 × contracepção
    T     = suave((nota − 0,45) / (0,90 − 0,45))           (suave(t) = 3t² − 2t³)
    piso  = 1,75 + 0,5 × (1 − contracepção)
    TFR   = (piso + (6,5 − piso) × (1 − T)) × (1 − 0,5 × fome de terra)

- **Sobrevivência:** a chance de passar dos 5 anos na mortalidade do mês, de 50 % (0) a 98 % (1).
  Os filhos morrem menos, as famílias querem menos filhos; o atraso entre as duas quedas sai da
  escolaridade.
- **Escolaridade das mães:** a alfabetização das mulheres do estado seguida com atraso de 20 anos
  (240 meses): a menina na escola hoje é mãe daqui a uma geração. Onde pouca gente lê, a menina vai
  à escola depois do menino: alfabetização das mulheres = alfabetização × (1 − 0,5 × (1 −
  alfabetização)) (`women_literacy`). Censos (Banco Mundial, WDI `SE.ADT.LITR.ZS` e `.FE.ZS`):
  Índia 1991, 48 % e 34 %; Egito 1986, 44 % e 31 %; Argélia 1987, 50 % e 36 %; Nigéria 1991, 56 % e
  44 %; Turquia 1990, 79 % e 69 % (lacuna de 0,5 a 0,65). México e Brasil têm menos (88 % e 85 %;
  75 % e 73 %): ali a regra põe as mães um pouco abaixo do real.
- **Fora do campo:** parcela da população fora das profissões rurais (`urban_share`), com peso
  pequeno (0,03). A literatura põe a escola das mães e a sobrevivência infantil como causas
  sustentadas da queda; a urbanização por trabalho informal, sem escola e sem saúde, não derruba a
  fecundidade (com peso 0,10 ela derrubava a do Brasil e da África cedo demais depois do #69).
- **Contracepção:** o efeito `fertility` das tecnologias (a pílula) vezes o alcance,
  0,05 + 0,95 × max(escolaridade, cobertura hospitalar): o planejamento familiar chegou pela
  escola e pela rede de saúde (médicos descalços na China, postos na Indonésia e em Bangladesh), e
  quase não chega a quem não tem nenhuma das duas. Uso de método moderno (Banco Mundial,
  `SP.DYN.CONM.ZS`): África subsaariana 11 % em 1990 e 16 % em 2000; Índia 9,5 % em 1971 e 28 % em
  1981; Bangladesh, pobre e sem escola, abaixo de 10 % em meados dos anos 1970 e 45 % em 1993–94,
  com agentes de saúde de porta em porta. Antes desta regra o piso era 0,25, e a Índia de 1975 chegava a
  0,47.
  Sem contracepção a transição para em 2,25 filhos (abstinência, coito interrompido, casamento
  tardio); com ela, em 1,75.
- **Fome de terra (freio preventivo):** a lavoura que cada lavrador tem, aberta e ainda por abrir
  na fronteira agrícola (`Region::land` + `Region::frontier`, níveis por mil camponeses e
  assalariados rurais). Onde ela está abaixo de 20 níveis por mil **e** abaixo da do próprio estado
  em 1925, a fecundidade cai na proporção do que se perdeu, até a metade. Em 1925: Brasil 140,
  Argentina 126, Austrália 329, Itália 11, Japão 6, Índia 5, China 4, Egito 0,8. Brasil, Argentina e
  África têm terra para abrir e não freiam; a Ásia rural freia quando os lavradores crescem. Uma
  medida só absoluta frearia o Nilo irrigado já em 1925, quando a fecundidade era 6,5; uma só
  relativa a 1925, sem fronteira, freava todo país que crescia (o mundo de 2025 caía a 5,5 bilhões).
- **Decomposição:** `DemographyState::fertility_breakdown(econ, mercado, colônia)` dá, por
  território, cada termo com valor, peso e contribuição (`FertilityTerm`), para a tela mostrar
  "por que nascem tantos aqui"; o `--history-out` grava os termos por território e ano
  (`fertility_terms`). Os termos são a lista `FERTILITY_TERMS`: termo novo entra como mais uma
  linha ali.

### Natalidade por classe

Cada pop (estado × profissão) tem o próprio TFR, pela mesma curva: sobrevivência infantil,
escolaridade das mães e contracepção do estado; o conforto da própria pop (`satisfaction[2]`) e
"fora do campo" 1 para as profissões urbanas e 0 para camponês, assalariado rural e proprietário
de terra (`is_urban`). Os nascimentos do estado saem da pirâmide com o TFR médio das pops,
ponderado pelo tamanho, e cada pop recebe os seus: parcela da pop × TFR dela
(`split_births`). As mortes continuam proporcionais ao tamanho. Não há pirâmide por pop: todas
as pops do estado têm a mesma estrutura de idade.

- **Escolaridade fica do estado.** Não há alfabetização por profissão nos pops nem em `data/`.
  A diferença entre classes vem só do conforto (peso 0,10) e do trabalho fora do campo (0,03).
- **A fome de terra freia todas as classes do estado**, como foi calibrada (#54). Medido em 51
  anos: só nos camponeses e assalariados rurais, o pobre das cidades da Ásia escapa do freio, a
  Índia de 1975 fica com camponês 0,81 vez o operário (a NFHS-1 de 1992–93 mede o rural 1,36
  vez o urbano) e China, Raj, URSS e Romênia sobem mais de 0,05 na razão de população de 1975.
- **Nenhum peso mudou.** A média dos TFR das pops em vez do TFR das médias desloca o TFR das
  regiões em no máximo 0,1 contra a base (tabela do PR): não houve o que recompensar.
- **Saída, não estado:** `Vital::tfr_by_class` é recalculado todo mês e nada na simulação o
  lê; fica fora do save e do hash, e o save antigo carrega (zerado até o primeiro mês).
- **Tela:** a de População mostra os filhos por mulher de cada classe do país ou do estado que
  a pirâmide mostra; o `--history-out` grava `class_share` e `class_fertility` por território.
- **CPU:** a demografia passa de 4,6 para 5,4 ms por mês (13 avaliações da curva por estado), cerca
  de 1 s em 100 anos.

**O diferencial contra a literatura.** Fontes (fecundidade marital, bruta de escolaridade e
renda): Kiser, *Milbank Memorial Fund Quarterly* 33(4), 1955, fig. 6 e 8 (EUA, filhos de mulheres
de 45–49 anos: em 1910 fazendeiro 5,60, braçal urbano 4,79, profissional liberal urbano 2,76; em
1940, 4,14, 3,23 e 1,96); censo da Inglaterra e Gales de 1911, *Fertility of Marriage*, parte II,
tabela XLIV (por classe, padronizado pela idade ao casar: trabalhador agrícola 5,56, não
qualificado 5,28, classe I 3,89); Lerch, MPIDR WP 2017-011 (a razão rural/urbano começa em 1,0 e
sobe a 1,3–1,7 durante a transição); Berquó e Cavenaghi (Brasil 1991, rural 4,3 e urbano 2,3).

| Unidade | Camponês ÷ liberal (simulado) | Operário ÷ liberal (simulado) | Referência |
|---|---:|---:|---|
| EUA 1925 / 1940 | 1,36 / 1,58 | 0,99 / 1,00 | fazendeiro ÷ liberal 2,03 (1910), 2,1 (1940); braçal ÷ liberal 1,74, 1,65 |
| Grã-Bretanha 1925 | 1,43 | 1,00 | trabalhador agrícola ÷ classe I 1,43; não qualificado ÷ classe I 1,36 (1911) |
| Itália 1930 / 1950 | 1,31 / 1,51 | 1,00 / 1,01 | – |
| Japão 1950 | 1,51 | 1,00 | – |
| Brasil 1960 / 1970 | 1,00 / 1,15 | 1,01 / 1,02 | rural ÷ urbano 1,87 (1991) |
| México 1970 | 1,22 | 1,00 | América Latina 1,3 a 1,7 (Lerch) |
| Índia 1975 | 0,99 | 1,04 | rural ÷ urbano 1,36 (1992–93) |

Critério **parcialmente atendido**: campo contra cidade com o sinal certo e a magnitude da
Inglaterra, metade da dos EUA em log, fraco no Brasil e no México; entre classes urbanas, zero.
Por quê: a diferença de nota entre classes vai a no máximo 0,13 (conforto e fora do campo),
contra 0,45 de largura da transição, e só aparece enquanto o estado está no meio da curva; nos
países ricos o conforto das classes urbanas está perto do teto e é igual entre elas. Pela curva
de hoje, os EUA de 1910 pedem 0,17 de nota entre o liberal e o braçal e 0,23 entre o liberal e o
fazendeiro, o que, com o peso da escolaridade (0,57), são **cerca de 0,30 de escolaridade das
mães entre a mulher do liberal e a do braçal**. Fecha com escolaridade por ocupação com fonte
(amostras de censo do IPUMS para os EUA de 1910–40, o censo inglês de 1911, WFS e DHS nos países
em desenvolvimento), como PR de dados; nenhum peso foi mexido para fabricar o diferencial.

### Migração entre países

Código: `crates/bf-sim/src/demography/migration.rs`. Unidade: território (metrópole ou colônias de
um país, `TerritoryId`). Tudo roda no mês da demografia, em três camadas: o fluxo contínuo, a
corrente da diáspora e as ondas.

**Atração** de um destino `j` sobre a gente de `i`:

    atração = brecha × distância × vagas × corrente × família × lei de imigração de j (até 1000)
    brecha  = min(1, max(renda, terra) + rede)

- **Renda:** PIB por habitante do país a preços-base, repartido entre metrópole e colônias pela
  renda monetária por habitante. Começa a puxar a 1,5 vez a renda da origem; cheia a 4,5 vezes.
- **Terra:** lavoura aberta e por abrir por mil lavradores. Puxa a partir de 2 vezes a da origem,
  cheia a 22 vezes, se o destino não for mais pobre que 0,2 da renda da origem: as fronteiras das
  Américas e da Austrália. Só puxa a parcela da origem que vive da terra (camponeses e
  assalariados rurais): um país de operários manda poucos colonos. Sem isso a Alemanha
  industrial mandava ondas "por pobreza" para o Brasil, 0,23 da renda dela, pela terra do Brasil.
- **Rede:** a diáspora puxa sozinha (cartas, passagem paga, emprego esperando): até 0,5 de
  brecha, para destino com pelo menos 0,20 da renda da origem. É o que leva italiano e português
  a um Brasil mais pobre que a Itália e que Portugal.
- **Distância:** 1 / (1 + (d / 8 000 km)²) entre os centros de população.
- **Vagas:** parcela dos operários do destino com emprego, à quarta potência.
- **Corrente:** 0,2 numa estrada sem ninguém do outro lado; sobe até ×4 quando a diáspora chega
  a 5 % da população da **origem** (quantas famílias têm alguém naquela estrada). Sem rede, pouca
  gente vai (teoria das redes de migração); com ela, a estrada se mantém. Medida contra a origem,
  a rede não nasce sozinha do fluxo contínuo de um país enorme.
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
origem (parentes e contratadores mandavam a passagem) e a paga pela lei abaixo. Os emigrantes se
repartem pelos destinos pela atração vezes a população; nenhum destino recebe do fluxo contínuo
mais de 2 500 por milhão ao ano (o Brasil e os EUA receberam perto de 2 400 e 2 600 nos anos 1920).

**Ondas.** A pressão de saída de uma origem soma a parcela ociosa do campo, a fome (1 − necessidades
básicas atendidas) e metade da melhor atração. Quando passa de 0,6 e a origem não tem onda, abre
uma para o destino de maior atração × tamanho (relativo à origem), entre os que têm atração de
0,25 ou mais, porta aberta, vagas (operários empregados ≥ 0,5) e estrada aberta: pioneiros
(diáspora de pelo menos 0,3 da corrente cheia) ou um programa dos dois governos. Por mês a onda
leva até 1 por mil ao ano da origem, vezes a força da pressão, quem pode pagar e a atração; o
destino recebe de ondas no máximo 3 por mil ao ano. A onda fecha, com motivo registrado, quando:

| Motivo (`WaveEnd`) | Condição |
|---|---|
| `DoorClosed` | a lei de imigração do destino ou a de emigração da origem cai abaixo de 1000 ‰ (cota, emigração regulada) |
| `PressureEased` | a pressão cai abaixo de 0,35 |
| `Full` | as vagas do destino caem abaixo de 0,5 |
| `NoPull` | a atração cai abaixo de 0,125 |

O motivo da abertura (`Push`) é o maior termo da pressão: falta de terra em casa (`Land`, os
camponeses ociosos), fome (`Hunger`) ou o que atrai na melhor estrada, separado pela parte que
mais pesa nela: renda (`Poverty`), terra para abrir (`Frontier`) ou a diáspora (`Chain`). `DemographyState::waves` guarda as abertas e `waves_ended` as fechadas; o
placar lista as maiores ("Ondas de migração").

**Passagem paga pela lei.** Valor da lei acima de 1000 é a parcela da passagem que o Estado paga
(`subsidy`): `subsidized_immigration` (2000, destino) e `sponsored_emigration` (2000, origem).
O subsídio do destino só paga a passagem de quem vem de origem com emigração livre ou patrocinada
(`regulated_emigration`, 700, a Itália de 1925: o rei proibiu a emigração subsidiada para o
Brasil); não recruta ninguém nem encurta a distância (a distância também é notícia e contato, que
uma passagem não traz). Um programa de colonização dos dois governos (a origem patrocina e o
destino subsidia, só para a metrópole: o Japão e o Brasil a partir de 1925) recruta como 0,05 de uma
corrente cheia, abre estrada para onda e encurta a distância pela metade. Em 1925: o Brasil
subsidia (lavoura paulista), o Japão patrocina, a Itália regula (fontes em
`data/scenario/1925/laws.ron`); a referência dá a cota brasileira de 1934.

**Porta.** Cotas (300 ‰) e emigração regulada fecham a porta para ondas, não para o fluxo contínuo,
que passa reduzido. Quando a porta de um destino fecha, a onda termina com `DoorClosed` e a próxima
abre para o melhor destino aberto: é o desvio de destino (EUA fecham → Brasil e Argentina). As leis
vêm do estado político (`Demography::migration_laws`, efeitos `immigration` e `emigration`); quando
a política muda as leis (o `nativism` fecha, o `immigration_reform` abre), as ondas reagem no mês
seguinte.

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

### Cultura e religião na natalidade (desenho, sem código)

Estado: desenho para o dono decidir junto com as decisões 1 a 3 (`architecture.pt-BR.md`). Nada
disto roda: os termos só entram quando as comunidades estiverem nos pops. Regra: cultura entra
como **difusão** da transição, nunca como "a cultura X tem mais filhos"; religião só com efeito
medido **líquido de escolaridade e renda**. Efeito sem essa medição não entra.

#### O que a literatura mede

**Difusão pela língua.** O Projeto Europeu de Fecundidade de Princeton (Coale e Watkins, orgs.,
*The Decline of Fertility in Europe*, 1986) datou a queda de 10 % da fecundidade marital (`Ig`)
por província e concluiu, como o resumem Spolaore e Wacziarg (2022), que a data segue mais a
língua e a região cultural do que a economia. As críticas: a província agregada mistura cidade e
campo e controla pouco a economia (Brown e Guinnane, *Economic History Review* 60(3), 2007; na
Baviera, Brown e Guinnane, *Population Studies*, 2002, acham que o Projeto deixou passar a
economia). A medida mais limpa que encontramos responde à crítica:

| Fonte | O que mede | Efeito |
|---|---|---|
| Spolaore e Wacziarg, "Fertility and Modernity", *Economic Journal* 132(642), 2022, tab. 1, 2, 5, 6 e A9 | Ano da queda de 10 % de `Ig` em 771 regiões europeias (média 1899, desvio-padrão 25 anos) contra a distância linguística (nós da árvore) ao francês de Paris, com efeito fixo de país | 2,26 anos de atraso por nó sem controles; 3,19 com a alfabetização de 1880 (413 regiões; a alfabetização sai não significativa, t = 0,89); 3,90 com todos os controles (296 regiões). Um desvio-padrão de distância (3 nós) ≈ 7 a 12 anos. O efeito some nas coortes de 1931–60 (tab. 5). A distância à região vizinha que já caiu pesa pouco (tab. A9): é difusão a partir da fronteira, não contágio entre vizinhos |
| Lesthaeghe, *The Decline of Belgian Fertility, 1800–1970*, 1977, citado em Spolaore e Wacziarg, p. 799–800 | 70 pares de aldeias valãs e flamengas vizinhas de economia parecida | A valã caiu primeiro em 62 pares, cerca de 20 anos antes; Liège e Bruges, com 59 % e 61 % de alfabetizados em 1880, caíram em 1875 e 1905 |

Falta renda direta nas duas (só carvão, urbanização e densidade como substitutos); a
alfabetização entra e não apaga o efeito.

**Religião, líquida de escolaridade e renda.**

| Fonte | Medida | Líquido de | Efeito | Entra? |
|---|---|---|---|---|
| Becker, Cinnirella e Woessmann, CESifo WP 2775, 2009, tab. 2 e 8 (334 condados da Prússia) | Crianças por mulher (1849); queda da fecundidade marital 1890–1905 | Matrícula escolar, indústria, agricultura, urbanização, densidade, casadas, poloneses, expectativa de vida (renda só pela ocupação) | Condado todo protestante: cerca de 6 % menos crianças por mulher em 1849 (coef. −0,038 a −0,045 sobre média 0,64) e queda cerca de 14 pontos mais rápida em 1890–1905 | **Não**: medida de antes de 1925; usá-la de 1925 a 2025 seria extrapolar |
| Pasupuleti, Pathak e Jatrana, *Journal of Biosocial Science*, 2017 (só o resumo lido) | Fecundidade completa de muçulmanas e hindus na Índia (NFHS) | Características socioeconômicas e demográficas | Diferença de mais de 1,3 filho; os controles explicam 31 %; sobra cerca de 0,9 | **Ainda não**: falta ler a tabela (ou a de Dharmalingam e Morgan, *Demography* 41(3), 2004) |
| Diferença bruta na Índia (Pew Research Center, 2021; NFHS) | TFR | Nada | 1,1 filho em 1992 (4,4 e 3,3), 0,5 em 2015, 0,4 em 2019–21 | Não (bruta) |
| Liefbroer, 2012, Historical Sample of the Netherlands | Intervalo entre nascimentos, casais de 1850–1920 | Só idade, parturição e coorte | Católicos com cerca de 1/3 da chance de espaçar dos protestantes liberais | Não (sem classe) |
| Westoff e Jones, 1979; Mosher, Williams e Johnson, 1992 (EUA) | Católicos, mórmons, protestantes | Educação, segundo Lehrer (IZA DP 3541, 2008) | Católicos convergem nos anos 1970; mórmons e protestantes de alta frequência acima | **Ainda não**: números não lidos |

McQuillan ("When does religion influence fertility?", *Population and Development Review*,
2004) dá a condição: a religião pesa quando a igreja tem norma sobre filhos, meios de fazê-la
cumprir e é centro da identidade. Lehrer (2008) lembra que controlar a escolaridade subestima o
efeito total, porque a religião também molda a escola: o efeito líquido é o piso, e é o que a
regra aceita.

#### Termo novo: difusão (`diffusion`)

    D_f   = parcela das pessoas da família linguística f, no mundo e fora do próprio estado,
            que vive em estados já em transição (TFR 10 % abaixo de 6,5), no mês anterior
    valor = Σ (parcela da comunidade c no pop) × D_família(c)

- **O que lê:** a composição de comunidades de cada pop (cultura → família, `CULTURE_FAMILIES`)
  e o TFR do mês anterior de todos os estados (`Vital::tfr`). Fora do próprio estado e com um
  mês de atraso, para o termo não se alimentar de si mesmo no mesmo mês.
- **Por que a família e não a distância na árvore:** os dados têm 34 famílias e nenhuma árvore.
  Dentro da família o termo dá o que Lesthaeghe e Spolaore e Wacziarg medem (quem fala a língua
  de quem já caiu cai antes); entre famílias, não há gradação (romance e germânico ficam tão
  longe quanto romance e banto). Uma árvore com fonte (Glottolog, CC BY 4.0) daria a distância
  em nós, como no artigo; fica como passo seguinte, se a família não bastar no placar.
- **Neutro por construção:** se todas as famílias tivessem o mesmo D, o termo seria igual para
  todos e não mudaria a ordem de ninguém. Não há valor por cultura nos dados; a diferença só nasce
  de onde a transição já chegou.
- **Some sozinho:** quando toda família passou da transição, D = 1 para todos e o termo vira
  constante, como o efeito que se apaga nas coortes de 1931–60.
- **Peso (proposta, a calibrar):** o índice da transição sobe, na mediana dos territórios, 0,0095
  por ano quando cruza o início da queda (`F_START`; base `origin/main` em fb95145, 31
  territórios com mais de 2 milhões). Um adiantamento de 10 anos entre D = 1 e D = 0 pede peso
  ≈ 10 × 0,0095 ≈ 0,10; os 20 anos de Lesthaeghe pedem ≈ 0,19; um desvio-padrão de Spolaore e
  Wacziarg (7 a 12 anos) pede 0,07 a 0,11. Proposta: **0,10**, com o mesmo valor descontado da
  soma dos outros pesos ou de `F_START`, para a média mundial de 1925 não se mexer, e recalibrado
  no placar.
- **CPU:** D_f é uma soma por família sobre as linhas da tabela (cerca de 45 mil no teto de 1925)
  por mês, mais uma soma por pop: menos de 0,2 s em 100 anos.
- **Teste contrafactual:** (1) teste de unidade com duas regiões iguais em tudo menos a família,
  uma com D = 0,9 e outra com D = 0,1: a queda de 10 % da segunda vem de 5 a 20 anos depois;
  (2) teste de neutralidade: com D igual para todas as famílias, o TFR de cada estado é o mesmo
  que sem o termo, a menos da constante descontada; (3) rodada de 100 anos com peso 0 e com o
  peso proposto: desvio-padrão do ano da queda de 10 % entre os estados europeus (Princeton: 25
  anos) e regressão do ano da queda sobre D, contra os 2 a 4 anos por nó do artigo.

#### Termo novo: religião (`religion`), sem número

    valor = Σ (parcela da religião r no pop) × efeito(r, região, década)

- **O que lê:** a religião das comunidades do pop e uma tabela de efeitos
  (`data/demography/religion_fertility.ron`), cada linha com religião, região, período, efeito em
  filhos por mulher, controles usados e a fonte com tabela.
- **Hoje a tabela fica vazia.** A única medida líquida lida (Prússia) é anterior a 1925; o
  diferencial muçulmano–hindu na Índia é o primeiro candidato, quando a tabela de Pasupuleti et
  al. (2017) ou de Dharmalingam e Morgan (2004) for lida; o dos católicos nos EUA depois de
  Westoff e Jones (1979) e Mosher et al. (1992). O efeito entra como diferença ao TFR (não como
  peso da nota) para que "0,9 filho líquido" seja o número lido, e só no período em que foi medido.
- **CPU:** uma busca por pop e por mês; desprezível.
- **Teste contrafactual:** trocar a religião de uma comunidade e conferir que o TFR muda pelo
  efeito da tabela e nada mais; com a tabela vazia, o hash de 100 anos é o mesmo que sem o termo.

#### O que a ligação das comunidades aos pops pede da demografia

Hoje o pop é estado × profissão (11 310 linhas). Com as comunidades vira estado × comunidade ×
profissão: no teto de 1925, 45 630 linhas (`communities_1925`), na prática menos, porque nem
toda comunidade ocupa toda profissão.

| Peça | Opção | Linhas de código (estimativa) | CPU em 100 anos (base: demografia 5,5 s de 198 s, 2 threads) |
|---|---|---|---|
| Filhos por linha | TFR por linha com os termos de hoje mais difusão e religião; cada linha recebe os seus (como a natalidade por classe) | 100–150 | ×3 a ×4 das avaliações de TFR: +2 a +3 s |
| Pirâmide | **(a) por estado**, todas as linhas com a mesma estrutura de idade (recomendado para começar) | 0 | 0 |
| | (b) por estado × comunidade (cerca de 3 700 pirâmides contra 870) | 400–600 | +6 a +10 s; save +0,5 MB |
| Migração internacional | A diáspora já guarda a cultura: quem chega entra na linha da sua comunidade no destino | 150–250 | desprezível |
| Migração interna | O êxodo rural e a mudança de profissão movem gente de cada comunidade na proporção da linha de origem | 300–500 (em `econ/migration`) | economia mensal ×3: +5 a +7 s |
| Assimilação (decisão 3) | Fluxo anual entre linhas | do lado das comunidades | anual: desprezível |

Recomendação: (a) e a pirâmide por estado. O que (a) erra: imigrante chega com 20 a 34 anos e
tem mais filhos por cabeça no começo; com a pirâmide do estado, a comunidade imigrante cresce
devagar demais nos primeiros 20 anos. Se o placar mostrar isso (diáspora no Brasil e nos EUA),
passa para (b). A difusão e a religião não precisam de pirâmide por comunidade: leem parcelas.

### O que falta

- **Guerra, fome e regime.** Não há guerra (1914–18 já passou; 1939–45 não acontece), nem fome
  (China 1928–30 e 1959–61, URSS 1932–33, Bengala 1943), nem política de natalidade (filho único,
  proibição do aborto na Romênia em 1966), nem a alfabetização forçada e a saúde pública dos
  regimes socialistas, que adiantaram a transição no Leste Europeu e na URSS.
- **Comunidade nos pops.** O imigrante chega como gente, não como comunidade: o `Pop` é estado ×
  profissão. A diáspora guarda origem e cultura até as comunidades entrarem nos pops.
- **Exclusão racial e refugiados.** A exclusão racial é lei dirigida e não pesa enquanto o
  migrante não carrega comunidade; não há refugiado (sem guerra).
- **Ensino e saúde coloniais.** As colônias se alfabetizam no ritmo da metrópole (colônias
  britânicas 0,79 em 1985, real 0,57) e recebem o remédio e a pílula com a força da metrópole,
  porque os efeitos de pesquisa são por mercado e a colônia divide o mercado (expectativa de vida
  78 em 1985 contra 54 reais). Daí a transição cedo demais depois de 1975 na Ásia do Sul e na
  África (placar em `history.pt-BR.md`, "Placar com a demografia"). É entrada da economia e da
  política (trabalho de ensino e orçamento coloniais), não parâmetro da demografia; a demografia
  se recalibra depois que ele entrar.
- **Pioneiro.** A fronteira agrícola (#77) só abre pela fazenda comercial; o lavrador que abre terra
  para si (e com ela o freio por fome de terra fora da Ásia) vem com o efeito `frontier_access`.
- **Planejamento familiar como lei.** Hoje a causa é a rede de saúde; quando a política tiver
  quem peça, vira lei (programa voluntário, programa de Estado, limite coercitivo como o filho
  único).
- **Datas das leis de migração.** As mudanças da referência (o fim do subsídio paulista por volta
  de 1930, a cota brasileira de 1934, a restrição fascista de 1927, a guerra civil espanhola) só
  acontecem se um movimento da política as fizer; na rodada de 100 anos o `nativism` tira o
  subsídio brasileiro em 1976 e põe cotas em 1981, e os EUA não saem das cotas de 1924.
- **Ondas de minorias.** As minorias têm fluxo contínuo próprio, não ondas.
- **Escolaridade por ocupação.** A natalidade por classe usa a escolaridade do estado; a do
  liberal e a do braçal são iguais (ver "Natalidade por classe"). Próximo PR de dados.
- **Cultura e religião na natalidade.** Desenho acima ("Cultura e religião na natalidade"): entra
  depois das decisões 1 a 3, com as comunidades nos pops.
- **Pirâmide por comunidade**; idade de trabalho fixa em 40 % de cada pop.
- **Baby boom.** Nada no modelo faz a fecundidade dos ricos subir de 1946 a 1964.
