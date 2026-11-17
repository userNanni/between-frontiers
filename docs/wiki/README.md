# Wiki do Between Frontiers

Between Frontiers (Entre Fronteiras) é um simulador de grande estratégia de 1925 a 2025, gratuito e
de código aberto. A economia, a população, a política, a pesquisa, a diplomacia e a guerra são
simuladas com causa visível e sem sorteio onde ele não é preciso. Sem o jogador tocar em nada, o
século deve andar parecido com o que andou; quando o jogador mexe, a história desvia.

Esta wiki explica **por que o jogo é assim**: as decisões que o autor tomou, o que ele pensou e o
que isso muda para quem joga. O detalhe técnico (fórmulas, números, nomes no código) fica nos
documentos de [`docs/`](../), e cada página daqui aponta para eles.

## Como a wiki se organiza

| Página | O que tem |
|---|---|
| [Princípios](principios.md) | as regras de desenho que valem para o jogo inteiro |
| [Estado](estado.md) | as fases 0 a 7: o que já está no jogo e o que falta |
| Glossário *(em breve)* | os termos do jogo e da literatura |
| [Como manter](como-manter.md) | a regra para atualizar a wiki junto com o código |
| `decisoes/` | uma página por tema, com as decisões e o efeito de cada uma |

Páginas de decisão prontas:

- [Alianças](decisoes/aliancas.md)
- [Fronteiras e países](decisoes/fronteiras-e-paises.md)
- [Tratados e acordos](decisoes/tratados-e-acordos.md)
- [Crise e guerra](decisoes/crise-e-guerra.md)
- [Comércio](decisoes/comercio.md)
- [Diplomacia e influência](decisoes/diplomacia-e-influencia.md)
- [Saves e interface](decisoes/saves-e-interface.md)

Em preparação: capital entre
países, população e comunidades, ensino, pesquisa, economia, tempo e ordem de cálculo.

## Como ler uma página de decisão

Toda decisão segue o mesmo molde:

- **Decisão:** uma frase com o que ficou decidido.
- **O que o autor pensou:** a frase dele entre aspas, polida (enxuta e correta, com o mesmo
  sentido; não é transcrição literal), e o raciocínio. Quando a
  decisão foi técnica (da coordenação do projeto) ou uma recomendação que o autor aceitou, a página
  diz isso.
- **Como o jogo funciona por causa disso:** o que o jogador vê e o que a simulação faz, com um
  exemplo.
- **Como é na vida real:** o caso histórico ou a fonte que sustenta a regra.
- **Estado:** no jogo (com o número do PR), em rascunho (PR aberto), desenhado (só no documento) ou
  ainda em estudo.
- **Onde está o detalhe:** o link para a seção do documento técnico e para o dado.

Números só aparecem com o link de onde vêm. Os outros jogos citados (Victoria 3, Europa
Universalis, Civilization, o mod Cold War Era) foram referência de qualidade para o autor; nada
deles é copiado, e a wiki não reproduz regra, texto ou número de nenhum.
