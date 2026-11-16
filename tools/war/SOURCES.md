# Forças de 1925: fontes

`forces_1925.py` gera `data/scenario/1925/forces.ron`. Cada linha do arquivo gerado traz a fonte;
este arquivo diz o que cada fonte é, a licença e como foi lida.

| Abreviação | Fonte | Licença | Uso |
|---|---|---|---|
| `LoN Armaments Year-Book 1924` | Liga das Nações, *Armaments Year-Book: General and Statistical Information*, 1º ano (Genebra, set. 1924, doc. A.37.1924.IX, 2ª ed.). Scan da National Library of Scotland: <https://deriv.nls.uk/dcn23/1939/6472/193964723.23.pdf> | domínio público (documento oficial da Liga, 1924) | efetivos do exército, aviões, navios por classe e tonelagem; os números são em geral de 1923 ou do orçamento de 1924; lidos nas imagens das páginas, porque o OCR do PDF tem erros |
| `Hansard ... 'European Armies'` | Câmara dos Comuns, resposta escrita de 18/06/1924, vol. 174 cc2150-2W: tabela do War Office com as forças de paz de 1924. <https://api.parliament.uk/historic-hansard/written-answers/1924/jun/18/european-armies> | Open Parliament Licence | exército de quem o Year-Book não totaliza (TUR, POR, LTU, NOR, CZE, YUG, NLD, AUT, HUN) |
| `Hansard ... 'Air Forces (International Comparisons)'` | Câmara dos Comuns, resposta escrita de 15/04/1930, vol. 237 cc2755-6W, coluna de 1925: aviões de primeira linha. <https://api.parliament.uk/historic-hansard/written-answers/1930/apr/15/air-forces-international-comparisons> | Open Parliament Licence | aviação de FRA, USA e ITA |
| `Brassey's Naval and Shipping Annual 1925` | seção "Ships of the Lesser Navies", archive.org: <https://archive.org/details/brasseysnavalshipping1925> | domínio público nos EUA (1925) | navios de POR, SIA e URY, contados nominalmente |
| `WP` | Wikipedia em inglês (CC BY-SA 4.0), só para fatos: "Red Army" (SOV, 1/10/1924), "Reichsmarine" (lista de navios, contada em 1/1/1925), "Turkish Naval Forces" (lista de 29/10/1923), "Iraqi Army", "Mexican Army" | CC BY-SA 4.0 | onde o Year-Book não dá o número |
| Farrokh | K. Farrokh, "Organization of the Iranian Army in 1921-1941", kavehfarrokh.com | direito autoral; só o fato citado | exército persa (1926) |
| RTÉ | RTÉ News, "Army mutiny 1924" (06/03/2024) | direito autoral; só o fato citado | meta de efetivo irlandês de março de 1924 |
| Tratado de Washington | FRUS 1922, vol. 1, pp. 247-266 (Treaty Series 671), <https://www.ibiblio.org/pha/pre-war/1922/nav_lim.html> | domínio público | conferência dos navios capitais retidos (teste `war_1925.rs`) |
| Tratado de Versalhes | Wikisource, "Treaty of Versailles/Part 5", arts. 160, 181, 191 | domínio público | teto do exército, frota e proibição de submarino e aviação alemães |

## Regras de leitura

- **Data.** O Year-Book de 1925-26 não foi achado em forma legível; os números são de 1923-1924, os
  mais próximos de 1º de janeiro de 1925 que as fontes imprimem, e entram como de 1925 sem
  marcação de estimativa. Linha com número de outro ano (Pérsia 1926, México 1920) é estimativa.
- **Exército.** Só o exército: gendarmaria, polícia, milícia e reserva ficam de fora. Todo o
  exército de 1925 entra como `infantry` (a artilharia e a cavalaria orgânicas estão no
  equipamento do elemento) até uma fonte dividir por arma. Tropas coloniais pagas pelo orçamento
  colonial entram como `garrison` nas colônias (FRA, GBR, NLD). As tropas britânicas na Índia
  contam no RAJ, não no GBR.
- **Teto de tratado.** Alemanha, Hungria e Bulgária: as fontes dão o teto do tratado como efetivo;
  a linha é estimativa com esse método.
- **Marinha.** Navios "na frota", ativos e na reserva juntos, como o Year-Book conta. Torpedeiros
  contam como `destroyer`, porque a fonte os soma; a tonelagem total da linha guarda o peso real.
- **Aviação.** Onde a fonte separa caça, bombardeio e reconhecimento, a linha vai para `fighter`,
  `attack` e `patrol`; aviação naval japonesa vai para `naval_aviation`. Onde só há o total (FRA,
  USA, ITA, a metrópole holandesa, FIN, CHL, CAN, RAJ), o total é dividido pelas parcelas das
  aviações que a fonte divide, por maior resto; essas linhas são estimativas.

## Fora por falta de fonte aberta

Exército: SAF, AUS (só o estabelecimento de tempo parcial), CHE (milícia: 500.000 mobilizáveis,
sem efetivo permanente), EGY, PER, COL, VEN, URY, BOL, PRY, ECU, CUB, SIA, AFG, ETH. Aviação: SOV,
POL, ROM, BEL, TUR, BRA, ARG, AUS, SAF, GRE, PRS, MEX. Marinha: os destróieres alemães (a lista
consultada é incompleta) e a Marinha Real Indiana (só embarcações auxiliares).
