# Política nacional: leis, grupos de interesse e movimentos

Estado: contrato e catálogo da fase 3. O código está em `crates/bf-sim/src/politics.rs`; os dados,
em `data/politics/` (catálogo) e `data/scenario/1925/laws.ron` (leis vigentes em 1925, método e
fontes em `data/scenario/1925/LAWS.md`); os nomes, em `assets/i18n/*/politics.ftl`. O estado que
muda no jogo (leis em vigor, peso dos grupos, movimentos, legitimidade, registro das mudanças) está
em `crates/bf-sim/src/polity/`, salvo e no hash, e roda: os efeitos com onde cair chegam à
economia, os movimentos crescem e decaem pela causa medida e os países sem jogador mudam de lei e de
forma de governo sozinhos (seção "Dinâmica").

## O desenho

Três peças, ligadas por **efeitos** (`politics::EFFECTS`, 37 números em milésimos):

```text
lei ──declara──▶ efeitos ◀──pesa── grupo de interesse (base: profissões)
                    │
                    └──medido por── causa do movimento ──quer──▶ lei (ou independência)
```

- **Lei.** Pertence a um grupo de leis; todo país tem exatamente uma lei de cada grupo (os dois
  grupos coloniais, só quem tem colônias). A lei não faz nada por si: declara efeitos sobre o que a
  simulação tem (migração, orçamento, renda) ou terá (sindicato, mídia, comunidades, colônias).
  Há três modos de somar efeitos de leis diferentes: multiplicador (1000 neutro), parcela somada
  (0 neutro) e teto (o menor vale).
- **Grupo de interesse.** Bloco com o mesmo interesse material, tirado das 13 profissões de
  `economy::PROFESSIONS`. Entre os grupos gerais, as parcelas de cada profissão somam 1000: todo
  mundo está em um bloco, ninguém em dois. Um grupo pode nomear uma comunidade (cultura + religião,
  por id) e então só recruta nela; o catálogo de comunidades vem em PR paralelo, e o vínculo é só o
  id, conferido pela sintaxe.
- **Movimento.** Quer uma lei (ou independência) e cresce só enquanto a **causa** vale: condições
  sobre indicadores da simulação (`politics::INDICATORS`) ou sobre os efeitos das leis em vigor,
  medidas em quem ele recruta.

### Causa visível

- **Postura sem rótulo.** A postura de um grupo diante de uma lei é a soma, efeito por efeito, do
  peso do grupo (−3 a 3) vezes a distância da lei ao neutro (`Politics::score`). A tela mostra os
  termos (`Politics::terms`): "os grandes proprietários se opõem porque a renda da terra cai 40 %".
  Não há ideologia atribuída ao grupo. Quem entra no grupo muda quando as profissões mudam, e a
  postura acompanha.
- **Movimento sem sorteio.** Nenhum movimento nasce por evento aleatório. Cada condição tem uma
  margem (quanto o valor passou do limite, em milésimos do espaço até o fim da faixa); a menor
  margem dá o ritmo (`MovementKind::pressure`) e a tela diz qual condição está segurando.
- **Persistência.** Sem a causa, o apoio cai pela metade a cada `persistence` anos. O movimento
  sobrevive à causa e volta mais forte se ela voltar.
- **Movimento que termina.** O teste recusa um movimento cuja lei-objetivo mantém a própria causa.

### Onde a lei vale

- **Metrópole:** leis dos grupos não coloniais e não dirigidas.
- **Comunidade visada:** as mesmas, mais as leis dirigidas (`targeted`: nacionalidade oficial,
  segregação, exclusão, lei religiosa, ateísmo de Estado, leis de passe, exclusão racial). O cenário
  diz, por id, quais comunidades cada lei dirigida atinge; em 1925 a lista fica vazia até existir o
  catálogo de comunidades, e a fonte diz em palavras quem é o alvo.
- **Colônias:** as leis coloniais e a política econômica da metrópole (tributação, propriedade,
  terra, comércio, capital estrangeiro). Trabalho, bem-estar, direitos e mídia da metrópole param
  na fronteira: as colônias tinham códigos próprios. Por isso o francês vota e o senegalês não.
- **País súdito** (RAJ, IRQ, protetorados): leis próprias; o movimento de independência mede as
  leis coloniais do suserano.

### O que é determinístico

Tudo: dados inteiros em milésimos, sem ponto flutuante, sem sorteio. Os tipos de cálculo
(`combine`, `margin`, `score`, `effects`) são funções puras e testadas.

## O catálogo

| | Quantidade |
|---|---|
| Categorias | 7 (economia, trabalho, bem-estar, direitos, informação, Estado, império) |
| Grupos de leis | 24 (2 coloniais; `government` é a forma de governo) |
| Leis | 99 (8 dirigidas a comunidades, 10 formas de governo) |
| Efeitos | 37 (10 ligados à economia) |
| Indicadores | 17 (12 medidos: 11 da economia e a legitimidade) |
| Grupos de interesse | 8 |
| Tipos de movimento | 23 (`income_tax`, `fiscal_reform`, `regime_change` e `revisionism` entraram na fase 3d) |

O que a fase 3 pediu e onde ficou:

- **Saúde ligada a remédios e hospitais:** grupo `health`, efeitos `health_state_share` (parcela do
  hospital paga pelo tesouro; hoje o Estado paga tudo) e `medicine_subsidy` (o bem `medicines` já
  existe).
- **Mídia:** grupos `press` e `broadcasting`, efeitos `censorship` e `state_media`; indicador
  `media_reach` para quando imprensa, rádio, TV e internet virarem edifícios e serviço consumido.
- **Corrupção:** grupo `civil_service`, efeitos `corruption` (parcela do orçamento perdida) e
  `tax_collection`; movimento `anti_corruption`.
- **Descolonização:** grupos `colonial_rule` e `colonial_labour`; movimentos `independence` e
  `colonial_labour_reform`, que recrutam nas colônias.
- **Estatal com lucro para o tesouro:** grupo `ownership`, efeitos `state_ownership`,
  `state_investment_share` e `state_profit_to_treasury`.
- **Leis que já existem:** `internal_passports` dá os mesmos 250/100 de
  `MigrationLaw::Restricted` (o teste confere); os tetos de `taxation` limitam as alíquotas por
  classe do orçamento.
- **Forma de governo** é o grupo de leis `government` (10 leis, as chaves de `government` em
  `countries.ron`). A lei de 1925 vem de `countries.ron` quando `laws.ron` não a nomeia, e muda
  pelo mesmo mecanismo das outras: comando, movimento (`regime_change`) ou grupos. O regime soma
  repressão, censura e privilégio da Igreja aos das leis de polícia, imprensa e religião; o voto
  continua em `franchise`.

## 1925

Os 83 países do cenário têm uma lei por grupo, a partir de 7 modelos (Europa parlamentar, domínio,
Europa centro-oriental, república latino-americana, monarquia tradicional, soviético, microestado)
com substituições país a país. Cada país tem a fonte (constituição e leis da época). Há 132
mudanças datadas de 1925 a 1994 como referência, sem disparar nada, entre elas propiska
(1932-12-27), hukou (1958-01-09) e as leis de passe sul-africanas (em vigor em 1925 pela Natives
(Urban Areas) Act 1923, abolidas em 1986-07-01). O teste confere que cada mudança troca a lei
vigente.

## Fontes

- **Catálogo:** desenho próprio. Os números dos efeitos são ordens de grandeza para calibrar
  quando cada sistema for ligado; o que vale agora é a ordem das leis em cada grupo e quais efeitos
  cada uma move.
- **1925:** os próprios atos legais (fatos, sem texto copiado), citados país a país em `laws.ron`;
  método, confiança e o que conferir em `data/scenario/1925/LAWS.md`.
- **Comparação:** o mod Cold War Era do Victoria 3 só como ordem de grandeza
  (`benchmarks.pt-BR.md`). Nenhum nome, texto ou dado dele nem do Victoria 3 entrou.

## No jogo (`polity.rs`)

- **Estado** (`GameWorld::politics`, `PoliticsState`): lei em vigor por país e grupo (parte de
  `laws_1925`), comunidades visadas, peso de cada grupo de interesse (milésimos), apoio, pressão e
  condição mais fraca de cada movimento, legitimidade (milésimos), pedidos pendentes e o registro
  de cada mudança (data, país, de, para, quem: comando, movimento ou grupos).
- **Comando** `GameCommand::SetLaw { tag, law }` (índice em `Politics::laws`): o pedido fica
  pendente e o sistema `politics` o aplica no mesmo dia; lei desconhecida, lei colonial de país
  sem colônias e etiqueta desconhecida não fazem nada; de dois pedidos para o mesmo grupo vale o
  último.
- **Leitura** para a interface: `PoliticsState::{laws_in_force, law_in_force, clout,
  active_movements, legitimacy, pending_laws, law_changes}` e `polity::{possible_laws, stance,
  effects_in_force, scope_of}`. `stance` compara o conjunto de leis em vigor com o mesmo conjunto
  trocando uma lei e devolve, efeito por efeito, peso × variação do efeito combinado (um teto de
  outro grupo conta como o povo o sentiria) e o total.
- **Peso dos grupos** (mensal, dia 1): metade da parcela das pessoas da metrópole, metade da
  parcela da renda, pelas profissões de cada grupo. Grupos de uma comunidade ficam com zero até as
  comunidades entrarem nos pops.

## Dinâmica

O sistema `politics` (`polity/system.rs`) roda todo dia, mas só faz conta no dia 1 de cada mês e,
para movimentos e mudança de lei, em 1º de janeiro.

### O que a lei faz na economia

O sistema escreve em `Market::laws.effects` (metrópole e colônias) o que as leis em vigor dizem; a
economia lê ali. Efeito que nenhuma lei em vigor declara fica com a regra de antes (sem lei, sem
mudança). Os 14 efeitos ligados e onde caem:

| Efeito | Onde cai | Arquivo |
|---|---|---|
| `migration_within_state`, `migration_between_states` | multiplicam o alcance da migração (junto com `SetMigrationLaw`) | `econ/migration.rs` |
| `tax_cap_lower`, `_middle`, `_upper` | teto das alíquotas de renda por classe, na regra anual, no comando e na troca de lei; o que o teto corta (média das três classes, até 12 %) vira imposto sobre o consumo | `econ/public.rs` (`cap_taxes`), `econ/week.rs` |
| `tax_collection` | parte do imposto lançado que chega ao tesouro; o resto fica com o contribuinte | `econ/week.rs` |
| `health_state_share`, `education_state_share` | parte do hospital e da escola que o tesouro paga; o usuário paga o resto da poupança | `econ/week.rs` |
| `school_enrolment_target` | teto da cobertura escolar que conta para a alfabetização | `econ/yearly.rs` |
| `landowner_rent` | parte do lucro da terra que vai ao proprietário (1 = o pagamento aos donos, como antes); abaixo de 1, a diferença vai ao camponês | `econ/week.rs` |
| `state_ownership`, `state_profit_to_treasury` | parte do pagamento aos donos da indústria que vai ao Estado em vez dos capitalistas; dessa parte, a que entra como receita do Tesouro (o resto é reinvestido) | `econ/week.rs` |
| `tariff`, `foreign_investment` | canal por onde o saber de fora entra na pesquisa: protótipo e indústria do que o mundo já faz, imitação e, com piso alto, a ciência que o mundo já conhece; tarifa até 25 % e capital aberto deixam a regra sem canal (`late-industrialization.pt-BR.md`, "A imitação com canal") | `research_system.rs`, `foreign_capital.rs` |

Os outros 23 efeitos esperam sistema (sindicato, previdência, investimento estatal, comunidades,
mídia, colônias, fronteiras, exército) e não fazem nada ainda.

### Imposto sobre o consumo

O modelo só tinha imposto de renda, e os Estados de 1925 presos à tributação indireta viviam de
tarifa e imposto de consumo. Sem isso, o teto de lei cortava a receita inteira: na primeira
versão, o mundo de 2025 ficava com urbanização de 0,475 contra 0,585 sem tetos, porque menos
receita é menos obra pública (moradia urbana, infraestrutura, escola).

- **Regra:** o que os tetos cortam das alíquotas que a regra pede (média das três classes) é
  cobrado sobre o que os pops compram na semana (`Budget::indirect_rate`), tirado do que sobra
  depois de comprar e poupar. Quem poupa mais paga menos: é regressivo, como foi.
- **Teto do imposto sobre consumo:** 12 % (`public::INDIRECT_CEILING`), derivado do cenário: a
  maior alíquota que a regra histórica pede da classe baixa (1975), que paga sobretudo pelo que
  compra. Sem fonte de receita indireta por país em 1925 no repositório; a tarifa sobre comércio
  de verdade vem com as fronteiras (fase 4).
- **Na tela:** `Budget::revenue_indirect` (imposto sobre consumo da semana) e
  `Budget::revenue_firms` (lucro de estatais ao Tesouro) estão dentro de `Budget::revenue`; o
  imposto de renda é o resto.
- **Na postura:** a classe paga o imposto de renda até o teto mais o indireto. Trocar a tributação
  indireta pela progressiva tira imposto do consumo do camponês e do operário e põe na renda do
  alto; eles passam a apoiar o imposto de renda.

### Postura: a regra geral

- **Efeito combinado.** A postura compara o conjunto de leis em vigor com o mesmo conjunto trocando
  uma lei; o termo é peso × variação do efeito combinado.
- **Teto não é alíquota.** Um teto conta pelo que ele deixa o Estado cobrar: a variação de
  `min(teto, alíquota que a regra pede)` de cada classe (`PoliticsState::taxes`, medido todo mês).
  Subir o teto da classe baixa de 15 para 20 % não custa nada a quem paga 5 %. Vale para todo efeito
  do tipo teto que vier a ter regra por trás.
- **Quem recebe do Estado ganha com a arrecadação.** O que os tetos acrescentam à receita (variação
  de alíquota de cada classe × parcela da classe na renda) entra no termo `tax_collection`, que o
  funcionalismo e os militares pesam a favor.
- No Brasil de 1925, trocar tributação indireta por imposto progressivo deixa camponeses e
  trabalhadores neutros (a regra pede 5 % e 9 %, abaixo dos dois tetos), proprietários e
  empresariado contra (o teto da classe alta passa de 8 % para 15 % efetivos) e o funcionalismo a
  favor.
- **Crise fiscal pesa mais.** O termo de arrecadação é multiplicado por 1 + 2 × `fiscal_stress`
  (até 3 vezes): com o Tesouro sem dinheiro, quem vive do Estado quer mais receita.
- **Falta o custo do gasto.** Lei de bem-estar (saúde, escola, previdência) aparece só pelo
  benefício: ninguém vê o imposto que ela vai exigir. É o próximo ajuste da postura.

### Peso dos grupos

Mensal. Da força política, a parcela com voto (`franchise`, em milésimos) vai por cabeça e o resto
por renda: sem voto só o dinheiro fala; com sufrágio universal, cabeça e dinheiro pesam metade cada.

### Movimentos

- **Pressão:** a margem da condição mais fraca (`margin`), medida em quem o movimento recruta
  (alfabetização, desemprego e necessidades pela profissão; os outros indicadores pelo país;
  efeitos pelas leis em vigor). Indicador que a simulação ainda não mede (preço, desigualdade,
  alcance da mídia, imigrantes, território perdido) dá margem zero: o movimento não cresce.
- **Apoio** (milésimos de quem ele recruta): sobe `growth × pressão` por ano (pelo menos 1), cai pela
  meia-vida `persistence` sem causa ou com a meta em vigor.
- **Mobilização:** apoio × recrutas ÷ população. **Legitimidade** = 1000 − 3 × a soma da
  mobilização dos movimentos sem resposta.
- Movimentos de comunidade (`Targeted`) ficam parados até as comunidades entrarem nos pops;
  `independence` e `revisionism` crescem mas nunca vencem (pedem fronteira e guerra).

### Crise fiscal

A lei de tributação limita o que o Estado arrecada. Duas pontas, cada uma com causa visível:

- **O gasto se ajusta à receita possível.** Com o caixa abaixo de 8 semanas de receita, a folha e a
  parte do Estado em escolas e hospitais caem um quarto por semana de receita que falta, até 10 %
  (`econ::public::pay_share`, `state.pt-BR.md`). As compras e as obras já caíam; a folha, que era
  obrigação, foi o que deixou 18 países com caixa negativo em 2025 na primeira versão.
- **A crise vira pressão política.** O indicador `fiscal_stress` (1 menos `austerity`: caixa que
  não se financia e juros que comem a receita) alimenta o movimento `fiscal_reform` (servidores,
  soldados, escriturários e parte dos trabalhadores, quer imposto de renda por degraus) e aumenta o
  peso da arrecadação na postura.
- **Paliativo: piso da folha em 10 %** (`econ::public::PAY_FLOOR`). Um Estado cuja folha (servidores
  e soldados) custa mais do que qualquer imposto que a lei deixa cobrar, como a China de 1925
  (folha perto de 30 % do PIB, receita abaixo de 7 %), só fecha o caixa pagando um décimo. Na
  história, o partido único vivia do lucro das próprias empresas e da compra forçada da safra. A
  empresa estatal já paga o Tesouro (`state_ownership`), mas nada ainda faz um regime
  revolucionário estatizar. **Condição de saída:** quando a agenda de regime (fase 7) levar a
  China à economia planificada, o piso volta a 1/4 e a China tem que fechar pelas estatais.

### Quando a lei muda sem jogador

Em 1º de janeiro, para cada país sem jogador e cada grupo de leis que não mudou nos últimos 5 anos:

- **Por movimento:** um movimento empurra o **próximo degrau** do grupo na direção da sua meta
  (reforma por degraus: seguro-saúde antes do serviço nacional de saúde); forma de governo cai de
  uma vez. Força do empurrão = mobilização × (1 − repressão). Muda se a mobilização passa de 4 %
  e `4 × empurrão + apoio dos grupos ≥ 300 × legitimidade`.
- **Apoio dos grupos** = soma, sobre os grupos de interesse, de peso × postura (postura de 500 ou
  mais conta inteira, a favor ou contra).
- **Por grupos, sem movimento:** só nos 5 anos depois de uma troca de forma de governo (o regime
  novo reescreve leis), um degrau por vez, com apoio de 70 % do peso.
- Constantes em `polity/system.rs` (`MIN_MOBILIZED`, `PUSH_WEIGHT`, `THRESHOLD`,
  `GROUPS_THRESHOLD`, `COOLDOWN_YEARS`, `REGIME_YEARS`), calibradas pelo placar abaixo.
- Nenhum evento datado nem pressão histórica: as 132 mudanças de `laws.ron` só medem.
- **Portas abertas para a fase 7.** `MovementState::leader` e `PoliticsState::leaders` (por grupo
  de interesse) guardam o personagem que lidera, e `system::leader_push` é o ponto onde o empurrão
  dele se soma à margem de uma causa que já vale (hoje zero; nunca cria causa). O `revisionism` é o
  movimento que a pressão histórica e o casus belli vão alimentar quando houver fronteira e guerra.

### Placar contra a história

`bf-bench --game` grava em `politics.events`, para cada mudança datada de `laws.ron`, uma pergunta
fechada: `id` (`law:<TAG>:<lei>`), `question`, `reference`, `window` (± 5 anos), `happened`,
`simulated`, `delay` (anos) e `verdict` (`on_time`, `early`, `late`, `never`). O placar histórico
(`tools/history/`) pode ler essa lista. `politics.extra` lista as mudanças simuladas fora da
referência e `politics.changes`, todas as mudanças.

Rodada de 100 anos sem jogador (seed 1, 2 threads, sobre a `main` em 971bdf1), data simulada contra a de referência:

| | Mudanças |
|---|---|
| no prazo (± 5 anos) | 4 |
| adiantadas | 7 |
| atrasadas | 28 |
| nunca acontecem | 93 |
| fora da referência | 589 |

Os 12 maiores (referência → simulado; – = nunca):

| País | Mudanças de referência | Outras mudanças simuladas |
|---|---|---|
| CHN | protective_tariffs 1929 → –; factory_acts 1929 → –; open_capital 1943 → –; land_reform 1950 → 1991; planned_economy 1953 → –; internal_passports 1958 → – | one_party_state 1975; compulsory_primary 1977; restricted_suffrage 1985; freehold_farming 1986; male_suffrage 1990; universal_suffrage 1995 |
| RAJ | collective_bargaining 1926 → – | compulsory_primary 1953; workers_insurance 1967; health_insurance 1970; social_security 1972; national_health_service 1975; regulated_market 1976; one_party_state 1979; male_suffrage 1984; universal_suffrage 1989; mixed_economy 1990; freehold_farming 2000 |
| SOV | planned_economy 1928 → –; collective_farming 1930 → –; internal_passports 1932 → –; police_state 1934 → – | restricted_suffrage 1947; male_suffrage 1952; universal_suffrage 1957 |
| USA | regulated_market 1933 → 1970; collective_bargaining 1935 → 1951; social_security 1935 → 1964; labour_code 1938 → –; selective_service 1940 → –; equal_citizenship 1964 → –; health_insurance 1965 → –; volunteer_army 1973 → – | workers_insurance 1959; mixed_economy 1975 |
| GBR | universal_suffrage 1928 → 1956; protective_tariffs 1932 → –; selective_service 1939 → –; universal_secondary 1945 → –; mixed_economy 1947 → –; national_health_service 1948 → –; social_security 1948 → – | separation 1964; free_labour 1967 |
| FRA | health_insurance 1930 → 1951; universal_suffrage 1944 → 1956; social_security 1945 → 2009; mixed_economy 1945 → –; free_labour 1946 → 1943 | labour_taxation 1938; national_health_service 1956 |
| GER | social_security 1927 → 1970; political_police 1933 → –; state_unions 1933 → –; no_free_elections 1933 → –; universal_service 1935 → –; legal_exclusion 1935 → – | mixed_economy 1966 |
| JAP | political_police 1925 → –; male_suffrage 1925 → 1930; health_insurance 1927 → 1935; forced_labour 1939 → –; universal_suffrage 1945 → 1935; collective_bargaining 1945 → 1956; land_reform 1946 → –; labour_code 1947 → –; separation 1947 → 1992 | public_hospitals 1930; national_health_service 1940; workers_insurance 1973; social_security 1978; free_labour 1984; mixed_economy 2003; one_party_state 2007; free_press 2016 |
| ITA | state_unions 1925 → –; prior_censorship 1926 → –; political_police 1926 → –; no_free_elections 1928 → –; legal_exclusion 1938 → –; universal_suffrage 1945 → 1966; national_health_service 1978 → 1942 | health_insurance 1937; labour_taxation 1942; free_labour 1947; social_security 1984; mixed_economy 1985; one_party_state 1988; free_press 1999 |
| BRA | state_unions 1931 → –; no_free_elections 1937 → –; labour_code 1943 → –; universal_suffrage 1985 → 1959 | compulsory_primary 1945; public_hospitals 1946; proportional_income_tax 1947; health_insurance 1951; progressive_income_tax 1952; male_suffrage 1954; national_health_service 1956; press_licensing 1961; free_press 1966; one_party_state 1975; mixed_service 1995; workers_insurance 1996; social_security 2001; collective_bargaining 2003; regulated_market 2019; mixed_economy 2024; freehold_farming 2024 |
| ESP | press_licensing 1931 → 1940; universal_suffrage 1931 → 1942; anticlerical_state 1931 → –; land_reform 1932 → –; state_unions 1938 → –; prior_censorship 1938 → –; press_licensing 1966 → 1940 | restricted_suffrage 1932; male_suffrage 1937; health_insurance 1940; labour_taxation 1943; national_health_service 1945; free_press 1945; free_labour 1948; mixed_service 1963; merit_service 1988; one_party_state 2000; social_security 2019 |
| MEX | mixed_economy 1938 → 2020; workers_insurance 1943 → 1995; universal_suffrage 1953 → 1977 | health_insurance 1940; national_health_service 1945; separation 1954; mixed_service 1968; merit_service 1973; social_security 2000; progressive_income_tax 2004 |

**Ruído ou plausível?** Das 627 mudanças, nenhuma volta à lei anterior; cada país troca de regime
no máximo uma vez (27 trocas no todo); os que mais mudam (Transjordânia e Brasil) fazem 18 mudanças
em 100 anos. O que está fora da referência são, quase todos, os degraus intermediários
(`health_insurance`, `public_hospitals`, `workers_insurance`, `male_suffrage`) e países pequenos
que a referência não cobre. O descanso de 5 anos basta.

**Imigração:** nenhum país muda a lei de imigração em 100 anos. O único movimento do grupo
(`nativism`) quer fechar e mede `immigrant_share`, que ainda não existe; nenhum quer abrir. Os
Estados Unidos ficam nas cotas de 1924 até 2025 (a referência, Hart-Celler, é de 1965). Falta um
movimento de abertura com causa medida (falta de mão de obra, ou a comunidade imigrante, quando as
comunidades estiverem nos pops).

Por que tão pouco no prazo, e qual peça falta:

- **Nenhum movimento pede** 43 das 97 que nunca acontecem: o pacote autoritário (`no_free_elections`
  13, `state_unions` 5, `prior_censorship` 5, `political_police` 4, `police_state`) e
  `public_broadcaster` 8. Só saem quando um regime novo reescreve as leis, e `regime_change` precisa
  de desemprego de 15 % com legitimidade baixa: sem a Depressão (ciclo de confiança, fase 7) isso
  não acontece nos anos 1930, e sim nos anos 1970 a 2010, nos países errados.
- **Guerra e ocupação** (fase 5): serviço militar de 1935 a 1940, trabalho forçado de 1939 e as
  reformas japonesas, alemãs e italianas de 1945 a 1947 (reforma agrária, sindicatos, separação).
- **Comunidades nos pops**: `legal_exclusion` 5 e `equal_citizenship` 3 têm movimento, mas ele é de
  comunidade e fica parado.
- **Agenda do regime revolucionário** (fase 7): plano quinquenal, coletivização, propiska e hukou
  (SOV, CHN) não nascem de causa medida em quem vive sob a lei.
- **Desemprego de massa** para o New Deal e o Estado de bem-estar de 1935 a 1948 (USA, GBR):
  `social_protection` mede desemprego de 8 % e a economia simulada não o produz nesses anos.
- **Atrasos**: o sufrágio cresce pela margem pequena da alfabetização acima de 50 % e chega 10 a 30
  anos tarde na Europa ocidental; adiantado onde o voto foi barrado por causa que o modelo não vê
  (alfabetização exigida no Brasil até 1985, raça na África do Sul).

## Mídia (fase 3)

**O que já existia:** as leis de imprensa (`free_press`, `press_licensing`, `prior_censorship`,
`state_press_monopoly`) e de radiodifusão (`private_broadcasting`, `public_broadcaster`,
`state_broadcasting`), com os efeitos `censorship` e `state_media`; só o apoio à guerra lê a
censura (decisão A2 do dono). O indicador `media_reach` estava declarado e não medido, e nenhum
movimento o lê.

**Fatia 1: o alcance medido (chave `media`, desligada; `media.rs`, `data/politics/media.ron`).**
Decisão MD1 (o orquestrador, pela regra do dono de calibrar sem forçar a história): alcance
**medido** do consumo e da alfabetização, liberado por tecnologia; nunca uma série fixa por país.
- **Jornais** seguem os alfabetizados: exemplares por alfabetizado sobem com a renda por pessoa até
  um teto, e o estoque fecha um décimo da distância por ano (o primeiro ano começa no alvo: a
  imprensa de 1925 já existia).
- **Aparelhos e usuários** são comprados com os eletrônicos das famílias (`consumer_electronics`):
  um décimo do que compram é aparelho de mídia, repartido entre os meios que as tecnologias liberam
  (rádio, depois televisão, depois a web), dividido pelo preço de um aparelho e somado ao estoque,
  que se desgasta pela vida do aparelho. Gradual, como na vida real: o estoque de aparelhos por mil
  é o número intermediário que a tela mostra.
- **Alcance** (`media_reach`, por mil): a parte das pessoas que pelo menos um meio alcança (um
  exemplar é lido por 2,5 pessoas, um aparelho alcança uma casa de 3,5, um usuário é uma pessoa).
- **Valores de desenho**, marcados no dado: a parte dos eletrônicos que é aparelho (0,1), os preços
  e as vidas dos aparelhos, a repartição entre os meios, o teto e a meia renda dos jornais, as
  pessoas por unidade.
- **Nada lê o alcance ainda:** nenhum movimento tem condição sobre `media_reach`. Quais movimentos,
  o apoio à guerra e a legitimidade passam a ler (MD2 a MD6) é a fatia 2, ao dono.

**Conferência (não é meta; UNESCO e ITU, com página em `target/media/research.md` do levantamento:
Liga das Nações 1931/32 tab. 119, *Historical Statistics* dos EUA pp. 796 e 809, UNESCO *Statistics
on Radio and Television 1950–1960*, *World Communications* 1975, WDI do Banco Mundial para 1970 e
2000). Por mil habitantes, jogo / real:**

| | jornais 1950 | rádios 1930 | rádios 1950 | rádios 1970 | TVs 1970 | TVs 2000 | internet 2000 |
|---|---|---|---|---|---|---|---|
| EUA | 318 / 356 | 118 / 98 | 353 / 560 | 379 / ~1.414 | 246 / 395 | 292 / 835 | 206 / 431 |
| Reino Unido | 95 / 612 | 44 / 94 | 142 / 244 | 104 / 624 | 64 / 324 | 98 / 652–892 | 0 / 268 |
| URSS | 64 / ~174 | 0 / ~3 | 93 / 62 | 66 / 430 | 36 / 185 | 106 / 538 | 0 / 20 |
| Brasil | 39 / 30 | 0 / 0,2–5 | 99 / 48 | 86 / 208 | 15 / 64 | 55 / 343 | 0 / 29 |
| Índia | 12 / 8 | 0 / ~0,02 | 49 / 1 | 29 / 31 | 14 / 0,1 | 22 / 78 | 0 / 5 |

- A ordem de grandeza bate em 1930–1950; depois os estoques do jogo ficam baixos nos países ricos
  (rádio e TV dos EUA e do Reino Unido em 1970–2000 a um terço ou menos do real) e altos nos pobres
  cedo (rádios da Índia em 1950). A parte dos eletrônicos que vira aparelho é uma só para todos os
  anos; na vida real ela subiu com a TV a cores e o computador.
- **A internet só chega aos EUA até 2000:** a web (1995) chega ao estágio industrial só lá; é a
  difusão de técnica, a mesma anotada no computador (o Reino Unido em 1972).
- Os jornais do Reino Unido (612 por mil em 1950) ficam a um sexto: o teto por alfabetizado é um
  só para todos.
- Itens de calibração dessa chave, no roteiro de `state.pt-BR.md`.

**Fatia 2: o que a mídia faz (chave `media_effects`, que precisa de `media`; em rascunho até a
resposta do dono a MD2, MD3 e MD6, com as recomendadas).** `media::channels` separa o alcance em
**livre** (mídia que o Estado não tem e não censura), **oficial** (a do Estado, pelo efeito
`state_media`) e a **censura** que sobra depois do vazamento da internet (MD6: acesso total tira
80 % da força da censura).
- **Movimentos** (`polity::system`, MD2 e MD3): crescem até 50 % mais rápido com alcance livre
  total e até 50 % mais devagar com alcance oficial total; o que a mídia do Estado segura vai para
  o movimento de troca de regime (a pressão escondida que sai no colapso, como a A2).
- **Apoio à guerra** (`war::support`, MD2 e MD3, só com `war`): a mídia oficial soma até 100 por
  mil à meta; a livre faz o apoio mudar até 50 % mais rápido; a censura é a de depois da internet.
- **Legitimidade:** segue a mobilização dos movimentos, sem regra nova.
- **Valores de desenho** (`media.ron`, `effects`).
- **Medida (101 anos, `media` contra `media` com `media_effects`, `BF_DIAG_POLITICS`):** as
  mudanças de lei no mundo quase não mudam (437 e 437 até 2025); a legitimidade média também
  (679 e 678). Onde a mídia é do Estado o movimento de troca de regime cresce escondido: URSS 0 →
  94 (1965), 169 (1985), 407 (2025) por mil; Itália 21 e Brasil 27–29 no meio do século. O PIB
  mundial fica dentro da faixa (−0,4 a +0,7 %).

**Decisões ao dono (fatia 2 e 3):**
- **MD2** Onde a mídia age: movimentos, apoio à guerra e legitimidade (recomendada) | só
  movimentos e guerra.
- **MD3** Mídia estatal: freia a oposição e soma ao apoio, e a pressão escondida sai no colapso
  (recomendada, como a A2) | um estoque de credibilidade que cai com o tempo de propaganda.
- **MD4** Propaganda paga: linha do Orçamento com "Regra histórica" × "Fixado pelo governo"
  (recomendada) | só as leis.
- **MD5** Rádio estrangeiro alcançando outros países: depois (recomendada) | agora.
- **MD6** A internet reduz a eficácia da censura na proporção do acesso (recomendada) | não.

## O que fica para ligar na simulação

1. ~~Estado salvo, `SetLaw`, efeitos ligados, movimentos, peso e mudança de lei sem jogador.~~
   Feito (acima).
2. Custo do gasto na postura (lei de bem-estar vista também pelo imposto que pede).
3. Movimento que devolve a democracia (hoje `regime_change` só vai para o partido único) e a
   reescrita de leis pelo regime novo (voto, imprensa, polícia), que hoje só sai pelo caminho dos
   grupos.
4. Indicadores que faltam: inflação, desigualdade, alcance da mídia, imigrantes, território
   perdido (pede as fronteiras de 1914, fase 4).
5. Agenda de regime (fase 7): o regime revolucionário que estatiza (China 1953, URSS 1928) e a
   compra forçada da safra; sem ela, a China fica no "nunca" do placar e no paliativo da folha.
6. Movimento de abertura da imigração já está no catálogo (`immigration_reform`); o `nativism`
   espera `immigrant_share`, que a demografia (#54) vai dar
   (`world.demography.immigrant_share(&world.econ, country)`), ligado em um ponto só
   (`polity::measure::immigrant_share`).
7. Tarifa sobre comércio (fase 4): o imposto sobre consumo faz as vezes dela por enquanto.
8. Os sistemas que os efeitos planejados esperam: sindicato e salário mínimo, previdência,
   investimento estatal,
   comunidades (discriminação, segregação, assimilação), mídia, colônias, e depois tarifas e
   fronteiras (fase 4) e exército (fase 5).
9. Radicalização: movimento forte e sem resposta vira greve, motim e insurgência (fases 3 e 5).
