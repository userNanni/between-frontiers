# Diplomacia: o mundo internacional de 1925

Estado: contrato e dados da fase 4, **ainda fora da simulação**. O contrato está em
`crates/bf-sim/src/diplomacy.rs`, os dados em `data/scenario/1925/diplomacy.ron`, os nomes em
`assets/i18n/*/diplomacy.ftl` e as invariantes em `crates/bf-sim/tests/diplomacy_1925.rs`.
`GameData` já carrega o arquivo (campo `diplomacy`); nada o lê ainda.

## O que o contrato cobre

| Parte | Conteúdo | Em 1925-01-01 |
|---|---|---|
| Organizações | Liga das Nações (membros, assentos permanentes, territórios que governa ou protege), União Pan-Americana, Pequena Entente, União Monetária Latina | 55 membros da Liga; Conselho permanente: Reino Unido, França, Itália, Japão |
| Tratados | 58: alianças, amizade, consulta, limite de armas, garantia, paz, reparações, dívida de guerra, ocupação, proteção, união aduaneira e instrumento de sujeição | 51 em vigor |
| Regime monetário | Um período por país e data, com paridade quando houver | ver abaixo |
| Tarifa | Tarifa média de importação de 1925, uma por país | de 2 % a 30 %, média simples de 12,8 % |

Distribuição dos regimes em 1925-01-01 (83 países): papel-moeda inconversível 29, moeda
estrangeira 16, fixo em outra moeda 12, padrão-divisa-ouro 8, prata 8, ouro 6, moeda estrangeira
a peso 4. O mundo estava, na maior parte, fora do ouro: o Reino Unido volta em 1925-04-28, com os
Países Baixos no mesmo dia; França, Itália, Bélgica, Japão e Brasil só voltam entre 1926 e 1930.

### Regras

- **Linhas são história.** Cada linha tem data de início, de fim (quando registrada) e fonte. O
  estado inicial é o que está em vigor na data de início (`treaties_on`, `members_on`,
  `regime_on`); o que começa depois (Locarno, assinado em 1925-12-01 e em vigor em 1926-09-14; a
  Alemanha na Liga em 1926; o retorno britânico ao ouro) é referência para calibrar, como os anos
  das tecnologias, e não roteiro.
- **Sujeição não se repete.** Domínio, mandato, protetorado e satélite estão em `countries.ron`.
  Um tratado do tipo `Subjection` só nomeia o instrumento (Tratado Anglo-Iraquiano, Acordo
  Anglo-Kuwaitiano) e o teste confere que o país é mesmo sujeito daquele suserano. País soberano
  sob tutela (Cuba pela Emenda Platt, Egito pelos pontos reservados, Panamá) entra como `Protection`.
- **Tratado tem dois lados.** Pelo menos duas partes; nos tipos desiguais, `senior` lista o lado
  forte (garantidores, vencedores, ocupantes, credores, o território aduaneiro).
- **União aduaneira impõe a tarifa:** Luxemburgo, Liechtenstein, Mônaco, Sarre e Danzig têm a
  tarifa de Bélgica, Suíça, França e Polônia.
- `DiplomacyFile::validate` recusa tag inexistente, tratado de um lado só, datas incoerentes (fim
  antes do início, vigência antes da assinatura, adesão antes da fundação), país sem regime na data
  de início, regimes sobrepostos, moeda que segue país sem moeda própria e tarifa fora de 0 a 50 %.

## Fontes

- **Tratados e organizações:** textos dos tratados e da Liga das Nações (LNTS), conferidos pela
  Wikipedia em inglês (CC BY-SA 4.0, a mesma licença dos dados; usada só para fatos) e pelo
  Wikidata (CC0). Datas de adesão à Liga: tabela "Member states of the League of Nations"; a saída
  conta dois anos depois do aviso (art. 1º do Pacto).
- **Regimes monetários:** os mesmos, por moeda (artigo de cada moeda e das leis de estabilização).
  Fato e não estimativa: as paridades de ouro (libra 7,322382 g, dólar 1,504632 g, coroa sueca
  0,403226 g, florim 0,6048 g, Reichsmark 0,358423 g) e as datas das grandes voltas ao ouro.
- **Tarifas:** a medida é a de Clemens e Williamson, receita de direitos de importação sobre o
  valor de todas as importações. Só a dos Estados Unidos é citada (U.S. Tariff Commission, cerca de
  14 % sob a tarifa Fordney–McCumber). As séries acadêmicas (Clemens & Williamson 2004; Liepmann,
  *Tariff Levels and the Economic Unity of Europe*, 1938; os *Tariff Level Indices* da Liga, 1927)
  têm direito autoral e não foram transcritas: os números delas podem entrar depois como fato
  citado, linha por linha.

## Estimativas

- **Tarifas: 82 de 83 são estimativas**, pela natureza do regime de cada país: tarifa fiscal de
  país que vive da alfândega (América Central, Venezuela, Grécia: 20 a 30 %), tarifa limitada por
  tratado desigual (China 4 %, Sião 3 %, Pérsia 6 %, Mascate 5 %, Etiópia 10 %), economias abertas
  (Bélgica, Países Baixos, Dinamarca: 4 a 5 %), protecionismo europeu (Espanha 16 %, Polônia
  14 %). A União Soviética tem monopólio do comércio exterior: a tarifa ali é ordem de grandeza.
- **Regimes: 71 dos 88 períodos têm `estimate`**, quase todos pelo dia de início (a suspensão da
  conversibilidade em agosto de 1914, a data em que uma moeda estrangeira passou a circular). As
  marcações que pesam: taxa da coroa húngara contra a libra, colón costa-riquenho a 4 por dólar,
  rupia indiana a 1s 6d antes da lei de 1927, data do abandono da paridade do złoty em 1925, e o
  emissor em Andorra (franco e peseta circulavam) e em Tuva.
- **Tratados:** 17 têm a vigência tomada na assinatura, por falta da data de ratificação; está
  dito na fonte da linha.

## Lacunas conhecidas

- Extraterritorialidade e capitulações (China, Sião até 1927, Pérsia até 1928), concessões e
  zonas de influência acordadas entre terceiros (o acordo tripartite de 1906 sobre a Etiópia).
- Dívidas interaliadas além da britânica (França, Itália e Bélgica com os Estados Unidos, todas
  acertadas entre 1925 e 1926) e as reparações dos aliados menores.
- Tratados comerciais e a cláusula de nação mais favorecida; a preferência imperial britânica.
- Organização Internacional do Trabalho e Corte Permanente de Justiça Internacional.

## O que fica para ligar na simulação

1. **Regime monetário no gancho da 3c.** `econ/money.rs` tem `exchange_target`: hoje toda moeda
   flutua pela paridade do poder de compra com prêmio de risco. A fase 4 troca o alvo conforme o
   regime em vigor (`regime_on`):
   - `Gold` e `GoldExchange` com paridade: o alvo é fixo, a razão entre a paridade e a do país de
     referência; defender a paridade custa reservas e juro, e a perda de reservas força a saída
     (o mecanismo de 1931–33). `GoldExchange` sem paridade legal segue o ouro com uma banda.
   - `Peg`: o câmbio da âncora vezes `per_anchor`; a moeda acompanha a âncora quando ela entra
     ou sai do ouro (a área da libra em 1931).
   - `Foreign`: o país não tem banco central; usa o câmbio e a inflação do emissor.
   - `Paper`, `Silver`, `Specie`: flutuam como hoje; a prata soma o preço mundial da prata (a
     China deflaciona em 1929–31 e sai da prata em 1935).
   Depois vêm as transições como regra e não como data: saída do ouro por perda de reservas,
   Bretton Woods (paridades contra o dólar, dólar a US$ 35 a onça, FMI), flutuação e euro, e a
   moeda de reserva barateando a dívida de quem a emite.
2. **Tarifa no comércio.** O comércio mundial é um pool sem tarifa. A tarifa vira cunha entre o
   preço mundial e o interno e receita do orçamento da 3; a união aduaneira junta os mercados com
   uma tarifa externa só. Frete por distância e porto vem junto.
3. **Reparações e dívida de guerra** entram como dívida em moeda estrangeira (`debt_fx`) da 3c:
   Plano Dawes para a Alemanha, acordo de 1923 para o Reino Unido.
4. **Tratados e Liga** alimentam a guerra (fase 5: aliança chama aliado, garantia chama
   garantidor), a influência e as sanções; ocupação (Ruhr até 1925-08-25, Renânia até 1930) pesa na
   produção e na opinião do ocupado.
