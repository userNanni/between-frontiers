# Aba Comércio do governo: a tarifa por bem e por parceiro (docs/trade.pt-BR.md, seção 13).
gov-tab-trade = Comércio
cmd-gov-tab-trade = Governo: tarifa por bem e por parceiro
    .short = Comércio
tariff-mean = Tarifa média { $rate }
tariff-mean-tip = A tarifa média é a média simples dos bens à venda neste ano, cada bem com o mesmo peso, antes dos acordos.
tariff-revenue = Receita aduaneira { $amount }
tariff-revenue-would = Receita aduaneira que haveria { $amount }
tariff-off = A tarifa que conta está desligada nesta partida: a tabela pesa só na escolha entre fornecedores, e nada vai ao Tesouro.
tariff-law = Lei: { $law }
tariff-basis = Cada bem paga a regra histórica (a lei de comércio em vigor, a partir da tarifa de 1925) ou a tarifa que o governo fixou. Os pontos de um parceiro somam-se à tarifa de cada bem, e os acordos em vigor limitam o resultado. A IA segue a regra histórica.
tariff-col-good = Bem
tariff-col-rule = Regra
tariff-col-rate = Tarifa
tariff-col-imports = Importação/ano
tariff-imports-tip = Por bem: a importação líquida da semana passada (o que o país comprou de fora menos o que vendeu), × 52, a preços de base. Um bem que o país exporta mais do que importa mostra um traço. Por parceiro: o que se comprou dele no ano.
tariff-col-revenue = Receita/ano
tariff-revenue-col-tip = A tarifa do bem sobre a importação líquida da coluna ao lado: o que ela cobraria num ano. A receita do país, no topo, conta a importação de todos os parceiros, a troca de variedades incluída, e por isso é maior que a soma desta coluna.
tariff-range = { $low } a { $high }
tariff-fixed-count = { $n } de { $of } { $n ->
    [one] fixado
   *[other] fixados
}
tariff-group-apply = Aplicar ao grupo:
tariff-apply = Aplicar
tariff-back-to-rule = Voltar à regra
tariff-rate-limits = De 0 % a 200 %, de um em um ponto percentual.
tariff-partners = Por parceiro
tariff-partners-basis = Os { $n } maiores vendedores ao país em { $year }, a preços de base, mais os que têm ajuste e os escolhidos pelo nome.
tariff-partners-basis-so-far = Os { $n } maiores vendedores ao país em { $year } até agora (o primeiro ano ainda não fechou), mais os que têm ajuste e os escolhidos pelo nome.
tariff-partners-none = Ainda sem compras de outro país; escolha um pelo nome para ajustar a tarifa.
tariff-col-partner = Parceiro
tariff-col-agreement = Acordo em vigor
tariff-col-points = Ajuste
tariff-col-effective = Tarifa efetiva
tariff-agreement-none = nenhum
tariff-points = { $sign }{ $n } { $n ->
    [one] ponto
   *[other] pontos
}
tariff-points-none = sem ajuste
tariff-points-tip = Pontos sobre a tarifa de cada bem para este parceiro: positivos, uma sobretaxa; negativos, uma preferência. De −100 a +100 pontos. A tarifa efetiva é a média dos bens, já dentro dos acordos em vigor.
tariff-limit-cap = O acordo limita a tarifa a { $rate }: a sobretaxa acima disso não vale.
tariff-limit-union = A união aduaneira não cobra tarifa deste parceiro: a sobretaxa não vale.
tariff-breach-cap = Passar de { $rate } com { $partner } ({ $points }) quebra a cláusula de tarifa do acordo em vigor: a sua reputação geral de honrar acordos cai, { $partner } ganha uma causa contra você e a cláusula deixa de valer entre os dois. Um ponto de restauração é gravado antes.
tariff-breach-union = Cobrar tarifa de { $partner } ({ $points }) quebra a união aduaneira do acordo em vigor: a sua reputação geral de honrar acordos cai, { $partner } ganha uma causa contra você e a união deixa de valer entre os dois. Um ponto de restauração é gravado antes.
tariff-breach-confirm = Quebrar a cláusula
tariff-breach-cancel = Cancelar

## Embargo (decisão CT6), nas Agendas
embargo-open = Embargo comercial ({ $n } em vigor)
embargo-title = Embargo comercial
embargo-basis = Uma decisão rara: o embargo fecha todo o comércio com um país, nos dois sentidos, enquanto durar. Escolha o país pelo nome; a decisão pede confirmação.
embargo-impose = Impor embargo
embargo-on = Embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country }
embargo-explain = Embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country }: o comércio entre os dois fecha nos dois sentidos a partir da próxima semana.
embargo-trade = No ano passado o país vendeu { $sold } { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country } e comprou { $bought } { $article ->
    [o] dele
    [a] dela
    [os] deles
    [as] delas
   *[other] de lá
}.
embargo-trade-so-far = Neste ano, até agora, o país vendeu { $sold } { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country } e comprou { $bought } { $article ->
    [o] dele
    [a] dela
    [os] deles
    [as] delas
   *[other] de lá
}.
embargo-lifted-slowly = Levantado, o comércio volta devagar (cerca de seis meses para refazer a rede).
embargo-restore-point = Um ponto de restauração é gravado antes.
embargo-already = O país já mantém embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country }.
embargo-yes = Impor o embargo
embargo-no = Cancelar
embargo-sent = Ordem enviada: o embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country } vale a partir da próxima semana.
embargo-lift-sent = Ordem enviada: o embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country } cai na próxima semana.
embargo-in-force = Embargos em vigor ({ $n })
embargo-none = Nenhum embargo em vigor, nem do país nem contra ele.
embargo-by-us = Nosso embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country }
embargo-on-us = Embargo { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $country } contra nós
embargo-on-us-note = Só quem impôs pode levantar; o comércio entre os dois segue fechado.
embargo-lift = Levantar…
embargo-lift-explain = Levantar o embargo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country }: o comércio volta devagar, cerca de seis meses para refazer a rede.
embargo-lift-yes = Levantar
embargo-lift-no = Manter
