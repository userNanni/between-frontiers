# Comércio

Quem vende o quê a quem, por quanto chega, e o que acontece quando a rota fecha. O desenho completo
está em `docs/trade.pt-BR.md` (PR #113).

Até o #113 o comércio mundial era um *pool* por bem: cada país vendia ao mundo, não a outro país.
Com o #113 o comércio passa a ser **com parceiro**: o par de países existe, com frete, tarifa e
preferência, e o valor comerciado por par fica gravado.

## Decisões

### Quem compra reparte entre fornecedores pelo custo de chegada

- **Decisão:** toda semana, cada importador reparte o que procura entre os vendedores pelo custo de
  chegada (preço de quem vende, frete da rota e tarifa de quem compra), de forma suave: o mais
  barato leva mais, não leva tudo.
- **O que o autor pensou:** recomendação aceita (T1).
- **Como o jogo funciona por causa disso:** o país que baixa a tarifa para um parceiro passa a
  comprar mais dele, sem deixar de comprar dos outros. A tela de parceiros mostra os principais
  parceiros por país e por bem e a conta do custo de chegada de cada par.
- **Como é na vida real:** ninguém compra só do mais barato; a parte de cada fornecedor cai com o
  custo de chegada (a elasticidade entre origens; Head e Mayer 2014).
- **Estado:** no jogo (#113); a tela de parceiros vem num PR do cliente.
- **Onde está o detalhe:** `trade.pt-BR.md`, seções 2 e 3; parâmetros em
  `data/economy/purchases.ron` (`trade`).

### O frete segue a rota e cai com a técnica e a logística

- **Decisão:** o frete é pela rota marítima entre os portos, mais o trecho por terra até o porto, e
  cai com a técnica de transporte e com a infraestrutura de cada lado.
- **O que o autor pensou:** "Considere o que pode baixar o custo, como o avanço da navegação e do
  transporte e da logística em geral."
- **Como o jogo funciona por causa disso:** o trecho marítimo barateia com a técnica difundida no
  mundo (o contêiner e o superpetroleiro servem ao exportador pobre também, porque o frete de navio
  é um mercado mundial). O trecho por terra e o porto dependem de cada país: ferrovia, rodovia e
  porto próprios. Quando um estreito fecha, a rota muda e o frete sobe.
- **Como é na vida real:** o contêiner barateou o frete depois de 1960; com Suez fechado (1956–57,
  1967–75), o navio ia pelo Cabo.
- **Estado:** no jogo (#113); as tecnologias que ainda não têm efeito no comércio (navio a
  diesel, graneleiro, ferrovia) entram quando a árvore os ganhar. A rota é calculada na carga;
  estreito fechado no meio do jogo entra quando o fecho de estreito existir.
- **Onde está o detalhe:** `trade.pt-BR.md`, seção 10.

### O parceiro tem inércia, mas a inércia não segura comércio impedido

- **Decisão:** a preferência por um parceiro anda só uma parte do caminho por mês (meia-vida de
  cerca de quatro meses). Embargo, sanção, guerra, bloqueio, não reconhecimento e estreito fechado
  sem alternativa fecham o par na mesma semana; reabrir volta devagar.
- **O que o autor pensou:** aceitou a inércia e lembrou "os casos que impedem o comércio, como os
  embargos".
- **Como o jogo funciona por causa disso:** com Suez fechado por cinco meses, o comprador europeu
  paga o frete mais caro pelo Cabo e quase volta ao que era quando o canal reabre, em vez de trocar
  de fornecedor na semana. Um embargo corta na hora; enquanto dura, a rede comercial se desfaz, e
  depois volta aos poucos, como o comércio depois de uma guerra.
- **Como é na vida real:** contrato, rede e costume seguram o fornecedor; a elasticidade de longo
  prazo é em média o dobro da de curto (Gallaway, McDaniel e Rivera 2003).
- **Estado:** inércia e par fechado no jogo (#113); embargo e sanção dormentes até as cláusulas e a
  guerra existirem.
- **Onde está o detalhe:** `trade.pt-BR.md`, seções 4 e 11.

### Quem promete entrega tem de arcar

- **Decisão:** a promessa de entrega acumulada ("vendo X de Y até tal data") sai da oferta do
  vendedor antes do mercado interno dele. Falta em casa, e só quebra se a produção não der. A
  cláusula guarda um modo (dura ou branda) para o jogador escolher mais tarde.
- **O que o autor pensou:** "Quem promete tem de arcar com o que prometeu. Talvez mais tarde dê
  para escolher isso."
- **Como o jogo funciona por causa disso:** o país que prometeu trigo a um parceiro entrega mesmo
  com o preço do pão subindo em casa. O entregue soma no tratado, e a barra mostra o saldo e o prazo.
- **Como é na vida real:** o trigo e o petróleo soviéticos entregues à Alemanha em 1939–41; o
  *clearing* alemão com os Bálcãs nos anos 1930 (Hirschman 1945).
- **Estado:** desenhado; a cláusula é do thread dos tratados e entra no fechamento por par quando
  existir
  ([Diplomacia e influência](diplomacia-e-influencia.md#favor-devido-é-uma-promessa-num-tratado-ato-único-ou-entrega-acumulada)).
- **Onde está o detalhe:** `trade.pt-BR.md`, seção 5.

### Tarifa e preferência no custo; receita e preço de chegada depois

- **Decisão:** o primeiro PR põe a tarifa e a preferência dos tratados no custo de chegada; a
  receita da tarifa e o preço de chegada no mercado interno vêm no segundo.
- **O que o autor pensou:** ordem dos PRs recomendada pela coordenação e aceita (T3).
- **Como o jogo funciona por causa disso:** preferência imperial, porta aberta dos mandatos e união
  aduaneira passam a mudar quem compra de quem; o efeito no Tesouro chega depois.
- **Como é na vida real:** a preferência imperial de Ottawa (1932), a área da libra, a CEE (1957).
- **Estado:** tarifa e preferência no custo de chegada no jogo (#113); receita e preço de chegada
  no PR 2.
- **Onde está o detalhe:** `trade.pt-BR.md`, seção 9, T3.

### O comércio liga o canal de influência

- **Decisão:** com o comércio por par, o canal de comércio da influência deixa de ser dormente. Não
  se estima o comércio bilateral para preencher o canal antes disso.
- **Estado:** o valor exportado por par, a preço-base, está no jogo (#113,
  `TradeState::exports`); o canal liga no tick da influência; ver
  [Diplomacia e influência](diplomacia-e-influencia.md#canais-que-ainda-não-existem-ficam-dormentes).

### O fechamento é por bem, não por grupo de bens, e custa ~9 % da simulação

- **Decisão:** o comércio por par fecha cada bem com o seu parceiro, não os grupos de bens
  (indústria pesada, alimentos...) em valor. O custo aceito é de ~9 % da CPU da simulação, contra o
  orçamento de 5 % para sistema novo.
- **O que o autor pensou:** escolheu a opção A da decisão T1 (2026-11-18), o fechamento por bem a
  ~9 %, contra a opção B, por grupo, que cabia nos 5 %.
- **Como o jogo funciona por causa disso:** influência, embargo e tarifa leem o par por bem: dá
  para saber de quem um país compra aço e de quem compra carvão, e cortar um sem cortar o outro.
- **O que foi medido:** fechando por grupo, o código do comércio caía de 9,1–9,4 % para 5,2–5,3 %
  da simulação. Mas o PIB mundial ficava de 5 a 8 % abaixo da faixa de reembaralhamento de 1945 em
  diante, de −4 a −9 σ, porque a fração do grupo aplicada a cada bem troca menos que o fechamento
  do próprio bem. Por bem, PIB e população ficam dentro da faixa. O custo está na aritmética por
  par, uns 300 mil pares por semana; os cortes que não mudam o resultado tiraram pouco (#201,
  #204).
- **Estado:** no jogo, ligado (`data/rules.ron`, `trade_by_pair: true`). O fechamento dos bens em
  paralelo entre os núcleos, que não muda o resultado nem a CPU mas baixa a espera do jogador,
  fica para a próxima rodada de performance.
- **Onde está o detalhe:** `trade.pt-BR.md`, seção 8a.

### Na união aduaneira, o serviço atravessa a fronteira (microestados)

- **Decisão:** entre dois países ligados pela cláusula de união aduaneira de um acordo em vigor,
  serviço e finança são vendidos de um ao outro pelo comércio com parceiro, como os bens
  comerciáveis. Fora de uma união, continuam vendidos só em casa.
- **O que o autor pensou:** pediu que a economia dos microestados funcionasse. O orquestrador
  escolheu esta opção entre três (A), e a união negociada na mesa vale igual à de 1925, porque a
  regra lê o acordo em vigor.
- **Como o jogo funciona por causa disso:** o banco de Liechtenstein serve a Suíça e o comércio
  de Mônaco atende a França. San Marino, em união com a Itália desde 1862, entra com a regra
  ligada. Andorra não tinha união e fica de fora.
- **Como é na vida real:** Mônaco vivia de turismo e cassino para estrangeiros, e Liechtenstein
  entrou no território aduaneiro suíço em 1924. O serviço vendido ao vizinho é a economia de um
  microestado.
- **Estado:** no código, desligado (`union_services`), e **não deve ser ligado como está**. Medido
  ligado em 101 anos, o mundo fica na faixa, mas Mônaco perde ~80 % do PIB e Luxemburgo até 75 %:
  o fechamento leva o serviço do sócio grande para o pequeno, não o contrário. O que falta é
  demanda estrangeira pelo serviço do pequeno (cassino, banco, turismo); as opções estão com o
  dono, para depois da fase 6. As alternativas, trabalho que atravessa a fronteira (B) e
  microestado dentro do mercado do vizinho (C), estão anotadas com o porquê.
- **Onde está o detalhe:** `trade.pt-BR.md`, seção 12.

## Fontes

- Gallaway, M. P.; McDaniel, C. A.; Rivera, S. A. "Short-run and long-run industry-level estimates of
  U.S. Armington elasticities". *North American Journal of Economics and Finance* 14 (1), 2003.
- Head, K.; Mayer, T. "Gravity equations: workhorse, toolkit, and cookbook". *Handbook of
  International Economics*, vol. 4, 2014.
- Hirschman, A. O. *National Power and the Structure of Foreign Trade*. University of California
  Press, 1945.
