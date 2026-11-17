# Capital entre países: o fundo que investe fora (fase 4)

Estado: desenho; decisões C1 a C7 tomadas pelo dono em 2026-11-17 (seção 5). O estudo e a conta
do retorno estão em `late-industrialization.pt-BR.md` (parte 2). O contrato puro está em
`crates/bf-sim/src/foreign_capital.rs`, fora do tick. Este documento diz como ele entra no tick:
em que passo do agendador, o que o save guarda, como se testa e o que o dono decide.

## 1. Como é na vida real, e se a diferença importa

- **O fluxo é gradual, mas feito de projetos.**
  - O investimento direto chega como fábricas, cada uma com data, dono e contrato.
  - O estoque cresce por camadas. Cada camada guarda as condições do ano em que entrou: a lei,
    a parcela do sócio e o prazo da empresa mista.
  - Na China, o investimento direto recebido no ano passou de 0,03 % do PIB em 1980 a 1,1 % em
    1991 e 6,2 % em 1993 (`data/history/fdi.csv`), depois que a lei e as zonas abriram. Subiu
    ano a ano, não de uma vez.
- **O fluxo responde ao retorno com atraso.** Entre decidir e produzir há a obra. No jogo, o
  atraso é a fila de construção que já existe; não é um atraso inventado.
- **O contrato não muda com a lei.** Uma empresa mista de 1985 segue com o prazo de 1985 mesmo
  que a lei mude em 1990. Só a estatização mexe na fábrica existente
  (`late-industrialization.pt-BR.md`, "A nacionalização futura").
- **A diferença importa?** Sim, em dois pontos.
  - A passagem da fábrica ao sócio no fim do prazo depende do ano de cada camada.
  - Uma estatização mede quanto de cada dono foi tomado.

  Um estoque único por país, sem camadas, não sabe quando cada fábrica passa ao sócio, nem de
  quem era o que foi tomado. Por isso o estoque é guardado por camada: lugar, tipo de edifício,
  dono, ano e contrato.

## 2. O que entra no estado

`GameWorld::foreign`, campo novo **no fim** do mundo salvo, com duas partes:

| Parte | O que guarda | Tamanho |
|---|---|---|
| `stakes` | camadas: estado, tipo de edifício, país dono, níveis de propriedade estrangeira, parcela do sócio local, ano, prazo e modo de passagem | esparso: só onde houve investimento |
| `expropriations` | por país: as estatizações sem pagamento (ano, parcela tomada), para o prêmio que dura uma geração | algumas linhas por país, no século |

- As camadas **não entram na `Plant`**. A planta continua sendo estado × tipo, com níveis; a camada
  diz quantos desses níveis são de quem.
- Quando a planta perde níveis (depreciação, fechamento por prejuízo), as camadas perdem na mesma
  proporção, em ordem fixa.
- O estado só entra no hash quando não está vazio. **Sem nenhum investimento feito, o hash é o da
  `main`.**

### As fábricas das colônias

**Como é hoje.** A colônia não é outro país na economia: é um conjunto de regiões do mercado da
metrópole, marcadas `colony` (`econ::Region`). Dividem com a metrópole o mesmo fundo, o mesmo
Tesouro e a mesma moeda. A lei colonial muda os efeitos nessas regiões. No lucro de uma fábrica
colonial (`week.rs`, divisão entre donos, Estado e fundo):
- o dividendo dos donos fica com os capitalistas e proprietários que moram naquela região, se
  houver;
- se não houver, vai para o fundo do mercado, que é o da metrópole;
- o que não é dividendo vai para o mesmo fundo, e a parte estatal vai para o Tesouro da metrópole.

O fundo da metrópole já investe nas regiões coloniais como investe em casa. Na independência
(`borders::op`), a região passa ao mercado novo com as fábricas, e a metrópole perde tudo, sem
indenização e sem nenhum resto de propriedade.

**O que a camada muda.** Nada enquanto a colônia é colônia. Para a economia, fábrica colonial é
fábrica de casa, e não há camada estrangeira entre a metrópole e a própria colônia. A camada passa
a existir em dois casos:
1. **Capital de um terceiro país.** Uma fábrica americana no Congo belga é camada americana numa
   região do mercado belga. Vale a lei de capital da Bélgica para as regiões coloniais.
2. **Na independência.** Esta é a pergunta para o dono (decisão 6). Na vida real, as fábricas da
   metrópole em geral seguiram dela depois da independência, até serem compradas ou estatizadas
   anos depois (o canal de Suez, o cobre do Congo, o petróleo da Argélia; as datas entram com
   fonte no PR que fizer a regra). No jogo de hoje, elas
   passam inteiras ao país novo no dia da independência. Com a camada, a independência poderia
   criar camadas da ex-metrópole sobre as fábricas que o fundo dela construiu. Isso daria ao jogo o
   ciclo real: remessa de lucro para fora, e mais tarde uma estatização com memória e prêmio de
   risco.

O jogo hoje não sabe quem construiu cada nível de uma fábrica colonial: o fundo é um só para
metrópole e colônias. Para semear a camada na independência, seria preciso uma de duas coisas:
- marcar, desde 1925, os níveis que o fundo construiu em regiões coloniais;
- ou, na independência, tratar como da metrópole todos os níveis não estatais, menos uma parcela
  dos donos locais.

## 3. Onde entra no agendador

| Passo | Sistema e cadência | O que faz |
|---|---|---|
| **Escolher onde investir** | `economy-investment`, mensal (dia 8), depois das obras em casa | Lê a sobra de cada fundo (`InvestmentChanges::leftover`, já exposta e fora do hash). Monta os lugares candidatos e reparte com `foreign_capital::allocate`. Põe as obras na fila do país de destino, pagas pelo fundo da origem, com a camada criada quando a obra termina |
| **Remessa do lucro** | `economy-week`, semanal (segunda), onde o lucro se divide entre donos, Estado e fundo (`week.rs`, `to_owners`) | A parte estrangeira do lucro de cada planta com camada sai do país de destino e entra no fundo da origem (`foreign_capital::remittance`), menos o que é reinvestido lá |
| **Contrato e estatização** | `economy-year`, anual (1º de janeiro), depois da política | Passa ao sócio as camadas cujo prazo venceu (`local_share`). Quando a lei de capital vira fechada com camadas no país, ou a economia vira planificada, as camadas viram do Estado e a estatização vai para a memória |
| **Técnica transferida** | `research`, mensal (dia 22) | O canal de imitação do #79 ganha o termo da planta estrangeira: `transfer_rate`, a parcela estrangeira dos níveis de cada grupo |
| **Influência** | `influence`, mensal, quando entrar no tick (desenho em `influence.pt-BR.md`) | O canal "investimento", hoje dormente, lê as camadas: o valor das plantas de B cujo dono é A, sobre o PIB de B |

**Quem pode investir onde:**
- a lei do destino abre a porta: `foreign_investment` maior que zero, com a parcela estrangeira
  permitida pela lei;
- o reconhecimento é condição: capital não vai de quem não reconhece para quem não é reconhecido
  (`treaties.pt-BR.md`, "Reconhecimento");
- a cláusula `capital_protection` de um tratado em vigor baixa o prêmio de risco entre as partes;
  estatizar contra ela é quebra de tratado.

**Candidatos a destino** (o custo depende disto):
- só os tipos de edifício que rodam cheios no país de origem (a planta estrangeira vende para fora:
  `late-industrialization.pt-BR.md`, seção 1 da parte 2);
- só os estados com trabalhadores ociosos;
- só os países cuja lei não fecha a porta.

**Dinheiro:** transferências em unidades reais a preço-base, as mesmas do fundo e do Tesouro. O
efeito no câmbio fica para quando entrar o balanço de pagamentos (decisão 4).

## 4. Como se testa

- **Conservação** (teste de unidade e uma semana do jogo):
  - investir fora tira do fundo da origem exatamente o que entra como obra no destino;
  - a remessa tira do destino exatamente o que entra no fundo da origem;
  - a soma mundial de fundos, tesouros e poupanças, antes e depois de cada passo, muda só pelo
    que a produção e o consumo da semana mudam, e não pelo movimento entre países;
  - a renda mundial da semana é a mesma com e sem a remessa: só muda quem recebe.
- **Neutralidade:**
  - com todas as portas fechadas por lei, o hash de 10 e de 101 anos é o da `main`;
  - com o estado vazio, nada entra no hash.
- **Direção** (critérios de `late-industrialization.pt-BR.md`, seção 7 da parte 2, rodadas com
  `BF_LAWS`):
  - China fechada: zero de investimento recebido;
  - China com empresa mista em 1979 e zonas em 1980: investimento recebido acima de 1 % do PIB
    antes de 1995;
  - Índia aberta em 1991: perto de 1 % nos anos 2000;
  - país rico e aberto não vira destino por salário;
  - quem estatiza recebe menos da metade do fluxo de antes por pelo menos dez anos;
  - referência: `data/history/fdi.csv`.
- **Save antigo:** o campo novo vai no fim do mundo salvo e vazio. O `decode_world` já tenta o save
  antigo completado com zeros no fim, e um vetor vazio é um zero em postcard. O hash não muda
  (estado vazio fora do hash). Entra no teste `saves_of_every_layout_load_with_their_hash`.
- **Determinismo:** a repartição já não depende da ordem dos destinos (teste do contrato). No tick,
  as camadas ficam em ordem de criação, e as reduções seguem a ordem de id. Rodada de 101 anos com
  troca de threads e salvar/recarregar.
- **CPU:**
  - por mês, poucos países com sobra, vezes os tipos cheios na origem, vezes os estados com
    ociosos nos países abertos: dezenas de milhares de avaliações de `site_return`, cada uma
    poucas multiplicações;
  - por semana, a remessa percorre as camadas, algumas centenas a poucos milhares no fim do
    século;
  - meta: menos de 5 % no sistema `economy-investment` e menos de 1 % na semana, contra a `main`.

## 5. Decisões do dono

Decididas em 2026-11-17:

| | Decisão | O que vale |
|---|---|---|
| C1 | Camadas fora da planta | sim, como na seção 2 |
| C2 | Estoque de 1925 | zero agora; **semear depois**, num PR de dados, quando houver fonte utilizável. Candidato para os EUA: Lewis (1938). Reino Unido e França ficam como dependência anotada |
| C3 | Veto nos dois sentidos | sim, `SetCapitalEmbargo` |
| C4 | Câmbio | remessa sem efeito no câmbio agora; o efeito entra com o balanço de pagamentos |
| C5 | Técnica pela planta estrangeira | sim, como **efeito de primeira ordem** ("isso é muito importante para o processo de globalização"): a tela do país mostra quanto da difusão vem de fábrica estrangeira, e a direção é testada contra `fdi.csv` |
| C6 | Fábricas da colônia na independência | decidida em 2026-11-17: **sim às cinco partes** do padrão da seção 7 (C6a acordo sem cláusula; C6b cláusula `ColonialPlants`; C6c revolta; C6d terra fora da camada; C6e indenização posterior, que pode ser renegociada e pode não ser cumprida) |
| C7 | Parte dos donos locais na independência | decidida em 2026-11-17: **a densidade de capitalistas da colônia sobre a da metrópole, até 1** (`foreign_stakes::local_owners`), no lugar da "fração do dividendo que, no último ano, ficou com os capitalistas da própria região" da decisão 6, que no código vira tudo ou nada (a divisão semanal dá o dividendo inteiro de um estado aos capitalistas dele se houver um só). Na Nigéria de 1925: 2.137 níveis britânicos em 108 plantas |

O texto abaixo é o que foi levado ao dono, com a recomendação de cada uma.

1. **Camadas fora da planta, por lugar, tipo, dono e ano.**
   - **Recomendação:** sim, como na seção 2.
   - **Teste:** a passagem ao sócio no fim do prazo e a estatização parcial dão os números do
     contrato, camada por camada.
   - **Custo:** uma tabela esparsa no save. A alternativa, um dono na própria planta, não guarda
     camadas de anos diferentes nem dois donos no mesmo estado.
2. **Estoque de 1925: zero ou semeado.**
   - **Fontes conferidas** (existência, cobertura e licença; nenhum número tirado delas):

     | Fonte | Onde está aberta | Cobertura | Licença | Serve? |
     |---|---|---|---|---|
     | Cleona Lewis, *America's Stake in International Investments* (Brookings, 1938) | Internet Archive, cópia da Digital Library of India `in.ernet.dli.2015.223914` (PDF e OCR legíveis) | **só os EUA como dono**: investimento direto por país de destino (apêndice D) e carteira por país devedor (tabela E-5), em 1897, 1908, 1914, 1919, 1924, 1929 e 1935 | obra americana de 1938; não achei renovação de copyright no registro de 1963 a 1968, logo está em domínio público nos EUA (inferência da busca, não parecer) | em parte: a linha dos EUA de 1924 e 1929 |
     | Royal Institute of International Affairs, *The Problem of International Investment* (1937) | Internet Archive (`bmshri.problemofinterna0000unse_m1m5`; reimpressão de 1965 em `in.ernet.dli.2015.149924`) | Reino Unido, EUA e França como donos, com os países devedores um a um, de anos variados e compilados de terceiros | incerta: obra britânica de autores nomeados, ainda vendida; a marca "domínio público" do arquivo não garante | não, até haver parecer da licença |
     | Liga das Nações, *Balances of Payments* (anual, anos 1920 e 1930) | National Library of Scotland (volumes de 1935 e 1937 abertos; 1926 a 1930 não achados) | fluxos por país declarante; alguns países trazem passivo e ativo externo, sem o país da outra ponta | copyright da ONU, uso não comercial e de pesquisa (CC BY-NC-ND), sem raspagem | não: não diz quem é dono de quê |
     | ONU, *International Capital Movements during the Inter-War Period* (1949) | só a reimpressão de 1979, em empréstimo controlado; a biblioteca da ONU não tem o arquivo | não conferida | copyright da ONU | não |
     | Twomey (tabelas do site do autor) e Dunning | Twomey só no Wayback Machine; Dunning só em livro | Twomey: países de destino em 1913, 1929 e 1938, parcela do PIB | sem licença declarada; copyright do autor | não |

   - **Recomendação:** começar em zero neste passo. Depois, um PR de dados semeia o estoque
     americano por país de destino a partir do Lewis: interpolado entre 1924 e 1929, direto
     separado de carteira (só o direto vira camada), com a interpolação anotada. Os donos
     britânico, francês, holandês e alemão ficam como dependência anotada até haver fonte de
     licença clara. O dono decide se esse semeio parcial entra. Ele torna os EUA o único dono
     estrangeiro em 1925, o que é perto do real no México, em Cuba e no Canadá e falso na
     Argentina, na Índia e no Egito, onde o dono era britânico.
   - **Teste:** com zero, o hash dos primeiros meses é o da `main`. Com o semeio, a soma das
     camadas americanas por país bate com a tabela do Lewis interpolada.
   - **Custo:** com zero, o capital estrangeiro de 1925 não existe no jogo até alguém investir. O
     semeio americano é um PR de dados pequeno, com a tabela transcrita à mão do apêndice D.
3. **O que o jogador comanda.** Já decidido em 2026-11-16: a lei do próprio país e uma lista curta
   de países vetados.
   - **Recomendação:** o veto entra como comando (`SetCapitalEmbargo`) e vale nos dois sentidos:
     não investir lá e não aceitar de lá.
   - **Teste:** com o veto, nenhuma camada entre os dois.
   - **Custo:** baixo, um comando e uma lista no estado do país.
4. **Câmbio.**
   - **Recomendação:** transferências em unidades reais agora, sem efeito no câmbio. O câmbio passa
     a ler o saldo de capital quando o balanço de pagamentos entrar com o comércio.
   - **Teste:** conservação em unidades reais.
   - **Custo:** um país que recebe muito capital não valoriza a moeda por isso até lá; fica
     anotado.
5. **Técnica pela planta estrangeira, já neste passo.**
   - **Recomendação levada:** sim, com o termo `transfer_rate`, que já existe.
   - **Decidido (C5): efeito de primeira ordem.** Isso muda três coisas:
     - **Na tela do país:** a aba de pesquisa mostra, por grupo de técnica, a difusão do mês
       repartida em três partes: a do canal de imitação sem fábrica estrangeira, a parte que vem
       das fábricas estrangeiras e a parte que vem da entrega de técnica ao sócio. Mostra também a
       parcela estrangeira dos níveis desse grupo no país.
     - **No contrato:** `FOREIGN_SPILLOVER` e `PARTNER_SPILLOVER` deixam de ser valores de desenho
       fixos e passam a ser calibrados na rodada de direção do passo 5, como o resto do canal.
     - **Na direção:** os tigres (Coreia, Taiwan, Singapura) e a China depois de 1978 devem ganhar
       técnica mais depressa à medida que o investimento direto recebido sobe em `fdi.csv`. Os
       mesmos países com as portas fechadas por `BF_LAWS` servem de contraprova: a mesma
       trajetória sem as fábricas estrangeiras precisa ficar visivelmente atrás.
   - **Teste de unidade:** com 20 % dos níveis de um grupo em mãos estrangeiras, a difusão anda
     1,2 vez mais rápido; com a exigência de entrega ao sócio, 1,4 vez (valores de desenho, até a
     calibração).
   - **Custo:** uma leitura das camadas por mês no sistema de pesquisa, e três números a mais por
     grupo no que a tela lê.
6. **A independência e as fábricas da ex-metrópole** (seção 2, "As fábricas das colônias").
   Aberta: a regra abaixo foi a primeira proposta, e o estudo da seção 7 a substitui.
   - **Recomendação levada:** na independência, os níveis privados das fábricas não agrícolas da região
     viram camada da ex-metrópole, sem contrato, menos a parte dos donos locais. Essa parte é a
     fração do dividendo que, no último ano, ficou com os capitalistas da própria região. É a
     segunda saída da seção 2, e não pede marcar nada desde 1925.
   - **O acordo de independência pode mudar o padrão** (com o thread dos tratados). Hoje
     `treaty_exec` já cria país novo por acordo (`ReleaseCountry`, que vira `borders::Secede`) e
     põe no livro a dívida com que o país nasce (`Debt`, `DebtTerm::Agreement`). Entra uma
     cláusula nova, `ColonialPlants { kept }`, executada logo depois do território, como a dívida:
     - **sem a cláusula, num acordo:** vale a regra padrão acima;
     - **`kept` = 1:** tudo fica com a ex-metrópole, inclusive a parte dos donos locais;
     - **`kept` entre 0 e 1:** a ex-metrópole guarda essa parte da regra padrão;
     - **`kept` = 0 com uma cláusula `Debt` no mesmo acordo:** nada fica com a ex-metrópole; o país
       novo compra as fábricas e a indenização vai para o livro de dívida, com os termos do acordo;
     - **independência por revolta, sem acordo** (a operação de fronteira sem tratado): nenhuma
       camada, como hoje. As fábricas passam inteiras ao país novo e não há indenização. Isso não
       conta como estatização, porque não havia camada para tomar. Enquanto a ex-metrópole não
       reconhecer o país novo, a regra do reconhecimento já barra o capital entre os dois.
   - **Teste:**
     - uma colônia sem capitalistas próprios sai da independência com camada da ex-metrópole em
       todas as fábricas privadas;
     - a remessa começa na semana seguinte;
     - estatizar em seguida gera a memória e o prêmio de risco;
     - antes de qualquer independência, o hash é o da `main`;
     - com o acordo, `kept` = 1 dá a camada inteira, e `kept` = 0 com dívida não dá camada e põe a
       linha no livro, de valor igual ao das fábricas pela regra padrão;
     - a mesma independência por revolta não deixa camada nem dívida.
   - **Custo:** uma regra na operação de fronteira, a leitura do dividendo por região (que hoje
     não é guardado) e uma cláusula a mais no catálogo dos tratados, sem tipo de tratado novo. Fica para o passo 4, junto com a estatização. A alternativa é deixar como
     hoje, com tudo passando ao país novo; aí o ciclo de remessa e estatização dos anos 1950 a
     1970 não aparece.

## 6. Ordem dos PRs

1. Este desenho, em rascunho.
2. Estado e camadas no save, sem fluxo, com hash igual ao da `main` (`foreign_stakes`, neste mesmo
   PR, assumindo a decisão 1).
3. Fluxo mensal e remessa semanal, com os testes de conservação (seção 8).
4. Contrato, estatização e técnica, em duas partes:
   - **4a. Fábricas da colônia (C6a, C6c sem garantia) e o 1º de janeiro.** Feito:
     - **Independência:** na secessão de uma região colonial, por acordo ou por revolta, os
       níveis privados das plantas que não são fazenda, nem públicas, nem informais viram camada
       da ex-metrópole, sem contrato, menos a parte dos donos locais
       (`foreign_stakes::colonial_stakes`). A parte do Estado (`state_ownership` da lei colonial
       da véspera) passa ao país novo. Níveis que já são de outro dono estrangeiro ficam com ele.
     - **Parte dos donos locais (C7, decidida pelo dono em 2026-11-17):** a regra levada
       era "a fração do dividendo que, no último ano, ficou com os capitalistas da região". No
       código ela vira tudo ou nada: a divisão semanal dá o dividendo inteiro do estado aos
       capitalistas dele se houver um só (`econ::week`). Toda região da Nigéria tem algum, e
       nenhuma camada nascia. Entra no lugar a densidade de capitalistas no estado sobre a da
       metrópole em casa, até 1 (`foreign_stakes::local_owners`). Com ela, a Nigéria de 1925
       sai com 2137 níveis britânicos em 108 plantas.
     - **1º de janeiro, depois da depreciação** (`foreign_stakes::january`):
       - sai a camada cujo contrato já passou a planta inteira ao sócio; os níveis ficam na
         planta, agora do país;
       - em cada estado cuja lei em vigor fecha o capital (`foreign_investment` até a do
         `closed_capital`) ou planifica a economia (`state_ownership` de 900‰ para cima), as
         camadas e as obras pagas de fora na fila passam ao país sem pagamento. Onde a lei de
         casa do país (não a das colônias) toma, o lucro retido de fora também passa;
       - a memória ganha uma linha por dono tomado, com a parcela do capital estrangeiro do país
         (camadas pelo custo de um nível ali) que era dele. As linhas de um mesmo janeiro somam a
         parcela tomada do todo, e o prêmio de risco já soma as linhas. O formato do save não
         muda.
       - A lei lida é a do dia: uma lei aprovada no ano toma no janeiro seguinte. Na revolta com
         o capital fechado ao nascer, a C6c pede "na hora"; aqui é no 1º de janeiro seguinte.
   - **4b.** A estatização com pagamento (c), a compra (j), a diluição pela lei (k), a cláusula
     `ColonialPlants` com garantia e paridade (C6b, com o thread dos tratados), a técnica
     (`transfer_rate` no canal de imitação, C5) e o termo do contrato nas leis.
5. Canal de influência e rodadas de direção (China e Índia).

Cada passo fecha com o hash ou com a rodada, e o placar de investimento direto contra `fdi.csv`
entra no passo 5.

## 7. As fábricas da metrópole na independência: o estudo da C6

O dono pediu: "veja historicamente como isso aconteceu, a história irá nos revelar quais os
fluxos possíveis". Esta seção traz os desfechos reais, quando cada um aconteceu e quanto tempo
levou. Deles saem as opções da cláusula `ColonialPlants` e o padrão de cada situação. As datas e
os valores ficam com a fonte na rodada de referência; a regra não depende de nenhum deles.

### Como é na vida real

**Os desfechos possíveis.** Os pedidos (a) a (g) apareceram todos, e o estudo achou mais seis,
de (h) a (m).

| | Desfecho | Casos |
|---|---|---|
| a | o dono estrangeiro fica e remete o lucro | Índia (casas britânicas, saindo aos poucos nos anos 1950 a 1970); Union Minière no Congo até 1967; Fria na Guiné; Malásia até os anos 1970 |
| b | compra com empréstimo ou com dinheiro da metrópole | Irlanda (terra comprada com empréstimos da Land Commission antes de 1922); Quênia (Million-Acre Scheme, desde 1962, com doações e empréstimos britânicos); Zimbábue nos anos 1980; Zâmbia 1969 (51 % comprados com títulos) |
| c | estatização com indenização | petróleo na Índia, 1974–77; petróleo da Argélia, 1971; Birmânia 1963; Suez, acertado em 1958 |
| d | estatização sem indenização, seguida de sanção ou litígio | Indonésia 1957–58; Union Minière 1967 (a SGM se recusa a vender o cobre); Guiné 1961; Uganda 1972; Zimbábue 2000; Irlanda 1932 (anuidades retidas, guerra tarifária) |
| e | garantia ou direito de paridade com prazo, no acordo | Mesa-Redonda (Indonésia, 1949); Évian (Argélia, 1962); Lancaster House (Zimbábue, 10 anos); Bell Trade Act (Filipinas, até 1974) |
| f | saída dos colonos | Argélia (quase um milhão em meses de 1962); Angola e Moçambique (463 mil retornados em 1977); Guiné 1958; asiáticos de Uganda em 1972 |
| g | o país novo indeniza a metrópole como dívida | Haiti (1825; os empréstimos em cadeia só terminam em 1947); Indonésia (dívida da colônia assumida em 1949) |
| h | a metrópole indeniza os próprios colonos | França, lei de 1970; Guiné 1958 |
| i | indenização disfarçada de contrato de gestão ou de taxa sobre as vendas | Congo (SGM, 4,5 % das vendas desde 1967, depois 6 %); Zâmbia 1973 |
| j | compra a preço de mercado ou em bolsa | Índia nos anos 1950; Malásia (Guthrie, comprada na Bolsa de Londres em 1981) |
| k | lei que obriga o dono estrangeiro a diluir sua parte | Índia (FERA, 1973: até 40 %); Malásia (NEP, 1971) |
| l | devolução posterior aos donos | Uganda (lei de 1982); um caso isolado na Argélia (1998) |
| m | confisco com o nome de "bem vacante", depois que o dono sai | Argélia (decretos de março de 1963) |

**O que se repete:**
1. **Independência negociada: o dono estrangeiro fica de 5 a 30 anos.** A tomada vem depois de uma
   ruptura política, quase nunca na data da independência. Exemplos: Birmânia (15 anos), Congo
   (6,5 anos, depois do golpe), Indonésia (8 anos), Índia (26 a 30 anos) e Malásia (24 anos).
2. **Guerra com muitos colonos: eles saem em meses, e a terra é confiscada em menos de um ano.** A
   mina e o petróleo das empresas da metrópole demoram mais: o petróleo argelino ficou francês
   por 8,6 anos, e a Fria continuou estrangeira. A terra cai rápido; a mina, devagar.
3. **A garantia do tratado tem prazo, e o conflito vem quando ela vence.** No Zimbábue, a cláusula
   valia 10 anos e a tomada veio aos 20. Nas Filipinas, a paridade durou 28 anos. Em Évian, a
   garantia virou letra morta em 8 meses, porque os colonos já tinham saído.
4. **A terra dos colonos se resolve de três jeitos:**
   - a metrópole paga a compra (Irlanda, Quênia, Zimbábue nos anos 1980);
   - o país novo toma quando a garantia acaba (Zimbábue, 2000);
   - a metrópole indeniza os próprios colonos (França; Portugal ainda não, 50 anos depois).
5. **Quase toda estatização termina num acordo de soma fixa,** abaixo do pedido, pago em parcelas
   e com juro baixo. Leva de 2 anos (Suez) a 17 (Indonésia) ou 40 (Zimbábue, ainda não pago).
6. **A metrópole consegue o acordo com pressão:** tarifas (Irlanda), ameaça de cortar a ajuda
   (Argélia), dinheiro bloqueado (Egito), boicote de venda (o cobre do Congo). Sem pressão, o caso
   fica aberto (colônias portuguesas).
7. **Depois de perder a propriedade, o antigo dono continua recebendo parte do fluxo** pela
   indenização em forma de contrato (Congo, Zâmbia).

**A diferença importa?** Sim, por três motivos.
- O fluxo de remessa dura de 5 a 30 anos e sai do país novo para a ex-metrópole. Entregar tudo no
  dia da independência, como o jogo faz hoje, apaga esse fluxo.
- A estatização gera a memória e o prêmio de risco que o capital lê depois. Sem camada, não há o
  que estatizar, e o país novo não carrega nem a reputação nem o acordo de indenização.
- A terra e a fábrica têm destinos diferentes (regularidade 2). Uma regra única para as duas erra
  uma delas.

### Como cada desfecho entra no jogo

| | No jogo | Peça |
|---|---|---|
| a | camada da ex-metrópole com remessa | passo 3 (já existe) |
| b, g | `kept` = 0 e uma linha no livro de dívida (o país novo deve à ex-metrópole, ou a metrópole empresta para a compra) | `Debt` no acordo (já existe em `treaty_exec`) |
| c | estatização com pagamento: a camada passa ao Estado, a indenização vira linha no livro, e o prêmio de risco cai na parte paga | passo 4, ação nova |
| d | estatização sem pagamento: memória e prêmio de risco cheios; a ex-metrópole pode responder com o veto (C3) | passo 4 |
| e | garantia com prazo: estatizar antes do fim do prazo é quebra de tratado, como a cláusula `capital_protection` com fim | campo `guarantee_years` da cláusula |
| e (paridade) | o fundo da ex-metrópole é tratado como nacional por N anos (sem o prêmio da lei, e entra mesmo com a porta filtrada) | campo `parity_years` da cláusula |
| f | os colonos (os donos locais e as comunidades de fora) saem | depende das comunidades e da migração (thread da demografia); fica anotado |
| h | transferência do Tesouro da metrópole para a poupança dos colonos que voltam | junto com (f) |
| i | a camada vira contrato: o antigo dono recebe parte do lucro por N anos, e depois a planta passa ao Estado (`Handover::AtTerm`) | passo 4, como forma de pagamento de (c) |
| j | o fundo do país novo compra a camada pelo valor | passo 4, ação nova |
| k | a lei de capital ganha a parcela máxima do estrangeiro; ao mudar, as camadas acima dela são vendidas pelo valor | leis de contrato, passo 4 |
| l, m | (l) fica fora por ser raro; (m) é (d) depois de (f) | — |

### Padrão, por situação (decidido: C6a a C6e)

1. **C6a. Independência por acordo, sem cláusula:** como na regra da decisão 6. A ex-metrópole fica
   com os níveis privados não agrícolas, menos a parte dos donos locais, sem prazo e sem garantia.
   O que vem depois sai das leis do país novo: fechar o capital ou planificar a economia estatiza
   pela regra de (c) ou (d). Isso dá a regularidade 1 sem impor uma data.
2. **C6b. Independência por acordo, com a cláusula `ColonialPlants`:**
   - `kept`: a parte da ex-metrópole, de 0 a 1;
   - `guarantee_years`: o prazo da garantia (e);
   - `parity_years`: o prazo da paridade (e);
   - com `kept` = 0, uma cláusula `Debt` no mesmo acordo faz a compra (b) ou a indenização (g).
3. **C6c. Independência por revolta, sem acordo.** Aqui a história mudou a proposta anterior, que
   era "nenhuma camada".
   - **Fábricas e minas:** a camada nasce como no acordo, mas sem garantia. A propriedade não some
     na data; quem a toma é o país novo, pelas suas leis. Se ele nasce com o capital fechado ou a
     economia planificada, estatiza na hora, sem pagamento, com memória e prêmio de risco (d).
   - **Terra:** sai da camada (regularidade 2). Fica com os donos locais da região, e a saída
     deles depende das comunidades (f).
4. **C6d. Terra agrícola, em qualquer situação:** fora das camadas. O dono é quem mora na região (o
   colono, no jogo um proprietário local). A compra financiada pela metrópole (b) entra como
   cláusula `Debt` do acordo quando a saída dos colonos (f) existir.
5. **C6e. Acordo de indenização posterior:** um tratado com cláusula `Debt` entre o país que
   estatizou e o antigo dono reduz o prêmio de risco na proporção do que foi **de fato pago**
   (regularidade 5). A pressão da regularidade 6 é o veto (C3) e, depois, a influência.

### Renegociar e não cumprir (C6e)

O dono lembrou que a dívida de indenização precisa poder ser renegociada e que os tratados podem
não ser cumpridos. As duas coisas já têm regra geral no thread dos tratados, que é o dono do livro
de dívida e de `clauses.ron` (`treaties.pt-BR.md`, seção 6, graus de calote, e seção 9). Aqui elas
só ganham o que é próprio do capital.

**Renegociar.** A indenização é uma linha do livro de dívida como qualquer outra. A revisão que o
`treaty_exec` já executa (`revise_debt`: prazo, juro, parte adiável, perdão parcial; só o credor dá)
vale para ela sem regra nova.
- **O prêmio de risco lê o pago, não o prometido.** Cada estatização na memória guarda o valor
  tomado. O livro guarda, em cada linha, o que foi de fato pago (`paid`) e o que uma revisão
  perdoou (`forgiven`), ao lado do que falta (`principal`) e do atraso (`arrears`). Isso vem do
  thread dos tratados, com o degrau de save.
  - A parte do prêmio que cai é `pago / tomado`, com o pago lido da linha da indenização entre os
    dois países.
  - Uma moratória congela a queda, porque não há pagamento.
  - O perdão parcial reduz a dívida mas não soma ao pago: o prêmio da parte perdoada fica.
  - Os tratados dão também `treaty::indemnity_paid_share` = `paid / (paid + principal + arrears +
    forgiven)`, a parcela paga do que foi acordado, que a tela mostra.
- Isso repete o caso da Indonésia: o acordo de 1966 fixou uma soma abaixo do pedido, paga em 30
  parcelas a 1 %. O capital olha o que entra, não o tratado.

**Não cumprir.** Qualquer das partes pode deixar de cumprir. A quebra é o teste mensal falhar,
como na seção 9 dos tratados, e custa pela regra geral:
- registro no acordo;
- suspensão recíproca: o lado prejudicado pode suspender o que dava em troca;
- causa medida pelo valor da cláusula quebrada;
- reputação do país que quebrou, que sobe com cada quebra e cai 10 % ao ano.

No capital, a quebra custa também:

| Quem quebra | Como | Custo próprio do capital |
|---|---|---|
| país novo | para de pagar a indenização (atraso, calote parcial, repúdio) | a memória de expropriação volta a pesar na parte não paga; a ex-metrópole pode aplicar o veto (C3) sem quebrar nada, por suspensão recíproca |
| país novo | estatiza dentro de `guarantee_years` | expropriação sem pagamento com memória e prêmio cheios, mais quebra da garantia (causa e reputação) |
| país novo | veta o fundo da ex-metrópole, ou o dilui à força, dentro de `parity_years` (ver abaixo) | quebra da paridade: causa e reputação; a ex-metrópole pode vetar o capital entre os dois sem quebrar, por suspensão recíproca |
| ex-metrópole | com paridade recíproca (como a do acordo Laurel-Langley de 1955 nas Filipinas), não trata o fundo do país novo como nacional | quebra da paridade dela: causa e reputação; o país novo pode suspender a garantia e a indenização em troca, sem quebrar |

**A IA decide pela mesma conta dos tratados:** quebra quando o valor de quebrar passa o custo
medido (causa, reputação, suspensão recíproca e, aqui, o prêmio de risco que o próprio país
passará a pagar ao capital de todos). A tela mostra a conta à vítima.

**Testes de jogo:**
1. **Renegociação.** O país estatiza 100 de capital da ex-metrópole e assina uma indenização de 60
   em 10 parcelas. Depois de pagar 30, o prêmio de risco é o de 70 % tomados sem pagar.
   - O credor dá moratória de 3 anos: o prêmio não cai nesses 3 anos e não é quebra.
   - O credor perdoa 20: a dívida fica em 10, e o prêmio continua sendo o de 70 %.
   - Pagos os 10 restantes, o prêmio é o de 60 % tomados sem pagar, decaindo pela meia-vida
     como sempre.
2. **Calote na indenização.** O mesmo país para de pagar:
   - a linha vai para atraso e depois para calote, pela regra do livro;
   - cada parcela não paga é quebra com a ex-metrópole;
   - a reputação dele sobe pelo valor não pago;
   - a ex-metrópole ganha causa e pode vetar o capital entre os dois sem quebra própria.
3. **Estatização dentro da garantia.** Independência com `guarantee_years` = 10; o país novo fecha
   o capital no ano 6:
   - as camadas viram do Estado sem pagamento;
   - a memória e o prêmio são cheios;
   - a garantia quebra: causa e reputação.
   - No ano 11, o mesmo ato é só estatização, sem quebra de tratado.
4. **Paridade não cumprida.** Com `parity_years` = 20:
   - se o país novo filtra a lei de capital no ano 5, o fundo da ex-metrópole continua com a
     porta aberta e não há quebra;
   - se o país novo veta a ex-metrópole no ano 5, é quebra da paridade, com causa da ex-metrópole,
     reputação do país novo e o direito dela de vetar de volta sem quebrar;
   - com a paridade recíproca, se a ex-metrópole veta o país novo no ano 5, é quebra dela, e o
     país novo pode suspender a indenização sem que isso conte como quebra.
5. **Sem quebra, nada muda.** Um acordo cumprido até o fim não deixa registro de quebra, e o prêmio
   segue só a memória das estatizações e o que foi pago.

**Como entra no catálogo dos tratados** (combinado com o thread dos tratados):
- **`ColonialPlants { kept, guarantee_years, parity_years }` é de execução única.** Reparte a
  propriedade das fábricas logo depois do território, no mesmo lote da `Debt` em `treaty_exec`.
  Na assinatura, ela gera duas cláusulas contínuas com fim, uma para cada teste:
  - **a garantia é a `capital_protection` que já existe:** o país novo dá, a ex-metrópole recebe,
    com `until` = assinatura + `guarantee_years`. O teste mensal dela é "expropriação sem pagamento
    = quebra", e é aí que cai estatizar dentro do prazo;
  - **a paridade é a cláusula nova `capital_parity`:** o país novo dá, a ex-metrópole recebe, com
    `until` = assinatura + `parity_years`. Com a paridade recíproca, cada lado dá uma.
- **O que é "tratado como nacional"** é o que o sistema de capital mede para o teste mensal da
  `capital_parity`. Enquanto a cláusula vigora, o fundo de quem recebe deve ter, no país de quem
  dá:
  - **a porta aberta:** `Openness::OPEN` para esse dono, qualquer que seja a lei de capital em
    vigor. Não entra o prêmio da lei, e não há parcela máxima, prazo nem transferência;
  - **nenhum veto:** nenhum `SetCapitalEmbargo` de quem dá contra quem recebe;
  - **nenhuma diluição forçada:** as camadas desse dono ficam fora da venda obrigatória quando a
    lei baixa a parcela máxima (desfecho k).

  O sistema de capital publica, a cada mês, uma função `parity_kept(world, giver, receiver) ->
  bool` com essas três condições. O teste da cláusula a lê, e a quebra passa pela regra única
  (`treaty::breach`, com preço pelo tamanho da obrigação). Enquanto a cláusula vigora, o próprio
  sistema de capital já aplica a porta aberta a esse dono. Por isso a quebra só acontece por ato
  de quem dá: o veto, ou uma diluição que não isenta esse dono.
- **Custo próprio do capital:** a memória de expropriação e o veto (C3) são aplicados por este
  sistema, lendo o registro de quebra.
- **Ordem:** as duas cláusulas entram no PR das cláusulas novas (promessa, embargo,
  `ColonialPlants`), depois do PR da negociação e da quebra, que traz `breach` e `reputation`. Até a
  estatização e a lei de capital estarem no tick, elas nascem dormentes (`front: ForeignCapital`).
  Os testes de jogo 1 a 4 são escritos nesse PR, contra o contrato deste sistema no `main`.

O livro, a revisão, os graus de calote, a reputação e a suspensão recíproca já são do thread dos
tratados.

### Testes

- **Acordo sem cláusula:** a colônia sem capitalistas próprios nasce com a camada da ex-metrópole
  em todas as fábricas privadas não agrícolas, e a remessa começa na semana seguinte.
- **Cláusula:**
  - `kept` = 0 com `Debt` dá zero camada e uma linha no livro;
  - `guarantee_years` = 10 torna quebra de tratado estatizar no ano 9, e não no ano 11.
- **Revolta:** com o capital fechado ao nascer, a camada vira estatização no mesmo dia, com
  memória e prêmio; com o capital aberto, a camada fica.
- **Antes de qualquer independência:** o hash é o da `main`.

### Fontes

Abertas, todas lidas. As marcadas com "trecho" só foram vistas no resultado da busca, porque a
página bloqueou o acesso.

- **Índia:**
  - Khanna e Palepu, NBER 2005: https://www.nber.org/system/files/chapters/c10272/c10272.pdf
  - FERA 1973 (ORF): https://www.orfonline.org/expert-speak/43065-70-policies-foreign-exchange-regulation-act-1973
- **Indonésia:**
  - Lei 86/1958: https://pasal.id/peraturan/uu/uu-no-86-tahun-1958
  - Tratado de 1966: https://wetten.overheid.nl/BWBV0004205
  - Mesa-Redonda (Wikipédia, que cita Kahin 1961): https://en.wikipedia.org/wiki/Dutch%E2%80%93Indonesian_Round_Table_Conference
- **Argélia:**
  - Évian: https://mjp.univ-perp.fr/france/1962-1903evian.htm
  - Decreto 63-88: https://archive.gazettes.africa/archive/dz/1963/dz-government-gazette-dated-1963-03-26-no-16.pdf
  - Saída dos colonos: https://refugeehistory.org/blog/2019/9/17/victims-of-decolonisation-the-french-settlers-of-algeria
  - Petróleo, 1971: https://www.larousse.fr/archives/journaux_annee/1972/115/afrique
- **Congo e Zâmbia:**
  - NRGI, *Copper Giants*: https://resourcegovernance.org/sites/default/files/nrgi_NMC_English.pdf
  - Lei Bakajika: https://www.stanleyville.be/loi_bakajika.html
- **Egito:**
  - FRUS: https://history.state.gov/historicaldocuments/frus1955-57v16/d1
  - Hansard 1958: https://api.parliament.uk/historic-hansard/written-answers/1958/may/05/suez-canal-company-and-egypt-agreement-1
  - Hansard 1959: https://api.parliament.uk/historic-hansard/commons/1959/mar/16/anglo-egyptian-financial-agreement
- **Quênia:**
  - Hansard 1962: https://api.parliament.uk/historic-hansard/written_answers/1962/dec/18/land-settlement
  - Hansard 1970: https://api.parliament.uk/historic-hansard/written_answers/1970/dec/18/kenya
- **Zimbábue:**
  - HRW 2002: https://hrw.org/reports/2002/zimbabwe/ZimLand0302-02.htm
  - ISS 2020: https://issafrica.org/iss-today/zimbabwes-unaffordable-billion-dollar-land-deal
- **Filipinas:**
  - Quasha, 1972: https://lawphil.net/judjuris/juri1972/aug1972/gr_l_30299_1972.html
  - Bell Trade Act (Wikipédia, que cita Shalom 1980): https://en.wikipedia.org/wiki/Bell_Trade_Act
- **Haiti** (Wikipédia, que cita Brière): https://en.wikipedia.org/wiki/Haitian_independence_debt
- **Irlanda:**
  - Acordo de 1938: https://www.difp.ie/docs/1938/British-Irish-tripartite-agreement-on-trade-finance-and-defence/2321.htm
  - Guerra tarifária: https://en.wikipedia.org/wiki/Anglo-Irish_trade_war
- **Uganda** (IRB Canadá, 1990): https://www.ecoi.net/de/dokument/197460
- **Malásia:** https://www.ehm.my/publications/articles/imperial-icons-to-malaysian-flagships-the-fall-of-british-business-in-malaysia
- **Guiné** (Rivière): https://webguine.site/bibliotheque/sociology/riviere/mobilization_people/chap3.html
- **Birmânia:** https://www.encyclopedia.com/social-sciences-and-law/economics-business-and-labor/businesses-and-occupations/burmah-castrol-plc
- **Angola e Moçambique:** https://observador.pt/2025/01/29/maioria-dos-partidos-quer-resolver-pedidos-dos-espoliados-das-ex-colonias-portuguesas/

**Sem fonte, ou só em trecho de busca:**
- datas das guerras do Mau Mau, da Irlanda e de Angola e Moçambique;
- o destino dos ativos americanos nas Filipinas depois de 1974;
- os valores pagos à Esso, à Caltex e à Burmah Shell (este último só em trecho);
- a devolução em massa em Uganda nos anos 1990;
- a indenização belga de 1974 no Congo (só numa compilação sem autor);
- o dia de fevereiro de 1971 no petróleo argelino e a lei francesa 70-632 (só em trecho).

## 8. O fluxo no tick (passo 3)

O que entrou, e onde cada parte do desenho ficou para depois.

- **Investimento mensal** (`econ::monthly::abroad`, no fim de `economy-investment`):
  - **Sobra para fora:** o fundo só manda para fora o que sobrou depois de esgotar a lista de casa.
    Se o teto de obras do mês (`MAX_BUILDS`) parou a lista de casa, a sobra para fora é zero: o
    fundo tem obra esperando em casa para o mês seguinte. Sem essa regra, a primeira rodada punha
    China e Índia entre os maiores donos de fábrica no exterior, só porque o teto deixava dinheiro
    parado no fundo deles.
  - **Lugares:** os tipos de edifício que o país de origem roda cheios (sem fazenda, serviço
    público nem trabalho informal), com o método da maior planta dele, nos 4 estados de cada país
    aberto com mais mãos ociosas. A mina respeita a jazida.
  - **Mãos para ocupar a fábrica:** o lugar só entra com as mãos ociosas do estado (camponeses,
    trabalhadores sem emprego e informais), menos as que as obras já na fila vão contratar
    quando terminarem. A fila chegava a se acumular em estados pequenos demais para ocupá-la.
  - **Salário com piso:** o retorno esperado usa o salário de mercado do país quando há leitura
    (pelo menos 10 mil empregados, `WAGE_READING_HANDS`). Sem leitura, usa o menor salário entre
    os países que têm uma (`site_wages`). Os microestados liam salário perto de zero, o que dava
    custo de trabalho zero e retorno máximo.
  - **Segunda trava, o teto de absorção:** um país recebe por ano no máximo 10 % do PIB do ano
    anterior (`MAX_INFLOW`), contando o reinvestido. O valor é o percentil 90 de `fdi.csv`.
    A primeira trava, a das mãos, tira a causa; esta segura o que sobra.
  - **Quantos lugares:** cada fundo olha os 8 tipos que o país mais tem entre os que roda cheios
    (`ABROAD_KINDS`); as firmas vão para fora nos ramos que lideram em casa. Os lugares que não
    batem o retorno de casa saem antes da repartição. Isso limita o custo de CPU.
  - **Repartição:** `foreign_capital::allocate`, contra o retorno mínimo da origem (`hurdle`).
    Cada lugar recebe os níveis inteiros que o dinheiro paga, e as mãos ociosas de cada estado
    se gastam na ordem dos fundos. As sobras pequenas vão, um nível por vez, aos lugares de maior
    peso.
  - **Pagamento:** o fundo da origem paga ao fundo do país de destino o custo do nível no ato. A
    obra entra na fila do país de destino e é paga por ele à medida que os construtores entregam.
    A obra paga de fora fica em `ForeignState::building` e vira camada quando termina.
- **Remessa semanal** (`econ::week::settle_region`): a parte estrangeira do lucro positivo de
  cada planta (`ForeignState::owners_in`, já descontado o que o contrato passou ao sócio) não
  entra na divisão de casa.
  - **Parcela remetida:** 60 % vão para o fundo do dono.
  - **Parcela reinvestida:** 40 % ficam no fundo do país de destino, como crédito do dono
    (`ForeignState::retained`, `foreign_capital::REINVESTED`).
  - O resto do lucro se divide como em casa. O prejuízo fica no país de destino, como manda
    `foreign_capital::remittance`. O PIB continua do país de destino.
- **O lucro reinvestido** (pedido do orquestrador: "lucro reinvestido no destino é parte grande do
  investimento direto").
  - **Como é na vida real:** cerca de um terço do lucro do investimento direto no mundo foi
    reinvestido em 2005–2011, e 49 % nas economias em desenvolvimento em 2011 (UNCTAD, *World
    Investment Report 2013*, fig. I.30, https://unctad.org/system/files/official-document/wir2013_en.pdf).
    Nas filiais de empresas americanas, foi de 0,55 a 0,66 em 2000–2003 (BEA, *Survey of Current
    Business*: setembro de 2002, tabela 6, https://apps.bea.gov/scb/pdf/2002/09September/0902USDIA.pdf;
    setembro de 2006, tabela 7, https://apps.bea.gov/scb/pdf/2006/09September/0906_USDIA_WEB.pdf).
    O valor de desenho é 0,40, dentro dessa faixa.
  - **A parcela varia com choques:** cai para cerca de 20 % numa crise (2008) e fica negativa
    numa anistia fiscal à repatriação: −48 % das filiais americanas em 2018 (BEA, *Survey*, agosto
    de 2019, https://apps.bea.gov/scb/2019/08-august/pdf/0819-direct-investment-positions.pdf).
    Isso fica para o equilíbrio.
  - **A diferença importa:** sem ela, a filial nunca cresce com o próprio lucro, e o país de destino
    vê sair todo mês um lucro que na vida real em boa parte ficava lá.
  - **Como entra no jogo:**
    - o crédito reinvestido paga primeiro, no mês seguinte, níveis inteiros nos lugares desse país
      que batem o retorno de casa do dono, sem transferência, porque o dinheiro já está no fundo de
      lá;
    - o que não acha obra volta para casa aos poucos, 8,3 % ao mês (`RETAINED_RELEASE`), cerca de
      um ano;
    - enquanto espera, o dinheiro está no fundo do país de destino, que pode usá-lo nas suas
      obras, como um depósito.
  - **Fonte para a China:** não achei.
- **Coerência com as plantas:**
  - fechamento por prejuízo (mensal) e depreciação (anual) chamam `plant_shrank`;
  - a operação de fronteira chama `border_changed`.
- **Ficou para depois:**
  - **Abertura pela lei:** sai de `foreign_investment` (`Openness::from_pull`). Fechada no valor
    da lei fechada ou abaixo; acima disso, dono estrangeiro pode ter a planta inteira, sem prazo
    nem transferência. Prazo, parcela do sócio e transferência entram com as leis de contrato
    (empresa mista, zonas de exportação), no passo 4.
  - **Produtividade:** a planta estrangeira roda com a produtividade do país de destino. A técnica
    do dono chega com a decisão C5, no passo 4.
  - **Relação e diáspora:** zero até a influência entrar no tick. O frete é a tarifa da origem.
  - **Veto** `SetCapitalEmbargo` (C3): passo 4.
  - **Reconhecimento:** ainda não é estado do mundo (os tratados não estão no tick). É dependência
    anotada.
- **Testes** (`tests/foreign_flow.rs`):
  - um mês de investimento move dinheiro só entre fundos: a soma mundial dos fundos é a mesma
    antes e depois, e algum fundo perde enquanto outro ganha;
  - toda obra paga de fora está na fila do país de destino, e nenhuma camada passa da planta;
  - uma semana com uma camada inteira põe 60 % do lucro da planta no fundo do dono e 40 % no
    crédito dele no país de destino. O que o dono ganha é o que o resto do mundo deixa de ter
    (fundos, tesouros e poupança), com diferença de no máximo um milésimo, que vem do
    arredondamento;
  - com a lei fechada em todos os países por três anos, o estado estrangeiro fica vazio e, por
    isso, fora do hash.
- **Primeira olhada** (10 anos, semente 7, sem calibrar; decisão de 2026-11-17: funcionalidade
  antes de equilíbrio):
  - em 1935, 16,3 mil níveis estrangeiros, 2,4 % de todos (com a parcela reinvestida; eram 2,1 %
    sem ela);
  - maiores donos: EUA, Alemanha, China e Itália;
  - maiores destinos: Portugal (17 % dos níveis de lá), Espanha, China, Holanda, Raj e Turquia;
  - tem coisa fora do lugar: a Alemanha como grande dona nos anos 30, e Portugal e Irlanda como
    destinos acima da Argentina. Isso fica anotado como dependência, não como meta: o estoque de
    1925 está zerado (C2), e relação, diáspora e técnica do dono ainda são zero. O passo 5
    julga pela conservação, pela direção (o capital vai de onde sobra, com técnica alta, para
    onde o salário é baixo e a porta está aberta) e pelos agregados contra `fdi.csv`, sem
    calibrar país por país (o equilíbrio fino vem depois da fase 6).
