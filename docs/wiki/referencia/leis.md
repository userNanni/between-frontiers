<!-- Página gerada de `data/` por `crates/bf-sim/tests/wiki_reference.rs`. Não edite à mão: regere com `BF_BLESS=1 cargo test -p bf-sim --test wiki_reference`. -->

# Leis e efeitos

Por categoria e grupo, na ordem do catálogo. Cada efeito em milésimos: os que multiplicam (alcance da migração, tetos de imposto) valem sobre 1.000; os que somam se somam entre as leis em vigor.

## Economia

### Tributação (`taxation`)

| Lei | Efeitos |
|---|---|
| Tributação indireta (`indirect_taxation`) | Teto do imposto, classe baixa 150, Teto do imposto, classe média 100, Teto do imposto, classe alta 80 |
| Imposto de renda proporcional (`proportional_income_tax`) | Teto do imposto, classe baixa 250, Teto do imposto, classe média 250, Teto do imposto, classe alta 250 |
| Imposto de renda progressivo (`progressive_income_tax`) | Teto do imposto, classe baixa 200, Teto do imposto, classe média 350, Teto do imposto, classe alta 500 |
| Imposto fortemente progressivo (`steep_progressive_tax`) | Teto do imposto, classe baixa 200, Teto do imposto, classe média 450, Teto do imposto, classe alta 600 |

### Propriedade (`ownership`)

| Lei | Efeitos |
|---|---|
| Laissez-faire (`laissez_faire`) | Parcela do Estado na pesquisa 250 |
| Mercado regulado (`regulated_market`) | Propriedade estatal 50, Parcela do Estado no investimento 100, Lucro das estatais para o tesouro 300, Parcela do Estado na pesquisa 450 |
| Economia mista (`mixed_economy`) | Propriedade estatal 250, Parcela do Estado no investimento 300, Lucro das estatais para o tesouro 500, Parcela do Estado na pesquisa 550 |
| Capitalismo de Estado (`state_capitalism`) | Propriedade estatal 500, Parcela do Estado no investimento 600, Lucro das estatais para o tesouro 700, Investimento estrangeiro 700, Parcela do Estado na pesquisa 700, Prioridade do Estado para a pesquisa 500 |
| Economia planificada (`planned_economy`) | Propriedade estatal 950, Parcela do Estado no investimento 1000, Lucro das estatais para o tesouro 1000, Investimento estrangeiro 100, Parcela do Estado na pesquisa 950, Prioridade do Estado para a pesquisa 1000 |

### Posse da terra (`land_tenure`)

| Lei | Efeitos |
|---|---|
| Servidão (`bonded_tenure`) | Renda dos proprietários de terra 1600, Trabalho não livre 300, Migração dentro do estado 500, Migração entre estados 500, Acesso dos colonos à fronteira 100 |
| Latifúndio (`great_estates`) | Renda dos proprietários de terra 1300, Acesso dos colonos à fronteira 300 |
| Propriedade livre da terra (`freehold_farming`) | — |
| Reforma agrária (`land_reform`) | Renda dos proprietários de terra 600 |
| Coletivização (`collective_farming`) | Renda dos proprietários de terra 100, Propriedade estatal 100, Migração dentro do estado 700, Migração entre estados 700, Acesso dos colonos à fronteira 0 |

### Comércio (`trade`)

| Lei | Efeitos |
|---|---|
| Livre comércio (`free_trade`) | Tarifa 20 |
| Tarifas fiscais (`revenue_tariffs`) | Tarifa 100 |
| Tarifas protecionistas (`protective_tariffs`) | Tarifa 250 |
| Autarquia (`autarky`) | Tarifa 600, Investimento estrangeiro 300 |

### Capital estrangeiro (`foreign_capital`)

| Lei | Efeitos |
|---|---|
| Capitulações (`capitulations`) | Investimento estrangeiro 1500, Arrecadação 900 |
| Aberto ao capital estrangeiro (`open_capital`) | — |
| Capital estrangeiro controlado (`screened_capital`) | Investimento estrangeiro 600 |
| Fechado ao capital estrangeiro (`closed_capital`) | Investimento estrangeiro 100 |


## Trabalho

### Sindicatos (`unions`)

| Lei | Efeitos |
|---|---|
| Sindicatos proibidos (`unions_banned`) | Repressão 100 |
| Sindicatos tolerados (`unions_tolerated`) | Poder de negociação sindical 200 |
| Negociação coletiva (`collective_bargaining`) | Poder de negociação sindical 600 |
| Sindicato oficial (`state_unions`) | Poder de negociação sindical 300, Salário mínimo 100, Repressão 100 |

### Condições de trabalho (`working_conditions`)

| Lei | Efeitos |
|---|---|
| Trabalho sem regulação (`unregulated_labour`) | Jornada de trabalho 1150, Trabalho infantil 300 |
| Leis fabris (`factory_acts`) | Jornada de trabalho 1050, Trabalho infantil 120 |
| Jornada de oito horas (`eight_hour_day`) | Jornada de trabalho 950, Trabalho infantil 50 |
| Código do trabalho (`labour_code`) | Jornada de trabalho 900, Trabalho infantil 20, Salário mínimo 400 |

### Previdência (`social_insurance`)

| Lei | Efeitos |
|---|---|
| Família e caridade (`family_and_charity`) | — |
| Assistência aos pobres (`poor_relief`) | Aposentadoria 30, Seguro-desemprego 30 |
| Seguro operário (`workers_insurance`) | Aposentadoria 300, Seguro-desemprego 100 |
| Seguridade social (`social_security`) | Aposentadoria 500, Seguro-desemprego 400 |


## Bem-estar

### Saúde (`health`)

| Lei | Efeitos |
|---|---|
| Medicina privada (`private_medicine`) | Parcela do Estado no atendimento hospitalar 50 |
| Hospitais públicos (`public_hospitals`) | Parcela do Estado no atendimento hospitalar 300, Subsídio a remédios 50 |
| Seguro-saúde (`health_insurance`) | Parcela do Estado no atendimento hospitalar 500, Subsídio a remédios 300 |
| Serviço nacional de saúde (`national_health_service`) | Parcela do Estado no atendimento hospitalar 900, Subsídio a remédios 700 |

### Educação (`education`)

| Lei | Efeitos |
|---|---|
| Ensino tradicional (`traditional_schooling`) | Meta de matrícula escolar 50, Meta de matrícula no ensino médio 20, Meta de matrícula no ensino superior 5, Privilégio da Igreja 300 |
| Ensino privado (`private_schooling`) | Parcela do Estado no ensino 200, Meta de matrícula escolar 250, Meta de matrícula no ensino médio 100, Meta de matrícula no ensino superior 20, Privilégio da Igreja 150 |
| Primário obrigatório (`compulsory_primary`) | Parcela do Estado no ensino 700, Meta de matrícula escolar 800, Meta de matrícula no ensino médio 250, Meta de matrícula no ensino superior 50 |
| Secundário universal (`universal_secondary`) | Parcela do Estado no ensino 900, Meta de matrícula escolar 1000, Meta de matrícula no ensino médio 900, Meta de matrícula no ensino superior 300 |


## Direitos

### Direito de voto (`franchise`)

| Lei | Efeitos |
|---|---|
| Sem eleições livres (`no_free_elections`) | — |
| Voto restrito (`restricted_suffrage`) | Direito de voto 150 |
| Sufrágio masculino (`male_suffrage`) | Direito de voto 450 |
| Sufrágio universal (`universal_suffrage`) | Direito de voto 950 |

### Cidadania (`citizenship`)

| Lei | Efeitos |
|---|---|
| Cidadania igual (`equal_citizenship`) | — |
| Nacionalidade oficial (`official_nationality`) · dirigida a comunidades | Discriminação 100, Assimilação 1500 |
| Segregação legal (`legal_segregation`) · dirigida a comunidades | Discriminação 400, Segregação 600, Assimilação 300, Direito de voto -500 |
| Exclusão legal (`legal_exclusion`) · dirigida a comunidades | Discriminação 700, Segregação 900, Assimilação 100, Direito de voto -1000 |

### Religião (`religion`)

| Lei | Efeitos |
|---|---|
| Lei religiosa (`theocratic_law`) · dirigida a comunidades | Privilégio da Igreja 900, Discriminação 300, Censura 100 |
| Igreja oficial (`state_church`) | Privilégio da Igreja 500 |
| Separação entre Igreja e Estado (`separation`) | Privilégio da Igreja 100 |
| Estado anticlerical (`anticlerical_state`) | Privilégio da Igreja -300 |
| Ateísmo de Estado (`state_atheism`) · dirigida a comunidades | Privilégio da Igreja -800, Discriminação 300, Repressão 100 |

### Circulação interna (`internal_movement`)

| Lei | Efeitos |
|---|---|
| Livre circulação (`free_movement`) | — |
| Passaporte interno (`internal_passports`) | Migração dentro do estado 250, Migração entre estados 100 |
| Leis de passe (`pass_laws`) · dirigida a comunidades | Migração dentro do estado 250, Migração entre estados 100, Segregação 200 |

### Imigração (`immigration`)

| Lei | Efeitos |
|---|---|
| Imigração subsidiada (`subsidized_immigration`) | Imigração 2000 |
| Imigração aberta (`open_immigration`) | — |
| Cotas de imigração (`quota_immigration`) | Imigração 300 |
| Exclusão racial (`racial_exclusion`) · dirigida a comunidades | Imigração 50 |
| Fronteiras fechadas (`closed_immigration`) | Imigração 50 |

### Emigração (`emigration`)

| Lei | Efeitos |
|---|---|
| Emigração patrocinada (`sponsored_emigration`) | Emigração 2000 |
| Emigração livre (`free_emigration`) | — |
| Emigração regulada (`regulated_emigration`) | Emigração 700 |
| Visto de saída (`exit_permits`) | Emigração 100 |


## Informação

### Imprensa (`press`)

| Lei | Efeitos |
|---|---|
| Imprensa livre (`free_press`) | — |
| Imprensa sob licença (`press_licensing`) | Censura 300 |
| Censura prévia (`prior_censorship`) | Censura 700 |
| Monopólio estatal da imprensa (`state_press_monopoly`) | Censura 950, Mídia estatal 900 |

### Radiodifusão (`broadcasting`)

| Lei | Efeitos |
|---|---|
| Radiodifusão privada (`private_broadcasting`) | — |
| Emissora pública (`public_broadcaster`) | Mídia estatal 400 |
| Radiodifusão estatal (`state_broadcasting`) | Mídia estatal 900, Censura 100 |


## Estado

### Polícia (`policing`)

| Lei | Efeitos |
|---|---|
| Liberdades civis (`civil_liberties`) | Repressão 50 |
| Poderes de exceção (`emergency_powers`) | Repressão 300 |
| Polícia política (`political_police`) | Repressão 600, Censura 100 |
| Estado policial (`police_state`) | Repressão 900, Censura 200, Trabalho não livre 50 |

### Serviço público (`civil_service`)

| Lei | Efeitos |
|---|---|
| Clientelismo (`patronage`) | Corrupção 250, Arrecadação 700 |
| Serviço público misto (`mixed_service`) | Corrupção 120, Arrecadação 900 |
| Serviço público por mérito (`merit_service`) | Corrupção 40 |

### Serviço militar (`conscription`)

| Lei | Efeitos |
|---|---|
| Exército voluntário (`volunteer_army`) | Convocação militar 20 |
| Serviço seletivo (`selective_service`) | Convocação militar 300 |
| Serviço militar universal (`universal_service`) | Convocação militar 800 |

### Forma de governo (`government`)

| Lei | Efeitos |
|---|---|
| Monarquia parlamentar (`parliamentary_monarchy`) | — |
| República parlamentar (`parliamentary_republic`) | — |
| República presidencialista (`presidential_republic`) | — |
| Cidade livre (`free_city`) | — |
| Monarquia constitucional (`constitutional_monarchy`) | Repressão 50 |
| Monarquia absoluta (`absolute_monarchy`) | Repressão 150, Privilégio da Igreja 200 |
| Teocracia (`theocracy`) | Privilégio da Igreja 500, Censura 200 |
| Administração colonial (`colonial_administration`) | Repressão 150 |
| Ditadura militar (`military_dictatorship`) | Repressão 300, Censura 150, Convocação militar 100 |
| Estado de partido único (`one_party_state`) | Repressão 300, Censura 300, Mídia estatal 300 |


## Império

### Governo colonial (`colonial_rule`) · colonial

| Lei | Efeitos |
|---|---|
| Governo de companhia (`company_rule`) | Tributo colonial 400, Discriminação 500, Repressão 500, Censura 500 |
| Governo direto (`direct_rule`) | Tributo colonial 300, Autogoverno colonial 50, Discriminação 400, Repressão 400, Censura 400 |
| Governo indireto (`indirect_rule`) | Tributo colonial 200, Autogoverno colonial 200, Discriminação 300, Repressão 300, Censura 300 |
| Governo dos colonos (`settler_rule`) | Tributo colonial 150, Autogoverno colonial 100, Discriminação 600, Segregação 500, Direito de voto 50, Repressão 400, Censura 300 |
| Tutela (`trusteeship`) | Tributo colonial 100, Autogoverno colonial 500, Discriminação 150, Direito de voto 300, Repressão 150, Censura 150 |

### Trabalho colonial (`colonial_labour`) · colonial

| Lei | Efeitos |
|---|---|
| Trabalho forçado (`forced_labour`) | Trabalho não livre 500 |
| Imposto que obriga ao trabalho (`labour_taxation`) | Trabalho não livre 150 |
| Trabalho livre (`free_labour`) | — |

