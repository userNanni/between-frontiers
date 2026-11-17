# Tratados e acordos

Estado: **desenho** (fase 4, etapa 1). Nada aqui roda no tick. O contrato atual de 1925
(`diplomacy.rs`, `data/scenario/1925/diplomacy.ron`, `diplomacy.pt-BR.md`) descreve tratados
como fatos datados com um tipo; este documento troca o tipo por **cláusulas** e diz o que cada
cláusula faz no mundo. As decisões abertas estão na seção 15, à espera do dono.

Princípios, os de sempre (`architecture.pt-BR.md`): **causa visível, pouco sorteio, mesmas
regras para a IA e para o jogador, determinismo.** Nenhum acordo entra por data: as datas reais
são referência para o placar.

Referência de qualidade, não de conteúdo: a negociação como troca de cláusulas lembra a de
outros jogos de estratégia, mas os nomes, a lista de cláusulas, os números e os textos daqui
saem dos acordos reais do século (seção 2) e das regras deste jogo. Por isso o arquivo de dados
se chama `clauses.ron` e o tipo, `Clause`: é o nome jurídico (cláusula, artigo de tratado), sem o
vocabulário de nenhum jogo.

## 1. A ideia em uma frase

Um acordo é **um conjunto de cláusulas, cada uma com quem dá e quem recebe**, assinado por partes
que o aprovaram por dentro, com prazo, condição de vigência e um teste medido de cumprimento. Paz,
independência, reconhecimento, sujeição e aliança não são tipos de tratado: são acordos com certas
cláusulas.

## 2. O que os acordos do século ensinam

Os casos e as fontes estão na seção 16. O que o desenho tira deles:

1. **O calendário é o objeto da negociação, não o total.** A reparação alemã foi refeita quatro
   vezes em treze anos (Londres 1921, Dawes 1924, Young 1929, Hoover 1931, Lausanne 1932): o que
   mudou foi a anuidade, o prazo, a parte adiável e a moratória. A dívida entre Estados precisa de
   calendário próprio, revisável por outro acordo. (Detalhe na seção 5.)
2. **O gargalo era transferir divisas, não arrecadar.** O Plano Dawes protegia a transferência:
   o agente só convertia marcos em moeda estrangeira quando o câmbio aguentava. Vira um parâmetro
   da cláusula de dívida.
3. **A dívida nasce com a independência e com o reconhecimento.** O Haiti pagou à França pelo
   reconhecimento (1825); a Indonésia aceitou parte da dívida colonial na Mesa Redonda de Haia
   (1949) e a repudiou em 1956; as Filipinas (1946) e a Argélia (Evian, 1962) nasceram com
   comércio, moeda e bases presos ao antigo dono.
4. **Sujeição é escada de cláusulas, não estado binário.** Os domínios britânicos perderam o
   vínculo cláusula por cláusula, cada um na sua data (Westminster 1931, adotado pela Austrália em
   1942 e pela Nova Zelândia em 1947; apelações ao Conselho Privado cortadas de 1933 a 2004).
   Iraque (1932) e Egito (1922) viraram soberanos mantendo bases, trânsito e aliança.
5. **O súdito saiu quando o suserano estava em crise ou em guerra, e quando não havia tropa
   dele em casa.** Cuba (Depressão, 1934), Iraque e Austrália (Segunda Guerra), Europa do Leste
   (falência soviética, 1989–91). A Iugoslávia, sem tropas soviéticas, saiu em 1948; Hungria
   (1956) e Tchecoslováquia (1968), com tropas, não.
6. **Quem sai precisa de patrono ou de folga.** A Iugoslávia teve ajuda americana; o Egito de
   1956, a pressão americana sobre o Reino Unido.
7. **Reconhecimento é ato de cada Estado, contável.** Quem reconhecia a República da China caiu
   de 71 Estados (1969) para 23 (1986) e 12 (hoje), em ondas depois de um gatilho (ONU 1971, EUA
   1979). Não reconhecer custou pouco (Manchukuo) até muitos reconhecerem ao mesmo tempo
   (Hallstein, 1969–72).
8. **Acordo tem prazo, renovação tácita e aviso de denúncia.** Molotov–Ribbentrop 10 anos + 5,
   OTAN denunciável depois de 20 anos com 1 de aviso, Varsóvia 20 + 10, Montreux 20 anos e 2 de
   aviso. São parâmetros da cláusula, não enfeite.
9. **Promessa sem sanção quebra quando o equilíbrio medido muda** (não agressão polaco-soviética
   de 1932, rompida em 1939; Locarno, rompido com a Renânia em 1936). O jogo não pune a quebra por
   "honra": registra a quebra, dá causa ao outro lado e encarece a próxima promessa.
10. **Aprovação interna decide.** O Senado americano recusou Versalhes; a ratificação é um passo
    com custo político à vista.
11. **União monetária dura enquanto quem garante a conversão aguenta** (franco CFA com a conta no
    Tesouro francês; União Latina morta pela inflação da guerra; área da libra encolhendo a cada
    desvalorização).
12. **Cessão paga acontece com vendedor apertado** (Alasca depois da Crimeia, Ilhas Virgens sob
    risco de guerra). O preço é o que a conta de cada lado aceita, não um valor tabelado.
13. **Dívida declarada e dívida exigível se separam.** Dos 132 bilhões de 1921, só os 50 bilhões
    dos títulos A e B eram cobráveis; os C eram política. A cláusula de dívida tem parte nominal.
14. **Executar pela força custa.** O Ruhr rendeu cerca de 900 milhões de marcos-ouro, quase tudo
    gasto na própria ocupação. A ocupação como garantia tem custo para quem ocupa, medido.
15. **Limite de armas é razão entre as partes** (5:5:3 em Washington, 35 % em 1935), e a corrida
    continua nas categorias sem teto. O teto da cláusula pode ser absoluto ou relativo à outra parte.

## 3. O acordo

```text
Treaty
  key            chave estável (texto: treaty-<key> nos .ftl; acordo novo: id numérico)
  sides          lados; cada lado é uma lista de países (multilateral: Locarno tem 5)
  clauses        lista de Clause (seção 4), cada uma com giver e receiver (lados)
  signed         data da assinatura (proposta aceita pelos governos)
  in_force       data em que vigora (ratificação de todos; seção 8)
  term           Fixed(anos) | UntilCondition | Indefinite
  renewal        None | Tacit(anos)                 renova se ninguém denunciar
  notice         anos de aviso para denunciar (0: não se denuncia antes do prazo)
  conditions     predicados medidos de fim (ex.: ocupação termina quando a dívida X cair abaixo
                 de Y); os mesmos predicados dos movimentos e do trilho histórico
  status         Proposed | Signed | InForce | Suspended(cláusulas) | Ended(motivo)
  record         registro: quem propôs, contrapropostas, votos internos, quebras
```

- **Cláusula é o átomo, acordo é o envelope.** Prazo e aviso valem para o acordo inteiro; uma
  cláusula pode ter prazo próprio menor (a evacuação da Renânia por zonas, o depósito de reservas
  do CFA que caiu de 100 % a 65 % e 50 %).
- **Cláusula de execução única** age na entrada em vigor e fica no registro (ceder estados, criar
  país, criar dívida, pagar). **Cláusula contínua** vale enquanto o acordo vigora e tem teste
  mensal de cumprimento (tarifa, limite de frota, base, tributo, sujeição).
- **Uma cláusula, um teste.** Toda cláusula contínua declara o que mede para saber se está sendo
  cumprida; a quebra é o teste falhar, não um evento (seção 9).
- **Estado salvo:** o livro de dívidas fica em `Economy::ledger` (é dinheiro, servido no passo da
  moeda); acordos, reconhecimento e reputação em `GameWorld::treaties` quando entrarem (
  reputação), com `#[serde(default)]`; só entra no hash quando não está vazio, como a fronteira.

## 4. Catálogo de cláusulas

Arquivo `data/diplomacy/clauses.ron`: uma linha por tipo de cláusula, com os parâmetros e seus
limites, o que lê e escreve, o teste de cumprimento, quem dentro do país se importa (seção 8) e o
que precisa de outras frentes. O código tem um `enum ClauseKind` com a mecânica; o arquivo dá os
limites, os pesos e os textos. Esboço:

```ron
(
    key: "debt",
    once: true,
    params: [
        (key: "amount", unit: Reference, min: 0),
        (key: "currency", kind: Currency),          // do devedor, do credor ou de referência
        (key: "rate", unit: PercentYear, min: 0, max: 15),
        (key: "years", unit: Years, min: 1, max: 99),
        (key: "deferrable", unit: Share, min: 0, max: 1000),   // parte adiável (Young)
        (key: "transfer_cap", unit: ShareOfRevenue, min: 0, max: 1000, optional: true), // Dawes
        (key: "pledge", kind: Revenue, optional: true),     // receita dada em garantia (alfândega)
        (key: "contingent", unit: Reference, min: 0, optional: true), // parte nominal (títulos C)
    ],
    test: ServicePaid,
    concerns: [("taxpayers", -2), ("creditor_treasury", 3)],
    source: "Versalhes art. 231–232; Plano Dawes 1924; Plano Young 1929 (seção 16)",
)
```

### Execução única

| Cláusula | O que faz | Lê | Escreve | Precisa de |
|---|---|---|---|---|
| `cede_states` | estados passam de um país a outro | estados, dono | `Transfer` da fronteira | fronteiras parte B |
| `release_country` | estados viram país novo, independente ou súdito | catálogo de candidatos | `Secede` | parte B |
| `union` | um país se junta a outro | — | `Annex` | parte B |
| `debt` | cria dívida do devedor e o crédito igual do credor, com calendário | câmbio, taxa mundial | livro de dívidas (seção 5) | `DebtTerm` da parte B |
| `assume_debt` | um país assume dívida que já existe de outro (sucessão, colônia que leva o empréstimo colonial) | livro, `Money` | livro, `Money` | `DebtTerm` |
| `revise_debt` | muda calendário, juro, parte adiável ou perdoa (moratória, Lausanne) | livro | livro | só o credor dá |
| `payment` | pagamento à vista do Tesouro de um ao outro (compra de território, indenização) | Tesouro | Tesouro | — |
| `techniques` | técnicas levadas ou entregues | pesquisa | `TechniqueTerm` | parte B |

### Contínuas

| Cláusula | O que faz | Teste de cumprimento | Precisa de |
|---|---|---|---|
| `recognition` | reconhece o Estado (e, opcional, a fronteira atual) | — (é uma relação; seção 6) | nada |
| `non_aggression` | promete não iniciar crise nem guerra contra o outro | nenhum degrau da escada contra ele | escada de crise (fase 5) |
| `defence` | entra se o outro for atacado (`casus foederis`), mútua ou de um lado só (garantia) | entrou quando chamado | guerra (fase 5) |
| `consultation` | consulta antes de agir numa região | — | guerra (fase 5) |
| `border_guarantee` | garante uma fronteira de terceiros (Locarno) | entrou quando a fronteira foi violada | guerra (fase 5) |
| `demilitarized` | estados sem tropa nem fortificação | forças nos estados = 0 | forças no tick (fase 5) |
| `arms_limit` | teto de frota, de exército ou razão entre partes; modo de verificação | forças ≤ teto | forças no tick (fase 5) |
| `base` | uma parte mantém base e guarnição em estado da outra; renda anual | base existe | forças (fase 5); `sea_zones.ron` `holders` já é a forma do dado |
| `passage` | trânsito de forças, ou regime de estreito (aberto, teto, fechado na guerra) | — | mar (fase 5) |
| `occupation` | ocupa estados como garantia, até condição | condição do acordo | guerra (fase 5) |
| `tariff_cap` | teto da tarifa de uma parte sobre a outra (tratado desigual, nação mais favorecida) | tarifa aplicada ≤ teto | tarifa no comércio (fase 4) |
| `tariff_preference` | margem de preferência entre as partes | margem aplicada | tarifa no comércio |
| `customs_union` | mercado comum com tarifa externa única; receita repartida | tarifas iguais | tarifa no comércio |
| `currency_peg` | moeda de uma parte presa à da outra; depósito de reservas | regime em vigor | regime monetário (fase 4) |
| `monetary_union` | moeda comum | — | regime monetário |
| `capital_protection` | não expropriar sem pagar; capital de uma parte entra com a lei da outra | expropriação sem pagamento = quebra | `foreign_capital.rs` |
| `tribute` | pagamento anual de uma parte à outra (renda de base, contribuição do súdito) | pago | livro de pagamentos |
| `foreign_policy` | uma parte não assina acordo sem a outra (Platt art. I, protetorado) | acordo assinado sem consentimento | — |
| `intervention` | uma parte pode impor lei ou governo à outra (Platt art. III, Brejnev) | — (é direito, não dever) | política |
| `minorities` | proteção de uma comunidade num país (tratados de minorias de 1919) | lei dirigida contra a comunidade | comunidades nos pops |
| `troops_quota` | uma parte põe tropas à disposição da outra | contingente entregue | guerra (fase 5) |

Cláusula que depende de frente que ainda não existe **pode ser assinada e fica dormente**: está
no acordo, aparece na tela com "sem efeito até a fase 5", não tem teste. Assim os tratados de 1925
(Washington, Locarno) entram inteiros desde já.

## 5. Dívida entre Estados

### O que existe hoje

`econ/money.rs`: o Estado deve em casa (`debt_home`), ao banco central (`debt_bank`), no exterior
em moeda própria (`debt_abroad`) e em moeda estrangeira (`debt_fx`, unidades de referência). O
credor externo é **anônimo**: a poupança ociosa de todos os países compra os títulos, e o
crédito de cada um fica em `foreign_assets` (famílias), recebendo juro e perdendo no calote **pro
rata**. Nenhuma dívida liga um Estado a outro Estado. A dívida de 1925 é igual para todos (25 % do
PIB em casa, 5 % no banco central); as dívidas de guerra ficaram para o cenário.

O desenho das fronteiras (decisões de 2026-11-17, PR #90) já pôs o termo `DebtTerm` (quanto, de
quem, para quem, em que moeda) e um crédito do Tesouro credor, `Money::claims`.

### O que falta: um livro nomeado

A reparação precisa de credor com nome: a moratória Hoover suspendeu pagamentos de uns a outros,
Lausanne perdoou só o que os credores aceitaram, e a força revisionista da Alemanha
(`history-rails.pt-BR.md`) tem como alavanca "perdão, só os credores". Um agregado por país não
diz a quem se deve.

```text
InterstateDebt                       Economy::ledger.debts, em ordem de criação
  id
  debtor, creditor                   índices de país (estáveis; lápide herda pela anexação)
  treaty                             acordo que a criou ou a revisou por último
  currency                           Debtor | Creditor | Reference
  principal                          unidades de referência (como debt_fx)
  rate                               juro anual
  schedule                           anuidade, anos restantes, parte adiável, teto de transferência
  arrears                            atraso acumulado (o que não foi pago na semana)
  suspended_until                    moratória
```

- **Fonte da verdade é o livro.** `Money::claims` (crédito do Tesouro credor) e a parte da dívida
  do devedor que é entre Estados são **somas do livro**, recalculadas, não guardadas duas vezes
  (decisão 1). Assim credor e devedor nunca divergem.
- **Conservação por construção.** Cada linha é uma dívida e um crédito do mesmo número. Criar,
  pagar, revisar, perdoar, inflacionar e dar calote mexem na linha uma vez. O teste soma, para o
  mundo, dívida entre Estados menos créditos entre Estados (= 0) e dinheiro dos Tesouros mais
  fundos mais poupança (igual antes e depois de cada cláusula e de cada semana de serviço).
- **Erosão igual dos dois lados.** A dívida em referência perde com a inflação mundial, como
  `debt_fx` já perde; como é uma linha só, o crédito perde igual.
- **Serviço semanal no mesmo passo de `settle_week`:** juro + amortização pelo calendário, do
  Tesouro do devedor ao Tesouro do credor, na mesma unidade que o empréstimo externo já usa. O
  peso para o devedor sobe com o câmbio real, como `debt_fx_local`.
- **Teto de transferência (Dawes):** com `transfer_cap`, o que passa do teto (parte da receita)
  vai para `arrears` sem ser calote; o atraso paga juro e pesa no prêmio de risco.
- **Calote:** o serviço entre Estados entra na mesma conta do limite de serviço sobre a receita
  (`DEFAULT_LIMIT`). No calote, o corte vale para a dívida de mercado e para o livro, na mesma
  proporção, e cada linha cortada é **quebra da cláusula** com aquele credor (seção 9): é a causa
  que a ocupação do Ruhr teve.
- **Revisão:** só por acordo novo com a cláusula `revise_debt`, que o credor dá. A IA credora
  aceita quando o valor de rever passa o valor esperado de manter (seção 10): moratória antes do
  calote vale mais que metade perdida no calote.
- **Convive com a dívida de mercado, sem misturar.** A dívida anônima de hoje continua sendo o
  crédito de mercado: títulos comprados pela poupança ociosa do mundo, em `debt_abroad` e
  `debt_fx` do devedor e `foreign_assets` das famílias dos credores, pagos e cortados pro rata. O
  livro é só a dívida **de governo a governo** criada por acordo. O devedor vê as duas no orçamento
  (o juro do livro entra em `Budget::interest`, o principal em `Budget::debt`), o prêmio de risco e o
  limite de calote somam as duas, e a operação de fronteira mexe em cada uma pela sua regra (a de
  mercado pela parcela **w**, a do livro pelas funções do livro). A soma mundial fecha com as duas:
  dinheiro dos Tesouros + fundos + poupança + créditos de mercado − dívida de mercado é igual antes
  e depois de cada passo, e o livro soma zero (dívida = crédito) em todo instante.
- **Garantia sobre receita** (`pledge`: alfândega, ferrovias, imposto de consumo, como em Londres
  1921, Dawes e no Haiti): a receita dada em garantia paga o serviço antes de entrar no orçamento.
  Deixa o calote mais difícil e a austeridade mais rápida; é cláusula que o devedor cede (pesa em
  soberania).
- **Parte nominal** (`contingent`, os títulos C de 1921): valor declarado que só vira dívida no
  livro se uma condição medida do acordo for cumprida. Fica no registro do acordo, fora das somas.
- **Graus de calote, cada um medido:** pagamento cheio; atraso (`arrears`); calote parcial pela
  regra de `settle_week`; **repúdio** (o devedor denuncia a cláusula: a linha sai do livro, o
  credor registra a perda inteira e ganha causa). O credor tem a mesma resposta que o mercado já
  tem (exclusão do crédito, como o Johnson Act de 1934) e a causa da seção 9 (o Ruhr de 1923, cujo
  rendimento mal pagou a própria ocupação).
- **Dívidas encadeadas:** um acordo pode ter como condição de vigência outro acordo (Lausanne só
  valeria com o acerto das dívidas com os Estados Unidos, que não veio). Usa as condições da seção 3.
- **Moeda:** `Reference` é o padrão (marco-ouro, dólar-ouro: o valor em ouro da época); `Debtor`
  vira dívida em moeda do devedor, que a inflação dele corrói (o credor perde, conservado);
  `Creditor` segue o câmbio do credor.

## 6. Reconhecimento

- **É uma relação de cada país com cada país**, guardada como lista dos pares que **não** se
  reconhecem (o mundo de 1925 se reconhece quase todo). Exceções de 1925, do cenário: quem não
  reconhecia a URSS (os Estados Unidos até 1933, entre outros) e os casos que a pesquisa do cenário
  achar.
- **País novo nasce reconhecido só por quem assinou o acordo que o criou** (o antigo dono, quando
  liberta; ninguém, quando sai por movimento). Os outros reconhecem um a um, por acordo de duas
  partes com a cláusula `recognition`, que pode vir com outras (o reconhecimento do Haiti veio com
  a dívida).
- **"País não reconhecido" não é um estado guardado:** é derivado, a parcela do PIB mundial que o
  reconhece. A tela mostra os dois números (quantos e quanto do mundo).
- **O que o reconhecimento muda, e só isso:**
  - acordo com quem não reconhece só pode ter a cláusula `recognition` (e cláusulas de execução
    única de troca, como `payment`): sem aliança, sem tarifa, sem base;
  - capital e empréstimo de governo não vão de quem não reconhece para quem não é reconhecido
    (a URSS sem crédito americano até 1933); o mercado anônimo de títulos continua;
  - entrar em organização exige o reconhecimento da maioria dos membros.
- **Duas pretensões ao mesmo Estado** (as duas Chinas, as duas Alemanhas): quem reconhece um como
  governo do todo não reconhece o outro; reconhecer os dois é a cláusula com fronteira
  (`recognition` com `border: true`, o Tratado Básico de 1972). Pretensão é dado do país
  (estados que reivindica), já previsto nos núcleos do catálogo de candidatos.
- **Não reconhecer uma mudança de fronteira** (Stimson, Manchukuo) é a mesma relação aplicada à
  transferência: quem não reconhece continua tratando os estados como do dono anterior para a
  cláusula `border_guarantee` e para a causa de crise. Custa pouco, como custou.

## 7. Súditos

### Graus como feixes de cláusulas

Sujeição é um acordo entre suserano e súdito com um conjunto de cláusulas. O grau é **o nome do
feixe**, não uma regra à parte; tirar uma cláusula muda o grau. Os feixes, com o que o súdito cede:

| Grau | Política externa | Defesa e tropas | Tarifa e comércio | Moeda | Tributo | Leis internas | Exemplo |
|---|---|---|---|---|---|---|---|
| Colônia da coroa (administração separada) | `foreign_policy` | `base` + `troops_quota` | `tariff_preference` | `currency_peg` | `tribute` | `intervention` | Raj |
| Protetorado | `foreign_policy` | `base` | `tariff_cap` | `currency_peg` | — | `intervention` | Marrocos, Kuwait |
| Mandato | `foreign_policy` | `base` | `tariff_cap` (porta aberta) | — | — | `intervention` | Iraque até 1932, Síria |
| Domínio | `consultation` | `defence` | `tariff_preference` | `currency_peg` | — | — | Canadá, Austrália |
| Satélite | `foreign_policy` | `base` + `troops_quota` | `customs_union` ou preferência | — | — | `intervention` | Mongólia, Tuva |
| Protegido (soberano sob tutela) | `foreign_policy` parcial | `base` | — | — | — | `intervention` | Cuba (Platt), Egito, Panamá |

- Os 14 súditos de 1925 (`countries.ron`, `Status::Subject`) e as 4 proteções do `diplomacy.ron`
  são convertidos para estes feixes, com o instrumento real como nome do acordo.
- O vínculo `borders::Subject` (suserano, tipo, desde quando) da parte B continua sendo o ponto de
  leitura (`GameWorld::overlord_of`). O tipo dele passa a ser derivado do feixe em vigor; quando o
  feixe fica vazio, a diplomacia chama `Status` com suserano vazio.

### Como o súdito sai: condições medidas

Três caminhos, nenhum por evento:

1. **Negociado, cláusula por cláusula** (Westminster, Iraque 1932, Platt 1934). O súdito propõe
   tirar cláusulas; o suserano avalia como qualquer acordo (seção 10). O que pesa a favor de soltar
   é medido: custo de manter (base e guarnição pagas pelo Tesouro do suserano), movimento de
   independência no súdito (mobilização, que a política já mede), crise do suserano.
2. **Denúncia unilateral** (Egito 1951, Iugoslávia 1948). O súdito denuncia o feixe ou parte
   dele. Se o **controle** do suserano for menor que a **pressão de saída**, a denúncia vale: o
   acordo acaba, o suserano ganha causa contra o ex-súdito (seção 9) e as cláusulas de dívida
   continuam devidas. Se o controle for maior, a denúncia **vira crise** (fase 5); até a guerra
   existir, a denúncia é recusada e a tela diz quanto falta.
3. **Perda pelo suserano** (Manchukuo 1945): suserano anexado, lápide, ou derrotado numa paz com a
   cláusula `release_country`. Fase 5.

```text
controle do suserano  = guarnição       tropas do suserano no súdito sobre o exército do súdito (fase 5;
                                        antes, a cláusula base conta como guarnição fixa)
                      + dependência     dívida com o suserano / PIB do súdito, parte do comércio
                      − crise           1 − legitimidade do suserano, estresse fiscal, exaustão de guerra
pressão de saída      = mobilização do movimento de independência no súdito
                      + apoio dos grupos de interesse do súdito à saída (postura medida, seção 8)
                      + patrono         um terceiro reconhece e oferece defesa ou crédito
```

Os pesos ficam em `rules.ron`, marcados como escolha de desenho, para o equilíbrio depois da
fase 6. A tela do súdito mostra os termos dos dois lados, como a postura dos grupos já mostra.

**O caso do dono: jogar com a Polônia sob a Rússia e se separar.** O jogador escolhe o súdito
(troca de país, parte B), acompanha os dois números, e age nos termos que são dele: a dívida (paga
ou para de tomar do suserano), o comércio, o movimento (leis que os grupos apoiam), o patrono
(propõe reconhecimento e defesa a um terceiro). Quando a pressão passa o controle, denuncia. Se
denunciar antes, na fase 5 isso é guerra contra quem tem tropa em casa.

**Libertar e jogar com o libertado:** a cláusula `release_country` com grau escolhido (soberano ou
um feixe de súdito); o jogador troca de país para o novo (parte B). Libertar o próprio território
não precisa de acordo (comando da parte B); o acordo é o caminho quando a libertação vem com dívida,
base ou reconhecimento.

## 8. Como se chega a um acordo

### Proposta, contraproposta, aceitação

1. **Proposta:** um governo monta as duas colunas (o que dá, o que pede) e envia. A proposta é um
   comando (`GameCommand::Propose`), enfileirado e resolvido no dia, como os outros.
2. **Avaliação do outro lado:** a mesma função para IA e jogador (seção 10), com os termos à vista.
3. **Resposta:** aceitar, recusar com o termo que mais pesou, ou **contraproposta** (a IA tira a
   cláusula de maior custo para ela ou acrescenta a que fecha o saldo). No máximo três rodadas por
   proposta; depois, a mesma proposta só volta em 6 meses (escolha de desenho, contra a repetição).
4. **Assinatura** pelos governos e **ratificação** por dentro (abaixo). Vigora quando todos
   ratificam.

### Saldo de cada lado, em conta aberta

Cada cláusula vale, para cada parte, uma soma de termos com nome:

| Termo | Medida | Unidade |
|---|---|---|
| Dinheiro | valor presente dos fluxos (dívida, tributo, pagamento, renda de base) à taxa mundial | unidades de referência |
| Comércio | variação da receita de tarifa e do excedente do comércio no ano, vezes anos do prazo | unidades de referência |
| Território | valor adicionado e população dos estados que mudam | unidades de referência + gente |
| Segurança | força de quem promete defender sobre a ameaça medida, e vice-versa | pontos (fase 5; zero antes) |
| Soberania | cláusulas que o lado cede (política externa, intervenção, base estrangeira) | pontos por cláusula, de `clauses.ron` |
| Política interna | oposição dos grupos de interesse ponderada pelo peso (abaixo) | legitimidade |
| Confiança | reputação de quem promete: desconta o que é entregue no futuro | multiplicador |

- O que não é dinheiro vira dinheiro por **pesos declarados** em `rules.ron` (quanto vale um
  ponto de soberania em PIB do país), à vista na tela. Não há rótulo de "atitude": a IA aceita
  porque a conta fecha, e mostra a conta.
- **Sem sorteio.** A mesma proposta, no mesmo mundo, tem a mesma resposta.

### Quem aprova dentro do país

A ratificação usa a máquina da política (`politics.pt-BR.md`), não uma regra nova:

- Cada cláusula declara, em `clauses.ron`, **pesos por efeito** sobre o que os grupos de interesse
  já medem (tarifa, renda da terra, imposto, serviço militar) e dois efeitos novos: `sovereignty`
  (ceder política externa, base estrangeira, território) e `burden` (dívida e tributo sobre a
  receita). A postura de cada grupo sobre o acordo é a soma, efeito por efeito, como a postura
  sobre uma lei (`Politics::score`); a tela mostra os termos ("os grandes proprietários são contra
  porque a tarifa do trigo cai").
- **Apoio** = soma de peso × postura dos grupos, a mesma conta do "apoio dos grupos" das leis.
- **Quem precisa aprovar** depende da forma de governo, lida das leis em vigor: com parlamento e
  sufrágio, o apoio tem de passar de um limiar (o Senado de 1919); no regime de partido único, só
  pesam os grupos com força política fora do voto (o peso por renda e as forças armadas); a
  monarquia absoluta ratifica sozinha.
- **Custo de assinar contra a vontade:** quando o apoio fica abaixo do limiar e o governo
  ratifica mesmo assim (decisão 5), a legitimidade cai em proporção à oposição, e os movimentos
  medem o acordo: `revisionism` ganha causa com território perdido e reparação sobre a receita
  (os indicadores que `history-rails.pt-BR.md` já pede).

## 9. Quando se quebra

- **Quebra é o teste mensal falhar** (pagamento não feito, tarifa acima do teto, frota acima do
  limite, tropa na zona desmilitarizada, acordo assinado sem o suserano). **Denúncia** é sair pelo
  prazo e aviso do próprio acordo: não é quebra.
- **O que a quebra faz, sempre medido:**
  1. **Registro** no acordo (data, cláusula, quanto).
  2. **Suspensão recíproca:** o lado prejudicado pode suspender as cláusulas que dava em troca (a
     regra geral do direito dos tratados, Convenção de Viena de 1969, art. 60), sem quebrar.
  3. **Causa:** o lado prejudicado ganha uma causa contra quem quebrou, com tamanho medido (o valor
     da cláusula quebrada). É o que a escada de crise da fase 5 lê como casus belli.
  4. **Reputação:** um número por país, como o `default_history` da dívida, que sobe com cada
     quebra pelo valor dela e cai 10 % ao ano. A reputação desconta o valor que os outros dão às
     promessas futuras daquele país (seção 8, termo "confiança"). Quem quebra paga na próxima mesa.
- **Mudança de equilíbrio não é quebra automática.** A IA quebra quando o valor de quebrar passa o
  custo medido (causa dada ao outro, reputação, suspensão recíproca); a tela da IA mostra essa
  conta ao jogador que é a vítima, para a quebra não surpreender sem causa visível.

## 10. Como a IA avalia

- **A mesma função** para todos: `evaluate(world, proposal, party) -> Valuation`, com a lista de
  termos (seção 8). O jogador vê, na mesa, a `Valuation` do outro lado: é por isso que a IA aceita
  ou recusa.
- **Aceita** quando o saldo é positivo **e** a ratificação passa. **Contrapropõe** pela cláusula
  de pior termo. **Mantém** um acordo em vigor enquanto o valor de manter for maior que o de
  quebrar (seção 9).
- **Propostas da IA na fase 4**, poucas e com causa medida (a IA estratégica é da fase 6):
  - devedor em aperto (serviço perto do limite de calote) pede revisão ao credor; o credor aceita
    quando o valor da revisão passa o valor esperado do calote: **a moratória Hoover sai da conta**;
  - súdito com pressão de saída acima do controle pede cláusulas de volta, e denuncia se recusado;
  - país novo pede reconhecimento aos vizinhos e às potências, oferecendo o que a conta pede.
- **Custo de CPU:** avaliação só quando há proposta ou no teste mensal; sem acordos novos, o custo
  é o teste mensal das cláusulas contínuas (algumas centenas) e o serviço semanal do livro.

## 11. Paz, independência e reconhecimento são acordos

| Caso | Cláusulas | Observação |
|---|---|---|
| Paz com reparação (o caso de 1919) | `cede_states`, `release_country`, `debt` (moeda de referência, calendário), `arms_limit`, `demilitarized`, `occupation` (até condição de pagamento) | a fase 5 põe a proposta na mesa pelas exigências da crise |
| Revisão da reparação (1924, 1929, 1931, 1932) | `revise_debt`, às vezes com `debt` novo (o empréstimo Dawes) e fim de `occupation` | acordo entre devedor e credores |
| Independência com dívida (Haiti 1825, Indonésia 1949) | `release_country` ou `recognition`, `debt` ou `assume_debt` | dívida ao antigo dono ou herança do empréstimo colonial |
| Independência com laços (Filipinas 1946, Evian 1962) | `release_country`, `base`, `tariff_preference`, `currency_peg`, `capital_protection` | soberano com cláusulas residuais |
| Reconhecimento | `recognition` (com `border` quando divide pretensão), às vezes com `debt` ou `payment` | Hallstein, Tratado Básico |
| Sujeição | um feixe da seção 7 | |
| Compra de território | `cede_states`, `payment` | Alasca, Ilhas Virgens |

## 12. Dados de 1925 no formato novo

`data/scenario/1925/diplomacy.ron` continua com organizações, regimes e tarifas. Os 58 tratados
viram acordos com cláusulas, um a um, com a mesma fonte por linha:

| Tipo de hoje | Vira |
|---|---|
| `DefensiveAlliance` | `defence` mútua (dormente até a fase 5) |
| `Friendship` | `non_aggression` e/ou `recognition`, conforme o texto |
| `Consultation` | `consultation` |
| `ArmsLimitation` | `arms_limit` com os tetos do tratado (Washington: tonelagem e razão) |
| `Guarantee` | `border_guarantee` |
| `Peace` | as cláusulas contínuas que ainda valem em 1925 (`arms_limit`, `demilitarized`) |
| `Reparations` | `debt` no livro, com o calendário Dawes (decisão 6) |
| `DebtSettlement` | `debt` no livro (Reino Unido aos Estados Unidos, 1923) |
| `Occupation` | `occupation` até a data ou condição |
| `Protection` | feixe "protegido" |
| `Subjection` | o feixe do grau de `countries.ron` |
| `CustomsUnion` | `customs_union` |

O teste atual (`tests/diplomacy_1925.rs`) continua valendo e ganha: todo súdito de
`countries.ron` tem um acordo de sujeição; toda dívida do livro tem credor vivo; a soma do livro
fecha.

### Feito no contrato (etapa 2, primeira parte)

- `crates/bf-sim/src/treaty.rs`: `Clause`, `ClauseKind` (30 cláusulas), `ClauseCatalogue`,
  `Proposal`, `evaluate` (dinheiro em valor presente, soberania e confiança já com preço; os outros
  termos aparecem com a frente de cada um), `Ledger` e `InterstateDebt` (serviço semanal, teto de
  transferência, moratória, corte no calote, anexação), `validate_treaties`, `ledger_1925`.
- `data/diplomacy/clauses.ron`: catálogo, feixes de súdito, chaves do dono (`policy`) e pesos.
- `data/scenario/1925/diplomacy.ron`: os 58 tratados com `clauses` (ou `bundle`, os 7 de
  sujeição). O `kind` antigo fica como rótulo histórico, e o teste exige que cada tipo tenha a
  cláusula que o define.
- `data/scenario/1925/war_debts.ron`: reparação alemã (títulos A e B, 50 bilhões de marcos-ouro,
  parte nominal de 82 bilhões, anuidade Dawes do primeiro ano, repartida pelas porcentagens de
  Spa: França 52 %, Reino Unido 22 %, Itália 10 %, Bélgica 8 %; os 8 % dos outros aliados ficam
  fora, sem repartição na fonte) e dívida britânica aos Estados Unidos (US$ 4,6 bilhões, 1922 a
  1984, 3 % até 1932). Juro zero e 50 anos na reparação são escolha de desenho, marcada.
- `assets/i18n/{pt-BR,en}/treaties.ftl`: nome e efeito de cada cláusula, e para cada frente o
  texto "sem efeito até…": toda cláusula dormente diz na tela por que dorme e o que a acorda
  (teste `every_text_exists_in_both_languages`).
- Pendente de fonte lida (fica sem número, anotado na cláusula): tetos de exército de
  Saint-Germain, Trianon e Neuilly; degraus da anuidade Dawes; a taxa de 3,5 % britânica a partir
  de 1932; a renda anual do canal do Panamá.

### Feito no tick (etapa 2, segunda parte): o livro de dívidas

- **Onde:** `Economy::ledger`, servido no fim de `econ::money::week` (`treaty_service`), depois do
  mercado de títulos. No hash só quando não está vazio: com a chave `war_debts_1925` desligada, o
  hash de 10 anos é o da `main`.
- **Unidade:** a da economia, dólares de 2011 a preço-base de 1925. As dívidas de 1925 entram pelo
  ouro (marco-ouro e dólar de 1925) e pelo fator 12,85 do CPI-U que `economy.pt-BR.md` já usa.
- **Serviço:** juro e anuidade da semana, do Tesouro do devedor ao do credor; pesa mais com a
  moeda fraca (câmbio real, como `debt_fx_local`), e o credor recebe o que saiu do devedor. Entra
  em `Budget::interest` e o estoque em `Budget::debt` do devedor.
- **Prêmio e calote:** a dívida do livro conta no prêmio de risco como dívida externa em moeda
  estrangeira e o serviço conta no limite de calote e na reestruturação. No calote, o livro perde a
  mesma fração que a dívida de mercado (metade, ou 40 % na reestruturação), e cada credor perde o
  seu crédito.
- **1925–1935, chave ligada:** a Alemanha começa devendo 39 % do PIB aos credores (França 52 %,
  Reino Unido 22 %, Itália 10 %, Bélgica 8 % da reparação) e chega a 25 % em 1935, sem calote; o
  título alemão paga 6,7 % contra 5,1 % do britânico em 1926. O Reino Unido deve 13 % do PIB aos
  Estados Unidos e tem 7 % de créditos.
- **Save:** o snapshot é postcard, que não aceita campo novo; save da `main` não carrega neste
  binário (vale para todo campo novo desde a demografia). O cliente ainda não grava o mundo; o que
  se confere é salvar e recarregar no mesmo binário com o mesmo hash. Proposta ao orquestrador:
  versão de formato com migração antes de o cliente gravar mundos.

### Feito: a ponte entre cláusula e operação de fronteira

- `treaty_exec::settle(catálogo, proposta, chave)` traduz as cláusulas de um tratado assinado;
  `treaty_exec::queue(mundo, acerto, ano)` põe em movimento. Quem assina chama os dois: a
  negociação dos tratados e a paz da guerra (fatia D).
- **Território** vira operação de fronteira, na fila do sistema `borders` (fim do dia, cada uma
  conferida antes de mexer em algo), na ordem do tratado: cessão → `Transfer`; país novo →
  `Secede`, independente ou súdito do lado que recebe quando a cláusula nomeia um feixe, com
  todas as partes como signatárias; união → `Annex`. Cessão e união pedem um dador e um
  recebedor; mais que isso é recusado na tradução.
- **Dívida do acordo** vai para o livro por um termo novo das operações,
  `DebtTerm::Agreement { debtor, creditor, treaty, terms }`, numa `Operation::Terms` depois do
  território: o país que nasce pelo mesmo tratado já pode dever ou receber (a independência que
  nasce devendo). Os nomes se resolvem quando a operação roda; nome desconhecido não cria linha
  de um lado só.
- **Pagamento** move Tesouros na hora; **revisão** muda as linhas do credor naquele tratado
  (juro, anuidade, prazo, carência; perdoa, nunca acrescenta).
- O resto fica guardado para valer enquanto o acordo vigora.
- A frente `Borders` passa a viva: cessão, país novo e união deixam de aparecer dormentes.
- Testes (`tests/treaties_bridge.rs`), cada um conferindo as somas das fronteiras, o saldo do livro
  e o dinheiro: paz com cessão de estado e reparação; independência da Nigéria devendo ao Reino
  Unido e como protetorado dele; união da Áustria à Alemanha com pagamento e perdão da parte
  francesa da reparação com carência; recusa de cessão com dois recebedores.
### Feito: reconhecimento

- `GameWorld::treaties.recognition` (`treaty::Recognition`): só os pares que não se reconhecem.
  No hash só quando não está vazio.
- `data/scenario/1925/recognition.ron`: a lista curta de 1925, cada linha com fonte lida: os
  Estados Unidos não reconhecem a URSS (até 1933, referência); a Mongólia só é reconhecida pela
  URSS; o Tibete, por ninguém. A lista cresce com fonte, não com memória. A Convenção de Simla
  passa a ser o Tibete aceitando a fronteira, sem o Reino Unido reconhecê-lo como Estado.
- `recognition_conflicts`: numa proposta, toda cláusula contínua que não seja o próprio
  reconhecimento exige que dador e recebedor se reconheçam; pagamento, cessão e dívida podem.
  `validate_recognition` confere isso nos tratados em vigor em 1925.
- País novo: `Recognition::on_new_country` (só quem assinou a criação reconhece); anexação:
  `on_annex`. A parte B das fronteiras chama as duas na operação.
- "Não reconhecido" é derivado: `recognised_by_share` (parcela do PIB mundial que reconhece).

## 13. Interface: a mesa de negociação

Painel registrado (`PanelId::Treaty`), escondido até ser aberto, teclado primeiro, textos em
`assets/i18n/{en,pt-BR}/treaties.ftl`. Esboço:

```text
┌ Mesa: Alemanha ⇄ Estados Unidos, Reino Unido, França ─────────────── [Esc] fechar ┐
│ Alemanha dá                           │ Credores dão                                │
│  ▸ Anuidade 2,0 bi RM até 1988        │  ▸ Fim da ocupação da Renânia em 1930       │
│    (moeda: referência; adiável 1/3)   │  ▸ Teto de transferência retirado           │
│                                       │  ▸ Empréstimo 0,3 bi a 5,5 %                │
│ [a] acrescentar cláusula  [x] tirar   │ [A] pedir cláusula                          │
├───────────────────────────────────────┴─────────────────────────────────────────────┤
│ Saldo da Alemanha        −1,9 bi │ Saldo dos credores            +2,4 bi            │
│  dinheiro (valor presente) −2,6  │  dinheiro                      +2,6               │
│  soberania (ocupação)      +0,5  │  segurança                     −0,2               │
│  política interna          +0,2  │                                                   │
│ Contra, na Alemanha:             │ Ratificação nos Estados Unidos: dispensa          │
│  indústria pesada −420 (imposto) │                                                   │
│  movimento revisionista +90      │ Resposta provável: aceita (saldo +2,4)            │
├─────────────────────────────────────────────────────────────────────────────────────┤
│ [Enter] propor   [c] ver contraproposta   [r] registro do acordo   [Tab] coluna    │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

- Duas colunas de cláusulas, uma por lado; cada linha abre os parâmetros (setas mudam valor).
- O saldo de cada lado por termo, com o total; quem dentro de cada país é contra e por quê.
- "Resposta provável" é a `Valuation` do outro lado: a IA não esconde a conta.
- Fora da mesa: a lista de acordos do país (em vigor, propostos, quebrados), o livro de dívidas
  entre Estados, e no súdito os dois números da seção 7.

## 14. O que cada cláusula precisa das outras frentes

| Precisa | Existe hoje | Quem faz |
|---|---|---|
| Dívida em moeda estrangeira | sim: `Money::debt_fx`, em referência, credor anônimo; calote e prêmio prontos | economia (3c) |
| Credor nomeado | não; `Money::claims` agregado decidido no #90 | **este trabalho** (livro), combinado com fronteiras parte B (decisão 1) |
| Operações de fronteira | desenhadas; no tick na parte B, com `DebtTerm` e `TechniqueTerm` | fronteiras parte B |
| Vínculo de súdito | `borders::Subject` e `overlord_of` na parte B; tipo em `countries.ron` | fronteiras parte B; regra de saída aqui |
| Tarifa no comércio | não: comércio em pool sem tarifa; tarifa de 1925 é dado | fase 4, frente de comércio (ainda sem thread) |
| Regime monetário | dado (`regime_on`); `exchange_target` sem regime | fase 4, frente de moeda (ainda sem thread) |
| Frota, exército, bases, guarnição | dado de 1925 (`forces.rs`); fora do tick | guerra (fase 5) |
| Zonas e estreitos com `holders` | dado e derivado do dono (`sea.rs`) | pronto; a cláusula `base`/`passage` vira a fonte dos `holders` |
| Escada de crise e casus belli | desenho (`war.pt-BR.md`) | fase 5 |
| Grupos de interesse e postura | sim (`Politics::score`, apoio dos grupos) | pronto; dois efeitos novos (`sovereignty`, `burden`) |
| Movimento de independência no súdito | sim (mede leis coloniais do suserano) | pronto |
| Comunidades (cláusula de minorias) | entrando nos pops | comunidades |
| Capital estrangeiro (proteção) | contrato fora do tick | fase 4 |

## 15. Decisões (decidido em 2026-11-17)

**Todas decididas em 2026-11-17, como recomendado:** 1, 2 e 7 pelo orquestrador (técnicas); 3,
4, 5 e 6 pelo dono. As quatro do dono ficam como chave em `data/diplomacy/clauses.ron`, bloco
`policy`, com os valores decididos: `subject_exit: PressureOverControl`,
`recognition_default: AllExceptListed`, `ratify_below_threshold: PlayerOnly`,
`war_debts_1925: true`. Na decisão 6, o efeito no placar de Alemanha, Reino Unido e França fica
anotado como dependência, não como meta.

Cada uma com recomendação, teste e custo de manter.

1. **Livro de dívidas entre Estados como fonte da verdade.** Recomendação: um livro com credor e
   devedor nomeados (seção 5), em `GameWorld::treaties`; o `Money::claims` do #90 vira soma do
   livro em vez de campo guardado, e o `DebtTerm` da parte B escreve no livro. Alternativa: só o
   agregado, e a reparação não sabe a quem se deve (sem moratória de um credor, sem perdão de
   Lausanne). Teste: soma mundial de dívida menos créditos entre Estados = 0 e dinheiro igual,
   antes e depois de cada cláusula e de cada semana; 100 anos de determinismo com o livro
   povoado. Custo: um sistema semanal pequeno; combinar o formato com o thread das fronteiras
   antes de a parte B fixar `claims`.
2. **Unidade do saldo.** Recomendação: dinheiro em valor presente mais termos sem preço
   (soberania, segurança, política) convertidos por pesos declarados em `rules.ron`, todos à vista.
   Alternativa: pontos abstratos de "interesse". Teste: mesma proposta, mesmo mundo, mesma
   resposta; trocar os lados troca o sinal. Custo: os pesos são de equilíbrio (depois da fase 6).
3. **Saída do súdito antes da guerra existir.** Recomendação: na fase 4, saída negociada e
   denúncia aceita só quando a pressão passa o controle (com a cláusula de base contando como
   guarnição fixa); denúncia contra controle maior fica recusada, com a conta na tela, até a fase 5
   transformá-la em crise. Alternativa: denúncia livre já (sem custo até a fase 5, explorável).
   Teste: súdito com movimento forte e suserano em crise fiscal sai; o mesmo súdito com suserano
   estável não sai; Iugoslávia sem base sai antes da Hungria com base. Custo: uma regra mensal por
   súdito (menos de 30).
4. **Reconhecimento.** Recomendação: guardar só os pares que não se reconhecem; 1925 quase todo
   reconhecido, com as exceções históricas do cenário; país novo reconhecido só por quem assinou a
   criação; "não reconhecido" derivado (parcela do PIB mundial). Teste: país saído por movimento
   não recebe empréstimo de governo nem assina aliança até ser reconhecido; o mercado anônimo
   continua. Custo: uma lista e uma verificação na proposta.
5. **Ratificação.** Recomendação: pela máquina da política, com limiar por forma de governo; o
   **jogador** pode ratificar contra o limiar, pagando legitimidade; a IA não ratifica abaixo do
   limiar (Versalhes nos Estados Unidos). Alternativa: ratificação automática. Teste: o mesmo
   acordo passa numa autocracia e falha numa república com os grupos contra. Custo: dois efeitos
   novos em `politics::EFFECTS` e pesos por cláusula em `clauses.ron`.
6. **Dívidas de guerra no cenário de 1925.** Recomendação: entram no livro **por cima** dos 25 %
   + 5 % genéricos: a reparação alemã pelo calendário Dawes (principal: os títulos A e B de 1921;
   50 bilhões de marcos-ouro; os 82 bilhões
   dos títulos C entram como parte nominal), com o calendário Dawes (1 bilhão de marcos no ano de 1924–25,
   subindo a 2,5 bilhões no quinto ano; os degraus intermediários a conferir), as dívidas interaliadas já acertadas em 1925 (Reino Unido aos
   Estados Unidos, 1923) e as não acertadas como dívida sem calendário até um acordo. Muda o placar
   da Alemanha, do Reino Unido e da França; pela regra do dono, a dependência fica anotada e não
   vira meta. Alternativa: começar sem elas e criar só por acordo jogado. Teste: livro de 1925 soma
   o que a fonte diz; Alemanha paga a anuidade Dawes em 1925–28. Custo: dado com fonte por linha.
7. **Nome do dado.** `clauses.ron` e `Clause` em vez de `articles.ron`: evita o vocabulário de
   outro jogo. Custo: nenhum.

## 16. Fontes

Fatos tirados de textos de tratados e de páginas públicas; a Wikipedia em inglês (CC BY-SA 4.0)
é usada só para fatos, com o artigo e a seção. Marcado **[conferir]**: número que a pesquisa não
confirmou no texto e que não entra em `data/` antes de conferido.

### Reparação e dívida

- **Versalhes, parte VIII** (1919): art. 231–235, anexo II §13 (unanimidade para cancelar), §16
  (juro de 5 %), §17–18 (sanção). Texto: Avalon Project, Yale,
  <https://avalon.law.yale.edu/imt/partviii.asp>.
- **Cronograma de Londres** (1921-05-05): 132 bilhões de marcos-ouro; títulos A e B, 50 bilhões,
  a dívida exigível; C, o restante, sem juro nem calendário. Calendário de 2 bilhões por ano mais
  26 % das exportações. Falta declarada em 1923-01-09 e ocupação do Ruhr em 1923-01-11. Total pago
  até 1932: 20,598 bilhões de marcos-ouro. WP "World War I reparations" (London Schedule of
  Payments; German defaults; Amount paid by Germany). Divisão A 12 / B 38 **[conferir]**.
- **Plano Dawes** (1924-08-16, em vigor 1924-09-01): anuidade de 1 bilhão subindo a 2,5 bilhões no
  quinto ano, sem total; garantias em alfândega, álcool, tabaco, açúcar, ferrovias e hipoteca
  industrial; empréstimo externo de 800 milhões; Comitê de Transferência; saída do Ruhr até
  1925-08-25. WP "Dawes Plan". Degraus intermediários **[conferir]**.
- **Plano Young** (assinado 1930-01-20, em vigor 1930-05-17, retroativo a 1929-09-01): 36 bilhões
  de Reichsmarks até 1988; parte incondicional em divisas (cerca de 600 milhões por ano) e parte
  adiável; BIS no lugar da Comissão; empréstimo Young de 1,47 bilhão a 5,5 %; Renânia evacuada até
  1930-06-30. WP "Young Plan" (The Plan; Subsequent events).
- **Moratória Hoover** (proposta 1931-06-20) e **Lausanne** (1932-06-16 a 07-09; 3 bilhões finais
  em títulos ao BIS; vigência condicionada a acerto com os Estados Unidos, que não veio; nunca
  ratificado). WP "Hoover Moratorium"; "Lausanne Conference of 1932" (Result and consequences).
- **Dívidas interaliadas:** acordo britânico de 1923 (3 % por 10 anos, 3,5 % depois); França,
  Mellon–Bérenger (1926, ratificado 1929); falta francesa em 1932-12-15; Johnson Act (1934).
  WP "Mellon–Berenger Agreement"; "Johnson Act"; FRUS 1934,
  <https://history.state.gov/historicaldocuments/frus1934v01/d442>. Principal e prazo (US$ 4,6
  bilhões, 62 anos) **[conferir]**.
- **Haiti** (ordenança de 1825-04-17): 150 milhões de francos pelo reconhecimento, reduzidos a 90
  em 1838; empréstimo de 30 milhões (24 recebidos). The New York Times, "The Ransom" (2022-05-20) e
  os dados em <https://github.com/nytimes/haiti-debt>; WP "Haiti indemnity controversy". Fim dos
  pagamentos (1888 ou 1947) **[conferir]**.
- **Indonésia**, Mesa Redonda de Haia (1949): dívida colonial assumida, repudiada em 1956. WP
  "Dutch–Indonesian Round Table Conference". Valor (4,3 ou 4,5 bilhões de florins) **[conferir]**.
- **Filipinas:** Tratado de Manila (1946-07-04), Bell Trade Act (preferência tarifária, peso a 2
  por dólar, paridade de direitos), Acordo de Bases (1947, 99 anos; 25 pelo Ramos–Rusk de 1966;
  renovação recusada pelo Senado filipino em 1991-09-16). WP "Bell Trade Act"; "United States
  bases in the Philippines".
- **Evian** (1962-03-18): Mers-el-Kébir por 15 anos, sítios do Saara, garantias aos europeus por
  3 anos, cooperação. WP "Évian Accords". Saída de Mers-el-Kébir (1967 ou 1968) **[conferir]**.
- **Lancaster House** (1979-12-21): terra só de vendedor voluntário por 10 anos, 20 % das cadeiras
  por 7. WP "Lancaster House Agreement"; N. Kriger (2007).

### Segurança e armas

- **Locarno** (assinado 1925-12-01, em vigor 1926-09-14): garantia das fronteiras ocidentais,
  Reino Unido e Itália garantidores, Renânia desmilitarizada; remilitarização em 1936-03-07. Texto:
  <https://avalon.law.yale.edu/20th_century/locarno_001.asp>; WP "Locarno Treaties".
- **Washington** (1922-02-06, em vigor 1923-08-17): capitais 525/525/315/175/175 mil t, porta-aviões
  135/135/81/60/60 mil t, art. XIX (não fortificação no Pacífico); denúncia japonesa 1934-12-29.
  WP "Washington Naval Treaty"; <https://history.state.gov/milestones/1921-1936/naval-conference>.
- **Londres** 1930 e 1936; **anglo-germânico** de 1935 (35 %, submarinos 45 %). WP "London Naval
  Treaty", "Second London Naval Treaty", "Anglo-German Naval Agreement".
- **Kellogg–Briand** (1928), **Molotov–Ribbentrop** (1939: 10 anos + 5 tácitos), **OTAN** (1949,
  art. 5, 12, 13), **Varsóvia** (1955, art. 4 e 11). Avalon: `kbpact.asp`, `nonagres.asp`,
  `nato.asp`, `warsaw.asp` em <https://avalon.law.yale.edu/20th_century/>. Não agressão
  soviético-polonesa (1932): WP. Garantia britânica à Polônia (1939): Avalon `wwii/blbk19.asp`.
- **Convenção de Viena sobre o Direito dos Tratados** (1969), art. 60: suspensão por violação
  substancial. UN Treaty Series, vol. 1155.

### Reconhecimento

- **Doutrina Stimson** (1932-01-07) e **Relatório Lytton** (adotado 1933-02-24, 42 a 1). WP
  "Stimson Doctrine"; "Lytton Report".
- **Montevidéu** (1933-12-26), art. 1, 3, 8 e 11. Avalon `intam03.asp`.
- **Doutrina Hallstein** (1955–1969/72) e **Tratado Básico** (1972-12-21). WP "Hallstein
  Doctrine"; "Basic Treaty, 1972".
- **China e Taiwan:** Resolução 2758 da ONU (1971-10-25, 76–35–17); Comunicado de Xangai
  (1972-02-28); reconhecimento americano em 1979-01-01; quem reconhece a República da China: 71
  (1969), 23 (1986), 12 (hoje). WP "Foreign relations of Taiwan"; "United Nations General Assembly
  Resolution 2758". Contagens intermediárias **[conferir]**.
- **Diretrizes da Comunidade Europeia** (1991-12-16) e **Comissão Badinter**, pareceres 1 a 3. WP
  "Arbitration Commission of the Peace Conference on Yugoslavia"; EJIL vol. 4 (1993),
  <http://www.ejil.org/pdfs/4/1/1209.pdf>.

### Sujeição, uniões, bases

- **Estatuto de Westminster** (1931-12-11), s. 2, 4, 10; adoção pela Austrália (1942) e Nova
  Zelândia (1947). <https://www.legislation.gov.uk/ukpga/Geo5/22-23/4>; WP "Statute of Westminster
  1931" (Adoption).
- **Pacto da Liga**, art. 22 (mandatos A, B, C). Avalon `leagcov.asp`.
- **Tratado anglo-iraquiano** (1930-06-30, 25 anos: aliança, bases de Shaibah e Habbaniya,
  trânsito). WP "Anglo-Iraqi Treaty (1930)".
- **Protocolo Japão–Manchukuo** (1932-09-15). WP "Japan–Manchukuo Protocol". Cartas anexas
  **[conferir]**.
- **Egito:** declaração unilateral de 1922-02-28 (quatro pontos reservados) e tratado de 1936 (até
  10 mil soldados na zona do canal). WP "Unilateral Declaration of Egyptian Independence";
  "Anglo-Egyptian treaty of 1936".
- **Pacto de Varsóvia e satélites:** tropas soviéticas na RDA e na Hungria (acordos de 1957),
  Tchecoslováquia (1968); Doutrina Brejnev (1968); "doutrina Sinatra" (1989-10-25). WP "Group of
  Soviet Forces in Germany", "Southern Group of Forces", "Brezhnev Doctrine", "Sinatra Doctrine".
  Efetivos na Polônia e na Tchecoslováquia **[conferir]**.
- **Finlândia:** tratado de 1948 (10 anos, renovado); Porkkala devolvida em 1956. WP
  "Finno-Soviet Treaty of 1948"; "Porkkala Naval Base".
- **Ruptura Tito–Stalin** (1948-06-28). WP "Tito–Stalin split".
- **Emenda Platt** (tratado de 1903-05-22, art. I–VII; revogada em 1934-05-29). Avalon
  `dip_cuba002.asp`; <https://history.state.gov/milestones/1899-1913/platt>.
- **Uniões:** Zollverein (1834), Benelux (1944/1948), CEE (1957; tarifa externa comum em
  1968-07-01), franco CFA (1945; conta de operações, depósito de reservas **[conferir]**), área da
  libra (1931, formal em 1939), União Monetária Latina (1865–1927), critérios de Maastricht (1992).
  WP, artigo de cada uma.
- **Bases e passagem:** Constantinopla (1888), Hay–Bunau-Varilla (1903; Avalon `pan001.asp`),
  Torrijos–Carter (1977), Montreux (1936: 20 anos e 2 de aviso), Destroyers for Bases (1940;
  <https://history.state.gov/milestones/1937-1945/lend-lease>), Guantánamo (Avalon
  `dip_cuba003.asp`), Diego Garcia. WP, artigo de cada um.
- **Compra de território:** Alasca (1867, US$ 7,2 milhões;
  <https://history.state.gov/milestones/1866-1898/alaska-purchase>), Ilhas Virgens (1917, US$ 25
  milhões; <https://history.state.gov/milestones/1914-1920/virgin-islands>).

## 17. Etapa 2: ordem e testes

Ordem justificada pela dependência (o que menos depende da guerra primeiro):

1. **Contrato** (`treaty.rs`: `Treaty`, `Clause`, `ClauseKind`, `Proposal`, `Valuation`;
   `clauses.ron`) e **conversão dos 58 tratados de 1925**, com o teste de 1925 ampliado. Fora do
   tick; hash igual ao da `main`.
2. **Livro de dívidas** no tick (serviço semanal, teto de transferência, calote integrado), com as
   dívidas de guerra de 1925 se a decisão 6 for sim. Teste de conservação por cláusula e por semana.
3. **Reconhecimento** e a cláusula `recognition` com o que ela bloqueia.
4. **Proposta, avaliação, ratificação** (comando, `evaluate`, postura dos grupos), com as três
   propostas da IA da seção 10.
5. **Súditos:** feixes, controle e pressão, saída negociada e denúncia.
6. Tarifa, moeda, capital: quando as frentes de comércio e moeda entrarem. Militares: fase 5.

Cada PR de simulação: determinismo de 100 anos (`--check-determinism` com troca de threads e
`--save-at 1975-01-01` recarregado com o mesmo hash), save antigo carregando, CPU de 10 anos contra
a `main`, conservação de dívida e dinheiro antes e depois de cada cláusula.
