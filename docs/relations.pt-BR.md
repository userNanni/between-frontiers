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
"CINC Score", p. 8). No jogo:

| Componente | De onde sai |
|---|---|
| gasto militar | o teto militar do Estado (`econ::public::military_room`) |
| pessoal militar | os soldados (a profissão `soldier` nos pops) |
| aço | a oferta de `steel` no mercado |
| energia primária | o consumo de carvão, petróleo cru e gás, em toneladas de carvão equivalente (NMC v7.0, pp. 63–65: petróleo cru × 1,429, Equation ENER 6; gás, 1 milhão de m³ = 1,33 mil tce, Formula ENER 9) |
| população | os pops |
| população urbana | as profissões urbanas (`migration::is_urban`) |

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
| minoria | o maior dos dois: a cultura de um na metrópole do outro, cheia em 200 por mil | +300 |
| regime | a parte dos grupos de leis em que os dois diferem; só entre vizinhos ou com uma grande potência | +200 |
| causas | uma causa aberta (quebra de acordo) de um contra o outro | +200 |
| corrida | o maior esforço militar dos dois (gasto sobre o PIB), de 20 a 80 por mil; só entre quem já tem território, minoria, causa ou guerra | +150 |
| guerra | em guerra um com o outro | +1000 |
| comércio | o comércio entre os dois sobre o menor PIB, cheio em 50 por mil | −200 |
| acordo | um acordo em vigor entre os dois | −200 |
| inimigo comum | um terceiro com hostilidade de 400 ou mais com os dois | −150 |
| dissuasão | os dois com ogivas nucleares | −200 |

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

- **CINC de 1925:** os EUA são o mais forte no jogo e no NMC, e a China o segundo nos dois. Depois o
  jogo põe o Reino Unido e a França à frente da URSS e da Alemanha, que o NMC põe em terceiro e
  quinto; Japão e Itália ficam no lugar. O pior erro é os Países Baixos: 29 por mil no jogo, perto
  de 7 no NMC, porque o jogo junta as Índias Orientais à metrópole. URSS e Alemanha ficam abaixo pelo
  gasto e pelo pessoal militar.
- **Grandes potências pelo limiar de 25 por mil:** as sete do COW e mais a China e os Países Baixos.
- **Disputas militarizadas:** dos 93 pares com disputa entre 1925 e 1939, **13** estão entre os 93
  mais hostis do jogo.
- **Os piores erros, pares com mais anos de disputa e pouca hostilidade:** Bolívia e Paraguai (a
  Guerra do Chaco; hostilidade 28), Equador e Peru (28), e França e Turquia, Bélgica e Alemanha,
  França e Alemanha, Bulgária e Grécia, Japão e Mongólia (0). A causa comum: a disputa de fronteira
  sem minoria nem terra perdida no jogo não tem termo, e o comércio e os acordos em vigor (Locarno)
  puxam França e Alemanha a zero. China e Japão, o par com mais anos de disputa, fica em 159.
- **Os falsos alarmes, os mais hostis sem disputa:** Andorra e Espanha (478), Albânia e Iugoslávia
  (473), Afeganistão e Índia (419), Suíça e França (393), Reino Unido e Irlanda (350): o termo de
  minoria domina nos países pequenos ao lado de um vizinho grande com a mesma cultura.
- **Ameaça em 1935** (por mil do próprio poder): URSS 757, Japão 727, Itália 562, França 375,
  Reino Unido 273, China 250, Alemanha 122, EUA 29.

**O que a medida diz à calibração (não mexido aqui):**
- falta um termo de disputa de fronteira entre vizinhos (território reclamado sem população: o
  Chaco, a fronteira entre Equador e Peru), com dado de reclamação e fonte;
- o termo de minoria pede um peso pelo tamanho do país que reclama, ou um limite em população
  absoluta, para os microestados;
- o comércio e os acordos pesam mais que o regime entre as grandes potências europeias.

**Custo:** a medida inteira do mundo, os 83 países e os 3.403 pares, leva 4,0 ms (release,
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
