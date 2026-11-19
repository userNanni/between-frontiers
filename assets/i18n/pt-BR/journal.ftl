## Janela de agendas (J) e o painel lateral do mapa.

bar-journal = Agendas
journal-title = Agendas
journal-not-ready = As agendas aparecem depois do primeiro dia da campanha.
journal-empty = Nada em curso: nem fila, pesquisa, alerta fiscal ou movimento para acompanhar.
journal-pin = Fixar no painel
journal-unpin = Desafixar
journal-pinned = fixada
journal-unknown = a simulação ainda não mede
journal-measured = agora { $value }
journal-limit-at-least = precisa ≥ { $value }
journal-limit-at-most = precisa ≤ { $value }
journal-weeks = { $weeks } sem.
journal-enter = O QUE ABRE
journal-milestones = MARCOS
journal-forces = FORÇAS
journal-exit = O QUE FECHA
journal-completes = AO COMPLETAR
journal-open-budget = Abrir o orçamento
journal-open-money = Abrir moeda e dívida
journal-open-laws = Abrir as leis
journal-open-research = Abrir a pesquisa
journal-group-politics = Política
journal-group-fiscal = Tesouro e moeda
journal-group-works = Obras
journal-group-research = Pesquisa

journal-works-title = Fila de obras
journal-works-cause = Todo nível novo, público ou privado, espera numa fila nacional pelos pontos de construção da construção civil. As obras recebem os pontos na ordem da fila, cada uma no máximo um vinte e seis avos do custo por semana.
journal-works-enter = Obras na fila
journal-works-exit = Fila vazia
journal-works-progress = { $public } públicas · { $private } privadas
journal-works-completes = Cada nível pronto começa a funcionar e as obras seguintes sobem na fila.
journal-work-line = { $building } · { $state }
journal-force-points = Pontos de construção entregues por semana
journal-bottleneck-builders = A construção civil atendeu só { $met } da demanda
journal-bottleneck-price = A construção custa { $price } do preço-base
journal-bottleneck-input = Falta { $good }: { $met } da demanda atendida

journal-large-title = { $building } em { $state }
journal-large-cause = A maior obra pública da fila: { $points } pontos de construção.
journal-large-enter = Lugar na fila
journal-large-points = Pontos de construção entregues
journal-large-exit = Pontos que faltam
journal-force-ahead = Obras à frente na fila
journal-force-min-weeks = Prazo mínimo de uma obra
journal-weeks-left = cerca de { $weeks } para terminar no ritmo da última semana
journal-weeks-beyond = mais de { $weeks } no ritmo da última semana
journal-waiting = esperando: as obras anteriores levam todos os pontos
journal-large-completes = A obra ({ $building }) abre e o Estado deixa de pagar por ela.

journal-research-title-focus = Pesquisando { $tech }
journal-research-title-auto = A caminho: { $tech }
journal-research-cause-focus = A tecnologia que o governo escolheu pesquisar.
journal-research-cause-auto = Sem foco escolhido: o país distribui a pesquisa sozinho. Estes são os protótipos mais perto da indústria.
journal-research-enter-focus = Foco escolhido
journal-research-enter-auto = Sem foco escolhido
journal-research-milestone = { $tech } → { $stage }
journal-research-exit = Chega ao estágio { $stage }
journal-research-progress = rumo a { $stage }
journal-research-completes = A tecnologia chega ao estágio { $stage }; no industrial, os métodos dela começam a subir a produtividade.

journal-cond-cash = Caixa do Tesouro
journal-cond-kept = Parte das compras planejadas mantida
journal-cond-room = Folga de déficit que a dívida deixa
journal-cond-defaults = Calotes até agora
journal-cond-year = Ano
journal-cond-inflation = Inflação
journal-cond-measure = { $measure }

journal-force-balance = Saldo da última semana, num ano nesse ritmo

journal-unfunded-title = Tesouro no vermelho
journal-unfunded-cause = Ninguém comprou os títulos que o Tesouro vendeu: o caixa ficou negativo. As compras caem com o buraco e a regra de imposto sobe a cada ano enquanto durar.
journal-unfunded-completes = O Tesouro volta a tomar emprestado e os cortes param.
journal-force-kept = Compras mantidas pela regra de orçamento
journal-force-tax-rule = Correção fiscal da regra de imposto

journal-purchases-title = Compras cortadas
journal-purchases-cause = A regra de orçamento corta as compras do Estado quando o Tesouro não conseguiu tomar emprestado ou quando os juros levam da receita mais do que a regra tolera.
journal-purchases-progress = { $kept } das compras planejadas mantidas
journal-purchases-completes = O Estado volta a comprar o plano inteiro.
journal-force-interest = Parte da receita que vai para juros

journal-debt-title = Dívida fechando o crédito
journal-debt-cause = A regra da dívida deixa o Estado tomar emprestado para compras enquanto dívida e juros estão baixos e estreita a folga conforme sobem. Esta entrada abre quando a dívida já tomou metade da folga que um país sem dívida teria.
journal-debt-closed = Sem folga: o Estado não toma nada emprestado para compras
journal-debt-progress = { $room } da folga sobrando
journal-debt-completes = As compras ficam só na receita; o déficit para de crescer.
journal-force-debt-gdp = Dívida sobre o PIB
journal-force-bond-rate = Juro dos títulos novos

journal-shutout-title = Sem crédito externo
journal-shutout-cause = Depois de um calote os credores estrangeiros somem por anos e o prêmio de risco fica alto ({ $defaults } até agora).
journal-shutout-back = Os credores estrangeiros voltam em { $year }
journal-shutout-completes = A poupança estrangeira volta a comprar os títulos.
journal-force-default-history = Histórico de calote que o prêmio lê

journal-inflation-title = Inflação acima da meta
journal-inflation-cause-subordinate = Os preços sobem mais rápido que a meta do banco central. O banco obedece ao Tesouro e compra os títulos que ninguém compra.
journal-inflation-cause-autonomous = Os preços sobem mais rápido que a meta do banco central. O banco autônomo sobe a taxa para segurar.
journal-inflation-completes = Inflação de volta à meta.
journal-force-policy-rate = Taxa do banco central
journal-force-monetized = Moeda criada fora da regra
journal-force-expected = Inflação esperada

journal-movement-title = Movimento: { $movement }
journal-movement-cause = { $desc }
journal-movement-progress = { $support } de apoio
journal-movement-exit = Sem a causa, o apoio cai à metade a cada { $years } anos
journal-goal-law = Aprovar { $law }
journal-goal-independence = Independência
journal-goal-revision = Recuperar território e posição perdidos
journal-force-pressure = Pressão da causa
journal-force-growth = Crescimento por ano com pressão máxima
journal-force-weakest = Condição mais fraca: { $measure }

journal-pending-title = Lei pedida: { $law }
journal-pending-cause = O governo pediu esta lei; o sistema de política a aprova.
journal-pending-enter = Pedida
journal-pending-exit = Aprovada
journal-pending-completes = { $law } em vigor.

journal-changes-title = Mudanças de lei
journal-changes-cause = Leis que mudaram nesta campanha. Legitimidade: { $legitimacy }.
journal-changes-enter = Mudanças
journal-change-line = { $from } → { $to }
journal-force-legitimacy = Legitimidade

outliner-title = Acompanhamento
outliner-journal = Agendas →
outliner-works = Obras
outliner-works-count = { $public } públicas · { $private } privadas
outliner-works-none = A fila está vazia.
outliner-weeks = ~{ $weeks } sem.
outliner-weeks-beyond = > { $weeks } sem.
outliner-waiting = esperando
outliner-work-tip-public = Obra pública, { $done } construída. Clique para o orçamento e a fila inteira.
outliner-work-tip-private = Obra privada, { $done } construída. Clique para o orçamento e a fila inteira.
outliner-research = Pesquisa
outliner-research-focus = foco
outliner-research-auto = sem foco
outliner-research-none = Nada a caminho.
outliner-pinned = Agendas fixadas
outliner-pinned-none = Fixe agendas na janela Agendas (J).
outliner-pinned-closed = Uma agenda fixada foi encerrada.

journal-difference = diferença de { $value }
journal-agendas = Agendas
journal-agendas-none = Nenhum alerta fiscal, movimento ou lei em curso agora.
journal-followed = Acompanhamento
journal-followed-hint = Obras e pesquisa: progresso para acompanhar, sem condição que as encerre.
journal-work-line-many = { $building } ×{ $count } · { $state }
journal-work-line-states = { $building } em { $states } estados
journal-noworks-title = Sem verba para obra pública nova
journal-noworks-cause = Todo mês o Estado começa obras públicas com o caixa acima da reserva de { $weeks } semanas de receita, menos o que as obras da fila ainda custam, mais o que a regra da dívida deixa tomar para obra. Essa soma está em zero ou abaixo: a fila não recebe obra pública nova.
journal-cond-works-budget = Verba para obra pública nova
journal-force-over-reserve = Caixa acima da reserva de { $reserve }
journal-force-committed = Ainda a pagar das obras públicas da fila
journal-force-borrowing = Empréstimo que a regra da dívida permite para obra
journal-noworks-completes = As obras públicas voltam a começar, pelos pesos do orçamento.
journal-bottleneck-cash = Sem verba para obra pública nova: o caixa acima da reserva não cobre a fila
outliner-research-focus-tip = O governo escolheu esta tecnologia na tela de Pesquisa (R).
outliner-research-auto-tip = Sem foco: o país distribui o esforço de pesquisa sozinho. Escolha um foco na tela de Pesquisa (R).

journal-revolt-title = Um país nasceu em revolta: { $country }
journal-revolt-cause = { $owner } recusou a independência e { $country } se proclamou sem acordo. Começou uma guerra colonial: a metrópole quer a união de volta e luta com a guarnição e os homens que desembarca; o país novo luta com irregulares, armados aos poucos.
journal-revolt-irregulars = Combatentes irregulares
journal-revolt-armed = Desses, armados
journal-revolt-landed = Homens desembarcados pela metrópole
journal-revolt-switch = Para jogar pelo país novo ou pela metrópole, troque de país no menu “…” da campanha, na tela de Campanhas.
