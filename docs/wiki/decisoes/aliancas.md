# Alianças

Quem promete o quê a quem numa guerra, e o que acontece quando a promessa não é cumprida. As
alianças são acordos como os outros: um feixe de cláusulas ([tratados](../../treaties.pt-BR.md)).
Esta página diz que nomes o jogo usa, quando o aliado entra e quanto custa trair.

## Os nomes da literatura

A ciência política tem duas bases de dados de alianças que o jogo usa como nomenclatura (não como
dado):

**Correlates of War, Formal Alliances** (v4.1, 1816–2012; Gibler 2009). Classifica cada aliança
pelo maior compromisso que ela traz
([página do conjunto](https://correlatesofwar.org/data-sets/formal-alliances)):

| Tipo | Nome | O que obriga |
|---|---|---|
| I | pacto de defesa | ir em socorro militar do aliado atacado por terceiro; o compromisso mais alto |
| IIa | pacto de neutralidade | ficar neutro se o aliado entrar em conflito |
| IIb | pacto de não agressão | não usar nem apoiar a força contra o aliado |
| III | entente | consultar o aliado numa crise ou num ataque; o compromisso mais baixo |

**ATOP, Alliance Treaty Obligations and Provisions** (v5.1, 1815–2018; Leeds, Ritter, Mitchell e
Long 2002). Em vez de um tipo por aliança, marca **cada obrigação de cada membro**, e uma aliança
pode ter várias (mais da metade tem). As obrigações não precisam ser simétricas e podem ser
condicionais: valer só contra um adversário, numa região, ou se o outro não provocou
([codebook v5.1](http://www.atopdata.org/documentation.html)):

| Obrigação | O que o membro promete |
|---|---|
| Defesa (*defense*) | apoio militar ativo se o aliado for atacado no seu território ou na sua soberania |
| Ofensa (*offense*) | apoio militar ativo em qualquer outro caso, sem ataque ao aliado |
| Neutralidade (*neutrality*) | não entrar do lado do adversário do aliado nem ajudá-lo |
| Não agressão (*nonaggression*) | não usar a força contra o aliado para resolver disputas |
| Consulta (*consultation*) | consultar o aliado numa crise que pode virar conflito, para coordenar a resposta |

Os dados das duas bases não entram no jogo. Aqui servem só os nomes e as definições, com a
citação.

## Decisões

### O jogo segue a nomenclatura da literatura

- **Decisão:** os tipos de aliança têm os nomes do Correlates of War e do ATOP, e cada tipo é um
  feixe de cláusulas do catálogo de tratados.
- **O que o autor pensou:** "Use o que a literatura diz, com a nomenclatura do Correlates of
  War." Ele também lembrou que a aliança é mais do que um simples pacto de defesa mútua.
- **Como o jogo funciona por causa disso:** o ATOP já pensa a aliança como feixe de obrigações,
  que é a forma dos tratados do jogo. Cada obrigação vira uma cláusula com quem dá e quem recebe.
  A tabela abaixo é a proposta da coordenação para os nomes que o jogo mostra:

  | Nome | Cláusulas no jogo |
  |---|---|
  | Garantia | `defence` de um lado só, ou `border_guarantee` sobre a fronteira de um terceiro (Locarno) |
  | Pacto de defesa (COW I; ATOP defesa) | `defence` mútua |
  | Aliança | `defence` mútua mais outras cláusulas: `consultation`, `base`, `troops_quota`, `passage`, às vezes `foreign_policy` |
  | Pacto de não agressão (COW IIb; ATOP não agressão) | `non_aggression` |
  | Entente (COW III; ATOP consulta) | `consultation` |
  | Pacto de neutralidade (COW IIa; ATOP neutralidade) | ainda sem cláusula própria no catálogo |
  | Pacto ofensivo (ATOP ofensa) | ainda sem cláusula própria no catálogo |

  Exemplo: a Pequena Entente de 1925 entra no cenário como bloco militar, com tratados de `defence`
  mútua entre os membros. O nome do tipo é um rótulo que a tela calcula das cláusulas; a regra está
  nas cláusulas.
- **Como é na vida real:** a maioria dos acordos reais junta mais de uma obrigação, e muitas são
  desiguais (um protege, o outro cede base). Por isso o jogo não tem "tipos de tratado", e sim
  cláusulas.
- **Estado:** cláusulas `defence`, `non_aggression`, `consultation`, `border_guarantee`, `base`,
  `troops_quota` e `passage` no catálogo (#93), assináveis e **dormentes** até a guerra rodar.
  Neutralidade e ofensa: desenhadas no ramo da crise, ainda sem cláusula no catálogo. O rótulo do tipo na tela: desenhado.
- **Onde está o detalhe:** [`treaties.pt-BR.md`, seção 4](../../treaties.pt-BR.md#4-catálogo-de-cláusulas)
  e [seção 12](../../treaties.pt-BR.md#12-dados-de-1925-no-formato-novo) (os tipos de 1925 viram
  cláusulas); [`influence.pt-BR.md`, seção 5](../../influence.pt-BR.md#5-blocos) (blocos);
  `data/diplomacy/clauses.ron`.

### O aliado só entra quando a cláusula prevê

- **Decisão:** quando estoura uma guerra, o aliado entra **somente quando as cláusulas do tratado
  preverem isso**. Nada de "aliado entra em qualquer guerra".
- **O que o autor pensou:** "O aliado entra somente quando as cláusulas preverem isso."
- **Como o jogo funciona por causa disso:** na declaração, o jogo lê as cláusulas em vigor entre os
  países. Quem tem `defence` com o atacado é chamado; quem tem só `consultation` é consultado, sem
  obrigação de lutar; quem prometeu `non_aggression` a um dos lados quebra a promessa se o atacar.
  A tela da crise mostra quem viria e por qual cláusula. Exemplo: se A tem pacto de defesa com B e B declara guerra a C,
  A não é chamado, porque B não foi atacado.
- **Como é na vida real:** em 1914 a Itália estava na Tríplice Aliança com a Alemanha e a
  Áustria-Hungria, uma aliança defensiva, e declarou neutralidade alegando que os aliados eram os
  agressores. O Pacto de Aço de 1939 entre Alemanha e Itália obrigava ao apoio militar sem a
  condição de defesa; ainda assim a Itália ficou fora da guerra até junho de 1940
  ([Pacto de Aço, Wikipedia](https://en.wikipedia.org/wiki/Pact_of_Steel)). Os dois casos são
  testes de jogo: o primeiro pela cláusula, o segundo pelo preço de não cumprir.
- **Estado:** decidido (W4); desenhado no ramo da crise e declaração (fase 5, fatia B), com os
  testes da Itália de 1914 e do Pacto de Aço. Ver [Crise e guerra](crise-e-guerra.md#quem-entra-e-por-qual-cláusula).
- **Onde está o detalhe:** [`war.pt-BR.md`, "Como começa e como acaba"](../../war.pt-BR.md#como-começa-e-como-acaba);
  a seção "Crise e declaração" entra no mesmo documento com a fatia B.

### Trair é possível, e custa muito caro

- **Decisão:** um país pode deixar de cumprir a aliança. A quebra tem um preço muito alto e medido.
- **O que o autor pensou:** "Um país pode ignorar o tratado e trair o aliado não cumprindo o pacto.
  Quebrar o tratado é possível, mas tem um preço muito alto."
- **Como o jogo funciona por causa disso:** quebra é o teste da cláusula falhar (o aliado chamado
  não entrou). O que já está decidido para toda quebra de tratado:
  1. **registro** no acordo: data, cláusula, quanto;
  2. **suspensão recíproca:** o traído pode suspender o que dava em troca sem quebrar nada;
  3. **causa:** o traído ganha uma causa contra quem quebrou, do tamanho da cláusula quebrada, que a
     crise lê como casus belli;
  4. **reputação:** cada quebra pesa na reputação do país, que só decai devagar; quem avalia uma
     proposta futura dele desconta as promessas pela reputação. Quem trai paga na próxima mesa;
  5. **boa vontade:** a quebra contra B baixa a boa vontade de B com quem quebrou
     ([Diplomacia e influência](diplomacia-e-influencia.md)).

  Na guerra, o jogador chamado por cláusula recebe uma entrada com prazo e duas saídas, honrar ou
  não honrar, com o preço à vista; é decisão rara, com ponto de restauração.

  A IA quebra só quando o valor de quebrar passa esse custo, e a tela mostra a conta ao jogador
  que é a vítima, para a traição não vir sem causa visível.
- **Regra geral:** não cumprir é possível em qualquer acordo, não só na aliança, e custa pela mesma
  regra ([Tratados e acordos](tratados-e-acordos.md#toda-dívida-de-acordo-se-renegocia-e-todo-acordo-pode-não-ser-cumprido)).
- **O que ainda está em desenho:** o preço próprio da traição numa guerra (quanto pesa a mais do
  que uma tarifa acima do teto, o efeito na legitimidade em casa, e se os outros aliados do traidor
  revisam os acordos). Os threads da guerra e dos tratados estão fechando isso.
- **Como é na vida real:** o caso de referência do preço é Munique (1938): a França não honrou o
  tratado com a Tchecoslováquia, Moscou concluiu que não podia contar com o Ocidente e os pequenos
  aliados viram que o tratado não os protegia. A reputação de cumprir é o que pesa na confiança entre Estados (Tomz
  2007, sobre a dívida soberana); a suspensão recíproca é a regra geral do direito dos tratados
  (Convenção de Viena de 1969, art. 60).
- **Estado:** regra geral da quebra desenhada e decidida; preço específico da traição na guerra em
  desenho.
- **Onde está o detalhe:** [`treaties.pt-BR.md`, seção 9](../../treaties.pt-BR.md#9-quando-se-quebra).

### Promessa a muitos vale menos

- **Decisão:** o valor de uma cláusula de defesa cai com o número de defesas que o país já
  prometeu. Algumas cláusulas são exclusivas: aliança defensiva com um exclui a aliança com o
  inimigo dele.
- **O que o autor pensou:** ele pediu o limite real por trás dos diplomatas dos jogos do gênero:
  "Não posso melhorar a minha relação com todo mundo ao mesmo tempo. Quais são esses limites?" A diluição da promessa de defesa foi um dos limites propostos, e ele
  aceitou.
- **Como o jogo funciona por causa disso:** a quinta aliança defensiva de um país vale menos para
  quem a recebe do que a primeira. Uma potência não colhe alianças sem custo.
- **Como é na vida real:** o dilema da aliança (Snyder 1984): o medo de ser abandonado contra o
  medo de ser arrastado.
- **Estado:** decidido; desenhado (termo da avaliação de proposta).
- **Onde está o detalhe:** [`influence.pt-BR.md`, seção 12](../../influence.pt-BR.md#12-por-que-não-se-melhora-a-relação-com-todo-mundo-ao-mesmo-tempo-pedido-do-dono-na-decisão-3).

## Fontes

- Gibler, D. M. *International Military Alliances, 1648–2008*. CQ Press, 2009. Correlates of War,
  Formal Alliances v4.1: <https://correlatesofwar.org/data-sets/formal-alliances>. Usado só pela
  nomenclatura; os termos de uso do COW limitam a redistribuição dos dados.
- Leeds, B. A., Ritter, J. M., Mitchell, S. M. e Long, A. G. "Alliance Treaty Obligations and
  Provisions, 1815–1944". *International Interactions* 28, 2002, 237–260. Codebook v5.1:
  <http://www.atopdata.org/documentation.html>. Usado só pela nomenclatura.
- Snyder, G. H. "The Security Dilemma in Alliance Politics". *World Politics* 36(4), 1984.
- Tomz, M. *Reputation and International Cooperation*. Princeton University Press, 2007.
- Pacto de Aço (1939): <https://en.wikipedia.org/wiki/Pact_of_Steel> (fatos; CC BY-SA 4.0).
- Acordo de Munique (1938): <https://en.wikipedia.org/wiki/Munich_Agreement> (fatos; CC BY-SA 4.0).
