# Confiança: medo do mercado e ciclo emocional

Estado: desenho, **sem código**, com as decisões do dono de 2026-11-16 (fim do documento). Entra no último bloco do plano (fase 7, ver
`architecture.pt-BR.md`), depois de política, diplomacia e guerra. A única parte que pode ser
adiantada é guardar as séries que este sistema vai ler (fim do documento).

A fatia 3c (`state.pt-BR.md`) já tem moeda, banco central, dívida, prêmio de risco, câmbio e o
investimento privado com taxa mínima de retorno. Tudo isso reage a números do presente: o
investidor constrói se o retorno passa do juro real mais 2 pontos, o banco empresta 2 % por semana
da poupança ociosa, o estrangeiro compra título enquanto o prêmio paga. Falta o que fez 1929, 1982
e 2008: **expectativa que se alimenta de si mesma**. Na calmaria todo mundo toma mais risco porque
nada deu errado há anos; quando uma perda passa do limite, todos recuam ao mesmo tempo, e o recuo
produz a perda seguinte. A economia liberal pura (o mercado de Smith, que se ajusta por preço)
explica o equilíbrio; não explica a crise nem a exceção. Este documento propõe o estado que falta.

## A teoria e o que cada fonte dá ao modelo

| Fonte | Ideia | O que vira regra |
|---|---|---|
| Keynes, *The General Theory of Employment, Interest and Money* (1936), cap. 12 | "Espíritos animais": o investimento depende de uma confiança que não é cálculo de probabilidade; com ela baixa, juro baixo não basta | O investimento lê a confiança, não só o retorno e o juro; com medo, cortar a taxa não reacende a obra (armadilha de liquidez já existe no piso zero) |
| Minsky, *Stabilizing an Unstable Economy* (1986); "The Financial Instability Hypothesis" (Levy Institute, working paper 74, 1992) | Estabilidade gera instabilidade: na calmaria o financiamento passa de *hedge* (a renda paga juro e principal) a especulativo (paga só o juro) e a Ponzi (precisa de preço subindo ou de novo crédito) | Medida de fragilidade: parcela da dívida privada cujo projeto não paga o juro corrente. A confiança sobe na calmaria e empurra essa parcela para cima |
| Kindleberger, *Manias, Panics, and Crashes* (1978); *The World in Depression, 1929–1939* (1973) | Ciclo deslocamento → euforia → aflição → pânico; e a Depressão foi mundial e longa porque nenhum país estabilizou o sistema (o Reino Unido não podia, os Estados Unidos não quiseram): mercado aberto para a mercadoria em liquidação, empréstimo anticíclico, câmbio estável, coordenação | Contágio entre países; papel do país que estabiliza (fase 4, diplomacia; e a IA estratégica do hegemon) |
| Fisher, "The Debt-Deflation Theory of Great Depressions", *Econometrica* (1933) | Dívida alta + queda de preços: cada um vende para pagar, o preço cai, a dívida real sobe; quanto mais se paga, mais se deve | A dívida privada é nominal; a deflação (que a 3c já calcula, piso −10 %) aumenta seu peso real e as quebras |
| Bernanke, "Nonmonetary Effects of the Financial Crisis in the Propagation of the Great Depression", *American Economic Review* (1983); Bernanke, Gertler e Gilchrist, "acelerador financeiro" (1996, 1999) | Banco que quebra destrói informação de crédito: o tomador bom também fica sem empréstimo; o patrimônio do tomador cai com a crise e o prêmio de financiamento externo sobe | Crédito bancário cai junto com a confiança dos bancos e com o patrimônio (lucros acumulados) das empresas; o custo do crédito sobe acima do juro do título |
| Diamond e Dybvig, "Bank Runs, Deposit Insurance, and Liquidity", *Journal of Political Economy* (1983); Bernanke, Diamond e Dybvig dividem o Nobel de economia de 2022 | O banco transforma depósito líquido em empréstimo ilíquido; se todos temem que os outros saquem, sacar é racional; seguro de depósito e emprestador de última instância eliminam o equilíbrio ruim | Corrida bancária como limiar da confiança das famílias nos bancos; a lei de seguro de depósito e o banco central como emprestador desligam o limiar |
| Kahneman e Tversky, "Prospect Theory", *Econometrica* (1979); Tversky e Kahneman, *Journal of Risk and Uncertainty* (1992) | A perda pesa mais que o ganho do mesmo tamanho: coeficiente de 2,25 em 1992 (uma meta-análise de 2024 dá perto de 2) | Assimetria: a confiança sobe com o ganho realizado e cai com a perda multiplicada por λ |
| Akerlof e Shiller, *Animal Spirits* (2009); Shiller, *Narrative Economics* (2019) | Cinco fatores: confiança, justiça, corrupção e má-fé, ilusão monetária, histórias; as narrativas se espalham como epidemia (Shiller usa o modelo epidêmico de Kermack e McKendrick) e movem decisões | A confiança se espalha entre mercados ligados (comércio, dívida, moeda) como contágio; a "história" é a crise anterior, que vacina por uma geração e depois é esquecida |
| Eichengreen e Sachs, "Exchange Rates and Economic Recovery in the 1930s" (1985); Eichengreen, *Golden Fetters* (1992) | O padrão-ouro transmitiu a deflação; quem saiu antes do ouro se recuperou antes | Regime monetário como correia de contágio (fase 4 liga o regime a `money::exchange_target`); sair do ouro é uma alavanca do jogador |

As referências estão aqui pelo mecanismo, não pelo texto. Os números citados adiante estão com a
fonte ao lado; o que for estimativa nossa está marcado.

## O modelo

### O estado novo

Por mercado (país; colônias seguem a metrópole), três confianças em milésimos, de −1000 (pânico) a
+1000 (euforia), 0 neutro:

| Confiança | De quem | Onde pesa |
|---|---|---|
| `investor` | dono do fundo de investimento (`Market::pool`): capitalistas e lucros retidos | taxa mínima de retorno (`monthly::hurdle`), quanto do fundo vira obra |
| `bank` | bancos (o crédito da poupança ociosa ao fundo, `BANK_LENDING`) | volume e custo do crédito ao investimento |
| `household` | famílias | poupança de precaução (`SAVE_RATE`), consumo, e o limiar da corrida bancária |

Mais um olhar de fora: a confiança **estrangeira** em cada devedor, que não é estado novo, é a
`investor` dos credores ponderada pelo que cada um empresta ao país, menos o prêmio dele. Ela
corta a compra estrangeira de títulos (`FOREIGN_ABSORB`) e é a "parada súbita" de 1982 e 1997.

E três estoques medidos, também por mercado:

- **Dívida privada** (`private_debt`): o que os bancos emprestaram ao fundo de investimento menos o
  que voltou. Hoje esse dinheiro passa da poupança ao fundo sem deixar rastro; passa a ser um
  empréstimo com juro (o do título mais um prêmio bancário) e prazo. Sem esse estoque não há
  Minsky nem Fisher.
- **Fragilidade** (0 a 1000): a parcela da dívida privada cujos níveis financiados rendem menos
  que o juro corrente do empréstimo. É a passagem de *hedge* a Ponzi, contada nível a nível com o
  lucro que a simulação já calcula (`week.rs`).
- **Memória da crise** (anos desde a última queda de confiança acima de 400): a "vacina" de
  Akerlof e Shiller. Logo depois de uma crise o teto de euforia é baixo; ele sobe à medida que a
  memória envelhece.

O custo é de 7 números por mercado, 83 mercados: nada no orçamento do tick.

### A dinâmica: sobe devagar, cai de uma vez

Todo mês, para cada confiança:

1. **Sinal medido.** Ganho ou perda realizada no mês, contra a tendência de dois anos que a 3c já
   guarda: para o investidor, o lucro do fundo e a utilização das fábricas; para o banco, as
   quebras (níveis fechados por prejuízo com dívida) e o calote privado; para as famílias, o
   desemprego, a queda de renda e as quebras de banco.
2. **Calmaria.** Com sinal positivo, a confiança anda 1/36 da distância até o teto (anos para
   encher). O teto é `memória × crescimento recente`: depois de uma geração sem crise e com
   crescimento acima da tendência, a euforia pode chegar perto de +1000. Os números (1/36, a
   meia-vida da memória de cerca de 15 anos) são estimativa nossa, para calibrar contra os alvos
   abaixo.
3. **Perda.** Com sinal negativo, o passo é `λ × perda × (1 + fragilidade)`, com λ = 2,25 (o
   coeficiente de Tversky e Kahneman, 1992) e sem a amortização de 1/36: a queda é na hora.
4. **Limiar de pânico.** Se a perda do mês passa de `limiar = base × (1 − fragilidade)`, a
   confiança cai de uma vez para `−λ × fragilidade` (um degrau, não uma rampa). Fragilidade alta
   faz uma perda pequena disparar o pânico: é o ponto Minsky. O limiar é determinístico: a mesma
   perda, no mesmo estado, dá o mesmo pânico.
5. **Recuperação.** Da confiança negativa de volta a zero, o passo é de 1/60 por mês com sinal
   positivo (cinco anos; estimativa para calibrar com a duração média das crises sistêmicas, mais
   abaixo). Política encurta: ver "O que o jogador pode fazer".

Não há sorteio em nenhum passo. O ciclo não precisa de choque externo: na calmaria a confiança
sobe, a taxa mínima cai, a obra cresce, a utilização passa da tendência, o banco central de Taylor
sobe o juro (já faz isso, `money.rs`), o juro corrente passa do retorno de parte dos níveis
financiados, a fragilidade aparece, e a primeira perda acima do limiar dispara a queda. O aperto
de 1928 do Federal Reserve e a euforia de 1927–29 são exatamente este caminho.

### Onde a confiança entra no que já existe

Todos os efeitos são multiplicadores sobre regras da 3c; com confiança 0 o jogo de hoje não muda.

| Regra existente | Hoje | Com confiança |
|---|---|---|
| `monthly::hurdle` | max(5 %, juro real + 2 pontos), teto 25 % | `+ prêmio de medo`: até +8 pontos com `investor` = −1000; até −2 pontos com euforia (o investidor aceita projeto pior). A obra privada para em pânico mesmo com juro zero: os espíritos animais de Keynes |
| `BANK_LENDING` (2 % por semana da poupança ociosa ao fundo) | fixo | vezes `(1 + bank/1000)`, de zero (crédito congelado, Bernanke) a 4 % em euforia; o juro do empréstimo privado soma um prêmio que sobe com o medo e com a fragilidade |
| `SAVE_RATE` por profissão (`week.rs`) | fixo | `+ poupança de precaução`: até +10 pontos com `household` em pânico. O consumo cai, a utilização cai, o lucro cai: a demanda que some na Depressão |
| `premium` (`money.rs`) | dívida, inflação, moeda estrangeira, calote | `+ prêmio de medo` dos credores: a confiança estrangeira negativa soma até 6 pontos, e com ela o juro do título e a fuga de capital pelo câmbio (`exchange_target` já enfraquece a moeda 3 % por ponto de prêmio) |
| `FOREIGN_ABSORB` (0,5 % por semana da poupança ociosa de fora) | fixo | vezes a confiança estrangeira (de zero a 1,5 vez): a parada súbita |
| Fechamento de nível com prejuízo (13 semanas) | fecha o nível | o nível financiado com dívida privada, ao fechar, vira **calote privado**: o banco perde, a `bank` cai, o crédito encolhe. É o canal de Fisher e Bernanke |
| Deflação (`Money::inflation`, piso −10 %) | corrói nada; a dívida privada não existe | a dívida privada é nominal: com deflação, o peso real sobe e a fragilidade também |

### Corrida bancária

- Se `household` cai abaixo de −600 e o país **não** tem seguro de depósito (lei nova no grupo de
  bem-estar ou finanças; nos Estados Unidos, o FDIC da lei bancária de 1933) e o banco central **não** age como
  emprestador de última instância, abre-se a corrida.
- Na corrida, as famílias tiram 2 % por semana da poupança ociosa do sistema e a guardam parada
  (entesouramento): ela não compra título nem vira crédito. Os bancos chamam o crédito
  (`private_debt` cai à força), os níveis financiados que não pagam fecham, e uma parcela dos
  depósitos some com os bancos que quebram. Números de partida, para calibrar: estimativa nossa.
- A corrida para quando `household` volta acima de −300 ou quando o banco central empresta aos
  bancos (comando do jogador ou regra histórica, ver abaixo).

### Contágio

A confiança de um mercado puxa a dos outros pelo que os liga, com pesos medidos, uma vez por mês:

- **Comércio:** a queda de demanda já chega pelo comércio mundial; o que muda é que a perda
  realizada pelo exportador conta como sinal negativo na confiança dele (o café de 1929 no Brasil).
- **Dívida:** o credor em pânico chama o crédito externo (a poupança estrangeira para de comprar e,
  na fase 4, os empréstimos curtos vencem): 1931 na Europa Central,
  1982 na América Latina, 1997 na Ásia.
- **Regime monetário (fase 4):** no padrão-ouro, o país que perde reservas tem de subir o juro e
  aceitar a deflação para defender a paridade; a deflação aumenta a dívida real (Fisher) e a
  confiança cai. A correia corta quando o país sai do ouro: a moeda deprecia, o juro cai, a
  deflação para. É o resultado de Eichengreen e Sachs: o mapa da recuperação dos anos 1930 segue a
  data de saída do ouro. Com câmbio atrelado (fase 4), a defesa da paridade contra a fuga de
  capital é o mesmo canal em 1997.
- **Narrativa:** um peso pequeno para a confiança média do mundo ponderada pelo PIB (a "história"
  de Shiller). Sem ele, um pânico em Nova York não chega a Londres no mesmo mês, e chegou.

### Causa visível: "Por que o mercado está com medo"

O painel do país lista, para cada confiança, os termos do último mês, como `Politics::terms` faz
para a postura dos grupos:

```text
Investidores −620 (pânico desde março de 1930)
  perda realizada no mês     −180   lucro do fundo 22 % abaixo da tendência
  fragilidade                 ×1,6  38 % da dívida privada não paga o juro
  contágio: Estados Unidos   −90    peso 0,30 (comércio 0,12, dívida 0,18)
  padrão-ouro                −40    reservas caíram 15 % em 6 meses
  memória da crise           teto +350 (última crise há 4 anos)
Efeito: taxa mínima de retorno 13,4 % (sem medo: 6,1 %); crédito bancário 0,6 % por semana
```

Tudo inteiro em milésimos, sem ponto flutuante e sem sorteio, como a política.

## O que o jogador pode fazer

| Alavanca | Onde está | Efeito |
|---|---|---|
| Emprestador de última instância | comando novo no banco central (`SetLenderOfLastResort`) ou regra da autonomia | interrompe a corrida; a `bank` para de cair; custo: crédito do banco central (inflação depois) |
| Seguro de depósito | lei nova (grupo financeiro) | desliga o limiar da corrida; custo: risco moral (a fragilidade sobe mais rápido na calmaria) |
| Sair do ouro, desvalorizar | comando de regime monetário (fase 4) | corta a correia de deflação e o juro de defesa; custo: prêmio de risco e confiança estrangeira no curto prazo |
| Juro (já existe: `SetPolicyRate`) | banco central | juro baixo na alta alimenta a euforia; juro alto no pânico aprofunda Fisher |
| Gasto público (já existe: obra pública, regra de déficit) | orçamento | segura a utilização e o sinal das famílias; custo: dívida, prêmio |
| Controle de capital | lei `screened_capital` e afins (já no catálogo) | corta a parada súbita; custo: menos poupança estrangeira na calmaria |
| Regulação bancária | lei nova ou efeito no grupo financeiro | teto de alavancagem: a fragilidade sobe mais devagar; custo: menos crescimento na calmaria |
| Agravar | apertar o juro e cortar gasto no pânico, ficar no ouro, deixar banco quebrar | o caminho de 1930–32 nos Estados Unidos e na Alemanha |

A IA de país (fase 6) escolhe essas alavancas pela regra histórica do seu tempo: até os anos 1930
o banco central defende o ouro e não socorre; a lição de 1933 entra como **aprendizado medido**
(a memória da crise muda a regra do banco central para emprestador de última instância), não como
data.

## Alvos de calibração

Para o placar histórico (`tools/history/`, `docs/history.pt-BR.md`). O primeiro placar põe
"sem ciclo nem guerra" como a terceira causa dos desvios (peso 0,59): na Depressão o Ocidente de
colonização europeia cresce +5,5 % ao ano simulado contra −9,1 % real. Este sistema é a resposta
a essa causa, junto com a guerra (fase 5). Cada alvo diz o que a rodada sem
jogador tem de produzir, com a fonte do número histórico. A regra do placar: o modelo acerta a
**ordem de grandeza e o mapa** (quem caiu mais, quem saiu antes), não o número exato.

Fontes: Romer, verbete "Great Depression" da Britannica (2003), tabela de produção industrial;
NBER, datas de ciclo dos EUA; Federal Reserve History (ensaios sobre o choque do petróleo de
1973–74, a crise da dívida latino-americana e a Grande Recessão); van Wincoop e Yi, *Economic
Policy Review* do Fed de Nova York (2000); FMI, *World Economic Outlook* (atualização de julho de
2010); Reinhart e Rogoff, "The Aftermath of Financial Crises", *AER Papers and Proceedings* (2009);
FDIC, história do seguro de depósito.

### 1929–33: a Depressão

Queda da produção industrial do pico ao fundo (Romer, tabela 2):

| País | Queda | Leitura para o modelo |
|---|---|---|
| Estados Unidos | 46,8 % | origem: euforia, aperto de 1928, corrida bancária sem emprestador |
| Polônia | 46,6 % | |
| Canadá | 42,4 % | comércio com os EUA |
| Alemanha | 41,8 % | dívida externa curta, parada súbita de 1931, deflação de Brüning |
| Tchecoslováquia | 40,4 % | |
| Países Baixos | 37,4 % | |
| Itália | 33,0 % | |
| França | 31,3 % | fica no ouro até setembro de 1936 (Acordo Tripartite) |
| Bélgica | 30,6 % | |
| Argentina | 17,0 % | desvaloriza desde 1929 |
| Dinamarca | 16,5 % | |
| Reino Unido | 16,2 % | sai do ouro em 21 de setembro de 1931 |
| Suécia | 10,3 % | |
| Japão | 8,5 % | sai do ouro em dezembro de 1931 |
| Brasil | 7,0 % | desvaloriza desde 1929 |

- **Estados Unidos, do pico ao fundo:** PIB real −30 %, preços no atacado −33 %, oferta de moeda
  −31 % (Romer); ciclo de agosto de 1929 a março de 1933, 43 meses (NBER); cerca de 9 mil bancos
  suspensos de 1930 a 1933 (FDIC); desemprego acima de 20 % no pico (Romer). O PIB volta ao nível
  de 1929 em 1936–37 e à tendência só em 1942 (Romer).
- **Regra do placar:** a ordem das quedas (EUA, Alemanha e Polônia no alto, França perto; Reino Unido,
  Suécia, Dinamarca, Japão e América Latina embaixo) e a correlação com a data de saída do ouro valem
  mais que o número. Alvo proposto: correlação de postos acima de 0,6 entre a queda simulada e a
  histórica nos 15 países; EUA e Alemanha entre −30 % e −55 % de produção industrial; crise
  iniciada entre 1928 e 1931. Os limites são proposta nossa.
- PIB real por país fora dos EUA (Maddison) ainda não conferido: entra quando o placar tiver a
  série.

### 1973–75: o primeiro choque do petróleo

- Petróleo de US$ 2,90 o barril antes do embargo a US$ 11,65 em janeiro de 1974 (Federal Reserve
  History). Recessão dos EUA de novembro de 1973 a março de 1975, 16 meses (NBER).
- **Para o modelo:** choque de preço real (o mercado já faz), mais a perda realizada dos
  importadores entrando na confiança; com euforia baixa (a memória de 1929 já velha, mas Bretton
  Woods acabando) a queda é moderada. Alvo: recessão nos importadores grandes entre 1973 e 1975,
  rasa, sem corrida bancária; inflação alta ao mesmo tempo (a 3c já soma o índice de preços).
  Profundidade do PIB a conferir.

### 1980–82: a dívida latino-americana

- Juro do Federal Reserve em média de 19,1 % em junho de 1981; recessão dos EUA de julho de 1981 a
  novembro de 1982 (NBER). Em agosto de 1982 o México avisa que não paga uma dívida de cerca de
  US$ 80 bilhões; 16 países latino-americanos reescalonam (Federal Reserve History).
- PIB per capita da América Latina na década: as fontes divergem (cerca de −8 % em Cardoso e
  Fishlow, 1992); a série da CEPAL ainda não foi conferida.
- **Para o modelo:** é o caso da confiança estrangeira: dívida em dólar acumulada nos anos 1970
  (poupança dos exportadores de petróleo emprestada), juro mundial sobe, a confiança dos credores
  cai, parada súbita, calote (a 3c já tem calote e o pecado original). Alvo: calote de México,
  Brasil e Argentina entre 1981 e 1984, uma década de renda per capita parada ou em queda.

### 1997–98: a crise asiática

- O baht flutua em 2 de julho de 1997 e vai de cerca de 25 a 55 por dólar (Banco da Tailândia).
- PIB de 1998 (van Wincoop e Yi): Coreia −5,8 %, Malásia −7,5 %, Indonésia e Tailândia pior que
  −10 %. Outras revisões dão a Indonésia perto de −13 %.
- **Para o modelo:** câmbio atrelado (fase 4) com dívida externa curta em dólar; a primeira
  desvalorização é perda realizada dos credores, que puxam a confiança nos vizinhos pelo peso da
  dívida (contágio sem comércio forte entre eles). Alvo: queda de 5 % a 15 % do PIB em um ano em
  três ou mais economias do Leste Asiático com câmbio atrelado e dívida curta; recuperação em dois
  a três anos.

### 2007–09: a Grande Recessão

- EUA: PIB real −4,3 % do quarto trimestre de 2007 ao segundo de 2009, desemprego de 5 % a 10 %
  (Federal Reserve History); ciclo de dezembro de 2007 a junho de 2009, 18 meses (NBER). PIB
  mundial −0,6 % em 2009 (FMI, julho de 2010).
- **Para o modelo:** euforia longa (memória de 1929 apagada, banco autônomo de juro baixo),
  fragilidade alta no crédito privado, pânico bancário contido por emprestador de última instância
  e seguro de depósito. A diferença para 1929 tem de sair das alavancas que existem em 2008 e não
  em 1929. Alvo: queda de 3 % a 6 % nos EUA, não de 30 %; se a rodada sem jogador der 1929 em 2008,
  o modelo erra a política, não a confiança.

### A média das crises bancárias sistêmicas

Reinhart e Rogoff (2009): produto cai 9 % ou mais do pico ao fundo; ações −55 % em cerca de três
anos e meio; imóveis −35 % em seis anos; dívida pública real +86 %. Serve de alvo para o conjunto
das crises que a rodada produzir, não para uma só. O jogo não tem preço de ações nem de imóveis: o
alvo que vale é o do produto e o da dívida pública.

## Desempenho e determinismo

- Cadência mensal, 7 números por mercado, contágio sobre uma matriz 83 × 83 de pesos recalculada
  uma vez por ano. Custo desprezível ao lado do mercado semanal.
- Entra no hash e no save (`#[serde(default)]`), como a 3c.
- Teste de regra (sem rodada longa): um mercado com fragilidade alta e uma perda pequena entra em
  pânico; o mesmo mercado com seguro de depósito e emprestador de última instância não tem
  corrida; dois mercados ligados por dívida contagiam na ordem certa; a mesma entrada dá o mesmo
  resultado com 1 e com 16 threads.

## O que pode ser adiantado barato

Guardar desde já, no `YearRecord` (e numa série mensal curta por mercado), o que a confiança vai
ler, para que a calibração tenha histórico quando o sistema entrar:

- lucro do fundo de investimento e fechamentos de nível por prejuízo;
- crédito dos bancos ao fundo (o fluxo que já existe) e o juro do título;
- utilização e índice de preços (já há a tendência; falta a série);
- desemprego por país;
- reservas e regime monetário, quando a fase 4 ligar.

São campos de leitura, sem efeito na simulação: não mudam o hash do que existe além dos campos
novos.

## Decisões do dono

Todas decididas em 2026-11-16, como recomendado:

1. ~~Três confianças por país~~: investidor, banco e família.
2. ~~Dívida privada~~: entra como estoque novo.
3. ~~Ciclo sem choque externo~~: sim (Minsky).
4. ~~Janela da crise~~: não se crava data. Rodando sozinho, a crise vem na janela (início entre
   1928 e 1931); uma intervenção grande do jogador pode evitá-la, adiantá-la ou atrasá-la. A
   entrada histórica da Depressão (`history-rails.pt-BR.md`, fio 1) não soma pressão histórica: é
   só causa.
5. ~~A IA aprende com a crise~~: sim, pela memória da crise, sem data.

Nenhuma decisão aberta neste documento.
