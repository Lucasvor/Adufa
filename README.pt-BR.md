<div align="center">
  <img src="docs/assets/adufa-icon.svg" width="112" alt="Ícone do Adufa: um canal de som redirecionado para uma saída selecionada">
  <h1>Adufa</h1>
  <p><strong>Direcione cada aplicativo para o dispositivo de áudio certo.</strong></p>
  <p>Um seletor rápido e local de saída de áudio por aplicativo, com controle de volume.</p>
</div>

<p align="center">
  <a href="README.md">English</a> ·
  <strong>Português (Brasil)</strong> ·
  <a href="docs/readme/README.es.md">Español</a> ·
  <a href="docs/readme/README.fr.md">Français</a> ·
  <a href="docs/readme/README.de.md">Deutsch</a> ·
  <a href="docs/readme/README.it.md">Italiano</a> ·
  <a href="docs/readme/README.ja.md">日本語</a> ·
  <a href="docs/readme/README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <code>Beta para Windows</code> · <code>macOS planejado</code> · <code>Linux planejado</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — direcione cada aplicativo para o dispositivo de áudio certo](docs/assets/adufa-hero.svg)

## Por que o Adufa?

As chamadas de vídeo devem usar o headset. A música deve sair nos alto-falantes.
Um navegador pode precisar de um monitor ou cabo virtual. O Adufa permite ver os
aplicativos que estão emitindo áudio e enviar cada um para a saída correta — sem
precisar procurar no mixer de volume do Windows toda vez.

O Adufa é deliberadamente pequeno: fica na área de notificação, abre perto de onde
você está trabalhando, memoriza as rotas dos aplicativos e sai do caminho.

## Veja em ação

![Uma demonstração curta do Adufa mostrando o popup da bandeja, o painel complementar da barra de tarefas, o controle de volume e a seleção de saída](docs/assets/adufa-demo.gif)

A animação é uma renderização determinística da interface nativa para a documentação;
ela não contém captura da área de trabalho nem dados pessoais.

## O que funciona hoje

O beta público atual funciona no **Windows 10 22H2 e Windows 11**.

- Descobre automaticamente aplicativos com sessões de áudio ativas.
- Altera o dispositivo de saída de um aplicativo sem mudar o padrão do sistema.
- Memoriza as rotas dos aplicativos após reiniciar o Adufa ou o próprio aplicativo.
- Devolve um aplicativo a `Padrão do sistema` (`System default`) com uma seleção.
- Altera o volume atual por aplicativo e o estado de mudo.
- Inclui **Encontrar som** (`Find sound`), uma visualização temporária ao vivo que destaca o aplicativo com áudio mais alto.
- Abre um seletor rápido perto do cursor com `Ctrl + Alt + A`.
- Pode iniciar com a sessão do usuário; essa opção permanece desativada até você habilitá-la.
- Está disponível em inglês, português do Brasil, espanhol, francês, alemão, italiano,
  japonês e chinês simplificado.
- Detecta o idioma de exibição do Windows e permite escolher outro idioma manualmente.
- Funciona localmente, sem contas, métricas de uso, telemetria ou envio de áudio.

### Integração experimental com o Windows

Em versões compatíveis do Windows 11, clicar com o botão direito no ícone da barra de
tarefas de um aplicativo que está emitindo áudio pode abrir o painel compacto do
Adufa ao lado do menu nativo da barra de tarefas. O menu nativo continua disponível;
o Adufa o complementa com controles de volume e saída.

Essa integração depende de associar o ícone visível na barra de tarefas a um
aplicativo com sessão de áudio ativa. É uma funcionalidade beta e pode recorrer ao
atalho global ou ao popup da bandeja quando o Windows não fornece uma associação confiável.

## Instalar o beta

### Baixar a versão portátil

As versões marcadas com tags são compiladas pelo GitHub Actions. Abra a
[página de Releases](../../releases), baixe `Adufa-Windows-x64.exe` e execute-o.

O executável do beta inicial é portátil e não é assinado. O Windows pode exibir um
aviso do SmartScreen até que pacotes assinados estejam disponíveis. Antes de
permitir a execução, confira se o arquivo veio do release deste repositório.

### Compilar a partir do código-fonte

Requisitos:

- Windows 10 22H2 ou Windows 11, x64;
- [Rust](https://www.rust-lang.org/tools/install) 1.85 ou mais recente, com a toolchain MSVC;
- Visual Studio Build Tools com **Desenvolvimento para desktop com C++** e um Windows SDK.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

Execute:

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

Use uma compilação de release no uso normal. Compilações de release são aplicativos
com interface gráfica do Windows e não abrem uma janela de terminal. Compilações de
debug mantêm um console intencionalmente para diagnóstico.

## Como usar

### Direcionar um aplicativo pela bandeja

1. Inicie a reprodução de áudio no aplicativo que você deseja direcionar.
2. Abra os ícones ocultos da área de notificação e selecione o Adufa.
3. Selecione a linha do aplicativo.
4. Escolha uma saída ou selecione `Padrão do sistema` (`System default`) para remover a rota salva.
5. Clique fora do popup ou pressione `Esc` para fechá-lo.

A rota é armazenada usando uma identidade estável do aplicativo, não um ID de processo
temporário. Quando um aplicativo reinicia, o Adufa restaura a escolha salva se a
plataforma puder identificá-lo com segurança.

### Descobrir qual aplicativo está emitindo som

1. Abra o Adufa.
2. Selecione **Encontrar som** (`Find sound`).
3. Observe os indicadores de nível ao vivo; a fonte audível mais forte fica em destaque.
4. Selecione esse aplicativo para alterar sua saída.

Encontrar som não grava áudio. Ele lê os níveis de pico das sessões que o Windows já
fornece e para quando o popup compacto é fechado.

### Usar o painel complementar da barra de tarefas

1. Faça o aplicativo desejado reproduzir áudio pelo menos uma vez para que o Windows
   exponha uma sessão de áudio.
2. Clique com o botão direito no ícone dele na barra de tarefas.
3. Use o painel adjacente do Adufa para silenciar, ajustar o volume ou selecionar uma saída.
4. Selecionar uma saída fecha tanto o painel complementar quanto o menu nativo.

Se o painel não aparecer, use `Ctrl + Alt + A` enquanto aponta para o aplicativo que
está emitindo áudio ou abra o Adufa pela área de notificação.

### Alterar o idioma ou o comportamento de inicialização

Abra **Configurações** (`Settings`) para:

- seguir o idioma do Windows ou escolher qualquer idioma compatível;
- habilitar ou desabilitar a abertura do Adufa ao entrar na conta;
- abrir o mixer de volume do Windows para controles no nível do sistema.

## Referência de teclado e mouse

| Entrada | Ação |
| --- | --- |
| `Ctrl + Alt + A` | Abrir o seletor rápido perto do cursor para o aplicativo audível sob o ponteiro |
| Clique com o botão direito em um aplicativo audível na barra de tarefas | Abrir o painel experimental do Adufa ao lado do menu nativo |
| `Tab` ou `↓` | Ir para o próximo item |
| `↑` | Ir para o item anterior |
| `Enter` ou `Space` | Ativar o item em foco |
| `Esc` | Fechar o seletor atual ou voltar de Configurações |
| Clique fora | Fechar as janelas transitórias do Adufa |

O atalho global pode ficar indisponível quando outro aplicativo já o registrou; o
Adufa continua funcionando e o fluxo pela bandeja permanece disponível.

## Estado das plataformas

| Plataforma | Estado | Backend planejado |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **Beta disponível** | Windows Core Audio / WASAPI e interface Win32 nativa |
| macOS 14.2+ | Planejado | Core Audio com interface nativa na barra de menus |
| Linux, Wayland e X11 | Planejado | PipeWire com integração nativa ao desktop |

Ser multiplataforma é a direção do produto, não uma afirmação de paridade de recursos
hoje. Cada backend informa suas capacidades explicitamente para que o Adufa nunca
finja que uma operação de roteamento sem suporte funcionou. O ícone e a linguagem
central de interação são compartilhados; o comportamento e os materiais continuam
nativos de cada plataforma.

## Privacidade

O Adufa foi projetado para funcionar somente no dispositivo.

- Sem telemetria ou métricas de uso.
- Sem conta de usuário.
- Sem gravação ou envio de áudio.
- Nenhum serviço de rede é necessário para o roteamento.
- Rotas e preferências ficam armazenadas no seu computador.

No Windows, a configuração fica no diretório local de dados de aplicativos do usuário
atual. Remover o executável portátil não apaga automaticamente esse arquivo de preferências.

## Planejamento

Nenhuma data é prometida até que a implementação da plataforma correspondente esteja comprovada.

### Em desenvolvimento

- Reforçar a estabilidade do beta para Windows nas versões compatíveis do Windows 10 e 11.
- Instalador assinado e somas de verificação dos releases portáteis.
- Rótulos acessíveis, validação de alto contraste e fluxos de teclado aprimorados.
- Revisão das traduções por falantes nativos.
- Verificações de atualização confiáveis, opcionais e que preservem a privacidade.

### Próximos passos

- Pesquisa de aplicativos e saídas.
- Saídas favoritas.
- Atalhos globais configuráveis.
- Mini mixer expandido.
- Perfis e regras automáticas por aplicativo.
- Diagnóstico aprimorado e reassociação recuperável de rotas.

### Plataformas planejadas

- macOS 14.2+ usando Core Audio, distribuído para Apple Silicon e Intel.
- Linux usando PipeWire no Wayland e X11; um backend somente para PulseAudio não faz
  parte da versão inicial para Linux.

### Em avaliação

- Integrações mais profundas com menus nativos onde o sistema operacional oferecer uma API segura.
- Sincronização opcional de preferências sem enviar áudio nem dados de atividade.
- Vários conjuntos de dispositivos e perfis reutilizáveis para trabalho, jogos, chamadas e transmissões.

## Solução de problemas

### Um aplicativo não aparece

Inicie a reprodução e abra o Adufa novamente. Alguns aplicativos só criam uma sessão
de áudio depois de produzir som. Sessões protegidas ou pertencentes ao sistema podem
expor menos informações de identidade e aparecer agrupadas em outro aplicativo.

### Uma saída salva não está disponível

O Adufa mantém a rota desejada em vez de substituí-la silenciosamente por um dispositivo
com nome parecido. Reconecte exatamente o mesmo dispositivo, escolha outra saída ou
selecione `Padrão do sistema` (`System default`).

### O painel complementar da barra de tarefas não abriu

A integração é experimental e exige uma associação única entre o ícone da barra de
tarefas e um aplicativo com áudio. Tente o atalho global ou o popup da bandeja. O
Windows 10 usa o fluxo alternativo para integrações que só funcionam de forma confiável no Windows 11.

### `Ctrl + Alt + A` não faz nada

Outro programa pode já ter registrado esse atalho global. Abra o Adufa pela bandeja;
atalhos configuráveis estão no planejamento.

### Uma janela de terminal aparece

Provavelmente você está executando uma compilação de debug ou iniciando por `cargo run`.
Compile e inicie o executável de release mostrado em [Compilar a partir do código-fonte](#compilar-a-partir-do-código-fonte).

## Desenvolvimento

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Um teste de ida e volta com uma sessão de áudio real do Windows é ignorado por padrão,
pois altera temporariamente uma sessão real. Execute-o somente em uma máquina de
desenvolvimento com uma sessão de áudio ativa e descartável.

O repositório está dividido em:

- `crates/router-engine`: identidades, comandos e estado independentes de plataforma;
- `platforms/windows`: áudio do Windows, persistência, integração com a barra de tarefas e interface nativa;
- `docs/adr`: decisões de arquitetura aceitas;
- `docs/design`: pesquisa de interação e identidade;
- `docs/assets`: recursos gerados de documentação e marca.

O fluxo de CI valida formatação, Clippy e testes e depois gera o executável portátil
Windows x64. Enviar uma tag como `v0.1.0-beta.1` cria um GitHub Release e anexa
`Adufa-Windows-x64.exe`.

## Como contribuir

Relatos de erros devem incluir a versão do Windows, o aplicativo afetado, a saída
esperada, a saída observada e se foi usado o fluxo da bandeja, do atalho ou da barra
de tarefas. Nunca anexe gravações, arquivos de configuração ou logs que contenham
caminhos privados sem revisá-los antes.

As contribuições devem preservar as regras centrais: identidade estável do aplicativo,
observação orientada a eventos, relato honesto de capacidades, superfícies nativas da
plataforma e ausência de telemetria.

Leia [CONTRIBUTING.md](CONTRIBUTING.md) antes de abrir um pull request,
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) para as expectativas da comunidade e
[SECURITY.md](SECURITY.md) para relatar vulnerabilidades de forma privada. Consulte
[CHANGELOG.md](CHANGELOG.md) para mudanças e limitações conhecidas do beta.

## Licença e nome

O código-fonte é licenciado sob **GPL-3.0-or-later**; consulte [LICENSE](LICENSE).
“Adufa” e a arte do projeto identificam as compilações oficiais; a licença de código
aberto não implica endosso de distribuições modificadas.

O nome passou apenas por uma verificação preliminar de colisões na web e em repositórios.
Isso não equivale a uma análise jurídica de marca; a distribuição oficial deve realizar
uma pesquisa formal nos territórios de lançamento pretendidos.

---

<p align="center"><strong>Adufa</strong> — uma pequena superfície de controle para os caminhos de áudio que seu desktop esconde.</p>
