# Relações entre países: as medidas (fatia 1 da IA estratégica)

Primeira fatia de `docs/ai.pt-BR.md`, autorizada pelo orquestrador em 2026-11-24: as medidas, só
leitura, atrás da chave `relations`, desligada. Nada no tick lê estas medidas, e nada se salva: a
interface as calcula quando as mostra (`relations::measure`). Por isso a série e o hash são os da
`main` com a chave ligada ou desligada. As fatias seguintes (armar-se, alianças, crise e guerra)
esperam o dono.

Código em `crates/bf-sim/src/relations.rs`; pesos em `data/diplomacy/relations.ron`.

## Poder: o CINC

O Composite Index of National Capability do Correlates of War (Singer, Bremer e Stuckey, 1972) é a
média das parcelas mundiais de seis componentes (National Material Capabilities Documentation v7.0,
"CINC Score", p. 8). O NMC mede o Estado, não o império: "Colonial troops (such as Indian troops
under British command during India's colonial period) were usually not included in this total if
they were a separately administered force" (NMC v7.0, p. 12). No jogo, da metrópole:

| Componente | De onde sai |
|---|---|
| gasto militar | o teto militar do Estado (`econ::public::military_room`), o do país, como o NMC conta o orçamento |
| pessoal militar | os soldados das regiões não coloniais |
| aço | o que as usinas das regiões não coloniais fazem (`steel`, pela marcha de cada usina) |
| energia primária | o carvão, o petróleo cru e o gás que as usinas da metrópole queimam, mais a parte das famílias no resto do consumo do país pela parcela da população metropolitana (**desenho**: o jogo não guarda o consumo das famílias por região), em toneladas de carvão equivalente (NMC v7.0, pp. 63–65: petróleo cru × 1,429, Equation ENER 6; gás, 1 milhão de m³ = 1,33 mil tce, Formula ENER 9) |
| população | os pops das regiões não coloniais |
| população urbana | as profissões urbanas (`migration::is_urban`) das regiões não coloniais |

**Dependência:** a eletricidade hidráulica, nuclear e geotérmica, que o NMC soma, fica de fora,
porque o jogo não a separa da fóssil.

## Território perdido

Por país, por mil da própria população:
- a população das regiões que o país tinha em 1925 e que outro país tem ou controla hoje;
- a população da cultura principal do país nas regiões metropolitanas de um vizinho, onde ela não é
  a cultura principal do vizinho (os húngaros na Romênia; não os alemães da Alemanha como minoria da
  Suíça, nem os árabes de uma colônia francesa como minoria de um vizinho na França).

O segundo termo cobre os tratados de 1919–23 sem a fronteira de 1914, que o jogo não tem.
**Dependência:** a fronteira de 1914 por estado, com fonte, faria o termo exato.

**Não alimenta a política nesta fatia:** o indicador `lost_territory` do revisionismo segue sem
função. Ligá-lo faria o movimento crescer, e a fatia deixaria de ser neutra.

## Hostilidade por par

De 0 a 1000, termo a termo, cada termo de 0 a 1000 vezes o seu peso (por mil). Todos os pesos são
**valores de desenho**, em `relations.ron`; é o que a calibração vai mexer depois.

| Termo | Medida | Peso |
|---|---|---|
| território | o maior dos dois: a terra de um que o outro tem, por mil da população, cheia em 200 por mil (`people_full`) | +400 |
| reivindicação | uma reivindicação territorial aberta entre os dois (`claims.ron`, abaixo), inteira enquanto aberta | +400 |
| minoria | o maior dos dois: a cultura de um na metrópole do outro (até 1000 por mil da população de quem reivindica), vezes a parte da cultura que vive na metrópole de quem reivindica (`minority_by`), cheia em 200 por mil | +300 |
| regime | a parte dos grupos de leis em que os dois diferem, vezes o alcance: inteiro entre vizinhos ou entre duas potências (`regime_reach`), `reach_far` nos outros pares | +200 |
| causas | uma causa aberta (quebra de acordo) de um contra o outro | +200 |
| corrida | o maior esforço militar dos dois (gasto sobre o PIB), de 20 a 80 por mil; só entre quem já tem território, minoria, causa ou guerra | +150 |
| guerra | em guerra um com o outro | +1000 |
| comércio | o comércio entre os dois sobre o menor PIB, cheio em 50 por mil | −200 |
| acordo | um acordo em vigor entre os dois | −200 |
| inimigo comum | um terceiro com hostilidade de 400 ou mais com os dois | −150 |
| dissuasão | os dois com ogivas nucleares | −200 |

**Reivindicações** (`data/diplomacy/claims.ron`): 26 reivindicações territoriais entre governos,
codificadas por nós, cada linha com a sua fonte e o localizador (FRUS, por volume, documento e
páginas; Hansard, por data, volume e coluna; artigos de tratado; RIAA). **A lista não foi escolhida
pelo resultado:** o critério, escrito no cabeçalho do arquivo, é que um governo tenha contestado
formalmente a soberania de outro sobre um território nomeado entre 1925 e 1939 (nota, protesto ou
reserva; questão levada à Liga, à arbitragem ou à Corte; constituição ou declaração no
parlamento), aplicado região por região, inclusive às que nunca viraram disputa armada. Nenhum dado
de outro projeto é copiado: o ICOW (Issue Correlates of War) pede que os seus dados não sejam
redistribuídos, e o repositório é público. A escala é inteira enquanto a reivindicação está aberta
(**desenho**): o jogo não sabe quais estados cada reivindicação cobre, e uma escala pela população
da terra reclamada deixaria de fora o Chaco, quase sem gente e o mais violento dos anos 1930.
- **Fora pelo critério:** Alsácia-Lorena e Eupen-Malmedy (renunciadas em Locarno, outubro de
  1925), o Schleswig do Norte, Gibraltar, a linha Durand, a Hungria contra a Romênia e a Iugoslávia
  (agitação, sem nota do governo), a Macedônia, a saída da Bolívia ao mar (acesso, não território
  nomeado), Costa Rica e Panamá; fechadas antes de 1925: Åland, a Carélia Oriental e a Isla de
  Pinos.
- **Fora por falta de fonte que se pudesse abrir:** Danzig (1939), o Shatt al-Arab, Aqaba e Maan,
  o Tirol do Sul, o Burgenland, o Epiro do Norte, o Dodecaneso, as fronteiras da Albânia, a Galícia
  Oriental, a Trácia Ocidental e a Guiana Essequiba.

**Duas escolhas medidas lado a lado** (as duas estão em `relations.ron`; fica a de melhor
justificativa, não a que acerta mais pares):
- **Minoria (`minority_by`):** pela parte da cultura que vive na metrópole de quem reivindica
  (`HomeShare`, escolhida: quem abriga 0,1 % dos catalães não fala por eles, quem abriga a maioria
  dos albaneses ou dos húngaros fala) ou pela razão de população (`PopulationRatio`, que mexe duas
  vezes no tamanho, porque a ameaça já multiplica pelo poder).
- **Alcance do regime (`regime_reach`):** vizinhos ou duas potências (`Powers`, escolhida: EUA e
  URSS de 1917 em diante, sem fronteira e sem bomba) ou vizinhos ou quem tem ogivas (`Warheads`).

**Dependências:**
- **Memória de guerra:** só conta a guerra em curso, porque o jogo não guarda por par as guerras
  que acabaram. A memória de 1914–18 pede o dado dos beligerantes com fonte.
- **Corrida:** é o esforço militar, não a alta do gasto, porque não há histórico militar por país.

## Ameaça

Por país, por mil do próprio poder: a soma, sobre os outros, de hostilidade × o CINC do outro que
alcança, sobre o CINC próprio. Alcança inteiro o vizinho e quem tem ogivas; de quem não é vizinho,
300 por mil (`reach_far`, **valor de desenho**: a parte do poder que atravessa o mar ou o ar).

## Papel

O lugar no ranking do CINC, grande potência acima de 25 por mil do poder do mundo (`great_power`,
**valor de desenho**; o Correlates of War lista em 1925 Reino Unido, França, Alemanha, Itália,
Japão, EUA e URSS) e a tendência: a mudança da parcela do PIB do mundo em 10 anos, pela série anual
que a economia já guarda (`econ.history`), sem estado novo. Em ascensão ou em declínio com mudança
de 15 % ou mais (`rising`, **valor de desenho**). Rótulos: o mais forte, grande potência, em
ascensão, em declínio, pequeno.

## Conferência (medida, não meta)

Teste `relations::against_the_correlates_of_war`:
- o CINC de 1925 do jogo contra o do NMC v7.0 (os 30 mais fortes do NMC);
- os pares com disputa militarizada em algum ano de 1925 a 1939 (Dyadic MID v4.03; Maoz e Miner,
  *Dyadic MID Codebook v4.02*, 2021, p. 1) contra os pares mais hostis do jogo no mesmo período.

**Os dados do Correlates of War não estão no repositório.** Os termos do projeto ("Users agree
not to distribute the dataset to any third party without written permission of the COW director
and data host", correlatesofwar.org, "Data Sets") não deixam: o teste lê os arquivos baixados na
hora de `BF_COW_DIR` (`NMC-70-abridged.csv` do `NMCv7.zip`, `dyadic_mid_4.03.csv` do
`dyadic_mid_4.03_update.zip`) e fica fora do CI. Aqui entra só o resumo do resultado.

**Medido** (a regra ligada, 1925 a 1939, a hostilidade mais alta de cada par no período):

- **CINC de 1925, da metrópole:** os EUA são o mais forte no jogo e no NMC, e a China o segundo
  nos dois. URSS, Alemanha e Reino Unido vêm em seguida em ordem diferente do NMC, e Japão e Itália
  ficam no lugar. Os Países Baixos, o pior erro, caem de 29 por mil (com as Índias) a 14 (o NMC dá
  perto de 7): o que sobra é o gasto militar do Estado inteiro e o pessoal militar da metrópole, que
  o jogo ainda conta alto.
- **Disputas militarizadas, as duas contas** (dos 93 pares com disputa entre 1925 e 1939, quantos
  estão entre os 93 mais hostis do jogo; e, dos 93 mais hostis, quantos não tiveram disputa):

  | Medida | Pares com disputa no topo | Topo sem disputa |
  |---|---|---|
  | a primeira versão (#377) | 13 | 80 |
  | a escolhida (minoria pela parte em casa, regime pelas potências, reivindicações) | 37 | 56 |
  | a mesma com a minoria pela razão de população | 37 | 56 |
  | a escolhida **sem** o termo de reivindicação | 33 | 60 |

  O termo de reivindicação, com a lista pelo critério, acrescenta 4 pares; o resto do ganho sobre a
  primeira versão vem das correções de definição e de fórmula (o CINC da metrópole, a minoria
  pesada pela parte da cultura em casa e limitada à população de quem reivindica, o regime pelo
  alcance). Uma primeira lista de reivindicações, montada a partir das faltas da conferência, dava
  39; foi trocada pela do critério, porque escolhia os pares pelo resultado.
- **Os piores erros que restam:** Japão e URSS (9 anos de disputa; hostilidade 142), China e URSS
  (7; 144), Espanha e Itália (5; 0), Reino Unido e Itália (5; 76), Espanha e Alemanha (4; 2): as
  disputas da Guerra Civil Espanhola, da Abissínia e das fronteiras do Extremo Oriente sem
  reivindicação de governo com fonte.
- **Os falsos alarmes, os mais hostis sem disputa:** Reino Unido e Irlanda (750), China e Tibete
  (672), China e Mongólia (667), Romênia e URSS (603), Reino Unido e Guatemala (545), Chile e Peru
  (533), Alemanha e Sarre (519), Iraque e Turquia (519): reivindicações formais que não viraram
  disputa armada, como o critério pede que entrem.
- **Os dez mais hostis**, iguais nas duas formas de alcance do regime (as reivindicações e as
  minorias dominam o topo). Em 1925: URSS e Romênia, Peru e Chile, Alemanha e Sarre, Turquia e
  Iraque, França e Libéria, Polônia e Lituânia, Iugoslávia e Albânia, Equador e Peru, Honduras e
  Nicarágua, Bolívia e Paraguai. Em 1950: Reino Unido e Irlanda, China e Tibete, China e Mongólia,
  China e Japão, Reino Unido e Guatemala, Reino Unido e Argentina, Iugoslávia e Albânia, Honduras e
  Nicarágua, Afeganistão e Índia, Romênia e Bulgária.
- **Onde as duas formas de alcance diferem:** EUA e URSS dão 139 em 1925 e 95 em 1950 pelas
  potências, 25 e 0 pelas ogivas (sem jogador, ninguém faz ogivas no jogo); Alemanha e Sião dão 5 e
  0 nas duas; Alemanha e Polônia, 253 em 1925 (o corredor, até 1934).
- **Ameaça em 1935** (por mil do próprio poder): Japão 2.400 (a China reivindica a Manchúria, e a
  China tem mais poder), URSS 630, Itália 441, França 440, Reino Unido 360, China 279, Alemanha
  132, EUA 28.

**O que a medida diz à calibração (não mexido aqui):**
- falta um termo de disputa de fronteira entre vizinhos (território reclamado sem população: o
  Chaco, a fronteira entre Equador e Peru), com dado de reclamação e fonte;
- o termo de minoria pede um peso pelo tamanho do país que reclama, ou um limite em população
  absoluta, para os microestados;
- o comércio e os acordos pesam mais que o regime entre as grandes potências europeias.

**Custo:** a medida inteira do mundo, os 83 países e os 3.403 pares, leva 9,0 ms (release, com a
metrópole por região;
`relations::the_cost_of_a_measure`); não roda no tick, a interface a pede quando mostra.


## Para a interface

`relations::measure(&setup, &catalogue, &world, community_table, ano)` com o
`GameData::relations_setup()`:
- por país, `CountryMeasures`: `shares` (as seis parcelas, por mil), `cinc`, `rank`,
  `great_power`, `trend`, `role`, `lost_territory`, `threat`;
- por par, `Pair { a, b, terms, hostility }`, com os termos um a um (`Terms`), e
  `Relations::hostility(a, b)`.

Uma lente de relações (a hostilidade do país selecionado com cada outro, pintada no mapa) ou um
painel com a conta termo a termo, como a tela de interesses de `history-rails.pt-BR.md`.
