# Aba Comércio do governo: a tarifa por bem e por parceiro (docs/trade.pt-BR.md, seção 13).
gov-tab-trade = Comércio
cmd-gov-tab-trade = Governo: tarifa por bem e por parceiro
    .short = Comércio
tariff-mean = Tarifa média { $rate }
tariff-mean-tip = A tarifa média é a média simples dos bens à venda neste ano, cada bem com o mesmo peso, antes dos acordos.
tariff-revenue = Receita aduaneira { $amount }
tariff-law = Lei: { $law }
tariff-basis = Cada bem paga a regra histórica (a lei de comércio em vigor, a partir da tarifa de 1925) ou a tarifa que o governo fixou. Os pontos de um parceiro somam-se à tarifa de cada bem, e os acordos em vigor limitam o resultado. A IA segue a regra histórica.
tariff-col-good = Bem
tariff-col-rule = Regra
tariff-col-rate = Tarifa
tariff-col-imports = Importação/ano
tariff-imports-tip = Por bem: a importação líquida da semana passada (o que o país comprou de fora menos o que vendeu), × 52, a preços de base. Um bem que o país exporta mais do que importa mostra um traço. Por parceiro: o que se comprou dele no ano.
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
