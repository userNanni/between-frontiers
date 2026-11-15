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
stat-area = Área
stat-area-value = { $value } km²
stat-provinces = Províncias
stat-subjects = Súditos
stat-rank-population = { $rank }º do mundo em população
stat-rank-gdp = { $rank }ª economia do mundo
stat-source = Fonte: { $source }
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
