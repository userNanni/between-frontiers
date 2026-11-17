# Pesquisa

Como um país aprende uma técnica, quanto isso custa e por que saber não basta. O detalhe está em
[`research.pt-BR.md`](../../research.pt-BR.md).

## Decisões

### Saber não basta: quatro estágios por país

- **Decisão:** cada tecnologia passa, país por país, por ciência, protótipo, industrial e difundida.
  Só no estágio industrial os métodos dela podem ser construídos; a produtividade inteira só chega
  difundida.
- **O que o autor pensou:** desenho da fase 2, aceito pelo autor.
- **Como o jogo funciona por causa disso:** sem comprador (o mercado ou o Estado), a tecnologia fica
  parada no protótipo, o vale da morte. Operar e produzir são coisas distintas: depois que algum país
  industrializa um bem, qualquer país compra e usa pelo comércio, mesmo sem saber fazer. Vacina,
  antibiótico e pílula importados baixam a mortalidade e a fecundidade sem fábrica em casa.
- **Como é na vida real:** a escala de maturidade tecnológica (TRL) inspira os estágios; a difusão
  barateia o que os outros já sabem.
- **Estado:** no jogo (#30, #32).
- **Onde está o detalhe:** [`research.pt-BR.md`, "A ideia"](../../research.pt-BR.md#a-ideia).

### Uma árvore só, com as áreas se alimentando

- **Decisão:** 168 tecnologias em 12 campos e 6 eras formam uma árvore única. Pelo menos 40 % dos
  requisitos cruzam de área, e toda área alimenta outras.
- **O que o autor pensou:** o autor gostou da árvore única com dependências entre áreas, que é o
  oposto das linhas separadas que ele viu no mod Cold War Era.
- **Como o jogo funciona por causa disso:** o militar passa o radar, o foguete e a navegação por
  satélite para o civil; a escola e a universidade, para a ciência; a mineração, para os materiais.
  A tela é um grafo de nós, com o caminho até a tecnologia em foco aceso.
- **Como é na vida real:** as áreas do conhecimento se alimentam umas das outras.
- **Estado:** no jogo (#31, #33).
- **Onde está o detalhe:** [`research.pt-BR.md`, "A árvore de 1925 a 2025"](../../research.pt-BR.md#a-árvore-de-1925-a-2025)
  e ["A tela"](../../research.pt-BR.md#a-tela); `data/research/technologies.ron`.

### Pesquisa é gasto: laboratório, pesquisador e quem paga

- **Decisão:** o esforço de pesquisa vem dos pesquisadores que os laboratórios empregam, pagos pelas
  empresas (pelo valor adicionado e pela proximidade da fronteira) e pelo Estado (parte da receita,
  mais com as leis de capitalismo de Estado e de planificação, e com o peso que o jogador dá à
  pesquisa no orçamento). O pesquisador custa o salário de mercado do país.
- **O que o autor pensou:** decisão da coordenação para corrigir uma distorção: com a regra antiga,
  de cabeça contada, uma China pobre e fechada pesquisava mais que os Estados Unidos. Entrou abaixo
  do limiar combinado, pela regra de funcionalidade antes de equilíbrio.
- **Como o jogo funciona por causa disso:** um país pobre pesquisa pouco porque paga pouco, e um
  país longe da fronteira copia em vez de inventar. O jogador muda a pesquisa do Estado pelo
  orçamento.
- **Como é na vida real:** a escala do P&D das empresas vem da NSF (Estados Unidos, desde 1953).
- **Estado:** no jogo (#107). O que não fechou (pessoal de P&D americano abaixo do real, P&D militar
  da guerra fria, tecnologias de programa estatal) está anotado como dependência.
- **Onde está o detalhe:** [`research.pt-BR.md`, "Pesquisa que custa"](../../research.pt-BR.md#pesquisa-que-custa).

### Manter a conversão da árvore

- **Decisão:** com a regra nova, os custos da árvore foram convertidos para a unidade nova por
  script, uma troca de unidade, não ajuste por tentativa.
- **O que o autor pensou:** o autor decidiu manter a conversão.
- **Como o jogo funciona por causa disso:** as datas da fronteira ficam perto das de antes; o que
  mudou de sinal fica anotado.
- **Estado:** no jogo (#107); `tools/research/convert_tree.py`.
- **Onde está o detalhe:** [`research.pt-BR.md`, "A árvore na unidade nova"](../../research.pt-BR.md#a-árvore-na-unidade-nova).

### O computador é eficiência, não capacidade

- **Decisão:** computador e IA serão métodos nos prédios que pensam (laboratório, universidade e a
  administração do Estado), cada um comprando os bens correspondentes: cálculo manual, computador
  central, microcomputador e rede, IA.
- **O que o autor pensou:** o país sem computador pesquisa, mas com perda enorme de escala; a
  dinâmica dos fluidos foi feita à mão, com cálculos longos. A IA, do mesmo jeito, diminui o tempo
  de fazer tudo. Ele sugeriu modelar isso como métodos que aumentam a eficiência da administração
  do governo e das instituições de pesquisa e universidades.
- **Como o jogo funciona por causa disso:** quem não tem o bem fica no método antigo, mais lento,
  sem ser travado. Hoje, enquanto isso não entra, um efeito global das tecnologias de informática
  multiplica os pontos de pesquisa sem comprar computador nenhum. A mesma ideia já entra no serviço
  exterior: a informática multiplica as negociações simultâneas
  ([Diplomacia e influência](diplomacia-e-influencia.md#por-que-não-se-agrada-todo-mundo-ao-mesmo-tempo)).
- **Estado:** desenhado, para depois da fase 6.
- **Onde está o detalhe:** [`research.pt-BR.md`, "Depois: computação como método"](../../research.pt-BR.md#depois-computação-como-método).

### Técnica também viaja: comércio, fábrica estrangeira, paz e espionagem

- **Decisão:** além da pesquisa própria, a técnica chega pela imitação (mais rápida onde se lê e
  onde se comercia com quem sabe), pela fábrica estrangeira, por técnicas levadas numa paz e pela
  espionagem.
- **O que o autor pensou:** ver [Capital entre países](capital-entre-paises.md#a-fábrica-estrangeira-ensina-técnica-como-efeito-de-primeira-ordem)
  e [Fronteiras e países](fronteiras-e-paises.md#o-país-novo-herda-a-técnica-até-o-protótipo-e-técnica-pode-ser-levada).
- **Estado:** imitação e conhecimento estrangeiro pelas leis de comércio e capital no jogo (#32,
  #79); fábrica estrangeira, paz e espionagem desenhadas.
