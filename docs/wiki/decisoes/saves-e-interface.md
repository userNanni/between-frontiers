# Saves e interface

Como o jogo guarda a partida, como o jogador volta atrás e como a tela se organiza. O detalhe está
em [`saves.pt-BR.md`](../../saves.pt-BR.md) e nos PRs do cliente.

## Decisões

### Voltar atrás é jeito legítimo de jogar

- **Decisão:** o jogo facilita salvar e voltar, com pontos de restauração dentro de cada campanha.
- **O que o autor pensou:** o autor defendeu o save scumming como tática: ele deixa o jogador
  arriscar uma guerra, ou procurar, num país difícil, o caminho estreito que cumpre uma agenda.
- **Como o jogo funciona por causa disso:** cada campanha tem quatro tipos de ponto: automático (na
  virada do calendário), antes de decisão arriscada, salvar rápido (F5, carregar com F9) e salvo à
  mão com nome (Ctrl+Shift+S). Os automáticos giram; os salvos à mão nunca saem sozinhos. Antes de
  voltar com F9, o jogo grava onde o jogador estava, para desfazer a volta. A tela de Campanhas lista
  os pontos com a data do jogo, o tipo e o motivo.
- **Estado:** no jogo (#98).
- **Onde está o detalhe:** [`saves.pt-BR.md`, "Tipos de ponto"](../../saves.pt-BR.md#tipos-de-ponto)
  e "Rotação e disco".

### Ponto de restauração antes de decisão arriscada

- **Decisão:** todo sistema com decisão arriscada ou irreversível grava um ponto **antes** de mandar
  o comando, por um único caminho no código.
- **O que o autor pensou:** veio junto da decisão anterior: quem arrisca precisa poder voltar ao
  instante de antes. Recomendação da coordenação sobre como fazer.
- **Como o jogo funciona por causa disso:** mudar uma lei ou libertar território grava um ponto
  "Antes de…" com o motivo; se o resultado não agradar, basta voltar a ele. Liga e desliga em
  Configurações. Os motivos de aceitar acordo, assinar tratado e declarar guerra já existem e são
  chamados quando essas decisões chegarem à tela; a mobilização usa o de guerra no #115.
- **Estado:** no jogo para lei (#98) e libertar território (#96); mobilizar em revisão (#115);
  tratado e guerra quando chegarem.
- **Onde está o detalhe:** [`saves.pt-BR.md`, "Tipos de ponto"](../../saves.pt-BR.md#tipos-de-ponto),
  "Ponto de entrada único".

### Salvamento automático configurável, com o custo à vista

- **Decisão:** o jogador escolhe o intervalo do salvamento automático (mensal, trimestral,
  semestral, anual ou desligado), quantos pontos guardar e o aviso de pasta grande, e vê quanto cada
  escolha custa em tempo e em disco.
- **O que o autor pensou:** o autor pediu o salvamento automático configurável com o custo do save
  calculado. O intervalo padrão foi decisão da coordenação, a partir da medida.
- **Como o jogo funciona por causa disso:** Configurações mostram o tamanho de cada ponto e o total
  da rotação. O custo medido: o automático mensal toma 5,5 % do tempo de simular um mês de 1975 em
  diante, o trimestral cerca de 1,8 %, o anual cerca de 0,5 %; um ponto tem de 1,2 a 1,8 MB. A
  gravação roda no thread da simulação, e a interface não trava.
- **Estado:** no jogo com padrão **anual** (#98). O PR #114, em revisão, muda o padrão para
  **trimestral** e mostra o custo de cada intervalo na tela; quem já escolheu mantém a escolha.
- **Onde está o detalhe:** [`saves.pt-BR.md`, "Custo do save"](../../saves.pt-BR.md#custo-do-save).

### Save que sobrevive às atualizações

- **Decisão:** o save tem versão e uma cadeia de migração, e guarda as chaves do catálogo
  (tecnologias, bens, prédios). Campo novo no mundo sobe a versão e ganha um degrau de migração.
- **O que o autor pensou:** decisão técnica, para que os pontos de restauração de uma campanha
  continuem abrindo depois de cada atualização do jogo (a condição do princípio de voltar atrás).
- **Como o jogo funciona por causa disso:** uma tecnologia nova inserida no meio da árvore não
  estraga o save: ele se realinha pela chave. Campanha gravada numa versão mais nova diz "Não abre
  nesta versão"; arquivo danificado diz "Arquivo danificado"; quando há ponto de restauração, a ação
  principal vira "Voltar a este ponto".
- **Estado:** em revisão, para entrar em breve (#114).
- **Onde está o detalhe:** PR #114; a seção "Formato do mundo e migração" entra em
  `saves.pt-BR.md` com ele.

### Decisão rara não fica a um toque

- **Decisão:** libertar território é uma decisão na janela de Agendas, com passo de confirmação
  próprio; jogar com outro país fica no menu "…" de cada campanha, na tela de Campanhas.
- **O que o autor pensou:** "Não precisa ser tão fácil libertar território e jogar com outro país.
  Não é uma decisão comum; num fluxo comum, ela ocupa espaço na tela e o jogador pode fazer sem
  querer." O princípio geral está em [Princípios](../principios.md#decisão-rara-fica-fora-do-fluxo-comum).
- **Como o jogo funciona por causa disso:** "Libertar território" (tecla J abre as Agendas) lista os
  países que o jogador pode libertar ou um estado seu, independente ou como súdito, mostra o que sai
  e o que fica e pede confirmação. O "…" da campanha tem Renomear, Apagar, Abrir pasta e Jogar com
  outro país: o jogador pode trocar de país a qualquer momento, por campanha.
- **Estado:** no jogo (#96 e #108).
- **Onde está o detalhe:** [`borders.pt-BR.md`](../../borders.pt-BR.md);
  `crates/bf-client/src/screens/liberate.rs` e `screens/saves.rs`.

### Interface modular e por teclado

- **Decisão:** painéis ancorados que o jogador arruma, com modo de edição, predefinições e layout
  por usuário; tudo acionável por teclado, com dica de tecla na tela e editor de atalhos; painel
  lateral e diário de Agendas fixáveis; toda tela cabe de 960×540 a 4K.
- **O que o autor pensou:** o autor pediu um HUD modular em vez de um HUD único "certo", pensando
  também em quem transmite o jogo ao vivo, e tudo pelo teclado.
- **Como o jogo funciona por causa disso:** toda tela nova nasce como painel registrado e com
  comando no registro de atalhos, então já tem tecla e dica desde o primeiro dia.
- **Estado:** no jogo (#56, #58, #59, #63).
- **Onde está o detalhe:** [`architecture.pt-BR.md`, "Cliente em Linux e Windows"](../../architecture.pt-BR.md#cliente-em-linux-e-windows)
  e os PRs citados.
