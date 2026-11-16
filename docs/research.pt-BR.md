# Pesquisa: tecnologia com maturidade por país

Estado: fase 2 em andamento. O contrato está em `crates/bf-sim/src/research.rs`, os dados em
`data/research/technologies.ron` e os nomes em `assets/i18n/*/research.ftl`.

## A ideia

Não basta pesquisar: a tecnologia tem de chegar ao mercado. Cada tecnologia passa, país por país,
por quatro estágios inspirados na escala TRL:

| Estágio | O que significa | Como se avança |
|---|---|---|
| Ciência | A ideia é entendida | Esforço de pesquisa (engenheiros e profissionais, vezes a alfabetização) |
| Protótipo | Funcionou uma vez | Mais esforço e engenheiros empregados no ramo |
| Industrial | É feita em escala; os métodos dela podem ser construídos | Precisa de comprador: demanda do mercado pelos bens do ramo, ou o Estado como comprador (armas, espaço, nuclear) |
| Difundida | Está em toda parte, com a produtividade inteira | Parcela dos níveis do país que já usa os métodos dela |

- **Operar e produzir são estados distintos.** Um bem existe no mundo quando algum país chega ao
  estágio industrial; daí em diante qualquer país compra e usa pelo comércio, mesmo sem saber fazer.
- **Vale da morte:** sem comprador, a tecnologia fica parada no protótipo.
- **Difusão:** pesquisar o que outros já sabem custa menos, tanto mais quanto maior a parcela do PIB
  mundial que já sabe e quanto mais o país comercia com quem sabe.
- **Curva de aprendizado:** o custo de chegar à indústria cai com o que o mundo já produziu.
- **Licença e engenharia reversa** pulam estágios com teto de eficiência; **embargo** corta o canal
  (fase 4).

## O que a pesquisa muda

- **Métodos:** um método listado por uma tecnologia só pode ser construído no estágio industrial.
  Método que nenhuma tecnologia lista fica sempre disponível. Substitui o `available_from` por ano.
- **Bens:** consumo de um bem novo começa quando algum país o industrializa.
- **Produtividade:** substitui o crescimento provisório da fase 1. Cada tecnologia dá um ganho por
  grupo de edifícios, metade no estágio industrial e inteiro quando difundida. Serviços continuam
  ganhando menos (efeito Baumol).
- **Demografia e sociedade:** vacinas e antibióticos baixam a mortalidade; a pílula, a fecundidade;
  escola de massa e rádio aceleram a alfabetização; computador e internet aumentam o rendimento da
  pesquisa; contêiner barateia o comércio.

## Escolha

O jogador escolhe a tecnologia em foco; o resto do esforço se espalha pelas candidatas. Os outros
países escolhem sozinhos, por regra determinística (barata, pedida pela demanda, do ramo forte do
país), até a IA da fase 6.

## Calibração

O benchmark de 100 anos compara o ano em que cada tecnologia chega ao protótipo e à indústria com os
anos históricos (`demonstrated`, `industrialized`) e acusa desvio grande. A produtividade da
fronteira deve crescer perto de 1,5 a 2 % ao ano.
