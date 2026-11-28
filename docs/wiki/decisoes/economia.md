# Economia

Como os mercados fecham, quem investe, quanto custa crescer e de onde vêm os números de 1925. O
detalhe está em [`economy.pt-BR.md`](../../economy.pt-BR.md) e em
[`state.pt-BR.md`](../../state.pt-BR.md) (orçamento, obra pública, moeda e dívida).

## Decisões

### Mercados que fecham toda semana, com atores autônomos

- **Decisão:** cada país é um mercado que fecha preço toda semana pela oferta e pela demanda, com
  estoques e comércio mundial. O investimento privado, o trabalho e a demografia rodam todo mês, por
  conta própria, para todos os países, com as mesmas regras da IA e do jogador.
- **O que o autor pensou:** princípios do projeto desde o início: a paz é o jogo, e a IA não conjura
  recurso.
- **Como o jogo funciona por causa disso:** o jogador não constrói cada fábrica; ele mexe em leis,
  impostos, orçamento e prioridades, e o investidor privado responde ao lucro. Uma fábrica só é
  construída onde o tipo já roda cheio.
- **Estado:** no jogo (#23, #29); o passo a passo de cada semana está em
  [Tempo e ordem de cálculo](tempo-e-ordem-de-calculo.md#a-ordem-no-main).
- **Onde está o detalhe:** [`economy.pt-BR.md`, "Ordem de trabalho do mercado"](../../economy.pt-BR.md#ordem-de-trabalho-do-mercado).

### Crescer custa: capital, troca de método e desgaste

- **Decisão:** um nível de fábrica custa alguns anos do que ele adiciona na produtividade em que vai
  rodar; trocar de método é investir e paga o equipamento novo; o capital se gasta todo ano.
- **O que o autor pensou:** correções da coordenação para o crescimento não disparar quando o 1925
  ficou no tamanho certo.
- **Como o jogo funciona por causa disso:** um país rico paga mais por fábrica nova, porque ela
  produz mais; mecanizar não é de graça e só compensa onde o trabalho é caro.
- **Como é na vida real:** a razão capital-produto e a depreciação das economias reais dão a ordem dos
  números.
- **Estado:** no jogo (#29, #55).
- **Onde está o detalhe:** [`economy.pt-BR.md`, "Investimento e crescimento"](../../economy.pt-BR.md#investimento-e-crescimento).

### Capital por método

- **Decisão:** cada prédio pode declarar quanto capital custa um nível em cada método. Onde declara,
  a escolha de método pesa o lucro sobre esse capital, e a obra paga por ele; onde não declara, nada
  muda.
- **O que o autor pensou:** o autor disse sim à proposta.
- **Como o jogo funciona por causa disso:** sem isso, a usina nuclear, que faz mais eletricidade com
  o mesmo pessoal, ganhava a escolha sem pagar o capital a mais, e em 2020 toda a eletricidade do
  mundo era nuclear. Com os valores das usinas, o problema muda de lado (o gás toma tudo), porque
  faltam o preço regional do combustível, a idade da usina e o despacho; por isso os valores das
  usinas ficam guardados até essas peças existirem.
- **Como é na vida real:** o custo de capital por kW de cada tipo de usina (EIA, 2020).
- **Estado:** a regra no jogo (#111), neutra enquanto nenhum prédio declara; os valores das usinas
  num ramo à parte.
- **Onde está o detalhe:** [`economy.pt-BR.md`, "Capital por método"](../../economy.pt-BR.md#capital-por-método).

### O que era regra do Estado passa para a mão do jogador

- **Decisão (2026-11-17):** dez regras do Estado viram decisão do jogador, com a regra de hoje como
  padrão ("regra histórica") e a opção de fixar o valor, como já acontece com os impostos: tamanho
  da folha, salário público, compras do Estado e a parte militar, excedente e reserva do Tesouro,
  teto de déficit, calote e reestruturação, câmbio, fila de obras públicas, dívida interna ou
  externa, e construir tropas de propósito (D1 a D9 e D13).
- **O que o autor pensou:** ele leu a lista do que o país decidia sozinho e escolheu o que passa ao
  jogador, com notas: o clero nem sempre é folha do governo, depende da lei de religião e Estado; a
  fila de obras é "uma lista de prioridades que o jogador define e em que pode ordenar tudo"; o
  mercado decide quem compra a dívida, e o jogador escolhe onde emite; o calote é decisão rara, em
  Agendas; o câmbio entra com o padrão-ouro; o efetivo de paz continua pela lei.
- **Como o jogo funciona por causa disso:** quem não mexe joga com a regra histórica; quem quer
  governar a fundo fixa a folha, o déficit ou a ordem das obras no Orçamento, e a regra deixa de
  valer para aquele item. Continuam por regra a correção fiscal, o recorte de zeros da moeda e a
  bolsa colonial.
- **Estado:** decidido; simulação e telas a fazer (o câmbio espera o padrão-ouro da fase 4).
- **Onde está o detalhe:** [O que o jogador decide](../o-que-o-jogador-decide.md), seção "O que o
  autor decidiu", que entra com o PR #137.

### Sem industrialização por regra

- **Decisão:** o jogo não constrói por regra o que o país importa. Industrializar pede capital,
  técnica ou um Estado que planeje.
- **O que o autor pensou:** o princípio de [não forçar o resultado histórico](../principios.md#não-forçar-o-resultado-histórico-que-dependeu-de-política).
- **Como o jogo funciona por causa disso:** a regra de substituir importações foi testada e
  retirada: industrializava colônias sem capital nem política por trás. A industrialização tardia vem
  com o capital estrangeiro e com o regime.
- **Estado:** decidido; o capital estrangeiro está entrando ([Capital entre países](capital-entre-paises.md)).
- **Onde está o detalhe:** [`economy.pt-BR.md`, "Saída do campo"](../../economy.pt-BR.md#saída-do-campo).

### Desequilíbrio que espera funcionalidade fica anotado

- **Decisão:** as colônias ricas demais e outros desvios medidos ficam anotados com o número, à
  espera do sistema que falta, em vez de corrigidos por ajuste.
- **O que o autor pensou:** [funcionalidade antes de equilíbrio](../principios.md#funcionalidade-antes-de-equilíbrio):
  talvez a falta de alguma funcionalidade seja o que desequilibrou o jogo.
- **Como o jogo funciona por causa disso:** a regra de salário e técnica colonial foi estudada e
  estacionada; a bolsa colonial (escola e saúde da colônia pagas pelo imposto colonial) entrou e já
  corrigiu metade do excesso.
- **Estado:** estudo anotado (#84); bolsa colonial no jogo (#82).
- **Onde está o detalhe:** [`economy.pt-BR.md`, "Colônias: salário e técnica"](../../economy.pt-BR.md#colônias-salário-e-técnica-estudo-regra-estacionada);
  [`state.pt-BR.md`, "Orçamento colonial"](../../state.pt-BR.md#orçamento-colonial).

### O Estado não emite dinheiro invisível

- **Decisão:** a diferença entre despesa e receita vira título; cada país tem moeda, banco central,
  inflação, câmbio e dívida, com calote quando a conta não fecha.
- **O que o autor pensou:** fatia 3c da política nacional.
- **Como o jogo funciona por causa disso:** o jogador fixa alíquotas, prioridades de obra, a taxa do
  banco central e a autonomia dele; um país que gasta demais paga juros, e quem deve em dólar pode
  dar calote. O caso japonês (dívida alta em moeda própria, comprada em casa) sai das regras.
- **Estado:** no jogo (#40).
- **Onde está o detalhe:** [`state.pt-BR.md`, "Moeda, banco central e dívida"](../../state.pt-BR.md#moeda-banco-central-e-dívida).

### Os dados de 1925 têm fonte, e a licença mais restritiva vence

- **Decisão:** todo número do cenário tem fonte anotada. Quando a mesma fonte traz licenças
  diferentes, o projeto segue a mais restritiva: os arquivos derivados do HYDE (lavoura, pasto,
  floresta, irrigação) ficam sob CC BY-NC-SA 4.0, separados do resto, que é CC BY-SA 4.0.
- **O que o autor pensou:** o autor decidiu marcar o HYDE como CC BY-NC-SA.
- **Como o jogo funciona por causa disso:** o jogo segue gratuito e aberto; quem quiser uso comercial
  dos dados de terra precisa respeitar a licença do HYDE.
- **Como é na vida real:** o pacote do HYDE 3.3 declara CC BY 4.0 num lugar, CC BY 3.0 em outro e CC
  BY-NC-SA 4.0 num terceiro.
- **Estado:** no jogo (#91).
- **Onde está o detalhe:** [`data/scenario/1925/LAND.md`](../../../data/scenario/1925/LAND.md), "Licença do
  HYDE"; [`README.pt-BR.md`, "Licença"](../../../README.pt-BR.md#licença).

### O que o servidor faz, a obra privada e honrar a dívida (alavancas do orçamento)

- **Decisão (2026-11-17):** três pontos do desenho das alavancas do orçamento e da dívida, todos
  como a coordenação propôs.
  1. **O servidor dá capacidade de cobrar.** Abaixo dos servidores que a regra pediria para a
     renda do país, a arrecadação cai na proporção que falta; acima, não sobe. Entra desligada
     (`fiscal_capacity`).
  2. **A obra privada fica fora da fila ordenável.** O jogador ordena só as obras públicas; a
     privada segue na ordem de chegada.
  3. **"Honrar a dívida a todo custo" existe** como postura nas Agendas: o calote automático vira
     austeridade forçada, até a folha cair ao piso.
- **O que o autor pensou:** sem efeito, cortar servidores seria dinheiro de graça (Besley e
  Persson, 2009, sobre capacidade fiscal). Sobre a obra privada: "o governo não tem acesso a essas
  decisões". Honrar a todo custo é uma escolha histórica real (a Romênia de 1981 a 1989 pagou a
  dívida externa cortando o consumo).
- **Como o jogo funciona por causa disso:** a folha tem custo e benefício, e um Estado pequeno
  demais arrecada menos; a fila de obras do jogador mexe só no que o Estado paga; quem quer evitar
  o calote pode, pagando com austeridade.
- **Estado:** desenho ([`state.pt-BR.md`, "Alavancas do orçamento e da dívida"](../../state.pt-BR.md#alavancas-do-orçamento-e-da-dívida-desenho)); o código entra em quatro fatias.

### O governo fixa o método de produção e constrói indústria

- **Decisão (2026-11-28):** o governo pode fixar o método de produção de um tipo de prédio e
  mandar construir fábricas do Estado; as duas coisas nasceram do primeiro teste do autor jogando.
  1. **M1, alcance:** o método fixado vale para toda planta do tipo no país, privada ou do Estado
     (é regulação, não só gestão das estatais).
  2. **M2, custo:** os donos perdem o lucro da diferença, e nada mais. O lado político (os
     industriais contra a regulação) fica para a fase dos grupos de interesse.
  3. **Laissez-faire não fixa:** sob a lei de propriedade `laissez_faire` (a única sem
     propriedade do Estado) o governo não fixa método nenhum. Nas outras quatro (mercado
     regulado, economia mista, capitalismo de Estado, economia planejada) pode. Um método fixado
     antes de o país passar ao laissez-faire fica suspenso, e os donos voltam a escolher.
  4. **E0, modelo da fábrica:** os níveis do Estado ficam dentro do prédio comum (uma siderúrgica
     com níveis privados e níveis do Estado), não num tipo de prédio à parte.
  5. **E1, método da fábrica nova:** usa o mesmo método das outras plantas do tipo e acompanha as
     trocas, do governo ou do mercado; se o governo fixou o método do tipo, usa o fixado e não
     troca; sob o laissez-faire, usa o que der mais lucro, como as privadas.
  6. **E2, prejuízo:** o Tesouro cobre; os níveis do Estado não fecham sozinhos, e fechar é
     comando do jogador.
  7. **E3, privatizar:** depois, como decisão nas Agendas.
  8. **E4, limite:** só o caixa do Tesouro.
- **O que o autor pensou:** "a fábrica do Estado usa o que foi escolhido para as outras até o
  governo ou o mercado atualizarem; se o governo fixar, daí não tem como mudar; se o governo for
  laissez-faire, não pode fixar, e a fábrica usa o que der maior lucro". M1, M2 e E0, E2 a E4 são
  as recomendações da coordenação, aceitas.
- **Como o jogo funciona por causa disso:** em Economia → Métodos, cada prédio mostra "Regra
  histórica (os donos)" ou "Fixado pelo governo". Fixado, as plantas trocam em janeiro, com o
  fundo de investimento do país pagando o equipamento antes das obras novas (a troca é gradual: a
  tela diz quantos estados já usam o método), e as plantas novas do tipo já nascem com ele. Um
  país sob laissez-faire vê a escolha desligada, com a frase de que a lei de propriedade é que
  decide. Exemplo: a Suíça de 1925 (mercado regulado) fixa o comércio tradicional no lugar das
  grandes lojas, e em janeiro de 1926 os sete estados usam o comércio tradicional.
- **Como é na vida real:** a regulação técnica de um setor inteiro por lei (a eletrificação
  ferroviária obrigatória, o padrão de combustível) cabe na economia mista; a gestão das estatais
  pelo governo, em qualquer regime que tenha estatais.
- **Estado:** método fixado no jogo, regra desligada (`fixed_methods`, PR #NNN); fábrica do Estado
  desenhada (próximo PR).
- **Onde está o detalhe:** [`econ/industry.rs`](../../../crates/bf-sim/src/econ/industry.rs),
  a troca anual em [`econ/yearly.rs`](../../../crates/bf-sim/src/econ/yearly.rs), o teste
  [`tests/fixed_methods.rs`](../../../crates/bf-sim/tests/fixed_methods.rs); as leis em
  [`data/politics/laws.ron`](../../../data/politics/laws.ron).
