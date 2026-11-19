# Forças armadas (fase 5, fatia A).

gov-tab-forces = Forças
forces-none = Sem forças armadas carregadas.
forces-basis = Efetivo, geração, equipamento e treino de cada tipo, lidos uma vez por mês. O equipamento repõe-se aos poucos com o que o Estado compra; com sobra no orçamento militar, parte do efetivo passa à geração seguinte a cada semana e o resto do orçamento compra equipamento para a reserva de guerra, em depósito, de onde as formações sem equipamento tiram primeiro (reservistas chamados, geração nova). O treino sobe semana a semana.
forces-summary = Resumo
forces-men = Homens em serviço
forces-reserve = Reserva treinada
forces-share = Tamanho em paz: parcela dos homens em idade de serviço (fixada em janeiro)
forces-goods = Bens das forças
forces-goods-value = { $week } por semana ({ $gdp } do PIB por ano)
forces-peace = Em paz: a reserva fica em casa.
forces-mobilizing = Mobilizando: { $reported } de { $total } reservistas apresentados.
forces-units = Tipos
forces-col-unit = Tipo
forces-depot = Reserva de guerra em depósito
forces-col-generation = Geração
forces-col-domain = Domínio
forces-col-size = Elementos
forces-col-men = Homens
forces-col-equipment = Equipamento
forces-col-training = Treino
forces-called-up = { $unit } (reservistas)
forces-decision-hint = Mobilizar é decisão do diário, em Agendas, com confirmação. Dissolver tira elementos inteiros de um tipo: escolha o número e confirme.
forces-disband = Dissolver…
forces-disband-title = Dissolver: { $unit }
forces-disband-count = Elementos
forces-disband-count-tip = Elementos a dissolver: 1.000 homens em terra, um navio, um avião, uma arma
forces-disband-reserve = Homens que vão para a reserva
forces-disband-home = Homens que voltam para casa
forces-disband-depot = Volta ao depósito
forces-disband-good = { $good } { $value }
forces-disband-goods-more = { $goods } e mais { $count ->
    [one] { $n } bem
   *[other] { $n } bens
}
forces-disband-no-goods = Sem equipamento: nada volta ao depósito.
forces-disband-note-peace = O exército de paz volta ao tamanho de paz em janeiro: dissolver agora alivia o orçamento até lá. Para ficar menor de vez, mude a lei do serviço militar.
forces-disband-note-raised = Os homens encomendados vão para a reserva, com o treino que tinham; o equipamento fica no depósito para a próxima encomenda.
forces-disband-note-service = Saem de serviço e não voltam: o que se aproveita deles vai para o depósito. Uma nova encomenda leva meses.
forces-disband-waiting = Dissolução enviada; a tabela se atualiza na próxima semana.
forces-disband-yes = Dissolver
forces-disband-no = Manter
forces-domain-land = Terra
forces-domain-sea = Mar
forces-domain-air = Ar
forces-domain-strategic = Estratégico

journal-group-forces = Forças armadas
journal-forces-title = Forças armadas em paz
journal-forces-mobilizing-title = Mobilização em curso
journal-forces-cause = O exército de paz acompanha os homens em idade de serviço e a lei de conscrição. Mobilizar chama a reserva, que se apresenta ao longo de { $weeks } semanas e sai do trabalho.
journal-force-in-service = Homens em serviço
journal-force-reserve = Reserva ainda em casa
journal-force-reported = Reservistas apresentados
journal-forces-progress = { $reported } de { $total } apresentados
journal-open-forces = Abrir as forças
journal-decision = Decisão
journal-mobilize = Mobilizar a reserva…
journal-mobilize-confirm = Confirmar a mobilização
journal-mobilize-explain = A reserva deixa o trabalho e se apresenta aos poucos; a produção sente a falta. Um ponto de restauração é gravado antes.
journal-demobilize = Mandar a reserva para casa…
journal-demobilize-confirm = Confirmar a desmobilização
journal-demobilize-explain = Os reservistas voltam para casa e para o trabalho.
journal-decision-cancel = Cancelar

unit-irregular = Milícia e guerrilha
unit-garrison = Guarnição
unit-infantry = Infantaria
unit-light_infantry = Infantaria leve
unit-cavalry = Cavalaria
unit-artillery = Artilharia
unit-engineers = Engenharia
unit-anti_air = Antiaérea
unit-armor = Blindados
unit-mechanized = Motorizada e mecanizada
unit-airborne = Paraquedistas
unit-battleship = Encouraçados
unit-carrier = Porta-aviões
unit-cruiser = Cruzadores
unit-destroyer = Contratorpedeiros
unit-submarine = Submarinos
unit-escort = Escoltas
unit-coastal = Defesa costeira
unit-mine_warfare = Guerra de minas
unit-transport = Transportes
unit-fighter = Caças
unit-attack = Ataque e bombardeio tático
unit-strategic_bomber = Bombardeiros estratégicos
unit-patrol = Reconhecimento e patrulha
unit-air_transport = Transporte aéreo
unit-naval_aviation = Aviação naval
unit-helicopter = Helicópteros
unit-drone = Drones
unit-ballistic_missile = Mísseis balísticos
unit-missile_submarine = Submarinos lança-mísseis
unit-nuclear_warhead = Ogivas nucleares
unit-satellite = Satélites
unit-missile_defense = Defesa antimíssil

# Encomendas de forças, na aba Forças.
orders-title = Encomendas
orders-rule = As encomendas são pagas na ordem da fila, depois de manter abastecidas as forças que já existem; quando o orçamento militar não basta, elas esperam.
orders-room = Sobra do orçamento militar nesta semana, depois de abastecer as forças: { $room }
orders-none = Nenhuma encomenda.
orders-generation = { $unit }, geração { $generation }
orders-colonial-unit = { $unit } (colonial)
orders-delivered = { $done } de { $ordered } entregues
orders-raising = { $share } dos homens já se apresentaram; o treino vem depois
orders-building = { $share } do equipamento construído; entram inteiros, no fim
orders-weeks = entregue { $weeks } semanas depois da encomenda, prevista para { $month } de { $year }
orders-waiting = { $count ->
    [one] Esperando folga no orçamento militar há { $weeks } semana
   *[other] Esperando folga no orçamento militar há { $weeks } semanas
}
orders-cancel = Cancelar
orders-cancel-explain = O que foi construído vai para o depósito; os homens já chamados ficam.
orders-cancel-yes = Cancelar a encomenda
orders-keep = Manter
orders-new = Nova encomenda
orders-nothing = Nada a encomendar ainda.
orders-pick = Escolha um tipo para ver quanto custa.
orders-count = Elementos
orders-count-tip = Elementos: mil homens em terra, um navio, um avião, uma arma
orders-amount-men = { $men } homens
orders-amount-ships = { $count ->
    [one] { $n } navio
   *[other] { $n } navios
}
orders-amount-aircraft = { $count ->
    [one] { $n } avião
   *[other] { $n } aviões
}
orders-amount-weapons = { $count ->
    [one] { $n } arma
   *[other] { $n } armas
}
orders-colonial = Recrutada nas colônias
orders-cost-build = Para construir, aos preços desta semana
orders-cost-yearly = Todo ano, depois de pronta
orders-cost-yearly-share = { $money } ({ $share } do orçamento militar)
orders-lead = Até entrar em serviço
orders-lead-land = { $weeks } semanas para os homens se apresentarem, depois { $training } de treino
orders-lead-build = { $weeks } semanas no estaleiro ou na linha
orders-confirm = Encomenda grande: { $unit }, { $amount }. Custa { $build } para construir e { $yearly } todo ano depois.
orders-confirm-yes = Fazer a encomenda
orders-confirm-no = Voltar
orders-place = Encomendar

orders-yearly-parts = Por ano: { $pay } de soldo e { $upkeep } de manutenção e desgaste; fica enquanto a força existir.

## A guerra em terra (fatia C): a janela da guerra
cmd-window-war = Guerra
    .short = Guerra
war-title = Guerra
war-off = A guerra em terra está desligada nesta partida.
war-none = O país não está em guerra.
war-sides = { $ours } contra { $theirs }
war-since-declared = Declarada em { $date }, há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}.
war-since-declared-on = Declarada contra nós em { $date }, há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}.
war-at-stake = Em jogo
war-support = Apoio à guerra
war-support-soon = O apoio de cada lado decide a paz e a rendição; o valor, o que o move e a revolução chegam com a próxima parte da simulação.
war-occupation = Ocupação
war-taken = Ocupados por nós:
war-lost = Nossos, ocupados:
war-states-none = nenhum
war-occupation-soon = O que o território ocupado paga, a devastação e a guarnição chegam com a próxima parte da simulação.
war-fronts = Frentes
war-fronts-basis = Uma frente por país inimigo, lida da última semana de combate: segmentos, forças, baixas e avanço.
war-front = Frente com { $country }
war-posture = Postura:
war-hold = Segurar
war-attack = Atacar
war-hold-note = Segurando: as tropas defendem e só lutam se o inimigo atacar.
war-attack-note = Atacando: as tropas avançam em todos os segmentos, com baixas dos dois lados.
war-front-none = Sem fronteira de terra com este inimigo nesta semana.
war-both = { $ours } nós · { $theirs } eles
war-men = Homens na frente
war-power = Poder de fogo
war-week-losses = Baixas na semana
war-advance = Avanço na semana
war-advance-tip = Soma, em km, do que a frente avançou ou recuou em cada segmento nesta semana; não é uma distância no mapa. Cada segmento avança até a profundidade do estado vizinho, e aí o estado muda de mãos.
war-enemy-losses = Baixas do inimigo desde o início
war-our-losses = Nossas baixas em terra desde o início
war-km-gained = ganhamos { $km } km
war-km-lost = perdemos { $km } km
war-km-none = parada
war-segments-show = Ver { $count ->
    [one] { $n } segmento
   *[other] { $n } segmentos
}
war-segments-hide = Esconder { $count ->
    [one] { $n } segmento
   *[other] { $n } segmentos
}
war-segment = { $ours } – { $theirs }
war-quiet = sem combate
war-segment-line = Poder { $ours } nós, { $theirs } eles · baixas { $lost } nós, { $killed } eles · { $advance }
war-terms-ours = Nossos termos:
war-terms-theirs = Termos deles:
war-term = { $term }: { $value }
war-term-number = Número (milhares de homens)
war-term-equipment = Equipamento
war-term-training = Treino
war-term-supply = Suprimento
war-term-terrain = Terreno
war-term-air = Ar
war-into-theirs = Avançamos { $share } do estado deles; em 100 % ele passa a nós.
war-into-ours = Eles avançaram { $share } do nosso estado; em 100 % ele passa a eles.
war-segment-density = Frente de { $width } km · { $ours } homens por km nós, { $theirs } eles · linha cheia: { $full } por km
war-segment-density-tip = Quanto mais denso o defensor, mais devagar o atacante avança: diante de uma linha cheia, na velocidade de base; diante de uma linha cinco vezes mais densa, 25 vezes mais devagar; diante de uma linha rala, até 1,75 vez mais rápido. A largura é a fronteira entre os dois estados.
war-segment-dug = Entrincheirado { $share }: parado há { $weeks ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}; cheio em { $full } semanas, quando o terreno do defensor conta em dobro.
war-peace = Propor paz a { $country }…
war-peace-note = A paz é um acordo na mesa de negociação, nas Agendas; assinado entre os líderes, a guerra acaba e a ocupação entre eles termina.
war-support-thresholds = Na barra, o traço claro é a paz e o vermelho, o colapso. Um governo da IA abaixo de { $peace } pede a paz; abaixo de { $collapse }, entra em contagem para cair.
war-support-you = { $country } (você)
war-support-tends-down = O apoio tende a cair para { $target }, por:
war-support-tends-up = O apoio tende a subir para { $target }, por:
war-support-tends-still = O apoio tende a ficar em { $target }, por:
war-support-why-start = causa da guerra
war-support-why-casualties = mortos
war-support-why-duration = guerra longa
war-support-why-deprivation = privação em casa
war-support-why-territory-lost = terra perdida
war-support-why-territory-won = terra tomada
war-support-why-result-down = frente que recua
war-support-why-result-up = frente que avança
war-support-pace = Muda devagar: { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
} para andar metade do caminho
war-support-censored = a censura atrasa a mudança
war-support-censored-fall = a censura atrasa a queda
war-support-vote-wide = com voto amplo, o apoio reage mais rápido
war-support-vote-narrow = com voto restrito, a opinião pesa devagar
war-support-you-thresholds = Para você: abaixo de { $peace } a IA pediria a paz, mas quem decide é você; abaixo de { $collapse }, o governo cai em { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
} e o novo governo pede a paz.
war-devastation-ours = Nosso lado
war-devastation-theirs = Lado deles
war-devastation-total = { $count ->
    [one] { $civilians } civil morto
   *[other] { $civilians } civis mortos
} em { $n ->
    [one] { $regions } região
   *[other] { $regions } regiões
}
war-devastation-more = e mais { $count ->
    [one] { $n } região com perda menor
   *[other] { $n } regiões com perdas menores
}
war-support-term = { $term } { $value }
war-support-group = { $group } { $value } (peso { $clout })
war-support-speed-tip = O apoio anda para o alvo aos poucos. O voto livre faz a opinião pesar mais depressa; a censura das leis em vigor a freia. Mudar a lei de imprensa ou o voto muda essa velocidade.
war-support-countdown = Abaixo do colapso: o governo cai em { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}; até lá, ainda dá para fechar a paz nos próprios termos.
war-support-fallen-regime = O governo caiu e veio outra forma de governo ({ $form }); o novo governo pede a paz. A reputação de segurança caiu à metade.
war-support-fallen-cabinet = O gabinete caiu; o novo governo pede a paz.
war-occupied-line = { $state } — de { $by } há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}; resistência { $resistance }, guarnição { $garrison } da necessária; leva { $extraction } da produção ({ $taken } por semana; sobe até { $cap })
war-occupied-tip = A resistência sobe com o que o ocupante leva e com a guarnição curta; a guarnição é a parte mantida da tropa que o território pede. O ocupante leva uma parte da produção que cresce com as semanas até o teto, cortada quando a guarnição fica curta.
war-devastation = Devastação
war-wreck-ours = { $state } (nosso): { $share } dos edifícios destruídos, { $count ->
    [one] { $civilians } civil morto
   *[other] { $civilians } civis mortos
}
war-wreck-theirs = { $state } (deles): { $share } dos edifícios destruídos, { $count ->
    [one] { $civilians } civil morto
   *[other] { $civilians } civis mortos
}

composition-title = Composição das forças
composition-basis = Uma vez por ano, a infantaria troca de equipamento rumo à parcela-alvo sobre rodas e em lagarta: no máximo 2 % dos homens de terra por ano, pagos com a sobra do orçamento militar. O exército não cresce: os homens são os mesmos, com o treino que tinham.
composition-target-none = Sem alvo: o país ainda não faz o equipamento da infantaria.
composition-wheels-now = Sobre rodas hoje
composition-tracks-now = Em lagarta hoje
composition-target = Alvo da composição
composition-rule-line = Regra histórica: { $wheels } sobre rodas e { $tracks } em lagarta, pelo que o país já sabe fazer.
composition-fixed-note = O governo fixa as duas parcelas; a conversão anda no mesmo ritmo, só muda o destino.
composition-step-tip = Um ponto percentual por clique; as duas parcelas somam no máximo 100 %.
composition-converting = Em conversão para { $unit }
composition-converting-done = { $done } de { $of } mil homens
composition-stall-maker = Parada há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}: o país ainda não sabe fazer o equipamento deste tipo.
composition-stall-room = Parada há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}: o teto militar das compras do Estado vai todo para abastecer as forças e as encomendas; não sobra para converter.
composition-stall-infantry = Parada há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}: não há mais infantaria em casa para converter.
composition-stall-prices = Parada há { $count ->
    [one] { $weeks } semana
   *[other] { $weeks } semanas
}: a sobra do orçamento militar não paga os veículos aos preços de hoje no mercado.
composition-no-wheels = Nenhum tipo sobre rodas que o país equipe: a parte sobre rodas do alvo fica sem converter.
composition-no-tracks = Nenhum tipo em lagarta que o país equipe: a parte em lagarta do alvo fica sem converter.
composition-wheels = Sobre rodas
composition-tracks = Em lagarta

nuclear-balance = Balanço de dissuasão
nuclear-warheads = { $n } ogivas
nuclear-warheads-opaque = { $n } ogivas, como você lê um arsenal que não foi testado
nuclear-umbrella = Sob o guarda-chuva de { $who }: os arsenais deles contam deste lado
nuclear-second-strike = Depois de sofrer o primeiro golpe, ainda entrega { $delivered } ogivas; o outro tem por inaceitável { $needed }.
nuclear-secure = Segundo golpe garantido: o outro não escapa da resposta.
nuclear-not-secure = Sem segundo golpe garantido: um primeiro golpe do outro o deixa sem resposta que pese.
nuclear-first-strike = Ganha golpeando primeiro: só ele tem segundo golpe garantido.
nuclear-ceiling = Até onde pode subir contra o outro: { $rung }.
nuclear-mutual = Dissuasão mútua: a guerra direta fica aberta, mas carrega { $risk } % por semana de risco de escalada; a coerção não tem crédito.
nuclear-not-mutual = A dissuasão não é mútua: o teto de cada lado está acima.
nuclear-opacity-note = Um arsenal não testado conta { $share } % para os outros; o seu conta inteiro para você. As ogivas cabem nos lançadores (submarinos, mísseis, bombardeiros); sem lançador, não contam.
nuclear-rung-tension = tensão
nuclear-rung-rivalry = rivalidade
nuclear-rung-arms_race = corrida armamentista
nuclear-rung-crisis = crise, sem guerra
nuclear-rung-war = guerra convencional
nuclear-rung-coercion = coerção nuclear
nuclear-rung-use = uso da arma

war-expedition = Expedição a { $country }
war-expedition-by-land = A colônia é alcançável por terra: o exército chega sem desembarque e sem teto.
war-expedition-landed = Homens desembarcados
war-expedition-goal = Meta desta semana
war-expedition-cap = Teto (metade dos homens de terra)
war-expedition-rule = A meta segue a regra: homens bastantes para cobrir 20 por mil da população que você segura na unidade (O3, população hostil).
war-expedition-ordered = A meta é a sua ordem. Os homens desembarcam, ou voltam, pelos portos, no ritmo de cada semana.
war-expedition-tied = Amarrados na unidade
war-expedition-tied-value = { $men } ({ $share } % do exército de terra)
war-expedition-armed = Irregulares armados do inimigo
war-expedition-less = −10 mil
war-expedition-more = +10 mil
war-expedition-to-rule = Voltar à regra
war-expedition-note = Mais homens cobrem melhor a unidade (a guerrilha perde mais por baixa sua), mas amarram o exército e, com a G8, pesam mais na duração da guerra.
