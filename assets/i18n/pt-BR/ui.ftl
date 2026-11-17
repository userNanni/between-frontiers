## Nome e números

game-title = Entre Fronteiras
game-subtitle = Simulador político e econômico, 1925–2025
game-version = Versão { $version } · fase de fundação

number-thousands = { $value } mil
number-millions = { $count ->
    [one] { $value } milhão
   *[other] { $value } milhões
}
number-billions = { $count ->
    [one] { $value } bilhão
   *[other] { $value } bilhões
}
money-intl-dollars = US$ { $value }
date-long = { $day ->
    [1] 1º
   *[other] { $day }
} de { $month } de { $year }
month-1 = janeiro
month-2 = fevereiro
month-3 = março
month-4 = abril
month-5 = maio
month-6 = junho
month-7 = julho
month-8 = agosto
month-9 = setembro
month-10 = outubro
month-11 = novembro
month-12 = dezembro

## Menu principal

menu-continue = Continuar
menu-continue-detail = { $save } · { $date }
menu-continue-empty = Nenhuma campanha salva
menu-play = Iniciar jogo
menu-settings = Configurações
menu-quit = Sair
menu-back = Voltar

## Configurações

settings-title = Configurações
settings-language = Idioma
settings-language-hint = Muda todos os textos na hora, inclusive nomes de países.
settings-ui-scale = Tamanho da interface
settings-ui-scale-value = { $percent }%
settings-map-labels = Nomes de países no mapa
settings-autosave = Salvamento automático
settings-autosave-off = Desligado
settings-autosave-monthly = Mensal
settings-autosave-yearly = Anual
settings-saves-folder = Pasta das campanhas
settings-done = Concluir

## Campanhas salvas

saves-title = Campanhas
saves-subtitle = Continue uma campanha ou comece outra em 1º de janeiro de 1925.
saves-empty = Nenhuma campanha salva ainda.
saves-empty-hint = Crie a primeira ao lado.
saves-country-pending = País ainda não escolhido
saves-last-played = Jogado em { $when }
saves-load = Carregar
saves-delete = Excluir
saves-delete-confirm = Excluir “{ $name }”? Isso não pode ser desfeito.
saves-delete-yes = Excluir campanha
saves-delete-no = Manter
saves-new-title = Nova campanha
saves-new-name = Nome da campanha
saves-new-name-default = Campanha { $n }
saves-new-start = Início
saves-new-start-value = 1º de janeiro de 1925
saves-new-seed = Semente do mundo
saves-new-seed-hint = A mesma semente com as mesmas decisões repete a mesma história.
saves-new-create = Criar e escolher país
saves-new-name-taken = Já existe uma campanha com esse nome.
saves-new-name-empty = Dê um nome à campanha.
saves-error-io = Não foi possível gravar a campanha: { $error }

## Escolha do país

pick-title = Escolha seu país
pick-hint = Clique num país no mapa. Arraste para mover, use a roda do mouse para aproximar.
pick-back = Voltar às campanhas
pick-start = Começar campanha
pick-none = Nenhum país selecionado

## Painel do país

country-status-sovereign = País soberano
country-status-subject = { $kind } { $article ->
    [o] do
    [a] da
    [os] dos
    [as] das
   *[other] de
} { $overlord }
subject-kind-Dominion = Domínio
subject-kind-Mandate = Mandato
subject-kind-Protectorate = Protetorado
subject-kind-Satellite = Satélite
subject-kind-CrownColony = Colônia da Coroa
stat-capital = Capital
stat-government = Governo
stat-population = População
stat-gdp = PIB
stat-gdp-per-capita = PIB per capita
stat-gdp-unit = Dólares internacionais de 2011
stat-gdp-unit-live = Dólares a preços-base da simulação
stat-area = Área
stat-area-value = { $value } km²
stat-states = Estados
stat-subjects = Súditos
borders-title = Fronteiras
borders-switch = Jogar com { $country }
borders-switch-detail = O jogo continua; este país passa a ser o seu.
borders-liberate = Libertar { $state }
borders-liberate-subject = Libertar { $state } como súdito
borders-liberate-detail = O estado vira país próprio, com a sua gente, as fábricas e parte do fundo de investimento; o seu Tesouro fica com a dívida.
borders-select-state = Clique num estado seu para libertá-lo.
stat-rank-population = { $rank }º do mundo em população
stat-rank-gdp = { $rank }ª economia do mundo
stat-difficulty = Dificuldade
difficulty-great = Grande potência
difficulty-major = Potência regional
difficulty-minor = País pequeno
difficulty-subject = Súdito

## Mapa

map-sea = Mar
map-unowned = Sem dono
map-tooltip-province = { $province }, { $country }

## Campanha

game-paused = Pausado
game-speed = Velocidade { $speed }
game-menu = Menu
game-menu-save = Salvar
game-menu-saved = Campanha salva
game-menu-resume = Voltar ao jogo
game-menu-exit = Sair para o menu principal

## Barra superior

number-short-thousands = { $value } mil
number-short-millions = { $value } mi
number-short-billions = { $value } bi
bar-gdp = PIB
bar-population = População
bar-quality = Qualidade de vida
bar-literacy = Alfabetização
bar-life = Expectativa de vida
bar-urban = Urbanização
bar-unknown = —
bar-years = { $value } anos
bar-percent = { $value }%
bar-of-hundred = { $value } de 100
bar-rank = { $rank }º do mundo
bar-gdp-tip = Valor de tudo o que o país produz num ano.
bar-gdp-per-capita = Por habitante: { $value }
bar-population-tip = Pessoas vivendo na metrópole. As colônias contam à parte.
bar-density = Densidade: { $value } hab./km²
bar-quality-tip = Média geométrica de renda, saúde e educação, de 0 a 100. Uma dimensão fraca puxa o índice todo para baixo.
bar-quality-income = Renda
bar-quality-health = Saúde
bar-quality-education = Educação
bar-quality-missing = Faltam dados para calcular o índice deste país.
bar-literacy-tip = Adultos (15 anos ou mais) que sabem ler e escrever.
bar-life-tip = Anos que um recém-nascido viveria com a mortalidade de hoje.
bar-urban-tip = Parcela da população que vive em áreas urbanas.
bar-country-tip = Seu país. Clique para ver o resumo.
bar-speed-down = Mais devagar
bar-speed-up = Mais rápido
bar-pause = Pausar
bar-play = Continuar
bar-date-tip = Velocidade { $speed } de 5. Clique para pausar ou continuar.
bar-menu = Menu
source-economy = Fontes: Maddison Project 2023 e Our World in Data; estimativas próprias onde faltam dados.
source-social = Fontes: Our World in Data e HYDE; estimativas próprias onde faltam dados.

## Painel do canto

hud-per-capita = Renda per capita
hud-per-capita-tip = PIB dividido pela população. A barra vai de US$ 300 a US$ 40.000, a faixa do século.
hud-density = Densidade
hud-density-value = { $value }/km²
hud-density-tip = Pessoas por km² de território.
hud-area = Área
hud-area-tip = Território da metrópole. As colônias contam à parte.
hud-states = Estados
hud-states-tip = Estados da metrópole, a unidade da simulação. As colônias contam à parte.
hud-subjects = Súditos
hud-subjects-tip = Países sob a suserania deste: domínios, mandatos, protetorados e satélites.
hud-subjects-none = Nenhum súdito.
hud-gdp-share = Fatia do PIB mundial
hud-gdp-share-tip = Parte de tudo o que o mundo produz que sai deste país.
hud-pop-share = Fatia da população mundial
hud-pop-share-tip = Parte da humanidade que vive neste país.
hud-rank-quality = Posição em qualidade de vida
hud-rank-population = Posição em população
hud-rank-value = { $rank }º
hud-due-to = Isso se deve a:
hud-world-total = Total do mundo: { $value }
hud-life-bar = A barra vai de 20 a 85 anos.
hud-medal-tip = { $rank }ª economia do mundo
hud-metropole = Metrópole
hud-colonies = Colônias
hud-total = Total
hud-includes-colonies = Inclui as colônias.
country-colonies = Colônias
country-colonies-value = { $population } · { $states } estados
settings-map-relief = Relevo do terreno no mapa
saves-error-load = Não foi possível abrir a campanha: { $error }
hud-engine = Dia { $tick } da simulação · estado { $hash }

## Números da simulação

unit-scenario = Em dólares internacionais de 2011 (Maddison Project), os do cenário de 1925.
unit-live = Em dólares a preços-base da simulação: preços fixos do catálogo, sem inflação. Não se somam nem se comparam direto com os dólares de 2011 do cenário.
source-live = Fonte: a simulação, lida a cada mês.
source-live-gdp-year = PIB de { $year }, o último ano completo.
source-live-gdp-calibration = Ainda sem ano completo de jogo: vale o PIB calibrado em 1925.
hud-gdp-split = Metrópole e colônias negociam no mesmo mercado: o PIB dele se divide entre as duas pela renda das pessoas, e no primeiro ano como no cenário.
hud-life-scenario = Dado de 1925: a simulação ainda não calcula a expectativa de vida.
hud-quality-life-scenario = A saúde usa a expectativa de vida de 1925, que a simulação ainda não calcula.

## Modos de mapa

lens-title = Modo do mapa
lens-political = Político
lens-resources = Recursos
lens-climate = Clima
lens-farms = Agro
lens-no-data = Os dados de recursos e de terra ainda não foram carregados.
legend-deposit = Tamanho da jazida
legend-small = Pequena
legend-large = Classe mundial
legend-suitability = Aptidão
legend-low = Baixa
legend-high = Alta
tip-deposit = { $resource }: jazida de tamanho { $size } de 5
tip-climate = Clima { $climate } em { $share } do território
tip-farm = { $farm }: aptidão { $value }
lens-infrastructure = Infra
legend-local = Demanda atendida no estado
legend-unmet = Falta
legend-met = Atendida
tip-local = { $good }: { $share } da demanda atendida

## Pesquisa
bar-research = Pesquisa
research-title = Pesquisa
research-close = Fechar
research-summary = { $points } pontos de pesquisa por mês · foco: { $focus }
research-focus-auto = escolha automática
research-focus-clear = Deixar o país escolher
research-set-focus = Pesquisar
research-focused = Em foco
research-locked = Requer outras tecnologias
research-locked-short = Bloqueada
research-requires = Requer: { $list }
research-unlocks = Libera: { $list }
research-history = Na história: protótipo em { $demonstrated }, indústria em { $industrialized }
research-first-industrial = Feita em escala pela primeira vez neste jogo em { $year }
research-state-buyer = O Estado pode ser o primeiro comprador
research-level = Nível { $n } · ≈ { $cost } pontos
research-level-short = Nível { $n }
research-cost = Nível { $n } · custo de { $cost } pontos até a ciência
research-zoom-in = Aproximar
research-zoom-out = Afastar
research-fit = Ajustar
research-fit-hint = Mostrar a árvore inteira
research-all-fields = Todas as áreas

## Janela do governo
bar-government = Governo
gov-title = Governo
gov-tab-budget = Orçamento
gov-tab-money = Moeda e dívida
gov-tab-laws = Leis
gov-close = Fechar
gov-coming = Ainda não feito.

## Leis (aba do governo)
laws-missing = Os dados de leis não foram encontrados.
laws-not-held = Não se aplica: sem colônias
laws-not-held-hint = O país não tem colônias; estas leis só valem para quem tem.
laws-in-force = Em vigor
laws-targeted = Dirigida
laws-no-effects = Não muda nenhum efeito.
laws-scope-metropole = Vale na metrópole
laws-scope-targeted = Vale nas comunidades que a lei visa
laws-scope-colonies = Vale nas colônias
laws-source = Fonte legal das leis em vigor (1925)
laws-propose = Propor lei
laws-propose-pending = Já proposta; a simulação decide no próximo dia.
laws-propose-in-force = Esta lei já está em vigor.
laws-propose-not-held = O país não tem colônias, e esta lei só vale nelas.
laws-changes = O que muda
laws-changes-from = Em relação à lei em vigor, { $law }:
laws-changes-none = Nada muda nos efeitos.
laws-stances = Grupos de interesse
laws-stances-hint = Lei em vigor: cada termo é o quanto um efeito da lei pesa para o grupo, em pontos (o peso que o grupo dá ao efeito vezes o quanto a lei o afasta do neutro). A soma é o que o grupo ganha com ela; não há ideologia.
laws-stances-hint-change = Cada termo é o quanto um efeito da troca pesa para o grupo, em pontos (o peso que o grupo dá ao efeito vezes o quanto ele muda, contadas todas as leis em vigor). A soma é a posição do grupo sobre a troca; não há ideologia. Entre parênteses, a força do grupo.
laws-stance-for = a favor
laws-stance-against = contra
laws-stance-indifferent = indiferente
laws-terms-none = Nenhum efeito que este grupo pese.
laws-value-cap = teto de { $value } %
laws-value-no-cap = sem teto
laws-value-share = { $value } %
laws-value-multiplier = ×{ $value }
laws-pending = Proposta
laws-row-pending = { $law } · proposta pendente
laws-legitimacy = Legitimidade
laws-movements-none = Nenhum movimento ativo.
laws-movement-support = { $value } % de apoio
laws-balance = Força dos grupos: { $for } % a favor, { $against } % contra.
laws-group-clout = { $group } ({ $clout } %)
laws-history = Mudanças na campanha
laws-history-line = { $date }: { $from } → { $to } ({ $by })
laws-by-command = por decisão do governo
laws-by-groups = pelos grupos de interesse
laws-by-movement = pelo movimento { $movement }
laws-balance-none = Nenhum grupo com força apoia esta troca. Quando a tramitação existir, uma lei sem apoio não passa; por enquanto a proposta é promulgada no dia seguinte.
laws-balance-minority = A maior parte da força é contra. Quando a tramitação existir, a lei vai depender de mudar essa conta; por enquanto a proposta é promulgada no dia seguinte.
laws-balance-majority = A maior parte da força apoia esta troca.
laws-effects-inert = Os efeitos das leis ainda não agem na economia: a troca muda a lei em vigor e a posição dos grupos, não os números do país.
laws-migration-in-budget = A migração interna que a simulação usa hoje se ajusta na lei de migração, na aba Orçamento.
