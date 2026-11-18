### SPDX-License-Identifier: CC-BY-SA-4.0
### Tratados como conjuntos de cláusulas (fase 4). Chaves: clause-<chave> (nome),
### clause-<chave>-effect (o que faz, para quem dá), front-<chave> (por que a cláusula está sem
### efeito e o que a acorda), term-<chave> (termos da conta), bundle-<chave> (graus de súdito).

## Cláusulas: nome e efeito
clause-cede_states = Cessão de estados
clause-cede_states-effect = Os estados passam a quem recebe, com a gente, as fábricas e as obras; a dívida fica com quem cede.
clause-release_country = Criação de país
clause-release_country-effect = Os estados viram um país novo, soberano ou súdito, que nasce sem dívida salvo cláusula de dívida.
clause-union = União
clause-union-effect = Quem dá deixa de existir e se junta a quem recebe, com Tesouro e dívida.
clause-debt = Dívida entre governos
clause-debt-effect = Quem dá passa a dever a quem recebe, com juro e anuidade; o serviço sai do Tesouro toda semana.
clause-assume_debt = Assunção de dívida
clause-assume_debt-effect = Quem recebe assume parte da dívida de mercado de quem dá.
clause-revise_debt = Revisão de dívida
clause-revise_debt-effect = O credor muda anuidade, juro ou prazo, suspende o serviço ou perdoa parte da dívida.
clause-payment = Pagamento
clause-payment-effect = Um pagamento único do Tesouro de quem dá ao de quem recebe.
clause-techniques = Técnicas
clause-techniques-effect = Técnicas, gente e projetos passam de quem dá a quem recebe.
clause-recognition = Reconhecimento
clause-recognition-effect = Quem dá reconhece quem recebe como Estado (e, com fronteira, as fronteiras de hoje); sem isso não há aliança, tarifa nem empréstimo de governo entre os dois.
clause-non_aggression = Não agressão
clause-non_aggression-effect = Quem dá não abre crise nem guerra contra quem recebe.
clause-defence = Defesa
clause-defence-effect = Quem dá entra na guerra se quem recebe for atacado.
clause-consultation = Consulta
clause-consultation-effect = Quem dá consulta quem recebe antes de agir na região.
clause-border_guarantee = Garantia de fronteira
clause-border_guarantee-effect = Quem dá defende a fronteira de quem recebe contra qualquer um.
clause-demilitarized = Zona desmilitarizada
clause-demilitarized-effect = Quem dá não mantém tropa nem fortificação nos estados.
clause-arms_limit = Limite de armas
clause-arms_limit-effect = Quem dá mantém exército e frota abaixo dos tetos.
clause-base = Base
clause-base-effect = Quem recebe mantém base e guarnição no território de quem dá.
clause-passage = Passagem
clause-passage-effect = Quem dá deixa passar as forças de quem recebe.
clause-occupation = Ocupação
clause-occupation-effect = Quem recebe ocupa estados de quem dá como garantia, até o prazo ou a condição.
clause-tariff_cap = Teto de tarifa
clause-tariff_cap-effect = A tarifa de quem dá sobre os bens de quem recebe não passa do teto.
clause-tariff_preference = Preferência tarifária
clause-tariff_preference-effect = Quem dá cobra de quem recebe uma tarifa menor que dos outros.
clause-customs_union = União aduaneira
clause-customs_union-effect = Quem dá aplica a tarifa de quem recebe e entra no território aduaneiro dele.
clause-currency_peg = Moeda atrelada
clause-currency_peg-effect = A moeda de quem dá segue a de quem recebe, com depósito de reservas.
clause-monetary_union = União monetária
clause-monetary_union-effect = As partes usam uma moeda comum.
clause-capital_protection = Proteção do capital
clause-capital_protection-effect = Quem dá não toma sem pagar o capital de quem recebe.
clause-financial_control = Controle financeiro
clause-financial_control-effect = Funcionários de quem recebe arrecadam e aplicam parte da receita de quem dá.
clause-tribute = Tributo
clause-tribute-effect = Quem dá paga todo ano uma quantia a quem recebe.
clause-foreign_policy = Política externa
clause-foreign_policy-effect = Quem dá não assina tratado sem o consentimento de quem recebe.
clause-intervention = Direito de intervenção
clause-intervention-effect = Quem recebe pode impor lei ou governo a quem dá.
clause-minorities = Proteção de minorias
clause-minorities-effect = Quem dá não aplica lei dirigida contra a comunidade protegida.
clause-troops_quota = Contingente
clause-troops_quota-effect = Quem dá põe tropas à disposição de quem recebe.
clause-offence = Aliança ofensiva
clause-offence-effect = Quem dá entra na guerra que quem recebe começar, contra as partes nomeadas.
clause-neutrality = Neutralidade
clause-neutrality-effect = Quem dá não ajuda quem estiver em guerra com quem recebe.

## Frentes: por que a cláusula está sem efeito e o que a acorda
front-borders = Sem efeito até as operações de fronteira entrarem no jogo (fronteiras, parte B).
front-ledger = Sem efeito até o livro de dívidas entre governos entrar no jogo (tratados, etapa 2).
front-recognition = Sem efeito até o reconhecimento entre Estados entrar no jogo (tratados, etapa 2).
front-trade = Sem efeito até a tarifa entrar no comércio mundial (fase 4).
front-money = Sem efeito até o regime monetário mover o câmbio (fase 4).
front-war = Sem efeito até forças, crises e guerra entrarem no jogo (fase 5).
front-communities = Sem efeito até as comunidades entrarem na população.
front-foreign_capital = Sem efeito até o capital estrangeiro entrar no jogo (fase 4).
front-research = Sem efeito até a transferência de técnicas entrar na pesquisa.
front-politics = Sem efeito até a política entrar no jogo.

## Termos da conta de cada lado
term-money = Dinheiro (valor presente)
term-trade = Comércio
term-territory = Território
term-security = Segurança
term-sovereignty = Soberania
term-politics = Política interna
term-trust = Confiança na outra parte

## Graus de súdito
bundle-crown_colony = Colônia da coroa
bundle-protectorate = Protetorado
bundle-mandate = Mandato
bundle-dominion = Domínio
bundle-satellite = Satélite

## A mesa de negociação (Agendas): propor, responder, contrapropor
table-title = Propor acordo…
table-propose-to = Propor acordo { $article ->
    [o] ao
    [a] à
    [os] aos
    [as] às
   *[other] a
} { $country }…
table-counter-title = Contraproposta
table-off = A mesa de negociação está desligada nesta partida.
table-pick-country = Selecione no mapa o país a quem propor.
table-parties = { $me } e { $other }
table-gives = { $country } dá
table-gives-nothing = Nada por enquanto.
table-add = Acrescentar cláusula…
table-once = De uma vez, na assinatura
table-in-force = Enquanto o acordo vigorar
table-remove = Tirar
table-param-amount = Valor
table-param-yearly = Por ano
table-param-principal = Principal
table-param-rate = Juro
table-param-years = Prazo da dívida
table-param-deferrable = Parte adiável
table-param-share = Parcela
table-param-margin = Margem
table-per-year = { $money } por ano
table-annuity = Anuidade: { $money }
table-recognition-border = Com as fronteiras de hoje
table-out-of-limits = { $param } fora do limite do catálogo.
table-unrecognised = As partes não se reconhecem: esta cláusula não pode ser assinada sem reconhecimento mútuo.
table-clause-value = Vale { $me } para você, { $other } para a outra parte.
table-clause-mine = Vale { $value } para você.
table-states = { $count ->
    [one] { $n } estado
   *[other] { $n } estados
}
table-states-pick = Escolher estados…
table-term = Prazo do acordo
table-term-none = Sem prazo
table-term-tip = Quanto tempo o acordo vigora
table-years = { $n } anos
table-notice = Aviso para denunciar
table-notice-tip = Anos de aviso para sair do acordo sem quebra
table-notice-some = Com aviso, cada lado pode sair no fim do aviso, sem quebra.
table-notice-none = Sem aviso, sair do acordo antes do prazo é quebra.
table-account = Conta para { $country }:
table-account-empty = Nada a pesar ainda.
table-likely-accept = Resposta provável: { $country } aceita.
table-likely-counter = Resposta provável: { $country } faz uma contraproposta, com { $count ->
    [one] { $n } cláusula
   *[other] { $n } cláusulas
}.
table-likely-refuse = Resposta provável: { $country } recusa; pesou { $term }.
table-term-unknown = o conjunto
table-blocked = Uma proposta igual foi recusada em { $refused }; ela só volta à mesa em { $back }.
table-deadline = { $country } tem { $weeks } semanas para responder; o governo responde na segunda-feira seguinte.
table-refusal-empty = Acrescente uma cláusula para propor.
table-refusal-recognition = Tire as cláusulas marcadas: sem reconhecimento mútuo, só cabem reconhecimento e trocas de uma vez.
table-refusal-limits = Ajuste os parâmetros marcados: estão fora dos limites do catálogo.
table-refusal-cooldown = Mude a proposta: a igual está bloqueada.
table-weighing = Fazendo a conta…
table-propose = Propor
table-send-counter = Enviar a contraproposta
table-counter-drop = Desistir da contraproposta
table-received-title = Proposta recebida: { $country }
table-round = Rodada { $round } de 3.
table-answer-by = Responda até { $date }; sem resposta, a proposta vence e fica recusada.
table-accept = Aceitar
table-refuse = Recusar
table-counter = Contrapropor…
table-accept-confirm = Aceitar assina o acordo: as cláusulas de uma vez agem na assinatura, as outras valem enquanto o acordo vigorar.
table-refuse-confirm = Recusar fecha a proposta; uma igual só volta à mesa daqui a 6 meses.
table-accept-yes = Assinar o acordo
table-refuse-yes = Recusar a proposta
table-answer-back = Voltar
table-last-round = Terceira rodada: não cabe mais contraproposta.
table-received = { $country } propõe um acordo ({ $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
})
table-section = Mesa de negociação
table-outgoing = Proposta a { $country } em { $date }: aguardando resposta.
table-closed-signed = Acordo com { $country } assinado em { $date }.
table-closed-refused-them = { $country } recusou em { $date }; pesou { $term }.
table-closed-refused-me = Você recusou a proposta recebida ({ $country }) em { $date }.
table-closed-countered = Contraproposta na rodada com { $country } em { $date }.
table-closed-lapsed = A proposta com { $country } venceu em { $date }.
table-closed-failed = A proposta com { $country } não pôde ser cumprida ({ $date }).
table-revise-of = Dívida: { $treaty }
table-annuity-label = Anuidade
table-param-grace_years = Carência
table-renegotiate-title = Renegociar a dívida
table-peace-title = Propor a paz
