# Como manter a wiki

A wiki só vale se acompanhar o jogo. A regra vale para pessoas e agentes.

## A regra

1. **PR que implementa ou muda uma decisão do autor atualiza a página da wiki no mesmo PR:** a
   frase da decisão, o **Estado** (com o número do PR) e os links de **Onde está o detalhe**.
2. **Decisão nova entra na wiki quando o autor decide**, não quando o código chega. O Estado fica
   "desenhado" ou "em estudo" até o código existir.
3. **[`estado.md`](estado.md)** muda quando uma fase ou frente muda de estado. Troque a data no topo.
4. **Contradição** entre a wiki, o documento técnico e o código: não escolha sozinho. Corrija o que
   for erro de cópia e leve o resto para quem decide.

## O molde de uma decisão

```markdown
### Título curto

- **Decisão:** uma frase.
- **O que o autor pensou:** a frase dele entre aspas, polida (veja abaixo), e o raciocínio. Se a
  decisão foi técnica ou uma recomendação aceita, diga.
- **Como o jogo funciona por causa disso:** o que o jogador vê e o que a simulação faz, com um
  exemplo concreto.
- **Como é na vida real:** o caso ou a fonte.
- **Estado:** no jogo (PR #N) / em rascunho (PR #N) / desenhado / em estudo.
- **Onde está o detalhe:** link para a seção do doc técnico e para o dado.
```

## A frase do autor é polida, não literal

O autor escreve rápido e às vezes de modo prolixo, e pediu que a frase dele entre polida na wiki.

- Reescreva a frase enxuta e correta, com o mesmo sentido e o exemplo que ele deu. Primeira pessoa
  quando ajudar: "Quem promete tem de arcar com o que prometeu."
- Não invente posição que ele não tomou, nem junte duas frases numa conclusão nova.
- Na dúvida sobre o sentido, use discurso indireto: "o autor pediu que…".

## O que não entra

- Nome, e-mail ou dado pessoal do autor. Ele é "o autor".
- Frase atribuída ao autor que ele não disse, ou que muda o sentido do que disse.
- Número sem link para o documento, o dado ou o PR de onde vem.
- Regra, texto ou número de outros jogos. Pode dizer com o que o autor comparou.
- Cópia do documento técnico. A wiki explica o porquê e o efeito para o jogador, e aponta para o
  detalhe.

## O que fica para depois

- **Referência gerada:** bens, prédios, tecnologias, leis e cláusulas serão gerados dos dados em
  `data/` por script. Não escreva essas listas à mão.
- **Guias de como jogar:** depois da primeira versão estável.
- **Publicação** (por exemplo, com mdBook): um PR próprio. Por enquanto, Markdown simples que lê
  bem no GitHub, com links relativos.
