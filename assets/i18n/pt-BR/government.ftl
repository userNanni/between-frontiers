## Janela do governo: orçamento, moeda e dívida. Valores em dólares a preços-base de 1925.

gov-pct = { $value } %
gov-pct-gdp-year = { $value } % do PIB/ano
gov-no-economy = Este país não tem economia simulada.
gov-pending = Pausado: as mudanças valem quando o tempo voltar a correr.
gov-budget-basis = Contas da última semana × 52, em dólares a preços-base de 1925. Passe o mouse num número para ver a conta.
gov-money-basis = Taxas e inflação em % ao ano; índices com 1925 = 100. Passe o mouse num número para ver a conta.

## Orçamento
gov-revenue = Receita por ano
gov-revenue-total = Receita total
gov-class-0 = Classe baixa
gov-class-1 = Classe média
gov-class-2 = Classe alta
gov-class-0-members = camponeses, trabalhadores rurais, operários, clero e soldados
gov-class-1-members = qualificados, escriturários, engenheiros, profissionais liberais e servidores
gov-class-2-members = gerentes, capitalistas e proprietários de terra
gov-tip-class-tax =
    { $people } pessoas: { $members }.
    Renda depois do imposto: { $income } por ano.
    Alíquota: { $rate }. Imposto = renda × alíquota ÷ (1 − alíquota) = { $tax }.
gov-tip-revenue =
    Imposto de renda da última semana: { $week } × 52 = { $year } ({ $gdp } do PIB).
    Média do trimestre: { $avg } por ano.
gov-spending = Despesa por ano
gov-spending-total = Despesa total
gov-spend-0 = Funcionalismo
gov-spend-1 = Compras
gov-spend-2 = Escolas e hospitais
gov-spend-3 = Obra pública
gov-spend-4 = Juros
gov-tip-payroll =
    { $workers } servidores, clérigos e soldados, pagos a { $pay } do salário médio do país vezes o peso da profissão.
    É obrigação: paga mesmo com o caixa no vermelho.
gov-tip-goods =
    O que o Estado compra, a preço-base: serviços 30 %; transporte 10 %; papel 10 %; remédios 10 %; armas leves 10 %; munição 10 %; navios de guerra 10 %; artilharia 5 %; aviões militares 5 %.
    Regra da semana, por ano: receita média { $revenue } + déficit que a regra da dívida permite { $room } − folha, escolas e hospitais, obras e juros { $committed } = { $planned }.
    No mínimo 60 % da folha: { $floor }. Fica { $kept } do plano (corte com caixa negativo ou juros acima de 10 % da receita) = { $total }.
gov-tip-services = O Estado compra tudo o que escolas e hospitais produzem; vira salário de professores, médicos e funcionários.
gov-tip-works = Pontos de construção pagos às obras públicas da fila, conforme chegam.
gov-tip-interest =
    Juros da dívida em mãos privadas e no exterior ({ $cost } ao ano sobre { $debt } de dívida).
    Os juros do banco central voltam ao Tesouro e não entram aqui.
gov-tip-yearly = Última semana: { $week }; × 52 = { $year } ({ $gdp } do PIB).
gov-tip-spending = { $gdp } do PIB.
gov-balance = Saldo
gov-amount-gdp = { $amount } · { $gdp } do PIB
gov-tip-deficit =
    Receita { $revenue } − despesa { $spending } = déficit de { $balance } ({ $gdp } do PIB).
    O Tesouro vende títulos para cobrir a diferença; o banco central comprou { $bank } deles.
gov-tip-surplus =
    Receita { $revenue } − despesa { $spending } = superávit de { $balance } ({ $gdp } do PIB).
    O que passa da reserva vira obra pública nova.
gov-treasury = Caixa do Tesouro
gov-tip-treasury =
    Caixa: { $cash }. Reserva: { $weeks } semanas de receita média ({ $avg } por semana) = { $reserve }.
    A barra mostra o caixa contra a reserva; o que passa dela vira obra pública.
gov-issued = Títulos vendidos ao banco central
gov-tip-issued = Moeda nova que o banco central emprestou ao Tesouro comprando títulos, na última semana × 52. Só o banco subordinado faz isso.

gov-taxes = Imposto de renda
gov-tax-rule = Regra histórica
gov-tax-fixed = Fixadas pelo governo
gov-tip-tax-rule =
    Regra de { $year }: { $base } × correção fiscal { $adjust } = { $rule }.
    A regra sobe em linha reta de 1925 a 1975. Teto de { $max } por classe.
gov-tax-adjust = Correção fiscal da regra: × { $adjust } (sobe quando a dívida custa caro, cai quando sobra caixa).

gov-works = Obra pública
gov-cat-0 = Infraestrutura ({ $share })
gov-cat-1 = Educação ({ $share })
gov-cat-2 = Saúde ({ $share })
gov-cat-3 = Moradia urbana ({ $share })
gov-tip-priority =
    Peso { $weight } de { $sum } = { $share } do dinheiro de obra nova (padrão: { $default }).
    Na fila: { $works } obras desta categoria, { $done } dos pontos entregues.
gov-works-stopped = Todos os pesos em zero: nenhuma obra pública nova.
gov-queue = Fila: { $public } obras públicas, { $private } privadas
gov-tip-work = { $done } de { $points } pontos de construção entregues.
gov-queue-more = e mais { $count } obras públicas.

gov-migration = Lei de migração
gov-migration-open = Aberta
gov-migration-restricted = Passaporte interno
gov-migration-open-tip = Qualquer um pode se mudar para a cidade que pagar melhor.
gov-migration-restricted-tip = Só 1/4 do fluxo dentro do estado e 1/10 entre estados passam.

## Moeda e dívida
gov-prices = Preços e câmbio
gov-inflation = Inflação
gov-tip-inflation-subordinate =
    Agora { $now } ao ano; no ano passado { $last }; esperada { $expected }.
    Com banco subordinado a expectativa segue o passado e o mundo, e só um pouco a meta de { $target }.
    Moeda criada fora da regra: { $monetized } do PIB ao ano (cada ponto soma 1,5 ponto à inflação).
gov-tip-inflation-autonomous =
    Agora { $now } ao ano; no ano passado { $last }; esperada { $expected }.
    Banco autônomo: a expectativa fica ancorada na meta de { $target }.
    Moeda criada fora da regra: { $monetized } do PIB ao ano (cada ponto soma 1,5 ponto à inflação).
gov-price-level = Nível de preços (1925 = 100)
gov-tip-price-level =
    Em 1º de janeiro: { $jan }.
    Reformas monetárias (três zeros cortados): { $reforms }.
gov-exchange = Câmbio (1925 = 100)
gov-tip-exchange =
    Moeda local por unidade de referência: { $rate }. Paridade de poder de compra: { $parity }.
    Câmbio real = câmbio ÷ paridade × 100 = { $real } (acima de 100: moeda barata).
    O câmbio anda 1/26 por semana para a paridade, enfraquecido pelo prêmio de risco.
gov-central-bank = Banco central
gov-policy-rate = Taxa básica
gov-tip-rate-fixed = Fixada pelo governo em { $rate } ao ano.
gov-tip-rate-taylor =
    Regra de Taylor: juro real { $real } + inflação { $inflation } = neutra { $neutral };
    + metade da inflação acima da meta de { $target }: { $infl_term };
    + metade do hiato de utilização das fábricas: { $gap_term };
    = { $rate } (piso zero).
gov-regime = Regime em vigor
gov-regime-subordinate = Subordinado ao Tesouro
gov-regime-autonomous = Autônomo
gov-tip-regime-historical = Regra histórica: subordinado até 1990 ou até um plano de estabilização depois de inflação acima de 200 %.
gov-tip-regime-historical-since = Regra histórica: autônomo desde { $year }.
gov-tip-regime-autonomous = Escolhido pelo governo: segue a sua regra e não compra título novo do Tesouro.
gov-tip-regime-subordinate = Escolhido pelo governo: compra os títulos que ninguém mais compra.
gov-credit = Crédito aos bancos
gov-tip-credit =
    Dinheiro que o banco central criou e emprestou aos bancos, que o põem no fundo de investimento das empresas: { $gdp } do PIB.
    Importa porque financia fábricas e obras privadas sem tirar da poupança das famílias; retirado, o investimento cai.
gov-monetized = Moeda fora da regra
gov-tip-monetized =
    Moeda que o banco central criou além da sua regra: títulos do Tesouro que comprou porque ninguém mais comprou, ou a taxa fixada abaixo da regra. Média do último ano, em % do PIB.
    Importa porque cada ponto soma 1,5 ponto à inflação; a compra de títulos dentro da regra não conta aqui.
gov-bank-controls = Política monetária
gov-bank-historical = Histórico
gov-bank-autonomous = Autônomo
gov-bank-subordinate = Subordinado
gov-rate-rule = Regra de Taylor
gov-rate-fixed = Taxa fixada
gov-tip-rate-step = Meio ponto por clique. Mexer na taxa fixa a taxa; volte à regra pelo seletor acima.

gov-debt = Dívida pública por credor
gov-debt-home = Em casa
gov-debt-bank = Banco central
gov-debt-abroad = Exterior, moeda local
gov-debt-fx = Exterior, em dólar
gov-tip-debt-home = Poupadores e bancos do país: { $amount } ({ $gdp } do PIB, { $share } da dívida).
gov-tip-debt-bank = Títulos no banco central: { $amount } ({ $gdp } do PIB, { $share } da dívida). Os juros voltam ao Tesouro.
gov-tip-debt-abroad = Estrangeiros, em moeda do país: { $amount } ({ $gdp } do PIB, { $share } da dívida). A inflação corrói.
gov-tip-debt-fx =
    Em moeda estrangeira: { $fx } em unidades de referência × câmbio real { $real } ÷ 100 = { $amount } ({ $gdp } do PIB, { $share } da dívida).
    Não cai com a inflação do país e cresce quando a moeda perde valor.
gov-debt-total = Total
gov-debt-total-value = { $amount } · { $gdp } do PIB
gov-tip-debt-total = Soma dos quatro credores contra o PIB do último ano ({ $gdp_value }) = { $gdp }.
gov-interest = Juros
gov-interest-paid = Juros pagos por ano
gov-tip-interest-paid =
    Última semana: { $week } × 52.
    { $revenue } da receita, { $gdp } do PIB.
gov-bond-rate = Título novo em moeda local
gov-tip-bond-rate = Metade da taxa do banco central + metade da taxa mundial corrigida pela inflação do país, + prêmio de risco.
gov-fx-rate = Título novo em dólar
gov-tip-fx-rate = Taxa mundial + prêmio de risco.
gov-cost-local = Custo médio do estoque, moeda local
gov-cost-fx = Custo médio do estoque, dólar
gov-tip-cost = O custo da dívida antiga segue a taxa de mercado em cerca de cinco anos.
gov-defaults = Calotes
gov-defaults-count = Calotes na história
gov-tip-defaults = Histórico que o prêmio de risco lê: { $history } (perde 10 % ao ano; cada calote soma 1).
gov-foreign-credit = Crédito estrangeiro
gov-excluded-until = Fechado até { $year }
gov-tip-excluded = Depois de um calote, o país fica três anos sem vender títulos no exterior.
gov-credit-open = Aberto
gov-tip-credit-open = Estrangeiros compram títulos do país.
gov-foreign-assets = Títulos de outros países
gov-tip-foreign-assets = O que os poupadores do país emprestaram a outros Estados.
gov-waiting = As contas aparecem quando o primeiro dia passar.
gov-no-player = Nenhum país para governar nesta campanha.
