# Padrão-ouro: câmbio, reservas e capital (aba Moeda e dívida)
points-reason-exchange = Antes de mudar o regime de câmbio ou o controle de capital
gold-points = { $value } { $n ->
    [one] ponto percentual
   *[other] pontos percentuais
}
gold-title = Câmbio e reservas
gold-basis = O banco central segura a paridade enquanto as reservas cobrem a moeda que emitiu. Mudar de regime é decisão das Agendas.
gold-regime = Regime de câmbio
gold-regime-float = Câmbio flutuante
gold-regime-gold = Padrão-ouro
gold-regime-control = Paridade sob controle cambial
gold-regime-float-tip = O câmbio anda até a paridade do poder de compra; os juros seguem a regra de Taylor.
gold-regime-gold-tip = O câmbio fica na paridade; a cobertura das reservas manda nos juros: abaixo da cobertura legal, os juros sobem acima da regra de Taylor e o banco não cria moeda.
gold-regime-control-tip = A paridade fica, mas atrás do controle cambial: sem a regra da cobertura e sem saída por ela; o que o país não consegue pagar fica bloqueado lá fora na moeda dele.
gold-by-rule = pela regra histórica
gold-by-government = fixado pelo governo
gold-pending = Pedido ao governo: { $choice }. Vale na próxima semana.
gold-parity = Paridade
gold-parity-value = { $parity } (câmbio de hoje { $today })
gold-parity-tip = Índice do câmbio, 1925 = 100. A paridade é o câmbio que o banco central segura.
gold-cover-line = Cobertura { $cover } · legal { $legal } · piso { $floor }
gold-cover-tip = Cobertura: reservas líquidas do banco central (ouro e divisas, menos a moeda do país nas reservas de fora) sobre a moeda que ele emitiu. A marca branca é a cobertura legal; a vermelha, o piso.
gold-below-legal = Abaixo da cobertura legal há { $weeks } { $weeks ->
    [one] semana
   *[other] semanas
}: juros { $points } acima da regra de Taylor, e o banco não cria moeda. Em { $left } { $left ->
    [one] semana
   *[other] semanas
} começa a retirar moeda.
gold-below-floor-rule = Abaixo do piso há { $weeks } { $weeks ->
    [one] semana
   *[other] semanas
}: pela regra histórica, sai do padrão-ouro em { $left } { $left ->
    [one] semana
   *[other] semanas
}.
gold-below-floor-defend = Abaixo do piso: o governo defende a paridade com os juros até as reservas acabarem.
gold-below-floor-devalue = Abaixo do piso: o governo desvaloriza a paridade em { $share } em vez de sair.
gold-posture = Abaixo do piso
gold-posture-rule = Regra histórica: sai do ouro
gold-posture-fixed = Fixado pelo governo
gold-posture-defend = Defender
gold-posture-devalue = Desvalorizar
gold-posture-rule-means = Depois de algumas semanas abaixo do piso, o país sai do ouro e o câmbio flutua; o prêmio de risco sobe por alguns anos.
gold-posture-defend-means = Segura a paridade com juros acima do piso até as reservas acabarem: deflação e desemprego no lugar da desvalorização.
gold-posture-devalue-means = Move a paridade em { $share } e fica no ouro: quem tem a nossa moeda perde, e a cobertura volta a subir.
gold-premium = Prêmio de risco
gold-premium-tip = Pontos que uma saída do ouro deixou nos juros dos títulos; caem a zero em alguns anos.
gold-reserves = Reservas do banco central
gold-reserves-tip = Ouro e divisas, em dólares a preços-base, como no balanço de pagamentos.
gold-reserves-gold = Ouro
gold-reserves-fx = Em divisas: { $country }
gold-reserves-fx-tip = Divisas de { $country }. Se a moeda de { $country } desvalorizar { $share }, estas reservas perdem { $loss }.
gold-owed = Nossa moeda nas reservas de fora
gold-owed-tip = O que os bancos centrais de fora guardam na nossa moeda: sai das nossas reservas líquidas e perde valor com ela.
gold-capital = Capital privado
gold-capital-tip = O capital segue o retorno esperado: os juros sem o prêmio, menos a inflação e o risco do câmbio. O de curto prazo foge mais depressa.
gold-controls = Controle de capital
gold-controls-on = ligado
gold-controls-off = desligado
gold-controls-tip = Com controle de capital, o país não empresta lá fora nem recebe capital novo, e as posições ficam congeladas; os juros continuam pagos.
gold-controls-ask-on = Ligar o controle de capital…
gold-controls-ask-off = Desligar o controle de capital…
gold-controls-on-means = Ligar congela todas as posições, as nossas lá fora e as de fora aqui, e corta capital novo nos dois sentidos; o prêmio de risco sobe { $points }. Um ponto de restauração é gravado antes, se os Ajustes pedirem.
gold-controls-off-means = Desligar solta as posições: o capital volta a seguir os juros, e o de curto prazo pode sair depressa se o retorno daqui for baixo. Um ponto de restauração é gravado antes, se os Ajustes pedirem.
gold-controls-turn-on = Ligar o controle
gold-controls-turn-off = Desligar o controle
gold-choice-prewar = aderir ao ouro na paridade de antes da guerra
gold-choice-market = aderir ao ouro no câmbio do dia
gold-choice-float = sair do ouro e flutuar
gold-choice-control = segurar a paridade sob controle cambial
gold-reserves-rest = Em { $n } outras moedas
gold-capital-abroad-long = Nossos poupadores lá fora, longo prazo
gold-capital-abroad-short = Nossos poupadores lá fora, curto prazo
gold-capital-here-long = De fora aplicado aqui, longo prazo
gold-capital-here-short = De fora aplicado aqui, curto prazo
