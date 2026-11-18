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
  [Tratados e acordos](tratados-e-acordos.md#dívida-entre-estados-com-nome-e-calendário).

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
- **Como o jogo funciona por causa disso:** se os estados libertados batem com o núcleo de um país
  do catálogo, nasce esse país. Se não (o jogador liberta São Paulo, que não está no catálogo), nasce
  um país genérico, com etiqueta `X00`, `X01`… O desenho dá a ele o nome do estado mais populoso e
  uma bandeira de três faixas desenhada pelo jogo, sem arte de terceiros.
- **Estado:** catálogo e etiqueta genérica no jogo (#95, #96). O nome pelo estado e a bandeira de
  três faixas ainda não: hoje o país genérico aparece sem nome traduzido e com a cor do mapa no
  lugar da bandeira.
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

### Independência por movimento: a colônia exige, a metrópole cede ou recusa

- **Decisão:** o movimento de independência é medido em cada colônia (o país candidato do catálogo,
  ou o súdito). Quando a mobilização passa a marca da **exigência**, a metrópole responde: concede
  governo próprio, solta como domínio, solta independente por acordo, ou recusa. Se recusa e a
  mobilização passa a marca da **revolta**, a colônia se separa sem acordo. A IA da metrópole
  responde por conta medida: o custo de segurar contra o valor da colônia.
- **O que o autor pensou:** aceitou as cinco recomendações (I1 a I5) e acrescentou à resposta da
  metrópole: "no pós-Segunda Guerra foi ficando menos aceitável ter colônias, e ter colônias hoje
  é também um motivo de descrédito mundial". Conferido: o custo material veio das superpotências
  quando quiseram (Indonésia em 1949, Suez em 1956), e o de reputação cresceu com a maioria da ONU
  (a Resolução 1514 de 1960, o isolamento de Portugal e da Rodésia). Hoje, a colônia mantida por
  consentimento local quase não custa; Chagos, sem consentimento, custa.
- **Como o jogo funciona por causa disso:** a Índia e a Gâmbia não sobem juntas; cada colônia tem
  o seu movimento, que nomeia a condição que o segura. Conceder governo próprio (a lei
  `trusteeship`) tira a causa e ganha tempo, porque o apoio só decai devagar. Por acordo, o tratado
  de independência leva as cláusulas da mesa (fábricas, dívida, base); por revolta, nasce sem o
  reconhecimento da metrópole e as fábricas seguem a regra da revolta. O jogador recebe a exigência
  em Agendas, com as quatro saídas, o preço à vista e ponto de restauração. A conta mostra a linha
  "custo internacional": boa vontade perdida com os países anticoloniais, pesada pelo tamanho
  deles, mais a pressão de quem tem influência sobre a metrópole. Sem data: o custo cresce porque
  cada país novo é anticolonial, e a onda acelera sozinha.
- **Como é na vida real:** a pressão cresce por décadas (o Congresso indiano de 1885 à
  independência de 1947); as metrópoles concedem aos poucos (as leis indianas de 1919 e 1935, a
  lei-quadro francesa de 1956, o domínio); a recusa leva à guerra (Indochina, Indonésia, Argélia,
  as colônias portuguesas).
- **Estado:** decidido; desenhado (#133). Até a guerra existir (fase 5), a revolta separa sem
  batalha.
- **Onde está o detalhe:** [`borders.pt-BR.md`, seção 11](../../borders.pt-BR.md#11-parte-c-independência-por-movimento-desenho-decisões-i1-a-i5-tomadas-em-2026-11-17).

### Governo próprio freia a independência, não a apaga

- **Decisão (2026-11-18, I1 a I3 do estudo da onda, #173):**
  - **I1:** conceder governo próprio não segura a colônia para sempre: o limite que ele impõe à
    causa sobe com a alfabetização da colônia.
  - **I2:** o governo próprio continua sendo decidido por dono, mas vale por tipo de vínculo
    (colônia, protetorado, mandato, domínio), e não para todas as unidades do dono de uma vez.
  - **I3 (opção A):** nenhuma regra simplificada de revolta. A revolta armada da colônia, que
    hoje nunca acontece, nasce com a guerra colonial, na fatia C da guerra (fase 5).
- **O que o autor pensou:** aceitou a recomendação do I1. No I2: "depende do tipo de vínculo que os
  países têm entre si".
- **Como o jogo funciona por causa disso:** com a lei de 1925 como estava, 73 das 83 colônias
  ficavam paradas para sempre. Com o limite subindo com a alfabetização, a colônia que estuda volta
  a pressionar. A metrópole pode dar governo próprio aos mandatos sem dar às colônias. Até a
  fatia C da guerra, a colônia se separa por acordo, nunca por revolta.
- **Como é na vida real:** as leis de autogoverno ganharam tempo sem segurar (as leis indianas de
  1919 e 1935, a lei-quadro francesa de 1956), e quem as levou à independência foi uma elite
  alfabetizada. O regime de um mandato não era o de uma colônia. A revolta que separou de fato foi
  guerra (Indochina, Indonésia, Argélia, as colônias portuguesas), e por isso fica com a guerra.
- **Estado:** a regra segue desligada. O I1 está no código do #173 (rascunho), e o I2 está com o
  thread das fronteiras, e o I3 com a fatia C da guerra.
- **Onde está o detalhe:** [`borders.pt-BR.md`, seção 11](../../borders.pt-BR.md#11-parte-c-independência-por-movimento-desenho-decisões-i1-a-i5-tomadas-em-2026-11-17).

## O que ainda não existe

- **Independência por movimento:** decidida e desenhada (acima), ainda sem código.
- **Fábricas da metrópole na independência:** decididas (viram capital estrangeiro), ainda sem
  código; ver [Capital entre países](capital-entre-paises.md#as-fábricas-da-metrópole-na-independência-ficam-como-capital-estrangeiro).
- **Divisão por ocupação** (Alemanha e Coreia em 1945): fica com a paz da fase 5.
