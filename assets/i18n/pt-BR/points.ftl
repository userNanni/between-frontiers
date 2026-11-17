# Pontos de restauração, salvamento rápido e o menu da campanha.

cmd-campaign-quick-save = Salvar rápido
    .short = Salvar rápido
cmd-campaign-quick-load = Carregar rápido
    .short = Carregar rápido
cmd-campaign-save-named = Salvar com nome
    .short = Salvar como

settings-autosave-quarterly = Trimestral
settings-autosave-semiannual = Semestral
settings-autosave-hint = Um ponto de restauração automático, e a campanha salva, a cada virada do calendário.
settings-autosave-keep = Pontos automáticos mantidos
settings-points-cap = Teto de pontos por campanha
settings-points-warn = Avisar quando a pasta da campanha passar de
settings-point-before-risk = Ponto antes de decisão arriscada
settings-point-before-risk-hint = Antes de mudar uma lei, libertar território ou aceitar um acordo (e, quando existirem, declarar guerra ou assinar tratado) o jogo guarda onde você estava, para voltar e tentar outro caminho.
settings-points-cost = Cada ponto: ~{ $each } MB · { $keep } automáticos: ~{ $auto } MB · teto de { $cap }: ~{ $full } MB
settings-points-cost-unknown = O tamanho de um ponto aparece depois que uma campanha for salva.

game-menu-quick-save = Salvar rápido
game-menu-quick-load = Carregar rápido
game-menu-save-named = Salvar com nome…

points-title = Pontos de restauração
points-summary = { $name }: { $count ->
    [0] nenhum ponto
    [one] um ponto
   *[other] { $count } pontos
    }, { $manual } fora da rotação · pasta { $size } MB
points-empty = Nenhum ponto de restauração ainda. O salvamento automático, o salvar rápido (F5) e o "Salvar com nome" criam pontos.
points-kind-auto = Automático
points-kind-quick = Salvar rápido
points-kind-manual = Salvo à mão
points-reason-law = Antes de mudar uma lei
points-reason-liberate = Antes de libertar território
points-reason-accept = Antes de aceitar um acordo
points-reason-war = Antes de declarar guerra
points-reason-treaty = Antes de assinar um tratado
points-reason-load = Antes de voltar a outro ponto
points-load = Voltar a este ponto
points-rename = Renomear
points-delete = Apagar
points-delete-confirm = Apagar este ponto?
points-delete-yes = Apagar ponto
points-name-title = Salvar com nome
points-name-save = Salvar
points-name-cancel = Cancelar
points-saved-named = Salvo "{ $name }"
points-saved-quick = Salvamento rápido gravado
points-no-quick = Esta campanha ainda não tem salvamento rápido.
points-folder-large = A pasta da campanha ocupa { $size } MB, mais que os { $limit } MB que você definiu.

saves-menu = Opções da campanha
saves-menu-rename = Renomear
saves-menu-delete = Apagar campanha
saves-menu-open-folder = Abrir pasta
saves-menu-play-other = Jogar com outro país
saves-rename-ok = Renomear
saves-cancel = Cancelar
saves-new-cancel = Voltar aos pontos de restauração
cmd-campaign-menu = Opções da campanha
    .short = Opções
saves-list-title = Campanhas salvas
month-short-1 = jan.
month-short-2 = fev.
month-short-3 = mar.
month-short-4 = abr.
month-short-5 = maio
month-short-6 = jun.
month-short-7 = jul.
month-short-8 = ago.
month-short-9 = set.
month-short-10 = out.
month-short-11 = nov.
month-short-12 = dez.
date-short = { $day } { $month } { $year }
date-month-year = { $month } { $year }
saves-played-today = hoje, { $time }
saves-played-yesterday = ontem
saves-last-played-full = Jogada pela última vez em { $date }, às { $time }
saves-size = Espaço que a campanha ocupa no disco, com os pontos de restauração.
saves-size-large = Passou dos { $limit } MB que você definiu nas configurações para ser avisado. O jogo não apaga nada sozinho: apague pontos de restauração antigos ou suba o aviso.
saves-newer = Gravada por uma versão mais nova do jogo ({ $by }, formato do mundo { $found }); esta ({ $this }) lê até o formato { $supported }. Atualize o jogo para abri-la, ou volte a um ponto de restauração mais antigo, abaixo.
saves-damaged = O estado atual não lê: { $error }. Os pontos de restauração abaixo podem estar bons: volte a um deles.
saves-unreadable-short = Não abre nesta versão
saves-cause-not-a-save = o arquivo não é um save deste jogo
saves-cause-container-newer = o arquivo foi gravado por uma versão mais nova do motor
saves-cause-truncated = o arquivo está cortado
saves-cause-checksum = o arquivo está danificado (a soma de verificação não bate)
saves-cause-wrong-game = o arquivo é save de outro jogo
saves-cause-malformed = uma parte do arquivo não decodifica
saves-damaged-short = Arquivo danificado
settings-autosave-cost = Tempo que tira da simulação (medido em 2025): { $costs }.
settings-autosave-cost-one = { $option } ~{ $percent } %
