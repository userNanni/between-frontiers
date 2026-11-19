# Crise e declaração (fase 5, fatia B), nas Agendas.

crisis-section = Crises e guerras
crisis-list = { $by } exige { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $target } · { $stage }
crisis-title = { $by } exige { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $target }
crisis-stage-answer = cabe a nós responder
crisis-stage-waiting = esperando a resposta
crisis-stage-decide = recusada: cabe a nós declarar ou recuar
crisis-stage-refused = recusamos: o outro lado decide
crisis-stage-accepted = aceita: vira tratado na semana que vem
crisis-deadline = { $count ->
    [one] Falta { $left } semana; sem resposta, conta como recusa
   *[other] Faltam { $left } semanas; sem resposta, conta como recusa
}
crisis-takes-union = O país inteiro: { $country } deixa de existir
crisis-takes-states = Estados: { $list }
crisis-takes-money = Dinheiro: { $money }
crisis-takes-nothing = Nenhuma cláusula ainda
crisis-no-cause = Sem motivo reconhecido: a exigência fica registrada como sem casus belli.
crisis-weighing = Como o alvo pesa
crisis-force = Força em terra, de quem exige sobre a do alvo
crisis-force-value = { $ratio } ({ $by } contra { $target } homens)
crisis-share = Parcela do país pedida
crisis-needed = Força que faria ceder
crisis-needed-never = nunca: é existencial
crisis-yields = Resposta esperada: cede
crisis-refuses = Resposta esperada: recusa
crisis-third-parties = Quem entraria e por qual cláusula
crisis-accept = Aceitar…
crisis-refuse = Recusar…
crisis-accept-confirm = Aceitar executa as cláusulas como tratado na semana que vem. Antes, fica salvo um ponto de restauração.
crisis-refuse-confirm = Recusar devolve a decisão a quem exigiu: declarar guerra ou recuar. Antes, fica salvo um ponto de restauração.
crisis-declare = Declarar guerra…
crisis-back-down = Recuar…
crisis-declare-confirm = A guerra não se desfaz. Antes, fica salvo um ponto de restauração.
crisis-back-down-confirm = Recuar fecha a crise sem nada. Antes, fica salvo um ponto de restauração.
crisis-confirm = Confirmar
crisis-cancel = Voltar
crisis-war-line = Em guerra: { $a } contra { $b }, desde { $year }
crisis-blocked-war = Já estamos em guerra com este país.
crisis-blocked-open = Já há uma crise aberta com este país.
crisis-demand-title = Exigir de outro país
crisis-demand-from = Exigir { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $country }…
crisis-demand-pick-target = Selecione no mapa o país de quem exigir, ou escolha pelo nome.
crisis-demand-of = { $article ->
    [o] Do
    [a] Da
    [os] Dos
    [as] Das
   *[other] De
} { $country }
crisis-demand-clauses = O que exigir
crisis-demand-union = O país inteiro (anexação)
crisis-demand-states = Estados a ceder
crisis-demand-payment = Pagamento único, % do PIB do alvo
crisis-demand-tribute = Tributo por ano, % do PIB do alvo
crisis-demand-of-gdp = Em porcentagem do PIB do ano passado do país que recebe a exigência
crisis-demand-deadline = Prazo para responder: { $weeks } semanas
crisis-demand-review = Revisar a exigência
crisis-demand-confirm = Enviar a exigência a { $country }?
crisis-demand-final = Recusada, cabe a nós declarar guerra ou recuar. Antes de enviar, fica salvo um ponto de restauração.
crisis-demand-send = Enviar a exigência
crisis-share-all = o país inteiro
crisis-third-parties-off = Sem acordos em vigor nesta campanha: ninguém é chamado (os acordos estão desligados nas regras).
crisis-third-parties-none = Nenhum acordo em vigor chama terceiros nesta crise.
crisis-agreement = { $name }, { $year }
crisis-duty-defend = { $party } — defesa ({ $agreement }) — { $article ->
    [o] com o
    [a] com a
    [os] com os
    [as] com as
   *[other] com
} { $partner }
crisis-duty-attack = { $party } — ofensiva ({ $agreement }) — { $article ->
    [o] com o
    [a] com a
    [os] com os
    [as] com as
   *[other] com
} { $partner }
crisis-duty-neutral = { $party } — neutralidade ({ $agreement }): não entra contra { $partner }
crisis-duty-not-attack = Declarar quebra a não agressão { $article ->
    [o] com o
    [a] com a
    [os] com os
    [as] com as
   *[other] com
} { $partner } ({ $agreement })
crisis-duty-suspended = suspensa: o parceiro quebrou o acordo antes
crisis-duty-barred = barrada: uma neutralidade impede
call-title = { $ally } chama: { $enemy } o atacou
call-by = Pelo acordo: { $agreement }
call-deadline = { $count ->
    [one] Falta { $left } semana; sem resposta, conta como honrar
   *[other] Faltam { $left } semanas; sem resposta, conta como honrar
}
call-honour-means = Honrar
call-honour-what = entrar na guerra: reserva chamada e custo semanal
call-refuse-means = Não honrar
call-refuse-what = a quebra fica registrada, e custa { $price } (um ano das forças)
call-honour = Honrar…
call-refuse = Não honrar…
call-honour-confirm = Honrar entra na guerra e não se desfaz. Antes, fica salvo um ponto de restauração.
call-refuse-confirm = Não honrar grava a quebra do acordo. Antes, fica salvo um ponto de restauração.

crisis-coerced = Quem exige pode coagir: tem segundo golpe garantido e o alvo não. No periférico, a vantagem nuclear dele soma à força; o existencial o alvo não cede nunca.
crisis-nuclear-term = Vantagem nuclear somada à força
crisis-deterred = O alvo tem segundo golpe garantido e quem exige não: a vantagem nuclear do alvo é descontada da força de quem exige.
crisis-declare-blocked = A declaração está bloqueada: o alvo (ou quem o protege por cláusula de defesa) tem segundo golpe garantido, e você não. Contra ele, o teto é a crise; só resta recuar.
