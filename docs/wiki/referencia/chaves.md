<!-- Página gerada de `data/` por `crates/bf-sim/tests/wiki_reference.rs`. Não edite à mão: regere com `BF_BLESS=1 cargo test -p bf-sim --test wiki_reference`. -->

# Chaves de regra

Cada regra nova entra desligada, com o jogo igual ao de antes; ligar é outra decisão, medida contra a faixa (`docs/wiki/principios.md`). O estado é o de `data/rules.ron`.

| Chave | Estado | O que faz |
|---|---|---|
| `capital_follows_rate` | desligada | O capital privado vai para onde o juro rende mais, e o país pode fechar a conta de capital (controle de capital). Precisa da liquidação em reservas. |
| `clause_tests` | desligada | Os acordos em vigor são testados todo mês: o tributo é pago e cobrado, e o acordo com prazo acaba quando o prazo vence. |
| `computer_efficiency` | desligada | O computador rende mais por pessoa: laboratórios e universidades ganham métodos com computador, e, com a capacidade fiscal, cada servidor arrecada mais. |
| `crisis` | desligada | Crises com prazo: um país exige, o outro aceita ou recusa pela força de cada lado, e a recusa pode virar guerra. |
| `customs_duties` | desligada | O imposto de importação vai para o Tesouro e encarece o que vem de fora. |
| `fiscal_capacity` | desligada | Um Estado com menos servidores do que precisa arrecada menos imposto. |
| `fixed_methods` | desligada | O governo fixa o método de produção de um tipo de prédio no país, fora do laissez-faire. |
| `forces` | ligada | Os soldados são os homens das formações de 1925 de cada país. |
| `foreign_contract` | desligada | As fábricas de uma colônia continuam da antiga metrópole na independência, e o país que fecha o capital estrangeiro estatiza as de fora. |
| `foreign_flow` | desligada | O dinheiro que sobra em casa constrói fábricas no exterior, e o lucro delas volta ao dono. |
| `foreign_technique` | desligada | A técnica se espalha mais depressa onde há fábricas de donos estrangeiros. |
| `gold_standard` | desligada | O padrão-ouro: quem está no ouro segura a paridade com os juros e as reservas, e sai quando a cobertura não dá. |
| `helicopter` | desligada | A tecnologia do helicóptero (1936/1942), de que as unidades de helicóptero e o assalto aéreo precisam. |
| `household_biomass` | desligada | As famílias do campo colhem a própria lenha, esterco e palha e compram só o resto do combustível. |
| `independence` | desligada | Os movimentos de independência das colônias pedem resposta à metrópole, e a recusa pode virar revolta. |
| `influence_ties` | desligada | O que um país vende a outro e o que tem investido nele pesam na influência que tem sobre ele. |
| `jobs_follow_use` | desligada | A fábrica que vende menos do que pode fazer demite parte da equipe a cada mês. |
| `ledger_record` | desligada | Cada linha do livro de dívidas entre governos guarda o que foi pago e perdoado, e a que acaba fica no registro. |
| `market_stock` | desligada | O bem estocável que não acha comprador fica guardado no mercado e volta à venda depois. |
| `media` | desligada | Mede o alcance da imprensa, do rádio, da televisão e da internet em cada país. |
| `negotiation` | desligada | A mesa de negociação: o outro país responde às propostas, faz contrapropostas e pode denunciar acordos. |
| `occupied_works` | desligada | As obras em terra ocupada ficam paradas até a terra voltar ao dono. |
| `payments_settle` | desligada | O comércio e os pagamentos entre países mexem nas reservas de cada banco central. |
| `radium` | desligada | As minas de urânio trabalham pelo rádio, que os hospitais compram, e o urânio sai como subproduto. |
| `rail_by_line` | desligada | As ferrovias de 1925 têm o tamanho do que as linhas carregavam, e o resto do transporte vai por carroça, tropa e barco. |
| `relations` | desligada | Mede o poder, a ameaça e a hostilidade entre os países, só para leitura: nenhuma decisão da IA ainda depende disso. |
| `reputation` | desligada | Cada país guarda a reputação dos acordos que quebrou, que pesa na mesa de negociação. |
| `reserve_cohorts` | desligada | A reserva militar por turma: quem termina o serviço entra na reserva, perde treino com o tempo e a mobilização chama os mais novos primeiro. |
| `schooling` | desligada | A escola por turma e por nível, e a campanha de alfabetização de adultos. |
| `small_market_wage_floor` | desligada | Um país pequeno demais para ter salário próprio usa o menor salário dos outros. |
| `trade_by_pair` | ligada | Cada comprador escolhe de quem compra pelo custo de chegada. |
| `trade_gross` | ligada | Cada país compra de fora uma parte do que também produz, não só o que falta. |
| `treaties_in_force` | ligada | Os tratados de 1925 valem como acordos: chamam aliados para as crises e a quebra fica registrada. |
| `union_services` | desligada | Em uma união aduaneira, serviços e finanças também se negociam entre os países. |
| `war` | desligada | A guerra em terra: frentes, combate, avanço e controle do território. |
