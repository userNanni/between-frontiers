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
gold-cover-tip = Cobertura: reservas líquidas do banco central (ouro e divisas, menos a moeda do país nas reservas de fora), em moeda local ao câmbio real, sobre a moeda que ele emitiu (títulos que comprou e crédito aos bancos). Vai de −1000 % a +1000 %. A marca branca é a cobertura legal; a vermelha, o piso.
gold-below-legal = Abaixo da cobertura legal há { $weeks } { $weeks ->
    [one] semana
   *[other] semanas
}:
gold-below-floor-rule = Abaixo do piso há { $weeks } { $weeks ->
    [one] semana
   *[other] semanas
}: pela regra histórica, sai do padrão-ouro em { $left } { $left ->
    [one] semana
   *[other] semanas
}.
gold-below-floor-defend = Abaixo do piso há { $weeks } { $weeks ->
    [one] semana
   *[other] semanas
}: o governo segura a paridade com os juros da regra e só sai do ouro quando as reservas (ouro e divisas) acabarem.
gold-below-floor-devalue = Abaixo do piso há { $weeks } { $weeks ->
    [one] semana
   *[other] semanas
}: em { $left } { $left ->
    [one] semana
   *[other] semanas
} o governo move a paridade em { $share } e fica no ouro; a contagem recomeça depois.
gold-posture = Abaixo do piso
gold-posture-rule = Regra histórica: sai do ouro
gold-posture-fixed = Fixado pelo governo
gold-posture-defend = Defender
gold-posture-devalue = Desvalorizar
gold-posture-rule-means = Depois de algumas semanas abaixo do piso, o país sai do ouro e o câmbio flutua; o prêmio de risco sobe por alguns anos.
gold-posture-defend-means = Segura a paridade com os juros da regra até as reservas (ouro e divisas) acabarem: deflação e desemprego no lugar da desvalorização.
gold-posture-devalue-means = Move a paridade em { $share } e fica no ouro: quem tem a nossa moeda perde, e a cobertura volta a subir.
gold-premium = Prêmio de risco
gold-premium-tip = Pontos que uma saída do ouro deixou nos juros dos títulos; caem a zero em alguns anos.
gold-reserves = Reservas do banco central
gold-reserves-tip = Ouro e divisas, em dólares a preços-base, como no balanço de pagamentos.
gold-reserves-gold = Ouro
gold-reserves-fx = Em divisas: { $country }
gold-reserves-fx-tip = Divisas de { $country }. Perdem valor quando a moeda de { $country } cai, numa desvalorização ou flutuando: se a paridade dela subir { $share }, estas reservas perdem { $loss }.
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
gold-below-legal-rate = juros { $points } acima da regra de Taylor;
gold-below-legal-rate-fixed = os juros estão fixados pelo governo, sem o acréscimo da regra;
gold-below-legal-stop = o banco não cria moeda e, em { $left } { $left ->
    [one] semana
   *[other] semanas
}, começa a retirar moeda.
gold-below-legal-withdrawing = o banco está retirando moeda: { $share } do PIB ao ano.
gold-reserves-net = Reservas líquidas
gold-reserves-net-tip = Reservas menos a nossa moeda nas reservas de fora: é o que a cobertura conta. Negativas, a cobertura fica abaixo de zero.

# A decisão do padrão-ouro (Agendas)
gold-decision-open-join = Aderir ao padrão-ouro…
gold-decision-open-leave = Sair do padrão-ouro…
gold-decision-title = Padrão-ouro
gold-decision-now = Hoje: { $regime }, { $by }. Cada escolha mostra o que daria nesta semana; vale na semana seguinte, com um ponto de restauração antes.
gold-decision-parity = Paridade { $parity } (1925 = 100): a moeda fica { $gap } acima do valor de mercado
gold-decision-cover = Cobertura que teria { $cover } · legal { $legal } · piso { $floor } · { $margin }
gold-margin-over = folga de { $value }
gold-margin-short = faltam { $value } para a legal
gold-decision-defense = Abaixo da legal: juros { $points } acima da regra de Taylor desde a primeira semana.
gold-decision-no-prewar = Sem paridade de antes da guerra nos dados deste país: só dá para aderir no câmbio do dia.
gold-decision-confirm = Confirmar
gold-choice-prewar-title = Aderir na paridade de antes da guerra
gold-choice-market-title = Aderir no câmbio do dia
gold-choice-float-title = Sair do ouro e flutuar
gold-choice-control-title = Segurar a paridade sob controle cambial
gold-choice-rule-title = Devolver o câmbio à regra histórica
gold-choice-prewar-means = A moeda volta ao valor de antes da guerra: prestígio e crédito, mas a paridade fica acima do mercado e as exportações perdem preço.
gold-choice-market-means = O câmbio fica onde está: sem perda de competitividade, e a cobertura é a de hoje.
gold-choice-float-means = O câmbio anda até a paridade do poder de compra e os juros voltam à regra de Taylor; quem tem a nossa moeda perde se ela cair, e o prêmio de risco sobe por alguns anos.
gold-choice-control-means = A paridade fica atrás do controle cambial: sem a regra da cobertura, e o que o país não consegue pagar fica bloqueado lá fora.
gold-choice-rule-means = A regra histórica volta a decidir: sai abaixo do piso e volta ao ouro com inflação baixa e cobertura bastante.
gold-choice-prewar-confirm = Aderir na paridade de antes da guerra vale na semana que vem. Se a cobertura ficar abaixo da legal, os juros sobem e o banco para de criar moeda. Um ponto de restauração é gravado antes.
gold-choice-market-confirm = Aderir no câmbio do dia vale na semana que vem. Daí em diante, a cobertura manda nos juros. Um ponto de restauração é gravado antes.
gold-choice-float-confirm = Sair do ouro vale na semana que vem: o câmbio flutua, o prêmio de risco sobe, e quem guarda a nossa moeda perde se ela cair. Um ponto de restauração é gravado antes.
gold-choice-control-confirm = O controle cambial vale na semana que vem: o capital fica congelado, e o que o país não consegue pagar fica bloqueado lá fora. Um ponto de restauração é gravado antes.
gold-choice-rule-confirm = Devolver à regra histórica vale na semana que vem. Um ponto de restauração é gravado antes.
