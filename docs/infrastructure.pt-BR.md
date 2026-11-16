# Infraestrutura: transporte e eletricidade por estado

Estado: fase 3b. O código está em `crates/bf-sim/src/econ/local.rs`; o frete entra no catálogo em
`crates/bf-sim/src/econ/catalogue.rs`; as fronteiras saem do mapa (`StateDef::neighbours`, gerado
pelo `bf-mapgen`).

## A ideia

Uma ferrovia em São Paulo não leva o minério de uma mina no Amazonas. Transporte (ferrovia,
estrada, porto) e eletricidade são **bens do estado**: feitos e usados ali, com preço próprio.
Fora isso, valem as regras de qualquer bem.

- **Alcance do bem:** `local: true` em `data/economy/goods.ron`. Hoje, transporte e eletricidade.
- **Rede nacional:** a eletricidade passa a correr pelo país inteiro quando o país leva a
  tecnologia da rede nacional (`national_with: "national_grid"`) ao estágio industrial. Antes
  disso, cada estado vive das próprias usinas e das vizinhas.

## Como fecha

Toda semana, para cada bem local:

1. **O estado usa o que faz.** O que sobra e o que falta ficam de cada lado.
2. **As vizinhas por terra recebem parte da sobra.** Metade da sobra é oferecida às vizinhas do
   mesmo mercado que estão em falta, na proporção da falta de cada uma; uma vizinha oferecida a
   mais do que precisa leva o que precisa e todas as ofertas a ela encolhem igual. Nada depende da
   ordem dos estados.
3. **O que ainda sobra** serve aos compradores que não estão em estado nenhum: as compras do
   Estado e a manutenção do capital.
4. **Preço do estado:** sobe com a falta e cai com a sobra, como o preço de mercado; o mercado
   nacional mostra a média ponderada pela demanda.

Estados sem vizinho por terra (ilhas, enclaves) dependem só do que constroem.

## Frete e acesso ao mercado

- **Frete:** minas, fazendas e fábricas que vendem bens comerciáveis compram transporte na
  proporção do valor do que produzem, a preço-base: extração 6 %, agropecuária e indústria pesada
  4 %, indústria leve e militar 2,5 %. Serviços e usinas não pagam frete (a usina manda pelo fio;
  o carvão que ela queima já pagou frete na mina).
- **Sem transporte, não chega ao mercado.** A parcela do frete que falta corta a produção com peso
  de pelo menos 35 %, não só pelo peso do frete no custo: a mina sem ferrovia fica com o minério.

## Quem constrói

- **Empresas:** o investidor avalia a ferrovia ou a usina pelo preço do estado, e só onde as que
  já existem ali andam cheias (antes, era a utilização do país inteiro).
- **Obra pública:** a categoria de infraestrutura leva ferrovia, estrada e usina para os estados
  com mais demanda sem atender, pela fila de obras; a regra antiga (preço nacional alto, onde há
  mais gente por nível) vale para o que já corre pela rede nacional.
- **1925:** ferrovias e usinas são postas por último, onde as fazendas, minas, fábricas e pessoas
  já postas pedem transporte e energia, arredondadas para cima estado a estado; o porto, só em
  estado com costa.

## Troca de método

A troca anual de método (1º de janeiro) vê transporte e eletricidade pelo preço e pela oferta do
estado, como o investidor. Uma troca que pede mais transporte ou energia do que o estado tem
sobrando espera, as de maior ganho primeiro: a fábrica eletrifica quando há usina para ligá-la.
Com a rede nacional, a eletricidade não limita a troca. No primeiro janeiro não há troca: o
mercado ainda não fechou nenhuma semana, e as usinas de 1925 foram dimensionadas para os métodos
com que as fábricas começam.

## Mapa

A lente "Infra" pinta cada estado pela parcela da demanda atendida (vermelho é falta, verde é
atendida), de eletricidade ou de transporte, com o número no tooltip. Lê o estado da simulação e
recolore uma vez por mês.

## Calibração

Teste de 100 anos (`bf-bench --game --years 100 --check-determinism`). O rótulo é a foto de 1º de
janeiro; o PIB é o do ano que acabou de fechar (o de 1934 na linha de 1935). "main, como media"
é o que o bench mostrava antes desta correção; "main, ano fechado" é a mesma rodada lida certo.

| | 1926 | 1927 | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|---|---|
| PIB (US$ tri), main como media | 5,23 | 3,55 | 5,85 | 18,4 | 57,0 | 97,6 |
| PIB, main ano fechado | 3,55 | 3,84 | 6,03 | 19,6 | 58,2 | 97,3 |
| PIB, esta correção | 3,11 | 3,27 | 5,70 | 19,5 | 59,9 | 100,6 |
| População (bi): main / correção | 1,97 / 1,97 | 1,99 / 1,99 | 2,16 / 2,16 | 3,29 / 3,29 | 4,94 / 4,95 | 6,71 / 6,73 |
| Urbanização: main / correção | 0,39 / 0,39 | 0,40 / 0,40 | 0,45 / 0,45 | 0,52 / 0,52 | 0,56 / 0,56 | 0,60 / 0,60 |
| Eletricidade atendida: main / correção | 38 % / 99 % | 97 % / 100 % | 100 % / 100 % | | | |

Determinístico entre contagens de threads e depois de salvar e recarregar em 1975.

- A demanda dos estados fica perto de toda atendida depois de 1945 (transporte e eletricidade);
  sobram em falta poucos estados, quase todos ilhas e enclaves sem vizinho por terra.

### A queda de 1926–27 não existia

- **O PIB do bench atrasava dois anos.** A foto é tirada em 1º de janeiro antes do tick do dia, e é
  nesse tick que o ano fecha: o "1927" era o PIB medido de 1925, e o "1926" era o PIB do cenário
  (Maddison, 5,23 tri), que o setup grava em `gdp_last_year` e que nada mediu. O PIB medido sobe
  todo ano desde 1925. A diferença entre o cenário e o primeiro ano medido existe desde a fase 1
  (#29: 5,23 → 3,60; com a fase 3c, 5,23 → 3,55). Os PIBs de `state.pt-BR.md` e das tabelas
  anteriores estão com esse atraso.
- **A eletricidade em 36–38 %** vinha da troca de método no primeiro tick (1º/1/1925), a preço-base
  e com todo bem "atendido", porque o mercado ainda não tinha fechado: 53 % dos níveis trocavam de
  método antes da primeira semana, quase todos para métodos elétricos (11 mil níveis de ferrovia a
  vapor viravam elétricos, 74 mil de comércio viravam lojas de departamento, linhas de montagem), e
  na sexta semana a demanda de eletricidade era 17 vezes o que as usinas de 1925 faziam.
  Fertilizante e combustível faltavam pelo mesmo motivo. A troca ainda lia eletricidade e transporte
  pela média do país, e não pelo estado. Ver "Troca de método".
- **Usinas de 1925 arredondadas por bloco:** 30 % da demanda de eletricidade ficava em estados sem
  usina nenhuma. Arredondando estado a estado, a eletricidade atendida no primeiro mês vai de 73 %
  a 93 %.
- **PIB de 1925 menor que na main** (3,11 contra 3,55): a troca no primeiro tick dava a metade das
  fábricas, de graça, métodos com mais valor adicionado. A diferença cai a 5 % em 1934, some em
  1964 e vira 3 % a mais de 1994 em diante, com a eletrificação no ritmo das usinas.

### O que fica: o nível de 1925

A economia de 1925 mede 60 % do PIB do cenário (68 % com a troca no primeiro tick). O PIB que
o jogo mostra em 1925 (`gdp_last_year`) ainda é o do cenário, e cai para o medido em 1926; a
pesquisa, os bancos centrais e a razão dívida/PIB também partem dele em 1925. As causas, medidas
no primeiro ano (antes da fase 3c, que não mexe no setup nem na troca de método):

- **O setup dimensiona as fábricas para cerca de 84 % do PIB:** consumo 70 %, investimento 10 %,
  compras do Estado 2,6 %, escola e hospital. A folha do Estado não entra no PIB medido. Valor
  adicionado a preço-base com tudo rodando: 4,0 tri.
- **Indústria e serviços são cortados** (até 40 % por bloco) para caber na mão de obra, porque
  os métodos de 1925 fazem US$ 5,1 mil por trabalhador e a produtividade do bloco supõe US$ 6 mil;
  fazendas e minas ficam dimensionadas para a demanda de antes do corte e sobram.
- **As fábricas rodam a cerca de 80 %** desde a primeira semana, e assim seguem pelo século: o que
  o setup supõe gastar não é o que as famílias, o fundo de investimento e o Estado gastam (a
  renda das famílias é 55 % do PIB, contra 70 % de consumo no setup; as compras militares começam
  em zero; os poços de petróleo de 1925 fazem quase 40 vezes o petróleo pedido).

Fechar isso é redimensionar o setup pelo gasto da simulação, o que mexe no século inteiro e no
fluxo de dinheiro da fase 3c (moeda, banco central, dívida).

## Aberto

- Rede nacional para o transporte (rodovia, contêiner) e alcance maior que um vizinho por semana.
- Porto ligando estados pelo mar e frete por distância no comércio mundial (fase 4).
- Rede elétrica entre países (interligação), e a hidrelétrica presa ao rio (água como recurso).
