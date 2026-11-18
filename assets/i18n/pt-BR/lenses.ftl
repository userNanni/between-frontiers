# Lentes de influência e de blocos no mapa.

cmd-map-mode-influence = Mapa de influência
    .short = Influência
cmd-map-mode-blocs = Mapa de blocos
    .short = Blocos
lens-influence-none = Sem dados de influência nesta campanha.
legend-influence-focus = { $country } (em foco)
legend-influence-nothing = Nada medido
tip-influence = { $share } do PIB · maior assunto: { $issue } · { $channels }
tip-influence-focus = País em foco
tip-influence-nothing = Nenhuma influência medida
lens-blocs-none = Sem dados de tratados nesta campanha.
lens-blocs-empty = Nenhum bloco deste tipo nesta data.
legend-bloc-line = { $name } · { $count ->
    [one] { $members } membro
   *[other] { $members } membros
}
legend-blocs-member = Tom claro: membro; tom forte: líder
legend-blocs-customs = União aduaneira
bloc-name-subjects = { $leader } e seus súditos
tip-bloc-leader = { $name } (líder)
tip-bloc-none = Fora de blocos deste tipo
lens-ties = Laços
lens-ties-influence_over = Influência do país sobre os outros
lens-ties-influence_on = Influência dos outros sobre o país
lens-ties-military = Blocos militares
lens-ties-monetary = Blocos monetários
lens-ties-subjects = Súditos
legend-influence-unit-over = Influência { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $country } sobre cada país, em % do PIB do país influenciado
legend-influence-unit-on = Influência de cada país { $article ->
    [o] sobre o
    [a] sobre a
    [os] sobre os
    [as] sobre as
   *[other] sobre
} { $country }, em % do PIB { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $country }
bloc-name-military = Bloco militar { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $leader }
bloc-name-monetary = Bloco monetário { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $leader }
bloc-name-customs = Bloco aduaneiro { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $leader }
legend-bloc-leaderless = { $name } · sem líder · { $count ->
    [one] { $members } membro
   *[other] { $members } membros
}
legend-influence-scope = Medidos hoje: crédito entre governos e boa vontade; o resto está desligado.
legend-influence-scope-tip = Medidos hoje: crédito entre governos e boa vontade. Comércio e investimento estão desligados nas regras do jogo. Proteção chega com a guerra (fase 5). Um súdito aparece com zero, e a janela de influência diz por quê.
legend-influence-unmeasured = Sem canal medido
tip-influence-unmeasured = Laço sem canal medido: a janela de influência diz o que falta
lens-ties-war = Guerras: quem ocupa o quê
legend-war-none = Nenhuma guerra em curso.
legend-war-at-war = País em guerra, na sua cor
legend-war-occupied = Estado ocupado, na cor de quem ocupa
legend-war-outside = Fora das guerras
legend-war-count = { $wars } países em guerra · { $states } estados ocupados
tip-war-occupied = Ocupado por { $by } (dono: { $owner })
tip-war-at-war = Em guerra, nas mãos do dono
tip-war-outside = Fora das guerras
