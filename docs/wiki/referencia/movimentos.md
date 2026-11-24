<!-- Página gerada de `data/` por `crates/bf-sim/tests/wiki_reference.rs`. Não edite à mão: regere com `BF_BLESS=1 cargo test -p bf-sim --test wiki_reference`. -->

# Movimentos

Na ordem do catálogo. Um movimento cresce quando todas as condições valem (a mais fraca dá o ritmo), em milésimos de quem ele recruta por ano, e perde metade do apoio a cada tantos anos sem causa.

| Movimento | Quer | Onde | Quem recruta | Condições | Crescimento | Meia-vida |
|---|---|---|---|---|---|---|
| Reforma agrária (`agrarian_reform`) | Reforma agrária | metrópole | Camponeses 100 %, Trabalhadores rurais 100 % | Renda dos proprietários de terra ≥ 1100; Parcela dos proprietários na renda agrícola ≥ 350; Necessidades básicas atendidas ≤ 650 | 60 por ano | 30 anos |
| Direitos trabalhistas (`labour_rights`) | Negociação coletiva | metrópole | Operários 100 %, Operários qualificados 100 %, Trabalhadores rurais 30 % | Poder de negociação sindical ≤ 300; Urbanização ≥ 150; Necessidades básicas atendidas ≤ 800 | 80 por ano | 25 anos |
| Proteção social (`social_protection`) | Seguridade social | metrópole | Operários 100 %, Operários qualificados 100 %, Funcionários 50 % | Desemprego ≥ 80; Seguro-desemprego ≤ 150; Urbanização ≥ 300 | 60 por ano | 20 anos |
| Sufrágio (`suffrage`) | Sufrágio universal | metrópole | Funcionários 100 %, Profissionais liberais 100 %, Engenheiros e técnicos 100 %, Operários qualificados 70 %, Operários 50 %, Camponeses 20 % | Direito de voto ≤ 500; Alfabetização ≥ 500 | 70 por ano | 40 anos |
| Liberdade de imprensa (`press_freedom`) | Imprensa livre | metrópole | Profissionais liberais 100 %, Engenheiros e técnicos 100 %, Funcionários 80 % | Censura ≥ 300; Alfabetização ≥ 400 | 50 por ano | 20 anos |
| Independência (`independence`) | independência | colônias | todos | Alfabetização ≥ 100; Autogoverno colonial ≤ 400; Discriminação ≥ 150 | 40 por ano | 60 anos |
| Fim do trabalho forçado colonial (`colonial_labour_reform`) | Trabalho livre | colônias | todos | Trabalho não livre ≥ 150; Alfabetização ≥ 50 | 40 por ano | 30 anos |
| Direitos civis (`civil_rights`) | Cidadania igual | comunidade visada | todos | Discriminação ≥ 300; Alfabetização ≥ 300 | 60 por ano | 40 anos |
| Liberdade de circulação (`freedom_of_movement`) | Livre circulação | metrópole | Camponeses 100 %, Trabalhadores rurais 100 %, Operários 100 % | Migração dentro do estado ≤ 500; Salário da cidade sobre o do campo ≥ 1500 | 40 por ano | 15 anos |
| Saúde pública (`public_health`) | Serviço nacional de saúde | metrópole | Operários 100 %, Operários qualificados 100 %, Funcionários 100 %, Profissionais liberais 50 % | Cobertura hospitalar ≤ 400; Parcela do Estado no atendimento hospitalar ≤ 500; Urbanização ≥ 350 | 50 por ano | 20 anos |
| Escola pública (`public_schooling`) | Primário obrigatório | metrópole | Funcionários 100 %, Profissionais liberais 100 %, Operários qualificados 100 %, Operários 50 % | Cobertura escolar ≤ 500; Meta de matrícula escolar ≤ 500; Urbanização ≥ 200 | 50 por ano | 20 anos |
| Estatização (`nationalization`) | Economia mista | metrópole | Operários 100 %, Operários qualificados 100 %, Servidores públicos 70 % | Desemprego ≥ 150; Propriedade estatal ≤ 100 | 70 por ano | 15 anos |
| Reforma de mercado (`market_reform`) | Mercado regulado | metrópole | Capitalistas 100 %, Gerentes 100 %, Profissionais liberais 50 %, Engenheiros e técnicos 50 % | Propriedade estatal ≥ 400; Necessidades básicas atendidas ≤ 700 | 50 por ano | 15 anos |
| Revolta fiscal (`tax_revolt`) | Imposto de renda proporcional | metrópole | Capitalistas 100 %, Proprietários de terras 100 %, Gerentes 100 %, Profissionais liberais 50 % | Carga tributária ≥ 250; Teto do imposto, classe alta ≥ 400 | 80 por ano | 8 anos |
| Imposto de renda (`income_tax`) | Imposto de renda progressivo | metrópole | Operários 70 %, Operários qualificados 100 %, Funcionários 100 %, Profissionais liberais 70 %, Servidores públicos 100 % | Teto do imposto, classe alta ≤ 250; Urbanização ≥ 250; Alfabetização ≥ 400 | 60 por ano | 20 anos |
| Reforma fiscal (`fiscal_reform`) | Imposto de renda progressivo | metrópole | Servidores públicos 100 %, Militares 100 %, Funcionários 100 %, Operários qualificados 50 %, Operários 30 %, Camponeses 15 % | Aperto fiscal ≥ 100; Teto do imposto, classe alta ≤ 250 | 120 por ano | 10 anos |
| Reforma da imigração (`immigration_reform`) | Imigração aberta | metrópole | Capitalistas 100 %, Gerentes 100 %, Proprietários de terras 60 %, Profissionais liberais 30 %, Engenheiros e técnicos 30 % | Imigração ≤ 500; Desemprego ≤ 40; Crescimento do PIB ≥ 30 | 50 por ano | 15 anos |
| Protecionismo (`protectionism`) | Tarifas protecionistas | metrópole | Operários 70 %, Operários qualificados 70 %, Capitalistas 50 %, Camponeses 30 % | Parcela importada ≥ 300; Desemprego ≥ 100; Tarifa ≤ 150 | 60 por ano | 10 anos |
| Anticorrupção (`anti_corruption`) | Serviço público por mérito | metrópole | Profissionais liberais 100 %, Engenheiros e técnicos 100 %, Funcionários 70 %, Gerentes 50 % | Corrupção ≥ 100; Alfabetização ≥ 500 | 50 por ano | 20 anos |
| Laicismo (`secularism`) | Separação entre Igreja e Estado | metrópole | Profissionais liberais 100 %, Engenheiros e técnicos 100 %, Funcionários 50 %, Operários qualificados 30 % | Privilégio da Igreja ≥ 400; Urbanização ≥ 400; Alfabetização ≥ 700 | 30 por ano | 30 anos |
| Reação clerical (`clerical_reaction`) | Igreja oficial | metrópole | Clero 100 %, Proprietários de terras 50 %, Camponeses 40 % | Privilégio da Igreja ≤ -200 | 80 por ano | 30 anos |
| Nativismo (`nativism`) | Cotas de imigração | metrópole | Operários 60 %, Operários qualificados 60 %, Camponeses 30 %, Funcionários 30 % | Parcela de imigrantes ≥ 100; Desemprego ≥ 80; Imigração ≥ 600 | 60 por ano | 15 anos |
| Troca de regime (`regime_change`) | Estado de partido único | metrópole | Funcionários 60 %, Operários qualificados 40 %, Operários 40 %, Camponeses 30 %, Militares 80 %, Proprietários de terras 30 %, Capitalistas 20 %, Profissionais liberais 30 % | Legitimidade ≤ 500; Desemprego ≥ 150 | 100 por ano | 15 anos |
| Revisionismo (`revisionism`) | revisão das fronteiras | metrópole | Funcionários 50 %, Profissionais liberais 40 %, Militares 100 %, Proprietários de terras 60 %, Operários qualificados 30 %, Camponeses 20 % | Território perdido ≥ 50; Desemprego ≥ 80 | 60 por ano | 40 anos |
