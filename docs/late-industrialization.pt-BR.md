# Industrialização tardia: capital que cruza fronteira, técnica transferida e decisão política

Estado: estudo e desenho, **fora da simulação**. O contrato puro está em
`crates/bf-sim/src/foreign_capital.rs` (fora do tick e do save) e a série de referência em
`data/history/fdi.csv`. As cinco decisões do fim foram tomadas pelo dono em 2026-11-16.

## O pedido

O dono pediu que a industrialização chinesa não saia de um "continuar do jeito que está": que ela
venha dos acordos com as multinacionais, das condições do partido único e da entrada de capital
atrás de mão de obra barata. Na `main`, a China chega a 2025 com 82 a 85 % do emprego na
agropecuária e 2 % na indústria (real: 22 e 32; `history.pt-BR.md`, "Convergência e campo"). O
investimento do jogo é só doméstico.

## O que o estudo concluiu

1. **Nenhuma fase da industrialização chinesa foi espontânea.** Os anos 1950 foram fábricas
   soviéticas inteiras, pagas com empréstimo e poupança forçada. Os anos 1970 foram fábricas
   inteiras compradas do Ocidente e do Japão, depois da reaproximação. A partir de 1979 vieram
   empresas mistas e zonas abertas por lei. Em cada fase, a técnica entrou por um canal aberto por
   decisão política.
2. **O dono tem razão sobre a técnica, a exportação e o emprego da costa, e menos sobre o volume
   do capital.** As empresas de capital estrangeiro passaram de 31,5 % das exportações chinesas em
   1995 para 58,3 % em 2005 e fizeram 92 % da exportação de computadores em 2003. Mas foram só 17 %
   do investimento fixo no pico (1994) e 7 % em 2003, e de 30 a 50 % do "estrangeiro" era capital
   chinês dando a volta por Hong Kong. A maior parte do investimento foi doméstica: poupança de 34
   a 51 % do PIB, mobilizada por bancos do Estado, governos locais e empresas de vila.
3. **A "nacionalização futura" não estava na lei de 1979.** A lei pedia pelo menos 25 % de
   capital estrangeiro, sem teto, presidente do conselho chinês e técnica "avançada". A passagem
   dos ativos ao sócio chinês no fim do prazo é da lei das empresas mistas contratuais, de 1988, e
   só quando o contrato a previa (o estrangeiro recuperava o investimento antes). A garantia
   contra a nacionalização só entrou na emenda de 1990.
4. **O mundo inteiro mostra três vias, nenhuma automática:** capital estrangeiro direto (costa
   chinesa, Sudeste Asiático, Europa Central, maquiladoras); capital nacional com empréstimo,
   ajuda e técnica comprada por licença (Japão, Coreia, Taiwan); e o Estado dono, que compra
   fábricas inteiras e força a poupança (URSS, China dos anos 1950). Quem fechou as três ficou
   parado: a manufatura indiana ficou entre 13 e 17 % do PIB de 1960 a 2024.
5. **Para o jogo:** faltam três peças. A primeira é capital que sai do país quando o retorno lá
   fora é maior. A segunda é técnica que passa por um canal (planta estrangeira, licença, fábrica
   comprada, ajuda), em vez de ser imitada de graça. A terceira é a via do plano, em que o Estado
   constrói sem esperar planta cheia. Cada uma tem sua lei e sua fase.

---

## Parte 1: o estudo

As fontes estão no fim, com chave entre colchetes. Números só de fonte aberta e anotada; o que
ficou sem fonte está dito.

### China, por fase

#### 1950–1960: a ajuda soviética e a indústria pesada do plano

- **Os 156 projetos** foram conglomerados de indústria pesada que replicavam fábricas soviéticas
  inteiras, com máquinas, projetos e treino. Foram o centro do primeiro plano quinquenal
  (1953–57). Valiam US$ 80 bilhões de 2020, ou 45,7 % do PIB chinês de 1949 [GL, §2.1].
- **Não foi doação.** A URSS não deu nada a fundo perdido. Emprestou US$ 300 milhões de 1955, um
  décimo do que a China pediu, e cobrou as máquinas a preço do mercado mundial. A China pagou
  antes do prazo [GL, nota 1 e nota 14].
- **Técnicos e escala.** Registros soviéticos contam 5 092 técnicos na China de 1952 a 1959
  [GL, nota 4]. Os projetos empregavam 5,5 milhões: 3 % da força de trabalho, mas cerca de 40 % do
  emprego industrial de 1952 [GL]. O aço levou 45 % do investimento [GL].
- **Emprego** (primário / secundário / terciário, %): 83,5 / 7,4 / 9,1 em 1952 e 81,2 / 9,0 /
  9,8 em 1957 [NBS 4-3].

#### 1960–1971: a ruptura, a fome e o isolamento

- Em julho de 1960 a URSS retirou os especialistas, que levaram a documentação, e parou de
  mandar máquinas. Veio também um embargo ocidental até pelo menos 1978, e por vinte anos a China
  contou quase só com recursos próprios [GL, pp. 7–8].
- A fome de 1959–61 matou entre 17 e 30 milhões [MQY]. A indústria caiu de 44,3 % do PIB em 1960
  para 31,2 % em 1962, e a formação de capital fixo de 32,6 % para 15,4 % [WDI].
- Ainda assim, o período do plano expandiu a capacidade industrial. De 1955 a 1978 as empresas
  industriais passaram de 125 mil a 348 mil e o emprego fabril de 5,9 para 61 milhões. Melhoraram
  muito a alfabetização, a saúde e a expectativa de vida [BMR].
- O "Terceiro Front", a indústria de defesa no interior, ficou sem número de fonte aberta.

#### 1971–1978: a reaproximação e as fábricas inteiras

- Kissinger foi a Pequim em segredo em 1971 e Nixon visitou a China em fevereiro de 1972 [OH]. O
  Japão normalizou as relações em 29 de setembro de 1972 (comunicado conjunto, só como ponteiro).
- **O "Plano 43".** Em 2 de janeiro de 1973, a Comissão de Planejamento pediu importar fábricas
  completas no valor de US$ 4,3 bilhões em três a cinco anos. Eram 26 projetos: 7 complexos
  petroquímicos e de fibra, 13 fábricas de fertilizante, 3 usinas elétricas (870 MW) e 1
  laminador de 2 Mt, entre outros. A importação terminou entre 1976 e 1982, e a produção
  "abasteceu de roupa, comida e eletricidade" o começo dos anos 1980 [Ma, pp. 18–20]. A embaixada
  chinesa cita o laminador de 1,7 m da siderúrgica de Wuhan [Emb].
- **O "salto para fora".** Em 1977–78, sob Hua Guofeng, as propostas de importação subiram para
  US$ 6,5 bilhões, depois 15 e 18 bilhões (1978–85). Baosteel foi planejada em 1977. A delegação a
  Hong Kong e Macau propôs as zonas de exportação que viraram as zonas econômicas especiais
  [Ma, pp. 24–25].
- Fertilizante químico usado: de 8,8 para 17,4 Mt entre 1978 e 1984, parte das fábricas do Plano
  43. Cereais: de 304,8 para 407,3 Mt [Ma, tabela 1].

#### Dezembro de 1978

O terceiro pleno do 11º Comitê Central (18 a 22 de dezembro de 1978) é a data convencional da
reforma. Ma argumenta que o pleno não trouxe política econômica nova: Hua já tinha anunciado, em
10 de novembro, a virada para o desenvolvimento a partir de 1º de janeiro de 1979. O peso do pleno
foi político, a ascensão de Deng [Ma, pp. 25–26]. Para o jogo, os marcos são as leis, não o pleno.

#### A lei das empresas mistas de 1979

Aprovada em 1º de julho de 1979 e em vigor em 8 de julho. O texto original de 1979 não foi aberto
palavra por palavra; vale a descrição de Decker (1980), que cita os artigos, e o texto consolidado
de 1990 e 2001.

| Ponto | O que a lei dizia |
|---|---|
| Parcela estrangeira | "Em geral", não menos de 25 % do capital (art. 4). Sem teto na lei; os comentaristas da época falavam em 49 %, mais para quem só exportasse [Decker, p. 233, nota 29] |
| Prazo | Não fixado na lei; acordado entre as partes, prorrogável com aprovação (art. 12) [Decker, p. 234] |
| Sócio chinês | Podia entrar com o direito de uso do terreno, avaliado sem o estrangeiro (art. 5) [Decker, p. 235] |
| Comando | Presidente do conselho chinês; o estrangeiro indica um ou dois vice-presidentes (art. 6) [Decker, p. 235] |
| Técnica | Incentivo à "qualidade da técnica"; no texto consolidado, técnica e equipamento "avançados, que atendam às necessidades do país", com indenização se forem atrasados e enganosos (art. 5) [Decker, p. 234; texto consolidado] |
| Lucro | Dividido pela parcela de capital; o estrangeiro pode remeter o lucro e, no fim, o que receber (arts. 8 e 11) [texto consolidado] |
| Nacionalização | **Não havia garantia em 1979.** A emenda de 4 de abril de 1990 incluiu no art. 2: "o Estado não nacionaliza nem expropria" empresa mista, salvo interesse público com indenização [texto consolidado] |
| Fábrica para o sócio no fim | **Não está na lei das empresas mistas de capital.** Está na lei das empresas mistas contratuais, de 13 de abril de 1988, se o contrato previr: os ativos fixos ficam com o lado chinês e o estrangeiro pode recuperar o investimento antes, durante a operação (art. 21 do texto de 2000) [CJV] |

- As disposições de 1990 sobre o prazo só o exigem em serviços, imóveis, recursos naturais e
  projetos restritos; o resto pode não ter prazo, e não há regra de reversão [Dur].
- Em 1980 já se temia que a experiência virasse "uma versão das fábricas da fronteira
  EUA–México", e que o chinês de ultramar levasse vantagem pela língua [Neh].

**O que isso diz da "nacionalização futura":** ela existiu como contrato, não como lei. O sócio
chinês tinha o comando e o terreno; a técnica tinha de ser avançada; e, nos contratos
cooperativos, a fábrica ficava com ele ao fim. A estatização pura nunca foi usada contra as
empresas mistas, e o regime sentiu a necessidade de prometer isso por escrito em 1990.

#### 1980–1991: as zonas e o capital da diáspora

- **Zonas econômicas especiais** (regulamento aprovado em 26 de agosto de 1980): Shenzhen, Zhuhai
  e Shantou, abertas a "cidadãos estrangeiros, chineses de ultramar e compatriotas de Hong Kong e
  Macau" (art. 1). Previam terra do Estado com prazo e taxa preferenciais (art. 12) e entrada sem
  tarifa de bens de capital e insumos (art. 13). O imposto de renda era de 15 % (art. 14), com a
  remessa de lucro permitida (art. 15). O trabalho era por contrato, e o trabalhador "pode ser
  demitido" (art. 20) [SEZ]. Xiamen foi a quarta zona, em 1980 [Emb].
- Deng, em 24 de fevereiro de 1984: a zona é "um meio de introduzir técnica, gestão e
  conhecimento" [Deng84]. As 14 cidades costeiras de 1984 e Hainan em 1988 ficaram sem fonte
  aberta, só como ponteiro.
- **As 22 regras de 1986** reduziram o imposto das empresas estrangeiras em todo o país, deram
  autonomia de gestão e facilitaram a remessa, com mais vantagens para quem exportasse pelo menos
  50 % ou fosse "tecnologicamente avançado" [BL, pp. 11–12].
- **Quem veio.** Nos anos 1980, o investimento foi "esmagadoramente" de Hong Kong e Taiwan, atrás
  de trabalho barato nas zonas para montar e exportar. Nos anos 1990 se diversificou (Japão, EUA,
  Europa) e se voltou mais ao mercado interno. Em 2003, os parceiros asiáticos somavam cerca de
  70 % do acumulado [BL]. Brandt, Ma e Rawski chamam de "acidente histórico" a abertura ter
  coincidido com a alta dos salários em Taiwan e Hong Kong. Os chineses de ultramar tinham
  privilégio de "compatriotas" [BMR].
- **A volta do dinheiro.** Hong Kong foi a origem oficial de 40 a 60 % do fluxo por duas décadas.
  Xiao estima que 30 a 50 % (provavelmente 40 %) do investimento "estrangeiro" era capital chinês
  que saía e voltava como estrangeiro para ter o tratamento dele. O Banco Mundial (2002) falava em
  até um quarto [Xiao].

#### 1992–2001: a virada e a OMC

- A viagem de Deng ao sul (18 de janeiro a 21 de fevereiro de 1992) retomou a reforma depois do
  recuo de 1989. O PIB cresceu 14,2 % em 1992, e o 14º Congresso (outubro de 1992) adotou a
  "economia socialista de mercado" [Ma; BMR].
- O investimento direto que entrou na China foi de US$ 0,06 bilhão em 1980 a 4,4 em 1991, 11,2 em
  1992 e 27,5 em 1993. Em proporção do PIB, de 1,1 % em 1991 a 6,2 % em 1993, o pico; foi 3,4 % em
  2000 e 3,9 % em 2010 (`data/history/fdi.csv`) [WDI].
- **OMC (2001).** Em três anos, toda empresa ganharia o direito de importar e exportar. A tarifa
  industrial consolidada iria a 8,9 % em média. Telecomunicações teriam teto estrangeiro de 25 →
  35 → 49 % [WTO].
- As empresas de capital estrangeiro fizeram 31,5 % das exportações em 1995, 50,0 % em 2001 e
  58,3 % em 2005 [XL]. Em 2003, fizeram 92 % dos US$ 41 bilhões exportados em computadores e
  componentes e 74 % dos US$ 89 bilhões em eletrônicos, com só cerca de 15 % de valor adicionado
  chinês [BL].
- O custo unitário do trabalho chinês, por volta de 2002, era de 25 a 40 % do americano. É bem
  menos vantagem do que o salário sozinho sugere [CG].

#### O lado doméstico: condição, não motor

- **Campo.** A responsabilidade familiar passou de 1 % das equipes de produção no fim de 1979
  para 98 % no fim de 1983. Lin (1992) atribui a ela cerca de metade do crescimento agrícola de
  1978–84, que foi de 7,7 % ao ano contra 2,9 % em 1952–78; há quem ache o efeito exagerado [Sun].
- **Empresas de vila e município.** Mais de 120 milhões de empregados em 1999, mais que o setor
  estatal, com emprego crescendo 7,4 % ao ano de 1978 a 1998 [JBS]. Ma vê nelas, na terra e na
  empresa privada a força real da reforma [Ma].
- **Herança do plano.** Expectativa de vida de 64,2 anos em 1980 e alfabetização adulta de 65,5 %
  em 1982 [WDI]. É o "trabalhador alfabetizado" que tornava o salário baixo útil.
- **Hukou.** Sob o plano, a coletivização e o registro de domicílio "facilitaram a extração de
  recursos" [BMR]. Prenderam no campo o excedente de trabalho que depois migrou: 242 milhões de
  trabalhadores migrantes em 2010 [NBS2010].
- **Poupança.** 33,7 % do PIB em 1982, 35,6 % em 2000 e 50,8 % em 2010; investimento fixo de
  29,3 % do PIB em 1978, 32,6 % em 2000 e 43,5 % em 2010 [WDI].

#### Estrutura do emprego

| Ano | Primário | Secundário | Terciário | Fonte |
|---|---|---|---|---|
| 1952 | 83,5 | 7,4 | 9,1 | NBS 4-3 |
| 1957 | 81,2 | 9,0 | 9,8 | NBS 4-3 |
| 1978 | 70,5 | 17,3 | 12,2 | NBS 4-3 |
| 1990 | 60,1 | 21,4 | 18,5 | NBS 4-3 (quebra de série) |
| 2000 | 50,0 | 22,5 | 27,5 | NBS 4-3 |
| 2009 | 38,1 | 27,8 | 34,1 | NBS 4-3 |
| 2025 | 21,7 | 31,8 | — | WDI (OIT, modelada), `data/history/wdi.csv` |

A indústria já era 47,6 % do PIB em 1978, por causa do viés do plano pela indústria pesada; depois
disso, o que muda é o **emprego** e o tipo de indústria, não a parcela da indústria no PIB [WDI].
Para o jogo isso importa: o plano dá máquina e aço com pouco emprego; a abertura dá emprego.

### O que atraiu o capital, e o que ele pagou

| Atração | O que a fonte diz |
|---|---|
| Salário baixo, trabalhador alfabetizado | Mão de obra "grande e barata" é o primeiro atrator [BL]; o custo unitário ficou em 25–40 % do americano, por desvalorização e produtividade, não só pelo salário [CG]; alfabetização de 65,5 % em 1982 [WDI] |
| O que o partido único oferecia | Imposto de 15 % e terra com prazo e taxa preferenciais nas zonas; contrato de trabalho com demissão [SEZ]; dinheiro de fora tratado melhor que o privado chinês [Huang via BL] |
| Infraestrutura e insumos a preço mundial | "Infraestrutura de exportação razoável" e regime de processamento com insumos a preço mundial [BL] |
| Escala do mercado | Nos anos 1990, investimento mais voltado ao mercado interno [BL]; a exigência de exportar foi afrouxada na prática [Naughton via BL] |
| Diáspora | Hong Kong e Taiwan primeiro; privilégio de "compatriota" [BMR, BL] |

| Cobrança | O que a fonte diz |
|---|---|
| Sócio local | Pelo menos 25 % estrangeiro, presidente chinês, o terreno como parte chinesa [Decker] |
| Técnica | Técnica "avançada que atenda às necessidades do país" [texto consolidado]; acesso ao mercado em troca de transferência em grande escala aos fornecedores e sócios [BMR] |
| Prazo e passagem | No contrato cooperativo, os ativos ficam com o lado chinês no fim [CJV] |
| Teto por setor | Na OMC, tetos estrangeiros por setor (telecomunicações 49 %) [WTO] |

### O contrafactual, e quem discorda

Nenhum autor lido quantifica uma China sem abertura. O que dizem:

- **Sem a abertura, o ritmo do plano.** De 1952 a 1978, o emprego secundário subiu dez pontos
  (7,4 → 17,3 %) e o primário caiu treze (83,5 → 70,5 %), com fábrica soviética, Plano 43 e
  poupança forçada [NBS 4-3]. Esse ritmo, mantido, deixaria a China de 2025 com algo perto de
  metade do emprego no campo. **É conta nossa, extrapolação linear, não fonte.** O "continuar como
  está" do dono não é estagnação: é a via do plano, mais lenta e de pouco emprego.
- **A favor da tese do dono:** a técnica soviética teve efeito duradouro nas fábricas e nas
  exportações seguintes [GL]. As fábricas do Plano 43 abasteceram os primeiros anos da reforma
  [Ma]. As estrangeiras dominaram a exportação de alta técnica [BL, XL].
- **Contra o capital estrangeiro como motor principal:**
  - Branstetter e Lardy: a contribuição ao capital foi "modesta" (17 % do investimento fixo em
    1994, 7 % em 2003). Eles tratam como teto a estimativa de que as estrangeiras fizeram mais de
    40 % do crescimento de 2003–04 [BL, pp. 17–18].
  - Bosworth e Collins: de 1978 a 2004, o crescimento do produto por trabalhador se divide quase
    meio a meio entre acumulação de capital e produtividade. A realocação do campo soma 1,2 ponto
    [BC].
  - Lin: o ganho veio de abandonar a estratégia de indústria pesada "contra a vantagem
    comparativa" [CL].
  - Ma: terra, empresas de vila e empresa privada, de baixo para cima, foram o motor; a liderança
    "permitiu" [Ma].
- **Huang** inverte o sinal. O investimento estrangeiro foi grande *por causa* dos defeitos do
  sistema chinês, que discriminava a empresa privada nacional (crédito, propriedade, comércio).
  Com um setor privado sadio, teria sido menor [Huang; BL, pp. 18–19]. Branstetter e Lardy aceitam
  as distorções, mas acham que o capital viria mesmo sem elas.
- **Brandt, Ma e Rawski:** várias causas. Contam a capacidade herdada do plano, o capital humano,
  o incentivo fiscal dos governos locais, a reforma parcial e a globalização [BMR].

**Leitura para o jogo.** As duas coisas valem ao mesmo tempo. Sem um canal aberto por decisão (fábrica
comprada, empresa mista, zona), o capital doméstico chinês não teria aonde ir em indústria de
exportação: a demanda interna não chamava. E sem o capital doméstico mobilizado pelo Estado, o
estrangeiro sozinho não faria a escala. O jogo precisa das duas peças, e as duas dependem de lei.

### Os outros casos

| Caso | De onde veio o capital | De onde veio a técnica | Que lei valia | Resultado |
|---|---|---|---|---|
| Japão, 1950–70 | Doméstico; investimento estrangeiro só o "desejável" [KO] | Licença sob triagem do Estado: 1 702 técnicas importadas por uma amostra de firmas, 1957–70 [KO] | Lei de Investimento Estrangeiro de 1950; aprovação vira regra em 1961 [KO] | Japão e Coreia restringiram o investimento direto e compensaram comprando conhecimento por licença [EAM, p. 36] |
| Coreia do Sul | 1953–61: a ajuda americana pagou cerca de 70 % das importações e 75 % do investimento fixo [Irwin, citando Haggard, Kim e Moon]. 1962–78: mais de 80 % do capital externo foi empréstimo; investimento direto entre 6 e 9 % [Hong, tabela 1] | Licença, bem de capital importado [EAM] | Câmbio e abertura em 1964–67; indústria pesada a partir de 1973 [Irwin] | Manufatura de 11,4 % do PIB em 1960 a 25,2 % em 1990 [WDI] |
| Taiwan | Ajuda americana; nos anos 1960 o investimento direto a substituiu, mas foi só 6 % da formação de capital, 80 % dele na manufatura [EAM, p. 145] | Investimento direto e licença | Zona de exportação de Kaohsiung, inaugurada em 3/12/1966: de 3 mil a 53 906 empregados em janeiro de 1972 [UNIDO] | Exportação de 12 % ao ano (1953–62) a 28 % (1963–72) [EAM, p. 145] |
| Sudeste Asiático | Investimento direto japonês depois do Acordo Plaza, depois dos tigres [EAM, p. 95] | Investimento direto | Tratamento "liberal" do capital estrangeiro [EAM, p. 95]; em Singapura, a multinacional só se instalava para exportar [EAM, p. 314] | Manufatura de 1960 a 2000: Malásia 10,3 → 30,9 % do PIB; Tailândia 12,5 → 28,4 %; Singapura 10,6 → 25,9 % [WDI] |
| Brasil, 1955–61 | Investimento direto com máquina importada sem cobertura cambial (Instrução 113 da SUMOC, 1955): US$ 497,7 milhões de 1955 a 1963, 38,1 % em veículos; EUA 43,5 %, Alemanha 18,7 % [CM] | Montadoras estrangeiras (Willys, GM, Ford, VW, Mercedes) [CM] | Decreto 41.018/1957: conteúdo nacional de 50 % em 1957 a 95 % em peso em julho de 1960, com tolerância de 90 % [Dec] | Em 1960, cerca de 133 mil veículos com 93 % de nacionalização média [FAPESP]. O emprego industrial ficou entre 20 e 25 % do emprego total de 1991 a 2024 [WDI] |
| México | Maquiladoras desde 1965: insumo sem tarifa para reexportar aos EUA; 147 empresas e 17 mil empregados em 1969 [Dallas] | Investimento direto | Programa de 1965; NAFTA em 1994; o art. 303 tirou a vantagem dos insumos de fora do bloco em 2001 [Dallas] | Manufatura entre 19 e 22 % do PIB de 1965 a 2024 [WDI]. Mais de 240 mil empregos perdidos em 2000–02 [Dallas] |
| Índia | Fechada: investimento direto de 0,0 a 0,1 % do PIB de 1970 a 1990 [WDI] | Licença com preço vigiado [EAM, p. 36] | Licença industrial e de importação [Pan]; o teto de 40 % da lei de 1973 e a saída da IBM e da Coca-Cola ficaram sem fonte aberta | Manufatura de 14,8 % do PIB em 1960, 16,6 % em 1990 e 13,1 % em 2024 [WDI]; investimento direto de 0,8 % do PIB em 2000 e 1,6 % em 2010 depois da reforma de 1991 [WDI] |
| Europa Central, depois de 1989 | Investimento direto: estoque de 1995 a 2020, Tchéquia 12 → 78 % do PIB, Hungria 24 → 62 %, Polônia 6 → 43 %, Eslováquia 6 → 60 % [UNCTAD] | Dentro das multinacionais; a VW comprou 31 % da Škoda por 620 milhões de marcos em 1991 [Škoda] | Privatização aberta ao estrangeiro | Integração rápida; Gál e Lux veem pouco transbordamento e sinais de esgotamento [GL22] |
| URSS, 1928–40 | O Estado: investimento de 8 % do PIB em 1928 a mais de 20 % em meados dos anos 1930 [Allen, p. 863] | Contrato de assistência técnica e máquina comprada pelo Estado, não investimento direto: Magnitogorsk projetada pela McKee, tratores pela Albert Kahn, Gorki com a Ford; "nenhuma grande fábrica foi esforço puramente soviético" [Sutton] | Plano, monopólio do comércio exterior, coletivização | PIB a 5,8 % ao ano, capital a 9,0 %, produtividade a 1,7 % [Allen, tabela 1]. Agricultura e comércio de 3/5 da economia (1913) a 1/4 (1940) [Harrison] |

- **Na URSS, o que contou foi o plano, não a coletivização.** No contrafactual de Allen, a
  prioridade à indústria pesada dá +90 % de capital em 1939 e o orçamento frouxo (empregar todo
  mundo) +23 %; a coletivização, só +8 %. "Stalin não era necessário" [Allen, p. 865].
  Cheremukhin e coautores discordam: medem perda de 24 % no bem-estar de 1928–40 e acham que a NEP
  com produtividade de 2 % ao ano ganharia [CGGT].
- **O debate do "milagre".** O Banco Mundial (1993) atribui o crescimento a políticas "amigas do
  mercado" e ao esforço exportador; a política industrial "em geral não mudou a estrutura" [EAM,
  pp. 7, 272]. Rodrik responde que a exportação era pequena demais no início e que o motor foi o
  investimento subsidiado e coordenado pelo Estado [Rodrik]. Krugman e Young mostram que foi
  sobretudo acumulação de capital e trabalho, como na URSS. Em Singapura, o investimento foi de 11
  a mais de 40 % do produto, financiado em grande parte com poupança doméstica apesar das
  multinacionais [Krugman; Young].

- **A via coreana não é subsídio.** O subsídio é gasto do orçamento hoje, pago com poupança de
  dentro e sem devolução. A via coreana é dívida em moeda estrangeira da empresa, com aval do
  Estado: traz capital de fora sem entregar a propriedade, tem de ser paga com exportação e deixa
  no Estado um passivo contingente. O subsídio fica dentro dela, implícito, no juro abaixo do
  mercado e no aval. Pela Lei de Indução de Capital Estrangeiro (1962), o governo coreano
  restringia o empréstimo direto das empresas no exterior e só o liberava às escolhidas, com aval
  do Banco de Desenvolvimento da Coreia, do Banco de Câmbio ou dos bancos comerciais que o Estado
  controlava [CL21, pp. 2 e 6]. O aval tirava o risco de calote e de desvalorização, e as
  empresas tomavam a juro real "negativo" [CL21, p. 6, nota 5]. O crédito externo saía a cerca de
  10 %, contra cerca de 20 % pagos ao depositante no país, e "constituía um subsídio" [CL21, p. 7].
  De 1973 a 1979, esse crédito às manufaturas somou cerca de US$ 16 bilhões de 2015, ou 11,4 % do
  PIB coreano de 1972 [CL21, p. 7]. Amsden explica o juro real negativo pela inflação acima da
  desvalorização, nos empréstimos externos de longo prazo de quase todos os anos 1960 e 1970. Os
  credores exigiam o aval do governo, e o governo usava o aval para decidir quem podia tomar
  [Amsden90]. No jogo, a via é empréstimo em moeda estrangeira (`debt_fx` da 3c) a empresa
  nacional, com aval do Tesouro e juro do mundo; o calote da empresa vira dívida do Estado
  (decisão 4).

**A regra mundial.** Em todos os casos houve uma decisão sobre como o capital e a técnica
entravam. Variaram a fonte do capital (estrangeiro, empréstimo, Estado) e o canal da técnica
(planta estrangeira, licença, fábrica comprada). Onde nenhum canal abriu, a indústria parou: Índia
até 1991, China de 1960 a 1971. Onde o canal era só capital estrangeiro, sem empresa nacional que
aprendesse, a integração foi rápida e rasa: maquiladoras, Europa Central. A regra do jogo tem de
ser essa, a mesma para todos, sem "regra da China".

---

## Parte 2: o mecanismo

### O que existe hoje e por que a China simulada fica no campo

| Peça | Como está | Efeito |
|---|---|---|
| Investimento | Só doméstico: o fundo do país (`Market::pool`) constrói no próprio mercado onde o tipo roda acima de 90 % e o retorno passa da barreira (`econ/monthly.rs`, `Investment`) | País pobre com demanda fraca não tem planta cheia, logo não tem investimento: a armadilha de demanda de `history.pt-BR.md` ("Por que todos convergem cedo e depois param") |
| Lei `foreign_capital` | Quatro leis (`capitulations`, `open_capital`, `screened_capital`, `closed_capital`) e o efeito `foreign_investment` (multiplicador), também em `autarky`, `state_capitalism` e `planned_economy` | Não cai em lugar nenhum: é um dos 25 efeitos que esperam sistema (`politics.pt-BR.md`) |
| Técnica | A imitação copia, sem pesquisa própria e sem canal, o que qualquer país faz em escala, à taxa (0,3 + alfabetização) × parcela do PIB mundial que já faz ÷ 60 meses (`research_system.rs`) | A China fechada tem em 1985 a produtividade industrial dos EUA (3,47 contra 3,28, `history.pt-BR.md`). O que falta não é técnica: é quem construa |
| Estatal | `state_ownership` e `state_profit_to_treasury` levam parte do lucro ao Estado; `state_investment_share` não faz nada | A economia planificada só muda quem recebe o lucro; não constrói |

Nenhum ajuste dos parâmetros domésticos acerta a China sem errar o resto. Uma saída do campo mais
rápida, por exemplo, faria a China fechada de 1962 se industrializar também.

### 1. Capital que cruza fronteira (fase 4)

**Quem investe.** O fundo de cada país, o mesmo de hoje. A ordem do mês começa pela regra de
hoje: constrói-se em casa o que passa da barreira, com o dinheiro que há. O que sobra pode sair,
até metade por mês (`foreign_capital::ABROAD_CAP`). Um fundo rico com a casa cheia manda dinheiro
para fora; um fundo que ainda tem o que construir em casa não manda. Não há multinacional como
entidade (decisão 2).

**O que ele constrói lá.** Os tipos que **a casa dele** compraria, não os que o destino compra: os
que rodam acima de 90 % no mercado de origem, ou cujo bem o mercado mundial está tomando. A planta
estrangeira vende para fora e por isso escapa da armadilha de demanda do país pobre. É a fábrica
de exportação de Shenzhen e a maquiladora, não uma fábrica para o mercado interno.

**A conta do retorno** (`foreign_capital::site_return`), por estado de destino e tipo de edifício,
em retorno anual, termo a termo:

```text
custo do trabalho lá = salário lá × trabalhadores ÷ (valor adicionado × (1 − frete) × alcance)
bruto                = (1 − custo do trabalho lá) ÷ CAPITAL_YEARS
alcance              = técnica do investidor no grupo (tech_tfp da origem)
                       × produtividade de base do estado de destino (escola, instituições)
líquido              = bruto − lei − entrega de técnica − prazo − expropriação − relação + diáspora
excesso              = líquido − retorno em casa
peso na repartição   = excesso × atração da lei (efeito foreign_investment), se o excesso > 0
```

- **Salário × técnica.** O bruto sobe quando o salário cai **e** a técnica alcança lá a
  produtividade de casa. É o custo unitário do trabalho, a medida de Ceglowski e Golub [CG].
  Salário baixo com técnica que não alcança (analfabetismo, sem energia, sem estrada) não atrai:
  o custo por unidade fica maior, não menor.
- **Lei.** Fechado dá zero: porta fechada, não prêmio. Aberto não soma nada. No meio, prêmio de
  `LAW_PREMIUM × (1 − atração)`, e a atração multiplica o peso.
- **Entrega de técnica** (`TRANSFER_COST × exigência`) e **prazo** são o preço da empresa mista.
  O prazo conta 1 ÷ anos do prazo por ano, porque o capital tem de voltar antes de a planta passar
  ao sócio.
- **Expropriação** (`expropriation_premium`). Cada estatização sem pagamento soma
  `parcela tomada × EXPROPRIATION_PREMIUM`, que cai pela metade a cada `EXPROPRIATION_HALF_LIFE`
  anos (10). Quem estatizou paga por uma geração. A promessa chinesa de 1990 existe porque esse
  prêmio é real.
- **Relação diplomática**, de −1 a 1: inimigo paga `RELATION_PREMIUM`. A China de 1960–71, sem
  relação com o Ocidente, fica fora mesmo se a lei abrisse.
- **Diáspora.** É a ligação de 0 a 1 entre o povo do investidor e o destino: a cadeia de migração
  do PR #54, lida ao contrário (gente da origem que vive lá). Tira até metade dos prêmios de risco
  (`DIASPORA_TRUST`). É o canal de Hong Kong, Taiwan e dos "compatriotas".
- **Frete.** Tarifa e distância, que vêm na fase 4, saem do valor adicionado. A zona com insumo
  sem tarifa e porto é frete baixo.

**Repartição** (`foreign_capital::allocate`). Cada destino recebe a parte do peso dele, nunca mais
que o seu espaço (`room`: ociosos lá × capital por trabalhador). O que um destino cheio não toma
vai aos outros. A conta é determinística e não depende da ordem de entrada.

**O lucro volta.** A planta guarda o dono (o mercado do fundo) e a parcela estrangeira. Do lucro da
parcela estrangeira, o que não é reinvestido lá vai ao fundo da origem
(`foreign_capital::remittance`). A divisão acontece no mesmo ponto em que hoje o lucro se reparte
entre donos, Estado e fundo (`econ/week.rs`, onde se calcula `state_take`). A remessa é saída na
conta corrente do destino e entrada na da origem; o câmbio da fase 4 (`money::exchange_target`) a
lê.

**Ordem de grandeza** (para calibrar, não é alvo). O investimento direto recebido pela China foi
de 0,03 % do PIB em 1980 a 1,1 % em 1991 e 6,2 % em 1993; pela Índia, de 0,04 % em 1980 a 1,1 % em
2001 (`fdi.csv`). Como parcela do investimento fixo
chinês, foi no máximo 17 % [BL]. Uma China aberta com mais de 10 % do PIB ao ano, ou uma China
fechada com mais que zero, está errada.

### 2. Técnica transferida

Hoje a imitação não tem canal. A proposta é multiplicar a imitação e a difusão de cada tecnologia,
em cada país, por um canal:

```text
canal = piso (livros, engenheiros, visitas)
        + exposição ao comércio com quem faz
        + (transfer_rate(parcela estrangeira dos níveis que usam a tecnologia, exigência) − 1)
        + fábrica comprada ou licença (contrato de Estado, como a URSS de 1930 e a China de 1973)
        + ajuda de governo a governo (os 156 projetos)
```

- `foreign_capital::transfer_rate`: com 20 % dos níveis de um grupo em mãos estrangeiras, a
  difusão no grupo anda 1,2 vez mais rápido; se a lei exige entrega ao sócio, 1,4 vez.
- **Encaixe nos estágios** de `research.pt-BR.md`. A planta estrangeira usa o método do dono mesmo
  que o país não tenha a tecnologia no estágio industrial (operar não é produzir). A tecnologia
  avança no país pela imitação com canal até o estágio industrial, quando o país passa a construir
  o método sozinho, e depois até a difusão. É o caminho Giorcelli e Li: máquina sem treino perde a
  vantagem, e o know-how fica [GL].
- **A fábrica comprada** é o mesmo canal sem investimento estrangeiro. O Estado paga a planta
  inteira com contrato de assistência; a técnica entra com exigência 1 e não há remessa (Sutton
  para a URSS, o Plano 43 para a China). Pede o comércio da fase 4 (comprar bem de capital do
  exterior) e dinheiro em moeda forte.
- **O piso é o ponto de calibração**, e é a mudança mais sensível. Piso alto devolve o mundo de
  hoje, em que todos convergem sem canal. Piso baixo trava os pobres fechados, que é o que o dono
  pede, mas também os pobres abertos sem capital. É a decisão 5.

#### A imitação com canal, primeira versão (antes da fase 4)

Proposta pelo orquestrador e aceita pelo dono: cortar a imitação de graça já, só com o que existe
hoje, sem esperar o fluxo de capital. Ligada no tick pelo PR da imitação com canal; as funções
são `foreign_capital::imitation_channel` e `foreign_capital::science_channel`.

**As quatro portas.** A primeira medição mostrou que cortar só a imitação não basta: a China
fechada chegava à mesma técnica industrial pela pesquisa própria, que já recebia de graça o que o
mundo sabe por três outras portas. As quatro portas por onde o saber de fora entra, em
`Research::compute`, e o canal de cada uma:

| Porta | Hoje | Com canal | Por quê |
|---|---|---|---|
| Ciência do que o mundo já conhece | custo × (1 − 0,75 × parcela que conhece) | parcela × canal da ciência | Ciência publicada atravessa fronteira fechada: revista, congresso, livro. A URSS fechada fez ciência e protótipo de ponta (bomba, foguete, reator); o que ela não fez foi difundir pela indústria civil. Por isso a porta tem piso próprio, alto |
| Protótipo do que o mundo já faz | custo × (1 − 0,75 × parcela que faz) | parcela × canal | Saber-fazer: vem com máquina, licença, técnico e fábrica |
| Passagem de protótipo a indústria | taxa × (1 + 2 × parcela que faz) | parcela × canal | Idem |
| Imitação | (0,3 + alfabetização) × parcela que faz ÷ 60 | × canal | Idem |

```text
canal da ciência = PISO_CIÊNCIA + (1 − PISO_CIÊNCIA) × canal
PISO_CIÊNCIA = 0,8 (valor de desenho, a calibrar)
```

Os dois pisos são parâmetro de desenho, sem número histórico por trás. O teste que separa as
portas: a URSS fechada chega à ciência e ao protótipo das tecnologias de base sem atraso grande e
atrasa na indústria e na difusão.

**Hoje** (`research_system.rs`, `Research::compute`, laço "Imitation"), por país `c` e tecnologia
`t` que ainda não é industrial em `c`:

```text
progresso por mês = (0,3 + alfabetização_c) × parcela do PIB mundial que faz t ÷ (5 anos × 12)
```

**O canal:**

```text
imitação por mês = (0,3 + alfabetização_c) × parcela do PIB mundial que faz t × canal_c ÷ 60
canal_c   = PISO + (1 − PISO) × comércio_c × capital_c
comércio_c = 1 com tarifa ≤ 25 %; cai em linha reta até 0 com tarifa de 60 %
capital_c  = PESO_COMÉRCIO + (1 − PESO_COMÉRCIO) × min(1, foreign_investment_c)
PISO = 0,2      PESO_COMÉRCIO = 0,5
```

| Termo | De onde sai | Valor |
|---|---|---|
| Alfabetização | `facts(world).literacy`, como hoje | 0 a 1 |
| Parcela que faz | `share_at(Stage::Industrial)`, como hoje | 0 a 1 |
| Tarifa | efeito `tariff` das leis em vigor (`trade`: livre comércio 2 %, tarifa fiscal 10 %, protecionista 25 %, autarquia 60 %) | comércio 1, 1, 1, 0 |
| Atração do capital | efeito `foreign_investment` combinado das leis em vigor (multiplicador: `open_capital` 1, `capitulations` 1,5, `screened_capital` 0,6, `closed_capital` 0,1; vezes `planned_economy` 0,1, `state_capitalism` 0,7, `autarky` 0,3) | capital 1, 1, 0,8, 0,55; plano e capital fechado juntos dão 0,505 |
| Piso | constante: livros, revistas, engenheiros que viajam, catálogos; a imitação nunca para | 0,2 |
| Peso do comércio | constante: metade do canal é máquina e bem comprado, metade é planta estrangeira e licença | 0,5 |

Valores do canal: país com tarifa até 25 % e capital aberto, **1 (a regra de hoje, igual)**;
capital triado, 0,84; capital fechado, 0,64; plano com capital fechado, 0,60; autarquia, 0,2 (o
piso). Um estágio que hoje se copia em 5 anos leva 6 anos com o capital triado, 8 com o capital
fechado e 25 em autarquia.

- **Degrada para a regra de hoje.** Com tarifa até 25 % e `foreign_investment` ≥ 1, o canal e o
  canal da ciência são exatamente 1 e o progresso é o mesmo número de hoje, sem arredondamento novo (a multiplicação
  por `Fixed::ONE` é exata). As tarifas protecionistas de 1925 a 1950 (EUA, Europa, Brasil)
  ficam em 25 % e não cortam: o protecionismo comprava máquina de fora, e só a autarquia cortava.
  As datas das tecnologias nos países abertos não mudam por este canal. Mudam só se mudar o
  mundo que as faz, o que a rodada mede.
- **O que fica de fora da primeira versão:**
  - **Vizinhança.** "De quem é vizinho de quem já tem" pede relação diplomática: a China de
    1960–71 fazia fronteira com a URSS e não recebia nada dela. Entra com a relação, na fase 4.
  - **Planta estrangeira e fábrica comprada** (`transfer_rate`). Entram com o fluxo.
  - **Difusão.** A passagem de industrial para difundida dentro do país fica como hoje; o canal
    corta a entrada da técnica, não o uso interno.
  - **Uso sem fabricar** (vacina, antibiótico, pílula). Fica como hoje.
- **Onde cai no código:**
  - `polity/system.rs`, `law_effects`: `LawEffects::tariff` (do #69) e o campo novo
    `LawEffects::foreign_investment` (neutro 1, com `serde(default)` e no hash), pelo mesmo
    caminho dos outros efeitos ligados (`politics.pt-BR.md`, "O que a lei faz na economia").
  - `research_system.rs`, `Research::compute`: o canal de cada país, uma vez por mês, nas quatro
    portas acima.
- **Os dois testes do placar** (rodada de 100 anos, com as leis forçadas por comando em 1925,
  porque a China do cenário fica com `screened_capital` e tarifa protecionista até a política
  mudar):
  1. **China fechada** (`planned_economy`, `closed_capital`, `autarky`): sem a técnica americana
     em 1985. A produtividade industrial da China (`tech_tfp` dos grupos industriais) fica abaixo
     de 60 % da americana em 1985 (hoje passa dela: 3,47 contra 3,28). A agropecuária fica acima
     de 70 % do emprego em 2025.
  2. **Europa Ocidental e EUA sem atraso:** o atraso médio de protótipo e indústria das 133
     tecnologias posteriores a 1925, contra `demonstrated` e `industrialized`, não piora mais de
     0,5 ano contra a `main`, e nenhum desses países fica mais de 3 anos atrás da própria data da
     `main` em média.

  Mais um, de não regressão do mundo: o PIB por habitante de 2025 dos países com capital aberto
  não cai mais de 5 % contra a `main`.

### 3. A "nacionalização futura"

Três coisas diferentes, cada uma com efeito próprio no fluxo seguinte:

| Forma | Como entra | O que faz à planta | O que faz ao fluxo seguinte |
|---|---|---|---|
| Empresa mista com passagem no fim | Contrato da planta, pela lei em vigor quando ela foi construída (a empresa mista contratual chinesa de 1988 [CJV]) | Parcela local fixa; no fim do prazo, a planta passa inteira ao sócio (`Handover::AtTerm`) | Já está no preço (termo "prazo"): reduz o retorno, não cria prêmio. Quem cumpre o contrato continua recebendo |
| Parcela local crescente | A mesma lei, variante (o conteúdo nacional crescente do decreto brasileiro de 1957 é o parente disso nos insumos [Dec]) | A parcela local sobe até a planta inteira ao longo do prazo (`Handover::Gradual`) | Idem; a remessa cai ano a ano |
| Estatização | Troca de lei: `closed_capital` com plantas estrangeiras no país, ou `planned_economy` | A parcela estrangeira vira do Estado sem pagamento | Registra uma expropriação (`Expropriation { years_ago, share }`). Com 100 % tomado, +15 pontos de retorno exigido no ano e +7,5 dez anos depois. Pagar a preço justo não é expropriação |

O contrato de cada planta não muda quando a lei muda; só a estatização mexe na planta. Uma China
que abre em 1979 com empresa mista e não estatiza recebe o fluxo cheio. Uma que abre e estatiza
em 1985 paga o prêmio até perto de 2005.

### 4. A via planificada

O #53 deixou a porta aberta: `state_ownership` e `state_profit_to_treasury` levam o lucro ao
Estado. Para a economia planificada construir indústria pesada sem capital estrangeiro faltam
quatro peças, na ordem do peso que Allen mede para a URSS [Allen, tabela 2]:

1. **O Estado investe por prioridade, não por planta cheia** (+90 % de capital em Allen).
   `state_investment_share`, hoje sem efeito, separa essa parte do fundo para obras do Estado nos
   grupos de prioridade (siderurgia, máquinas, energia, química), sem a regra dos 90 %. O plano
   compra o aço que ele mesmo encomenda: é o Estado como comprador, o "vale da morte" de
   `research.pt-BR.md`.
2. **Orçamento frouxo** (+23 %). A planta do plano emprega os ociosos e não fecha por prejuízo
   (hoje fecha em 13 semanas de perda, `LOSING_WEEKS_TO_CLOSE`).
3. **Técnica de fora por contrato de Estado.** A fábrica comprada da seção 2: tratado de ajuda ou
   compra, com técnicos. Romper o tratado tira os técnicos, como na ruptura de 1960 [GL].
4. **Poupança forçada** (+8 %). Compra obrigatória da safra abaixo do preço, um efeito novo da lei
   `collective_farming`: a diferença sai do camponês e entra no fundo do Estado. Pesa menos do que
   se pensa [Allen; CGGT].

E o custo: a planta do plano não troca de método por lucro, e a produtividade de base cresce
menos. A via do plano cresce em quantidade e fica para trás em técnica. Isso faz crescer a
distância de renda para os vizinhos, a causa do movimento `market_reform` (fio 9). Pelo emprego
chinês de 1952–78, essa via sobe a indústria no PIB e pouco no emprego.

### 5. Em que fase cai cada peça

| Peça | Tipo | Fase |
|---|---|---|
| Leis novas no grupo `foreign_capital`: `joint_ventures` (teto estrangeiro, prazo, entrega de técnica) e `export_zones` (aberto nos estados escolhidos); efeitos novos `foreign_share_cap`, `foreign_term_years`, `technology_transfer` | Efeito de lei, dado | 3 |
| Postura dos grupos de interesse sobre as leis novas (empresariado nacional contra a concorrência, trabalhadores a favor do emprego, o partido a favor se a técnica fica) | Efeito de lei | 3 |
| Plano: `state_investment_share` ligado, prioridade sem a regra dos 90 %, orçamento frouxo, compra obrigatória da safra | Efeito de lei | 3 |
| Fluxo do fundo para fora, dono e parcela estrangeira na planta, remessa, câmbio | Capital e comércio | 4 |
| Frete e tarifa no termo "frete"; relação diplomática; ajuda de governo e fábrica comprada | Capital e comércio, diplomacia | 4 |
| Memória de expropriação por país (entra no save) | Capital | 4 |
| Imitação com canal e `transfer_rate` | Pesquisa, com o fluxo da 4 | 4 |
| Fio 9 com marcos e pressão histórica (`history-rails.pt-BR.md`, reescrito) | Decisão de regime | 7 |

### 6. O ponto de leitura que preciso do thread da calibração (dono de `econ/`)

O módulo não lê nada de `econ/` sozinho. Para ligar o fluxo, a fase 4 precisa de pontos de
leitura, sem mudar o que já existe:

1. **O dinheiro que sobrou no mês, por mercado, depois das obras em casa.** Hoje é o `budget`
   local no fim do laço de `Investment::compute` (`monthly.rs`), que some. Basta guardá-lo em
   `InvestmentChanges`, sem efeito no estado e sem mudar o hash.
2. **A produtividade separada em base e técnica.** `plant_tfp` já é `Region::tfp_for(b)` ×
   `Market::tech_tfp[grupo]`; o alcance usa a técnica da origem com a base do destino. Basta que
   as duas partes continuem públicas.
3. **Salário e ociosos por estado.** `market_wages` e os camponeses sem emprego
   (`workers(peasant.size) − employed`), que o laço de candidatos já calcula.
4. **Onde o lucro se divide** (`week.rs`, `state_take`): o ponto em que a parcela estrangeira
   sairá como remessa. Pede um campo novo na planta (dono e parcela estrangeira) na fase 4; até
   lá, nada.

### 7. Como se testa

O teste do dono vira critério do placar (`history.pt-BR.md`) e do teste do desvio
(`history-rails.pt-BR.md`). São rodadas de 100 anos com uma lei forçada por comando:

| Rodada | Critério |
|---|---|
| China fechada a tudo o século todo: sem plano, sem ajuda, `closed_capital` | Agropecuária ≥ 70 % do emprego em 2025; indústria abaixo de 10 %; investimento estrangeiro zero |
| China do plano com a ajuda soviética até 1960, fechada depois | Emprego secundário na faixa de 1978 (17 %, ±5) por volta de 1978; agropecuária ≥ 50 % em 2025 |
| China do plano que abre com `joint_ventures` em 1979 e `export_zones` em 1980 | Investimento estrangeiro acima de 1 % do PIB antes de 1995; a indústria sobe na ordem histórica (primeiro a costa e a indústria leve de exportação, depois a pesada); agropecuária abaixo de 40 % em 2010 (real 36,7 %) e perto de 22 % em 2025 |
| Índia fechada até 2025 | Sem investimento estrangeiro; manufatura parada |
| Índia aberta em 1991 | Investimento estrangeiro perto de 1 % do PIB nos anos 2000 (real 0,8 % em 2000, 1,6 % em 2010); emprego industrial subindo (real 14,6 % em 1991, 25,8 % em 2025, `wdi.csv`) |
| Não regressão: país rico e aberto (EUA, Alemanha) | Não vira destino por salário: o que recebe vem de outro rico com técnica igual e é pequeno contra o PIB; o investimento em casa não cai mais de 10 % contra a `main` |
| Expropriação | O país que estatiza recebe menos da metade do fluxo de antes por pelo menos dez anos |

Os testes de unidade de `foreign_capital.rs` já cobram a direção de cada termo. Lei fechada dá
zero. Salário menor com a mesma técnica atrai mais. Salário baixo sem técnica não atrai. País rico
com o salário de casa não atrai. A memória de expropriação reduz o fluxo e se apaga. A empresa
mista acelera a difusão. A repartição respeita o teto e o espaço e não depende da ordem.

### 8. O fio 9 reescrito

O texto novo está em `history-rails.pt-BR.md`, seção "9. As reformas chinesas de 1978". Mudou em
três pontos. Os marcos são leis e fluxos medidos, não o pleno de 1978. O sucesso é o capital
entrar e a indústria subir, não a lei mudar. E a cadeia começa antes, na reaproximação e nas
fábricas compradas de 1972–73.

---

## Decisões para o dono

Todas decididas pelo dono em 2026-11-16, como recomendadas.

1. **O que o jogador comanda no investimento de fora.** Decidido pelo dono em 2026-11-16.
   - **Recomendação:** a lei do próprio país e uma lista curta de países vetados (embargo); o
     dinheiro sai sozinho pela conta acima, e a tela mostra os termos.
   - **Alternativa:** comandar quanto e onde.
   - **Como se testa:** a rodada sem jogador dá o mesmo fluxo com e sem a tela.
   - **Custo de manter:** baixo. O comando de valores pediria IA para os outros países e uma tela
     de carteira.
2. **Multinacional como entidade ou só o fundo do país.** Decidido pelo dono em 2026-11-16.
   - **Recomendação:** só o fundo, e a planta guarda o dono e a parcela estrangeira.
   - **Como se testa:** o save cresce dois campos por planta.
   - **Custo de manter:** baixo. A entidade pediria milhares de empresas no save, com lucro,
     sede e decisão próprios, e não muda nenhum critério do placar.
3. **Zonas especiais: por estado ou lei nacional.** Decidido pelo dono em 2026-11-16.
   - **Recomendação:** uma lei nacional, `export_zones`, que abre o capital só nos estados
     escolhidos, até um teto. O jogador escolhe; a IA escolhe os com porto e mais ociosos.
   - **Como se testa:** a China de 1980 recebe o fluxo na costa e não no interior.
   - **Custo de manter:** médio. A política ainda não tem lei com escopo de estado; o escopo
     seria uma lista no estado do país, sem lei por estado.
4. **A via do empréstimo (Coreia) junto ou depois.** Decidido pelo dono em 2026-11-16.
   - **Recomendação:** primeiro a planta estrangeira e a fábrica comprada pelo Estado (fase 4). O
     empréstimo estrangeiro a empresas nacionais com aval do governo fica para depois, sobre a
     dívida da 3c e a confiança da fase 7.
   - **Como se testa:** a Coreia, de 1962 a 1978, fica fora do critério até lá.
   - **Custo de manter:** sem a via, a Coreia simulada se industrializa por investimento direto,
     com a forma errada e o resultado perto do certo.
5. **Cortar a imitação de graça.** Decidido pelo dono em 2026-11-16.
   - **Recomendação:** sim, imitação com canal e piso. É a mudança que impede a China fechada de
     ter a técnica americana em 1985. Uma primeira versão, só com as leis que já existem, entra
     antes do fluxo de capital (seção "A imitação com canal, primeira versão").
   - **Como se testa:** a rodada da China fechada e a de não regressão acima, mais a data
     histórica das tecnologias (`research.pt-BR.md`, "Calibração"), que não pode atrasar mais de
     três anos em média.
   - **Custo de manter:** mexe na calibração do mundo inteiro, e o piso é um número a defender.

## O que ficou de fora

- **Sem fonte aberta:** o texto original de 1979, palavra por palavra; os números do Terceiro
  Front; o custo do laminador de Wuhan e do contrato de Baoshan; os livros de Naughton, Vogel,
  Huang, Amsden, Wade, Johnson, Shapiro e Panagariya (2008), além de *Farm to Factory*, citados só
  pelo que outros autores dizem deles; o teto de 40 % da lei indiana de 1973; a parcela da
  indústria no emprego soviético em 1928 e 1940; as 14 cidades costeiras e Hainan.
- **Dados antes de 1970:** não há tabela aberta de investimento direto por país. Em 1960, os EUA
  tinham cerca de metade do estoque mundial de saída, o Reino Unido 18 %, os Países Baixos 10 % e a
  França 6 % [Lipsey]. As tabelas de Dunning (1914, 1938, 1960) e o livro de Twomey são pagos.
- **Estoque de investimento direto** (UNCTAD, CC BY 3.0 IGO): o download oficial cobre de 1990 em
  diante; uma cópia antiga (DBnomics) tem fluxos de 1970 e estoques de 1980. Não entrou: os fluxos
  do Banco Mundial bastam para o placar.

## Fontes

Abertas em 2026-11-16, salvo indicação.

- [GL] Giorcelli, M. e Li, B. "Technology Transfer and Early Industrial Development: Evidence from
  the Sino-Soviet Alliance", NBER WP 29455 (rev. 2025).
  https://www.nber.org/system/files/working_papers/w29455/w29455.pdf
- [Ma] Ma, Q.-P. "Grassroots and Local Initiatives Versus the Architect's Design During China's
  Reform and Opening Process", *Essays in Economic & Business History* 39 (2021), 17–44.
  https://www.ebhsoc.org/journal/index.php/ebhs/article/download/442/373/846
- [BL] Branstetter, L. e Lardy, N. "China's Embrace of Globalization", NBER WP 12373 (2006).
  https://www.nber.org/system/files/working_papers/w12373/w12373.pdf
- [BMR] Brandt, L., Ma, D. e Rawski, T. "From Divergence to Convergence: Re-evaluating the History
  Behind China's Economic Boom", LSE Economic History WP 175/13 (2013).
  https://www.lse.ac.uk/asset-library/information/wp175.pdf
- [BC] Bosworth, B. e Collins, S. "Accounting for Growth: Comparing China and India", NBER WP 12943.
  https://www.nber.org/system/files/working_papers/w12943/w12943.pdf
- [Decker] "The Joint Ventures Law of the PRC in a Legislated and Negotiated Tax Climate",
  *Syracuse J. Int'l L. & Com.* 8(1) (1980).
  https://jilc.syr.edu/wp-content/uploads/2024/12/Decker-Joint-Ventures-Law.pdf
- Texto consolidado (1979, 1990, 2001) da lei das empresas mistas de capital:
  https://autonews.gasgoo.com/articles/news/law-of-the-peoples-republic-of-china-on-chinese-foreign-equity-joint-ventures-2267
- [CJV] Lei das empresas mistas contratuais (1988, texto de 2000).
  https://en.wikisource.org/wiki/Law_of_the_People%27s_Republic_of_China_on_Chinese-Foreign_Contractual_Joint_Ventures
- [Dur] Disposições provisórias sobre o prazo das empresas mistas (1990).
  https://lehmanlaw.com/resource-centre/laws-and-regulations/foreign-investment/interim-provisions-for-the-duration-of-chinese-foreign-equity-joint-ventures-1990.html
- [Neh] Nehemkis, P. "China's Law on Joint Ventures", *California Management Review* 22(4)
  (1980), resumo. https://cmr.berkeley.edu/1980/08/22-4-chinas-law-on-joint-ventures
- [SEZ] Regulamento das zonas econômicas especiais de Guangdong (1980), texto na OMC.
  https://www.wto.org/english/thewto_e/acc_e/chn_e/wtaccchn46_leg_8.pdf
- [Deng84] Deng Xiaoping, 24 de fevereiro de 1984.
  https://www.marxists.org/reference/archive/deng-xiaoping/1984/21.htm
- [Emb] Embaixada da China na Síria, "Opening Wider to the Outside World" (2009).
  https://sy.china-embassy.gov.cn/eng//xwfb/200910/t20091027_1726454.htm
- [OH] US Office of the Historian, "Rapprochement with China".
  https://history.state.gov/milestones/1969-1976/rapprochement-china
- [Xiao] Xiao, G. "PRC's Round-Tripping FDI: Scale, Causes and Implications" (2004).
  https://hiebs.hku.hk/wp-content/uploads/2020/09/wp1137.pdf
- [XL] Xu, B. e Lu, J. "The Impact of Foreign MNEs on Export Sophistication of Host Countries:
  Evidence from China", tabela 1 (de Wang e Wei 2007, Alfândega chinesa).
  https://www.ceibs.edu/facultyCV/xubin/xu-lu.pdf
- [WTO] OMC, comunicado 243 (17 de setembro de 2001).
  https://www.wto.org/ENGLISH/news_e/pres01_e/pr243_e.htm
- [CG] Ceglowski, J. e Golub, S., *The World Economy* 30(4) (2007), pp. 597–617, resumo. https://ideas.repec.org/a/bla/worlde/v30y2007i4p597-617.html
- [Huang] Huang, Y. *Selling China* (Cambridge UP, 2003), pela descrição do livro e por [BL].
  https://inbetweenreview.substack.com/p/yasheng-huang-selling-china-foreign
- [CL] Chen, B. e Lin, J. Y. "Development Strategy, Resource Misallocation and Economic
  Performance", PKU NSD WP (2016), resumo.
  https://www.nse.pku.edu.cn/en/Research/WorkingPaper/24632ac68d1842c8855bc6ff1d42b7ac.htm
- [MQY] Meng, X., Qian, N. e Yared, P. "The Institutional Causes of China's Great Famine", NBER WP
  16361. https://www.nber.org/papers/w16361
- [Sun] Sun, PKU NSD WP sobre a responsabilidade familiar (citando Lin 1992, *AER*).
  https://nsd.pku.edu.cn/attachments/0b4b3d2018ce4d4faa834ad84b72fb7a.pdf
- [JBS] Cambridge Judge Business School WP 33/2000, empresas de vila e município.
  https://www.jbs.cam.ac.uk/wp-content/uploads/2020/08/wp0033.pdf
- [NBS 4-3] China Statistical Yearbook 2010, tabela 4-3.
  https://www.stats.gov.cn/sj/ndsj/2010/html/E0403C.HTM
- [NBS2010] Comunicado estatístico de 2010 do NBS.
  https://www.stats.gov.cn/english/NewsEvents/201102/t20110228_26426.html
- [WDI] Banco Mundial, World Development Indicators, CC BY 4.0: `BX.KLT.DINV.CD.WD`,
  `BX.KLT.DINV.WD.GD.ZS`, `NV.IND.TOTL.ZS`, `NV.IND.MANF.ZS`, `NE.GDI.FTOT.ZS`,
  `NY.GNS.ICTR.ZS`, `SE.ADT.LITR.ZS`, `SP.DYN.LE00.IN`, `SL.AGR.EMPL.ZS`, `SL.IND.EMPL.ZS`.
  https://api.worldbank.org/v2/
- [EAM] Banco Mundial, *The East Asian Miracle* (1993) (páginas do PDF).
  https://documents1.worldbank.org/curated/en/975081468244550798/pdf/multi-page.pdf
- [KO] Kiyota, K. e Okazaki, T. "Foreign Technology Acquisition Policy and Firm Performance in
  Japan, 1957–1970", CIRJE-F-329 (2005).
  https://www.cirje.e.u-tokyo.ac.jp/research/dp/2005/2005cf329.pdf
- [Irwin] Irwin, D. "From Hermit Kingdom to Miracle on the Han", *KDI Journal of Economic Policy*
  47(2) (2025).
  https://kdijep.org/v.47/2/1/From+Hermit+Kingdom+to+Miracle+on+the+Han+Sources+of+Policy+Change+in+Korea%E2%80%99s+Trade+Transformation?view=print
- [Hong] Hong, K. "Foreign Capital and Economic Growth in Korea: 1970–1990", *Journal of Economic
  Development* 22(1) (1997), tabela 1. https://jed.cau.ac.kr/archives/22-1/22-1-6.pdf
- [Rodrik] Rodrik, D. "Getting Interventions Right", NBER WP 4964, resumo.
  https://www.nber.org/papers/w4964
- [UNIDO] UNIDO, "Industrial free zones in Taiwan" (c. 1972).
  https://downloads.unido.org/ot/46/91/4691351/00001-10000_04411.pdf
- [CM] Caputo, A. C. e Melo, H. P. "A industrialização brasileira nos anos de 1950: uma análise da
  instrução 113 da SUMOC", *Estudos Econômicos* 39(3) (2009).
  https://www.scielo.br/j/ee/a/ZpgwjzqDRC9bT4YrFhfxcvC/?lang=pt
- [Dec] Decreto 41.018, de 26 de fevereiro de 1957.
  https://legis.senado.leg.br/norma/463600/publicacao/15765117
- [FAPESP] Revista Pesquisa FAPESP, "Há 70 anos nascia o primeiro carro brasileiro".
  https://revistapesquisa.fapesp.br/ha-70-anos-nascia-o-primeiro-carro-brasileiro/
- [Dallas] Cañas, J. e Coronado, R. "Maquiladora Industry: Past, Present and Future", *Business
  Frontier* 2 (2002), Federal Reserve Bank of Dallas.
  https://fraser.stlouisfed.org/title/business-frontier-6348/issue-2-2002-607474/fulltext
- [Pan] Panagariya, A. "India in the 1980s: Weak Reforms, Fragile Growth".
  https://econwpa.ub.uni-muenchen.de/econ-wp/it/papers/0309/0309010.pdf
- [UNCTAD] UNCTADstat, "Foreign direct investment: inward and outward flows and stock", CC BY 3.0
  IGO. https://unctadstat.unctad.org/datacentre/dataviewer/US.FdiFlowsStock
- [Škoda] Škoda Storyboard, "Škoda and Volkswagen: three decades of partnership".
  https://www.skoda-storyboard.com/en/skoda-world/skoda-and-volkswagen-three-decades-of-partnership/
- [GL22] Gál, Z. e Lux, G. "FDI-based regional development in Central and Eastern Europe",
  *Tér és Társadalom* 36(3) (2022), resumo. https://m2.mtmt.hu/api/publication/33062008
- [CL21] Choi, J. e Levchenko, A. "The Long-Term Effects of Industrial Policy", NBER WP 29263
  (2021, rev. 2024), páginas do PDF rev0.
  https://www.nber.org/system/files/working_papers/w29263/revisions/w29263.rev0.pdf
- [Amsden90] Amsden, A. "Third World Industrialization: 'Global Fordism' or a New Model?", *New
  Left Review* I/182 (1990), introdução.
  https://newleftreview.org/issues/i182/articles/alice-h-amsden-third-world-industrialization-global-fordism-or-a-new-model
- [Allen] Allen, R. C. "The rise and decline of the Soviet economy", *Canadian Journal of
  Economics* 34(4) (2001). https://content.csbs.utah.edu/~mli/economics%207004/allen-103.pdf
- [CGGT] Cheremukhin, A., Golosov, M., Guriev, S. e Tsyvinski, A. "Was Stalin Necessary for
  Russia's Economic Development?", NBER WP 19425. https://www.nber.org/papers/w19425
- [Harrison] Harrison, M. "Foundations of the Soviet Command Economy, 1917 to 1941" (2016).
  https://warwick.ac.uk/markharrison/public/cambridge_communism_postprint.pdf
- [Sutton] Sutton, A. *Western Technology and Soviet Economic Development 1930 to 1945* (Hoover,
  1971). https://archive.org/details/AnthonySuttonTriologyOfWesternTechnologyAndSovietEconomicDevelopment1917To19301930To19451945To1965
- [Krugman] Krugman, P. "The Myth of Asia's Miracle", *Foreign Affairs* 73(6) (1994).
  https://vault.hanover.edu/~eiriksson/classes/214/files/MythOfAsiasMiraclePaulKrugman.pdf
- [Young] Young, A. "The Tyranny of Numbers", NBER WP 4680, resumo.
  https://www.nber.org/papers/w4680
- [Lipsey] Lipsey, R. NBER WP 8665 (2001), citando Nações Unidas (1988), tabela 1.2.
  https://www.nber.org/papers/w8665.pdf
