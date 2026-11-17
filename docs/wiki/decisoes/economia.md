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
  a página de tempo e ordem de cálculo (em preparação).
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
- **Estado:** no jogo (#55 e seguintes).
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
