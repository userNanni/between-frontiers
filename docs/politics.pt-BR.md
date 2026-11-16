# Política nacional: leis, grupos de interesse e movimentos

Estado: contrato e catálogo da fase 3. O código está em `crates/bf-sim/src/politics.rs`; os dados,
em `data/politics/` (catálogo) e `data/scenario/1925/laws.ron` (leis vigentes em 1925, método e
fontes em `data/scenario/1925/LAWS.md`); os nomes, em `assets/i18n/*/politics.ftl`. O estado que
muda no jogo (leis em vigor, peso dos grupos, movimentos, legitimidade, registro das mudanças) está
em `crates/bf-sim/src/polity.rs`, salvo e no hash; a dinâmica (efeitos, movimentos, mudança de lei
sem jogador) ainda não roda.

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
| Grupos de leis | 23 (2 coloniais) |
| Leis | 89 (8 dirigidas a comunidades) |
| Efeitos | 37 (10 sobre o que já existe na simulação) |
| Indicadores | 14 (10 já calculados pela simulação) |
| Grupos de interesse | 8 |
| Tipos de movimento | 19 |

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
- **Forma de governo** fica em `countries.ron` (`government`), não vira lei: o voto está em
  `franchise`.

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

## O que fica para ligar na simulação

1. ~~Estado salvo por país e o comando `SetLaw`.~~ Feito (acima). `Market::laws` passa a ser
   derivado das leis no passo 2.
2. Ler os efeitos que já têm onde cair: migração, tetos de imposto e arrecadação, parcela do Estado
   em escola e hospital, meta de matrícula, renda da terra.
3. Medir os indicadores por linha populacional (mensal) e mover os movimentos (anual).
4. Peso político de cada grupo (tamanho × riqueza × direito de voto) e a regra de quando uma lei
   muda: pressão dos movimentos contra a postura ponderada dos grupos, com o governo
   (`countries.ron`) decidindo como o peso conta. Sem isso, as leis ficam paradas no 1925.
5. Os sistemas que os efeitos planejados esperam: sindicato e salário mínimo, previdência, estatal,
   comunidades (discriminação, segregação, assimilação), mídia, colônias, e depois tarifas e
   fronteiras (fase 4) e exército (fase 5).
6. Radicalização: movimento forte e sem resposta vira greve, motim e insurgência (fases 3 e 5).
