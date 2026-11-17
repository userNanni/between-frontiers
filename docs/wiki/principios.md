# Princípios de desenho

As regras que valem para o jogo inteiro. Cada uma diz o que o autor pensou e o que muda para quem
joga. Os princípios técnicos de sempre (mesmas regras para a IA e para o jogador, determinismo,
causa visível, pouco sorteio, cada fase jogável sozinha) estão em
[`architecture.pt-BR.md`, "Princípios"](../architecture.pt-BR.md#princípios).

## Funcionalidade antes de equilíbrio

**Decisão (2026-11-17):** as fases 4 a 6 (diplomacia, guerra, IA estratégica) vêm primeiro; o
equilíbrio fino fica para depois da fase 6 e da otimização (abaixo).

**O que o autor pensou:** "Melhor andar mais rápido em funcionalidade e equilibrar depois: talvez
seja a falta de alguma funcionalidade que desequilibrou o jogo." A coordenação tinha proposto
equilibrar depois da fase 5; ele preferiu depois da fase 6. O que a coordenação tinha medido: vários desequilíbrios (colônias ricas
demais, a industrialização da China, o pessoal de pesquisa) esbarravam em sistemas que ainda não
existiam, e cada ajuste reembaralhava as trajetórias dos países.

**O que muda no jogo:** até a fase 6, um país pode crescer depressa demais ou devagar demais. O
teste de 100 anos continua rodando para avisar quando algo quebra (determinismo, mundo sem
colapsar), não como meta fina. O desequilíbrio que aparece vira dependência anotada com o número,
à espera do sistema que falta.

## Otimizar antes de calibrar

**Decisão (2026-11-17):** depois da fase 6 e antes da calibração vêm rodadas de otimização e
simplificação do código. A ordem fica: funcionalidade até a fase 6, depois desempenho, depois
calibração.

**O que o autor pensou:** "Para calibrar, vai ser preciso rodar muitas vezes e observar. Antes
disso, vale otimizar o código e ver o que dá para simplificar e melhorar em desempenho: as rodadas
de calibração já ganham com isso e servem também como mais testes de desempenho."

**O que muda no jogo:** o jogador não vê a ordem, mas sente o efeito: um ano de simulação mais
rápido. As rodadas de 100 anos da calibração passam a medir também o custo de cada sistema, como o
benchmark noturno já faz ([`benchmarks.pt-BR.md`](../benchmarks.pt-BR.md)).

## Gradual como na vida real, mas não tanto assim

**Decisão (2026-11-17):** toda regra confere se o processo real é imediato, gradual ou por coorte,
e se a diferença muda a simulação. Quando muda, o jogo mostra o estoque intermediário.

**O que o autor pensou:** "Na maior parte das situações, as coisas são graduais, mas não tanto
assim. Avalie sempre se na vida real o processo é mais gradual e se isso faz diferença na
simulação." O exemplo dele: uma campanha de educação não sobe a alfabetização na hora; sobe
primeiro a quantidade de gente na escola, e a alfabetização vem nos anos seguintes.

**O que muda no jogo:** o jogador vê a matrícula subir antes da alfabetização, a tropa treinar
semana a semana, a obra receber pontos até ficar pronta. Efeito instantâneo onde a vida real tem
defasagem daria alavancas irreais e esconderia a causa. O "não tanto assim" também vale: quando o
processo real anda por mês ou por ano, cadência mensal ou anual basta. Os documentos técnicos
trazem, em cada sistema, uma tabela "como é na vida real e se a diferença importa" (por exemplo,
[`war.pt-BR.md`](../war.pt-BR.md) e [`influence.pt-BR.md`, seção 2](../influence.pt-BR.md#2-como-é-na-vida-real-e-se-a-diferença-importa)).

## Decisão rara fica fora do fluxo comum

**Decisão (2026-11-17):** ação rara ou difícil de desfazer não ganha botão de um toque num painel
de leitura. Vai para a janela de Agendas, com passo de confirmação próprio, ou para um menu "…".

**O que o autor pensou:** ao ver os botões "Libertar estado" e "Jogar com Ucrânia" no painel do
país: "Não precisa ser tão fácil libertar território e jogar com outro país. Não é uma decisão
comum; num fluxo comum, ela ocupa espaço na tela e o jogador pode fazer sem querer." Ele sugeriu
pôr libertar numa aba de Agendas e jogar com outro país nos três pontinhos da campanha salva.

**O que muda no jogo:** libertar território já é uma decisão em Agendas, com passo de confirmação
e ponto de restauração antes; trocar de país fica no "…" de cada campanha. Mobilizar segue o mesmo
molde no PR #115; entrar ou sair de bloco, exigir e declarar guerra, no desenho. Ver
[Saves e interface](decisoes/saves-e-interface.md).

## Salvar e voltar atrás é jeito legítimo de jogar

**Decisão (2026-11-17):** o jogo facilita voltar atrás, em vez de punir.

**O que o autor pensou:** ele defendeu o save scumming como tática que deixa o jogador arriscar:
uma guerra arriscada, um país difícil com uma agenda de caminho estreito.

**O que muda no jogo:** pontos de restauração automáticos, antes de decisão arriscada, de salvar
rápido e salvos à mão, com o custo de cada intervalo à vista. Ver
[Saves e interface](decisoes/saves-e-interface.md).

## Nenhum número de memória; fonte e licença anotadas

**Decisão:** todo número dos dados tem fonte, e a licença vai junto do dado. Dado público pode ser
usado mesmo com licença não comercial, mas mantém a licença dele.

**O que o autor pensou:** a regra é do projeto (`CLAUDE.md`, "Dados"). A decisão do autor aqui foi
a da licença do HYDE (uso da terra histórica): marcar os arquivos derivados como CC BY-NC-SA,
separados do resto, que é CC BY-SA.

**O que muda no jogo:** o cenário de 1925 tem um `.md` de fontes por pasta e um bloco de fontes por
arquivo. Quando o dado não pode ser redistribuído, ele não entra: as missões diplomáticas de 1925
saem de uma regra, porque o dado do Correlates of War não pode ir para um repositório público
([Diplomacia e influência](decisoes/diplomacia-e-influencia.md)). Detalhe:
[`README.pt-BR.md`, "Licença"](../../README.pt-BR.md#licença) e `CLAUDE.md`, "Dados".

## Não forçar o resultado histórico que dependeu de política

**Decisão (2026-11-16):** um resultado real que dependeu de política, capital externo, guerra ou
regime não vira meta de calibração. Vira dependência anotada, e o processo é estudado antes de o
mecanismo ser desenhado.

**O que o autor pensou:** quando a coordenação propôs que a China chegasse a 2025 com menos da
metade do emprego no campo por mecanismo doméstico, ele corrigiu: "A industrialização da China só
aconteceu pelos grandes acordos dos anos 1970 e 1980 com as multinacionais, pela nacionalização
posterior das fábricas, pelos atrativos do partido único e pela entrada maciça de capital atrás de
mão de obra barata. Ela não é natural: continuar do jeito que estava não gera aquilo."

**O que muda no jogo:** uma China fechada continua agrária. A industrialização vem quando a causa
existe no jogo (capital estrangeiro, reformas), e um par de testes confere isso: sem a causa não
acontece, com a causa acontece. Ver
[`late-industrialization.pt-BR.md`](../late-industrialization.pt-BR.md).

## Nada copiado de outros jogos

**Decisão:** Victoria 3, Europa Universalis, Civilization e o mod Cold War Era servem só de
referência de qualidade. Nada deles é copiado: regra, texto, número, mapa ou ícone.

**O que o autor pensou:** o autor pediu que toda fase feche comparada com o mod Cold War Era. A
regra de não copiar é do projeto (`CLAUDE.md`, "Dados"): o Cold War Era não tem licença (todos os
direitos reservados), então dele só se comparam sistemas e ordens de grandeza.

**O que muda no jogo:** toda fase fecha com uma tabela de comparação em
[`benchmarks.pt-BR.md`](../benchmarks.pt-BR.md). Os nomes do jogo vêm do mundo real e da
literatura: a cláusula de tratado se chama `Clause` (o nome jurídico), e os tipos de aliança seguem
a ciência política ([Alianças](decisoes/aliancas.md)).

## Interface modular e por teclado

**Decisão (2026-11-16):** em vez de um HUD único, painéis que o jogador arrasta e arruma, com o
layout guardado por usuário; tudo acionável por teclado, com dica de tecla na tela e editor de
atalhos.

**O que o autor pensou:** não existe um HUD "certo" para todo mundo; ele pensou também em quem
transmite o jogo ao vivo. Painel lateral e diário de agendas fixáveis.

**O que muda no jogo:** toda tela nova nasce como painel registrado e com comando no registro de
atalhos. Ver [Saves e interface](decisoes/saves-e-interface.md).

## O trilho histórico que desvia com o jogador

**Decisão (2026-11-16):** sem o jogador tocar em nada, a história real acontece; quando ele age,
tudo vai desviando. Os grandes eventos são um roteiro com pontos: condição para entrar, condição
para sair e forças que empurram. Ir contra elas custa esforço. Entra por último, na fase 7.

**O que o autor pensou:** os exemplos dele: a Alemanha com um partido nacionalista que quer
expansão, os Estados Unidos financiando a reconstrução da Europa, a revolução na China. A crise de
29 nasce do medo do mercado, não de uma data: rodando sozinho, ela vem na janela histórica; se o
jogador mudar muito, pode não vir, vir antes ou depois. Na guerra, o empurrão age só na decisão de
atacar (casus belli e pressão popular), nunca no resultado.

**O que muda no jogo:** um modo só, o histórico, com uma chave "pressão histórica". Só a chave
ligada é calibrada; desligada, o projeto não promete resultado histórico, e a tela diz isso. A
pressão aparece pelo nome em cada decisão que ela empurra. Ver
[`history-rails.pt-BR.md`](../history-rails.pt-BR.md) e
[`confidence.pt-BR.md`](../confidence.pt-BR.md).
