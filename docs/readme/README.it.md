<div align="center">
  <img src="../assets/adufa-icon.svg" width="112" alt="Icona di Adufa: un canale audio reindirizzato verso un'uscita selezionata">
  <h1>Adufa</h1>
  <p><strong>Indirizza ogni applicazione al dispositivo audio giusto.</strong></p>
  <p>Un selettore rapido e locale dell'uscita audio per applicazione, con controllo del volume.</p>
</div>

<p align="center">
  <a href="../../README.md">English</a> ·
  <a href="../../README.pt-BR.md">Português (Brasil)</a> ·
  <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <strong>Italiano</strong> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <code>Beta per Windows</code> · <code>macOS pianificato</code> · <code>Linux pianificato</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — indirizza ogni applicazione al dispositivo audio giusto](../assets/adufa-hero.svg)

## Perché Adufa?

Le videochiamate dovrebbero usare le cuffie. La musica dovrebbe uscire dagli altoparlanti.
Un browser potrebbe aver bisogno di un monitor o di un cavo virtuale. Adufa mostra le
applicazioni che stanno emettendo audio e invia ognuna all'uscita appropriata, senza dover
cercare ogni volta nel mixer del volume di Windows.

Adufa è volutamente compatto: rimane nell'area di notifica, si apre vicino al punto in cui
stai lavorando, ricorda gli instradamenti delle applicazioni e non intralcia il lavoro.

## Guardalo in azione

![Una breve dimostrazione di Adufa con il popup dell'area di notifica, il pannello affiancato alla barra delle applicazioni, il controllo del volume e la selezione dell'uscita](../assets/adufa-demo.gif)

L'animazione è un rendering deterministico dell'interfaccia nativa creato per la documentazione;
non contiene acquisizioni del desktop né dati personali.

## Cosa funziona oggi

L'attuale beta pubblica funziona su **Windows 10 22H2 e Windows 11**.

- Rileva automaticamente le applicazioni con sessioni audio attive.
- Cambia il dispositivo di uscita di una singola applicazione senza modificare quello predefinito del sistema.
- Ricorda gli instradamenti delle applicazioni dopo il riavvio di Adufa o dell'applicazione.
- Riporta un'applicazione a `Predefinito di sistema` (`System default`) con una sola selezione.
- Modifica il volume corrente e lo stato di disattivazione audio per applicazione.
- Include **Trova suono** (`Find sound`), una vista temporanea in tempo reale che evidenzia l'applicazione più rumorosa.
- Apre un selettore rapido vicino al puntatore con `Ctrl + Alt + A`.
- Può avviarsi all'accesso; l'opzione rimane disattivata finché non viene abilitata.
- Supporta inglese, portoghese brasiliano, spagnolo, francese, tedesco, italiano,
  giapponese e cinese semplificato.
- Rileva la lingua di visualizzazione di Windows e consente di sceglierne una manualmente.
- Funziona localmente, senza account, analisi, telemetria o caricamento dell'audio.

### Integrazione sperimentale con Windows

Nelle versioni supportate di Windows 11, facendo clic con il pulsante destro sull'icona nella
barra delle applicazioni di un'app che emette audio è possibile aprire il pannello compatto di
Adufa accanto al menu nativo. Il menu nativo resta disponibile; Adufa lo completa con i controlli
del volume e dell'uscita.

Questa integrazione dipende dall'associazione dell'icona visibile nella barra delle applicazioni
a un'applicazione con una sessione audio attiva. È una funzione beta e può ripiegare sulla
scorciatoia globale o sul popup dell'area di notifica quando Windows non fornisce una corrispondenza affidabile.

## Installare la beta

### Scaricare una build portatile

Le versioni contrassegnate da tag vengono compilate da GitHub Actions. Apri la
[pagina Releases](../../../releases), scarica `Adufa-Windows-x64.exe` ed eseguilo.

L'eseguibile della beta iniziale è portatile e non firmato. Windows potrebbe mostrare un avviso
SmartScreen finché non saranno disponibili pacchetti firmati. Prima di consentirne l'esecuzione,
verifica che il file provenga dal release di questo repository.

### Compilare dal codice sorgente

Requisiti:

- Windows 10 22H2 o Windows 11, x64;
- [Rust](https://www.rust-lang.org/tools/install) 1.85 o successivo con toolchain MSVC;
- Visual Studio Build Tools con **Sviluppo di applicazioni desktop con C++** e un Windows SDK.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

Esegui:

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

Per l'uso normale utilizza una build release. Le build release sono applicazioni con interfaccia
grafica di Windows e non aprono una finestra del terminale. Le build debug mantengono intenzionalmente
una console per la diagnostica.

## Come si usa

### Indirizzare un'applicazione dall'area di notifica

1. Avvia la riproduzione audio nell'applicazione da indirizzare.
2. Apri le icone nascoste dell'area di notifica e seleziona Adufa.
3. Seleziona la riga dell'applicazione.
4. Scegli un'uscita oppure `Predefinito di sistema` (`System default`) per rimuovere l'instradamento salvato.
5. Fai clic fuori dal popup o premi `Esc` per chiuderlo.

L'instradamento viene associato a un'identità stabile dell'applicazione anziché a un ID di processo
temporaneo. Quando un'applicazione viene riavviata, Adufa ripristina la scelta salvata se la piattaforma
riesce a identificarla in modo sicuro.

### Scoprire quale applicazione sta emettendo suoni

1. Apri Adufa.
2. Seleziona **Trova suono** (`Find sound`).
3. Osserva gli indicatori di livello in tempo reale; la sorgente udibile più forte viene evidenziata.
4. Seleziona quell'applicazione per cambiarne l'uscita.

Trova suono non registra l'audio. Legge i livelli di picco delle sessioni già forniti da Windows
e si arresta quando viene chiuso il popup compatto.

### Usare il pannello affiancato alla barra delle applicazioni

1. Lascia che l'applicazione interessata riproduca audio almeno una volta affinché Windows esponga una sessione audio.
2. Fai clic con il pulsante destro sulla sua icona nella barra delle applicazioni.
3. Usa il pannello adiacente di Adufa per disattivare l'audio, regolare il volume o selezionare un'uscita.
4. La selezione di un'uscita chiude sia il pannello di Adufa sia il menu nativo.

Se il pannello non appare, usa `Ctrl + Alt + A` mentre punti all'applicazione udibile oppure apri
Adufa dall'area di notifica.

### Cambiare la lingua o il comportamento all'avvio

Apri **Impostazioni** (`Settings`) per:

- seguire la lingua di Windows o scegliere una delle lingue supportate;
- abilitare o disabilitare l'apertura di Adufa all'accesso;
- aprire il mixer del volume di Windows per i controlli a livello di sistema.

## Riferimento per tastiera e mouse

| Input | Azione |
| --- | --- |
| `Ctrl + Alt + A` | Aprire il selettore rapido vicino al puntatore per l'applicazione udibile sotto di esso |
| Clic destro su un'applicazione udibile nella barra delle applicazioni | Aprire il pannello sperimentale di Adufa accanto al menu nativo |
| `Tab` o `↓` | Passare all'elemento successivo |
| `↑` | Passare all'elemento precedente |
| `Enter` o `Space` | Attivare l'elemento con lo stato attivo |
| `Esc` | Chiudere il selettore corrente o tornare dalle Impostazioni |
| Clic all'esterno | Chiudere le finestre temporanee di Adufa |

La scorciatoia globale potrebbe non essere disponibile se un'altra applicazione l'ha già registrata;
Adufa continua a funzionare e il flusso dall'area di notifica resta utilizzabile.

## Stato delle piattaforme

| Piattaforma | Stato | Backend pianificato |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **Beta disponibile** | Windows Core Audio / WASAPI e interfaccia Win32 nativa |
| macOS 14.2+ | Pianificato | Core Audio con interfaccia nativa nella barra dei menu |
| Linux, Wayland e X11 | Pianificato | PipeWire con integrazione desktop nativa |

Il supporto multipiattaforma è la direzione del prodotto, non una dichiarazione di parità delle
funzioni attuali. Ogni backend comunica esplicitamente le proprie capacità, così Adufa non finge
mai che un'operazione di instradamento non supportata sia riuscita. L'icona e il linguaggio di
interazione principale sono condivisi; comportamento e materiali restano nativi della piattaforma.

## Privacy

Adufa è progettato per funzionare solo localmente.

- Nessuna telemetria o analisi.
- Nessun account utente.
- Nessuna registrazione o caricamento dell'audio.
- Nessun servizio di rete necessario per l'instradamento.
- Instradamenti e preferenze vengono memorizzati sul tuo computer.

In Windows, la configurazione è archiviata nella directory locale dei dati delle applicazioni
dell'utente corrente. Rimuovere l'eseguibile portatile non elimina automaticamente il file delle preferenze.

## Roadmap

Non vengono promesse date finché l'implementazione della piattaforma interessata non è comprovata.

### In sviluppo

- Consolidare la beta per Windows nelle versioni supportate di Windows 10 e 11.
- Installer firmato e checksum per i release portatili.
- Etichette accessibili, verifica del contrasto elevato e flussi da tastiera migliorati.
- Revisione delle traduzioni da parte di madrelingua.
- Controlli degli aggiornamenti affidabili, facoltativi e rispettosi della privacy.

### Prossimi passi

- Ricerca di applicazioni e uscite.
- Uscite preferite.
- Scorciatoie globali configurabili.
- Mini mixer ampliato.
- Profili e regole automatiche per applicazione.
- Diagnostica migliore e riassociazione recuperabile degli instradamenti.

### Piattaforme pianificate

- macOS 14.2+ con Core Audio, distribuito per Apple Silicon e Intel.
- Linux con PipeWire su Wayland e X11; un backend esclusivamente PulseAudio non fa parte
  della prima versione per Linux.

### In valutazione

- Integrazioni più profonde con i menu nativi quando il sistema operativo offre un'API sicura.
- Sincronizzazione facoltativa delle preferenze senza caricare audio o dati sulle attività.
- Più set di dispositivi e profili riutilizzabili per lavoro, gioco, chiamate e streaming.

## Risoluzione dei problemi

### Manca un'applicazione

Avvia la riproduzione e riapri Adufa. Alcune applicazioni non creano una sessione audio finché
non producono un suono. Le sessioni protette o di proprietà del sistema possono esporre meno
informazioni sull'identità e apparire raggruppate sotto un'altra applicazione.

### Un'uscita salvata non è disponibile

Adufa mantiene l'instradamento desiderato invece di sostituirlo silenziosamente con un dispositivo
dal nome simile. Ricollega esattamente quel dispositivo, scegli un'altra uscita oppure seleziona
`Predefinito di sistema` (`System default`).

### Il pannello della barra delle applicazioni non si è aperto

L'integrazione è sperimentale e richiede una corrispondenza univoca tra l'icona nella barra delle
applicazioni e un'app udibile. Prova la scorciatoia globale o il popup dell'area di notifica. Windows 10
usa il flusso alternativo per le integrazioni affidabili soltanto in Windows 11.

### `Ctrl + Alt + A` non fa nulla

Un altro programma potrebbe aver già registrato quella scorciatoia globale. Apri Adufa dall'area
di notifica; le scorciatoie configurabili sono nella roadmap.

### Appare una finestra del terminale

Probabilmente stai eseguendo una build debug o avviando il programma con `cargo run`. Compila e
avvia l'eseguibile release indicato in [Compilare dal codice sorgente](#compilare-dal-codice-sorgente).

## Sviluppo

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Un test completo su una sessione audio reale di Windows viene ignorato per impostazione predefinita
perché modifica temporaneamente una sessione reale. Eseguilo solo su un computer di sviluppo con
una sessione audio attiva e non importante.

Il repository è suddiviso in:

- `crates/router-engine`: identità, comandi e stato indipendenti dalla piattaforma;
- `platforms/windows`: audio di Windows, persistenza, integrazione con la barra delle applicazioni e interfaccia nativa;
- `docs/adr`: decisioni architetturali accettate;
- `docs/design`: ricerca sull'interazione e sull'identità;
- `docs/assets`: risorse generate per la documentazione e il marchio.

Il workflow CI verifica formattazione, Clippy e test, quindi genera l'eseguibile portatile
Windows x64. Il push di un tag come `v0.1.0-beta.1` crea un GitHub Release e allega
`Adufa-Windows-x64.exe`.

## Contribuire

Le segnalazioni di bug devono includere la versione di Windows, l'applicazione interessata,
l'uscita attesa, l'uscita osservata e l'eventuale uso dell'area di notifica, della scorciatoia o
della barra delle applicazioni. Non allegare registrazioni, file di configurazione o log contenenti
percorsi privati senza averli prima controllati.

I contributi devono preservare le regole fondamentali: identità stabile dell'applicazione,
osservazione guidata dagli eventi, comunicazione corretta delle capacità, superfici native della
piattaforma e nessuna telemetria.

## Licenza e nome

Il codice sorgente è distribuito con licenza **GPL-3.0-or-later**, come dichiarato dal workspace
Cargo. “Adufa” e la grafica del progetto identificano le build ufficiali; la licenza open source
non implica l'approvazione delle distribuzioni modificate.

Il nome ha superato solo una verifica preliminare delle corrispondenze sul Web e nei repository.
Ciò non costituisce un'autorizzazione legale del marchio; la distribuzione ufficiale dovrà
completare una ricerca formale nei territori di lancio previsti.

---

<p align="center"><strong>Adufa</strong> — una piccola superficie di controllo per i percorsi audio nascosti dal desktop.</p>
