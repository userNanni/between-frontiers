# Cenário de 1925: leis vigentes

`laws.ron` (formato `bf_sim::politics::ScenarioLawsFile`) dá, para cada um dos 83 países de
`countries.ron`, a lei de cada grupo do catálogo (`data/politics/laws.ron`) em 1º de janeiro de
1925, e uma lista de mudanças datadas depois disso, só como referência. Licença dos dados: CC BY-SA
4.0. Desenho e regras: `docs/politics.pt-BR.md`.

## Método

- **A lei escrita, não a prática.** Onde as duas se afastam muito (o voto do negro no Sul dos EUA,
  a jornada de oito horas na Alemanha depois de 1923, o seguro social português de 1919), a lei
  vale e a fonte diz a diferença. Quem a contorna é a simulação, pelos efeitos de outras leis
  (segregação, polícia, corrupção).
- **Modelos e substituições.** Sete modelos cobrem o comum; cada país troca o que diverge e cita
  as leis de todas as trocas e as mais importantes do modelo. Sem modelo, o país lista as 21 leis
  (EUA e Índia).
- **Ditadura em 1º de janeiro de 1925 conta como ditadura:** Espanha (Primo de Rivera), Chile
  (junta de 1924), Venezuela (Gómez), Albânia (Zogu). A Itália ainda é parlamentar nesse dia; as
  leis fascistíssimas vêm no histórico.
- **Leis dirigidas** (nacionalidade oficial, segregação, leis de passe, exclusão racial, lei
  religiosa, ateísmo de Estado) ficam com `targets` vazio até o catálogo de comunidades existir;
  a fonte diz em palavras quem é o alvo (negros americanos, africanos, mestiços e indianos na
  África do Sul, aborígines na Austrália, povos indígenas no Canadá e na Libéria, judeus na
  Hungria, curdos na Turquia, ainos no Japão, eslavos e alemães nas novas províncias italianas).
- **Colônias:** as 13 metrópoles com `colonies: Some` têm também governo e trabalho coloniais.

## Fontes

As fontes são os próprios atos (constituições, leis, decretos e tratados da época), citados por
nome e data no campo `source` de cada país e de cada mudança. Atos legais são fatos; nada foi
copiado de texto de terceiros. Não se usou base de dados NC nem GPL.

## Confiança

- **Alta:** voto, igreja e Estado, serviço militar, leis de passe e de imigração, tratados de
  capitulação e de tarifa, e as datas do histórico que são marcos conhecidos (propiska
  1932-12-27, hukou 1958-01-09, Nuremberg 1935-09-15, Wagner Act 1935-07-05, Influx Control 1986).
- **Média:** jornada de trabalho e previdência nos países pequenos, radiodifusão onde ainda não
  havia rádio (vale a regra que a lei ou o regime dava), regime de terras fora da Europa.
- **Conferidas em 2026-11-16** por busca: Terra Nova (lei do voto feminino em vigor em
  1925-04-03, não 04-13), Trade Unions Act indiano (sancionado 1926-03-25, em vigor 1927-06-01),
  Uruguai (Ley 8927 de 1932-12-16), Sümerbank (1933-07-11), Pérsia (lei do serviço militar de
  junho de 1925; o dia 6 é a conversão de 16 Khordad 1304, a conferir), Colômbia (Acto Legislativo
  1 de 1936-08-05 existe; que ele tenha dado o voto a todos os homens, a conferir no texto).
- **A conferir antes de ligar:** o restante das datas do histórico, contra o Diário Oficial de
  cada país ou o Wikidata (CC0).
