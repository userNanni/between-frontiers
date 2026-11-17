# Fronteiras e países

Como o mapa muda de dono, como um país nasce e morre e como o jogador troca de país. O detalhe está
em [`borders.pt-BR.md`](../../borders.pt-BR.md).

A regra de base: **nenhuma fronteira muda por data.** As datas reais (a independência da Índia, o
fim da URSS) ficam em `data/history/borders.ron` como referência para o placar. No jogo, o mapa só
muda por uma causa: um tratado, uma paz, um movimento de independência, uma entrada histórica com
condição, ou o jogador libertando o próprio território.

## Decisões

### Quatro operações, sobre estados inteiros

- **Decisão:** toda mudança de mapa é uma de poucas operações (transferir estados, separar estados
  num país novo, anexar um país inteiro, dissolver um país entre herdeiros, mudar o vínculo de
  súdito), sempre sobre estados inteiros, e conserva tudo o que passa de um dono a outro.
- **O que o autor pensou:** recomendação aceita (seção 6, decisão 5). Ele pediu que a operação
  fosse julgada por funcionar e conservar, não por acertar o placar de cada país.
- **Como o jogo funciona por causa disso:** quando a Índia se separa, saem dos estados dela a
  população, as fábricas, a poupança e a diáspora, e a soma mundial antes e depois é a mesma.
  Pedaço menor que um estado não muda de dono; fica anotado como parcial na referência.
- **Como é na vida real:** cessões, independências, uniões e dissoluções do século: a partilha da
  Índia, a reunificação alemã, a Tchecoslováquia de 1993.
- **Estado:** no jogo (#95, #102); as 260 linhas da referência histórica aplicam sem recusa no teste.
- **Onde está o detalhe:** [seção 2](../../borders.pt-BR.md#2-as-operações) e
  [seção 10](../../borders.pt-BR.md#10-dissolução).

### Quem fica com a dívida, e dívida criada por acordo

- **Decisão:** cada operação tem uma regra padrão para a dívida pública. A colônia que se emancipa
  nasce sem dívida; a metrópole que perde estados de si reparte a dívida pela renda; quem anexa
  leva tudo; a transferência de estados não mexe nela. **Além disso, um acordo pode criar ou mudar
  dívida**: paz com reparação, independência que nasce devendo, reconhecimento pago.
- **O que o autor pensou:** o autor aceitou as regras padrão e acrescentou a dívida por acordo:
  a paz com reparação de guerra como a da Primeira Guerra, a independência que gera dívida e o
  acordo de reconhecimento. Para os acordos, ele pediu a qualidade da lógica de tratados de outros
  jogos do gênero como referência, sem copiar.
- **Como o jogo funciona por causa disso:** uma independência pode sair com uma linha no livro de
  dívidas entre Estados (quem deve a quem, juro, calendário). A dívida da metrópole não some quando
  a colônia sai.
- **Como é na vida real:** a dívida da independência do Haiti é o caso extremo; as colônias que
  herdaram empréstimos coloniais, o caso comum; a anexação total passa a dívida inteira (a Alemanha
  de 1990).
- **Estado:** regras padrão no jogo (#95); dívida por acordo no jogo pelo livro de dívidas (#101,
  #104).
- **Onde está o detalhe:** [seção 6, decisão 1](../../borders.pt-BR.md#6-decisões-do-dono-decididas-em-2026-11-17);
  a página de tratados e acordos (em preparação).

### O país novo herda a técnica até o protótipo, e técnica pode ser levada

- **Decisão:** o país novo herda o que o antigo dono sabe limitado a protótipo, salvo as técnicas
  que as fábricas dos seus estados já usam. Toda operação aceita também técnicas levadas de um país
  a outro.
- **O que o autor pensou:** o autor aceitou a herança e pediu que se considerassem os eventos de
  transferência de técnica: a Operação Paperclip dos Estados Unidos e o roubo de projetos pelos
  soviéticos (espionagem).
- **Como o jogo funciona por causa disso:** a colônia que vira país sabe o que a metrópole sabe e
  opera o que já operava, mas não herda a indústria da metrópole. Numa paz, o vencedor pode exigir
  técnicas em que o derrotado está à frente; a espionagem sobe um estágio, nunca acima do alvo.
- **Como é na vida real:** cientistas e engenheiros alemães levados para os Estados Unidos e para a
  URSS depois de 1945; a bomba soviética e a espionagem.
- **Estado:** herança no jogo (#95). Técnica levada: o termo existe nas operações; a paz é da fase 5
  e a espionagem, da fase 4, ainda desenhadas.
- **Onde está o detalhe:** [seção 6, decisão 6](../../borders.pt-BR.md#6-decisões-do-dono-decididas-em-2026-11-17).

### O jogador liberta só o próprio território

- **Decisão:** o jogador pode criar um país, independente ou súdito seu, com estados que são dele.
  Tirar território de outro só por tratado ou paz.
- **O que o autor pensou:** recomendação aceita (seção 6, decisão 3); depois, o pedido de que a
  ação fosse rara: "Não precisa ser tão fácil libertar território e jogar com outro país."
- **Como o jogo funciona por causa disso:** em Agendas (J), "Libertar território" lista os países
  do catálogo cujo território o jogador tem, ou um estado só; mostra o que sai (pessoas, fábricas,
  dívida) e pede confirmação num passo próprio, com ponto de restauração antes. O jogo recusa
  libertar todos os estados do próprio país.
- **Estado:** no jogo (#96, #108).
- **Onde está o detalhe:** [seção 8, "Onde ficam as ações"](../../borders.pt-BR.md#onde-ficam-as-ações-decisão-do-dono-de-2026-11-17).

### Trocar de país a qualquer momento

- **Decisão:** o jogador pode trocar de país quando quiser, e escolhe outro quando o seu deixa de
  existir (anexado ou derrotado por uma revolta).
- **O que o autor pensou:** os casos que o autor deu: começar com a URSS, libertar uma república e
  jogar com ela; começar com o Brasil, libertar São Paulo e jogar com São Paulo; jogar como súdito
  e sair do controle do suserano. Derrotado por uma revolta, poder virar o país novo.
- **Como o jogo funciona por causa disso:** a troca fica no "…" da campanha, na tela de Campanhas
  ("Jogar com outro país"). Quem joga com qual país é escolha do jogador, não do mundo: a
  simulação trata todos os países pelas mesmas regras, e trocar não muda o resultado. Se o país do
  jogador some, o jogo salva e abre a escolha sozinho.
- **Estado:** no jogo (#96, #108).
- **Onde está o detalhe:** [seção 6, decisão 4](../../borders.pt-BR.md#6-decisões-do-dono-decididas-em-2026-11-17).

### Moeda própria na hora, sem salto

- **Decisão:** o país novo tem moeda própria desde o primeiro dia, com os números do antigo dono
  (preços, inflação, câmbio).
- **O que o autor pensou:** recomendação aceita (seção 6, decisão 2).
- **Como o jogo funciona por causa disso:** a independência não faz o preço nem a inflação saltarem
  no dia; a moeda nova se afasta da antiga aos poucos, pela economia do país.
- **Estado:** no jogo (#95).
- **Onde está o detalhe:** [seção 6, decisão 2](../../borders.pt-BR.md#6-decisões-do-dono-decididas-em-2026-11-17).

### Países que a história criou, e os que ela não criou

- **Decisão:** um catálogo de países possíveis traz os que nasceram de 1925 a 2025, com nome, cor,
  capital, bandeira livre e núcleo de estados. O que a história não produziu vira um país genérico.
- **O que o autor pensou:** desenho técnico da coordenação.
- **Como o jogo funciona por causa disso:** se o jogador liberta São Paulo, que não está no
  catálogo, nasce um país com o nome do estado e uma bandeira genérica desenhada pelo jogo, sem
  arte de terceiros. Se os estados batem com o núcleo de um país do catálogo, nasce esse país.
- **Estado:** no jogo (#95, #96).
- **Onde está o detalhe:** [seção 3](../../borders.pt-BR.md#3-países-que-ainda-não-existem).

### Reconhecimento: quem assinou a criação

- **Decisão:** o país novo nasce reconhecido só por quem assinou a criação dele. Os outros
  reconhecem por acordo.
- **O que o autor pensou:** decisão do autor nos tratados (seção 15, decisão 4), que aceitou a
  recomendação.
- **Como o jogo funciona por causa disso:** um país saído de uma revolta não recebe empréstimo de
  governo nem assina aliança até ser reconhecido; o mercado anônimo continua.
- **Como é na vida real:** a doutrina Hallstein e o Tratado Básico entre as duas Alemanhas.
- **Estado:** a lista de quem assinou no jogo (#95); os pares que não se reconhecem, nos tratados.
- **Onde está o detalhe:** [`treaties.pt-BR.md`, seção 6](../../treaties.pt-BR.md#6-reconhecimento).

## O que ainda não existe

- **Independência por movimento:** o primeiro gatilho automático, em que a colônia se separa quando
  o movimento de independência passa um limite e a metrópole não responde. Desenhado (parte C).
- **Fábricas de dono estrangeiro na independência:** em decisão com o autor; ver
  a página de capital entre países (em preparação).
- **Divisão por ocupação** (Alemanha e Coreia em 1945): fica com a paz da fase 5.
