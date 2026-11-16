# Economia: catálogo e regras

Estado: fase 1 concluída (#29). Mercados nacionais fecham toda semana, com comércio mundial em pool; o
investimento privado, o trabalho e a demografia rodam todo mês. O que ficou simplificado está no plano de
melhoria em `architecture.pt-BR.md`, distribuído pelas fases 2 a 4.

Os dados ficam em `data/economy/` (`goods.ron`, `buildings.ron`, `needs.ron`), com o contrato em
`crates/bf-sim/src/economy.rs` e os nomes em `assets/i18n/*/economy.ftl`. O teste
`crates/bf-sim/tests/economy_catalogue.rs` recusa referência quebrada, método que usa um bem antes de ele existir,
bem que ninguém produz e bem civil que ninguém consome.

## Bens por mercado

São 84 bens. Os preços-base estão em dólares de 2011 e ainda serão calibrados.

- **Alimentos básicos:** Cereais, Milho, Arroz.
- **Alimentos:** Carne, Laticínios, Pescado, Frutas e hortaliças, Açúcar, Café, Chá, Cacau, Tabaco, Bebidas, Alimentos industrializados.
- **Produtos agrícolas:** Algodão, Lã, Borracha natural, Fibras vegetais, Madeira.
- **Minérios:** Minério de ferro, Minério de cobre, Bauxita, Ouro, Minério de chumbo e zinco, Cassiterita, Minério de níquel, Minério de manganês, Cromita, Minério de tungstênio, Enxofre, Rocha fosfática, Potássio, Minério de urânio, Minério de terras raras (a partir de 1950).
- **Energia:** Carvão, Petróleo bruto, Gás natural, Combustíveis, Eletricidade.
- **Insumos industriais:** Aço, Alumínio, Cobre, Metais não ferrosos, Ferroligas, Produtos químicos, Fertilizantes, Explosivos, Cimento, Vidro, Papel, Tecidos, Borracha sintética (a partir de 1931), Plásticos, Componentes eletrônicos (a partir de 1950), Combustível nuclear (a partir de 1945).
- **Bens de consumo:** Roupas, Móveis, Utensílios domésticos, Rádios e eletrônicos, Eletrodomésticos, Automóveis, Medicamentos, Artigos de luxo.
- **Bens de capital:** Ferramentas, Máquinas, Motores, Equipamento elétrico, Material ferroviário, Navios, Aviões, Computadores (a partir de 1951).
- **Material militar:** Armas leves, Munição, Artilharia, Blindados, Navios de guerra, Aviões militares.
- **Serviços:** Transporte, Comércio e serviços, Serviços financeiros, Telecomunicações.
- **Construção e serviços públicos (fase 3):** Construção (pontos da fila de obras), Ensino, Atendimento hospitalar.

## Recursos minerais

Dezoito recursos existem desde o primeiro dia, cada jazida com tamanho de 1 a 5, produção de 1925, potencial e ano
de descoberta (`data/scenario/1925/deposits.ron`, contrato em `crates/bf-sim/src/resources.rs`).

- **Críticos desde 1925:** cobre e bauxita, com o alumínio feito por eletrólise (Hall-Héroult): a fundição consome
  muita eletricidade, então alumínio barato depende de hidrelétrica.
- **Ligas que decidem guerras:** manganês, cromo, níquel e tungstênio entram como ferroligas no aço especial, na
  artilharia e nos blindados.
- **Descobertos depois:** jazidas cujo ano de descoberta é posterior a 1925 existem no subsolo, mas ficam ocultas
  até serem achadas. É o caso das terras raras (Bayan Obo identificadas em 1936, mineradas a partir dos anos 1950;
  Mountain Pass, 1949) e de boa parte do petróleo do Oriente Médio, do Mar do Norte e da Sibéria.
- **Urânio:** minerado em 1925 como minério de rádio; vira combustível nuclear a partir de 1945.

## Agropecuária por clima

Cada estado tem a parcela de cada clima de Köppen-Geiger, a área de lavoura, pasto e floresta de 1925, se tem costa,
e a aptidão de 0 a 1 para cada tipo de fazenda (`data/scenario/1925/land.ron`; regras em `LAND.md`). Os tipos:

| Tipo | Produz | Onde |
|---|---|---|
| Lavoura de cereais | Trigo, centeio, cevada, aveia | Temperado oceânico e continental, estepe fria, mediterrâneo |
| Lavoura de milho | Milho | Subtropical úmido, continental quente, savana, planaltos tropicais |
| Arrozal | Arroz | Monção e tropical úmido; subtropical com água; irrigado |
| Pecuária extensiva | Carne, lã | Estepes, savanas, pastagens |
| Leiteria | Laticínios | Oceânico e continental, perto das cidades |
| Plantação de algodão | Algodão | Subtropical, savana, deserto irrigado (Egito, Punjab, Turquestão) |
| Canavial / beterraba | Açúcar | Cana nos trópicos; beterraba no clima temperado |
| Cafezal, chazal, cacaual | Café, chá, cacau | Planaltos tropicais; monção de altitude; floresta equatorial |
| Seringal | Borracha natural | Floresta equatorial |
| Tabaco, pomar, fibras, vinhedo | Tabaco, frutas, juta e sisal, bebidas | Conforme o clima de cada um |
| Exploração florestal, pesca | Madeira, pescado | Floresta; costa |

## Construções

São 85 tipos, cada um com métodos de produção que entram ao longo do século (o ano é um piso; a pesquisa da
fase 2 vai liberá-los por tecnologia).

- **Agropecuária:** Lavoura de cereais, Lavoura de milho, Arrozal, Plantação de algodão, Canavial, Lavoura de beterraba, Cafezal, Chazal, Cacaual, Seringal, Plantação de tabaco, Pomar e horta, Plantação de fibras, Vinhedo, Pecuária extensiva, Leiteria, Exploração florestal, Pesca.
- **Extração:** Mina de carvão, Mina de ferro, Mina de cobre, Mina de bauxita, Mina de ouro, Mina de chumbo e zinco, Mina de estanho, Mina de níquel, Mina de manganês, Mina de cromo, Mina de tungstênio, Mina de enxofre, Mina de fosfato, Mina de potássio, Mina de urânio, Mina de terras raras, Campo de petróleo, Campo de gás.
- **Indústria pesada:** Siderúrgica, Usina de alumínio, Fundição de cobre, Fundição de metais, Usina de ferroligas, Refinaria, Indústria química, Fábrica de fertilizantes, Fábrica de explosivos, Cimenteira, Refinaria de terras raras, Usina de combustível nuclear, Fábrica de borracha sintética, Fábrica de plásticos, Fábrica de máquinas, Fábrica de motores, Indústria elétrica, Fábrica de automóveis, Fábrica de material ferroviário, Estaleiro, Fábrica de aviões, Indústria eletrônica.
- **Indústria leve:** Vidraria, Fábrica de papel, Fábrica de tecidos, Confecção, Indústria de alimentos, Fábrica de móveis, Fábrica de utensílios, Fábrica de ferramentas, Indústria farmacêutica, Ateliê de luxo, Fábrica de eletrodomésticos.
- **Energia:** Usina elétrica.
- **Infraestrutura:** Ferrovia, Transporte rodoviário, Porto, Construção civil.
- **Serviços:** Comércio, Banco, Central de telecomunicações, Escola e Hospital (públicos; ver `state.pt-BR.md`).
- **Indústria bélica:** Fábrica de armas, Fábrica de munição, Fundição de artilharia, Fábrica de blindados, Arsenal naval, Fábrica de aviões militares.

## Profissões e necessidades

Treze profissões formam a chave da tabela de população (estado × comunidade × profissão). As necessidades têm quatro
níveis, do básico ao afluente; dentro de cada uma, a população troca de bem quando o preço muda (cereais por arroz
ou milho; carvão por lenha, querosene ou gás). Os números por habitante são de partida.

## Ordem de trabalho do mercado

1. Preço por estado e por mercado nacional: oferta e demanda fecham semanalmente, com preço sensível à escassez.
2. Estoques, para que a falta e o excesso não se resolvam no mesmo tick.
3. Comércio entre mercados nacionais, com custo de transporte.
4. Moeda, câmbio e inflação, por último.

Cada passo entra com o benchmark de 100 anos verde e com a calibração de 1925 conferida: produção mundial de cada
bem, PIB e consumo por habitante dentro das faixas dos dados históricos.
