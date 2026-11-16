# População: comunidades de 1925

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
