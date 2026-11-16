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
  já postas pedem transporte e energia; o porto, só em estado com costa.

## Mapa

A lente "Infra" pinta cada estado pela parcela da demanda atendida (vermelho é falta, verde é
atendida), de eletricidade ou de transporte, com o número no tooltip. Lê o estado da simulação e
recolore uma vez por mês.

## Calibração

Teste de 100 anos (`bf-bench --game --years 100 --check-determinism`), contra a `main` com a fase
3a:

| | 1935 | 1965 | 1995 | 2025 |
|---|---|---|---|---|
| PIB (US$ tri): main / 3b | 6,49 / 6,48 | 20,9 / 21,2 | 60,9 / 64,0 | 87,0 / 92,8 |
| População (bi): main / 3b | 2,18 / 2,17 | 3,48 / 3,46 | 5,12 / 5,09 | 6,78 / 6,74 |
| Urbanização: main / 3b | 0,45 / 0,45 | 0,52 / 0,52 | 0,56 / 0,57 | 0,60 / 0,61 |

- A demanda dos estados fica perto de toda atendida depois de 1945 (transporte e eletricidade);
  sobram em falta poucos estados, quase todos ilhas e enclaves sem vizinho por terra.
- **1926:** a oferta de eletricidade no mundo cobre só 36 % da demanda, e o PIB cai de 5,2 para
  3,8 trilhões entre 1926 e 1927. A `main` tem a mesma queda, com os mesmos números: é anterior a
  esta fatia e fica aberta.

## Aberto

- Rede nacional para o transporte (rodovia, contêiner) e alcance maior que um vizinho por semana.
- Porto ligando estados pelo mar e frete por distância no comércio mundial (fase 4).
- Rede elétrica entre países (interligação), e a hidrelétrica presa ao rio (água como recurso).
