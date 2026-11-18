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
