<div align="center">
  <img src="../assets/adufa-icon.svg" width="112" alt="Adufa-Symbol: Ein Audiokanal wird zu einem ausgewählten Ausgang umgeleitet">
  <h1>Adufa</h1>
  <p><strong>Leite jede App an das richtige Audiogerät.</strong></p>
  <p>Ein schneller, lokal arbeitender Umschalter für App-Audioausgänge mit Lautstärkeregelung.</p>
</div>

<p align="center">
  <a href="../../README.md">English</a> ·
  <a href="../../README.pt-BR.md">Português (Brasil)</a> ·
  <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> ·
  <strong>Deutsch</strong> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <code>Windows-Beta</code> · <code>macOS geplant</code> · <code>Linux geplant</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — leite jede App an das richtige Audiogerät](../assets/adufa-hero.svg)

## Warum Adufa?

Videoanrufe sollen das Headset verwenden. Musik soll über die Lautsprecher laufen.
Ein Browser benötigt vielleicht einen Monitor oder ein virtuelles Kabel. Adufa zeigt
Apps mit hörbarem Ton und leitet jede an den passenden Ausgang weiter – ohne jedes Mal
den Windows-Lautstärkemixer durchsuchen zu müssen.

Adufa ist bewusst klein: Es lebt im Infobereich, öffnet sich nahe am Arbeitsbereich,
merkt sich App-Routen und bleibt ansonsten aus dem Weg.

## In Aktion

![Eine kurze Adufa-Demonstration mit Infobereich-Popup, Taskleisten-Begleitfenster, Lautstärkeregelung und Ausgangsauswahl](../assets/adufa-demo.gif)

Die Animation ist eine deterministische Dokumentationsdarstellung der nativen Oberfläche;
sie enthält weder eine Desktopaufnahme noch persönliche Daten.

## Was heute funktioniert

Die aktuelle öffentliche Beta läuft unter **Windows 10 22H2 und Windows 11**.

- Erkennt Apps mit aktiven Audiositzungen automatisch.
- Ändert das Ausgabegerät einer App, ohne den Systemstandard zu ändern.
- Merkt sich App-Routen über Neustarts von Adufa und der jeweiligen App hinweg.
- Setzt eine App mit einer Auswahl auf `Systemstandard` (`System default`) zurück.
- Ändert die aktuelle Lautstärke und Stummschaltung pro App.
- Auf unterstützten Windows-11-Versionen öffnet ein Rechtsklick auf eine hörbare
  Taskleisten-App experimentelle Regler für Ausgang, Lautstärke und Stummschaltung
  neben dem nativen Menü.
- Enthält **Ton finden** (`Find sound`), eine temporäre Live-Ansicht, die die lauteste App hervorhebt.
- Öffnet mit `Ctrl + Alt + A` eine Schnellauswahl nahe am Mauszeiger.
- Kann bei der Anmeldung starten; die Option bleibt deaktiviert, bis sie eingeschaltet wird.
- Unterstützt Englisch, brasilianisches Portugiesisch, Spanisch, Französisch, Deutsch,
  Italienisch, Japanisch und vereinfachtes Chinesisch.
- Erkennt die Windows-Anzeigesprache und erlaubt eine manuelle Sprachauswahl.
- Arbeitet lokal, ohne Konten, Analysen, Telemetrie oder Audio-Uploads.

### Experimentelle Windows-Integration

Auf unterstützten Windows-11-Versionen kann ein Rechtsklick auf das Taskleistensymbol
einer hörbaren App das kompakte Begleitfenster neben dem nativen Taskleistenmenü
öffnen. Das native Menü bleibt verfügbar; Adufa ergänzt es um Lautstärke- und Ausgangsregler.

Diese Integration muss das sichtbare Taskleistensymbol einer aktiven Audiositzung zuordnen.
Sie befindet sich in der Beta und kann auf das globale Tastenkürzel oder das Infobereich-Popup
zurückfallen, wenn Windows keine zuverlässige Zuordnung bereitstellt.

## Beta installieren

### Portable Version herunterladen

Versionen mit Tags werden von GitHub Actions erstellt. Öffne die
[Releases-Seite](../../../releases), lade `Adufa-Windows-x64.exe` herunter und führe die Datei aus.

Die erste Beta-Datei ist portabel und nicht signiert. Windows kann eine SmartScreen-Warnung
anzeigen, bis signierte Pakete verfügbar sind. Prüfe vor dem Ausführen, dass die Datei aus einem
Release dieses Repositorys stammt.

### Aus dem Quellcode erstellen

Voraussetzungen:

- Windows 10 22H2 oder Windows 11, x64;
- [Rust](https://www.rust-lang.org/tools/install) 1.85 oder neuer mit der MSVC-Toolchain;
- Visual Studio Build Tools mit **Desktopentwicklung mit C++** und einem Windows SDK.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

Ausführen:

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

Verwende für den normalen Einsatz einen Release-Build. Release-Builds sind Windows-GUI-
Anwendungen und öffnen kein Terminalfenster. Debug-Builds behalten zur Diagnose absichtlich
eine Konsole bei.

## Verwendung

### Eine App über den Infobereich umleiten

1. Starte die Audiowiedergabe in der App, die du umleiten möchtest.
2. Öffne die ausgeblendeten Symbole des Infobereichs und wähle Adufa.
3. Wähle die Zeile der App.
4. Wähle einen Ausgang oder `Systemstandard` (`System default`), um die gespeicherte Route zu entfernen.
5. Klicke außerhalb des Popups oder drücke `Esc`, um es zu schließen.

Die Route wird anhand einer stabilen App-Identität und nicht anhand einer temporären Prozess-ID
gespeichert. Nach dem Neustart einer App stellt Adufa die gespeicherte Auswahl wieder her, sofern
die Plattform die App sicher identifizieren kann.

### Herausfinden, welche App Ton wiedergibt

1. Öffne Adufa.
2. Wähle **Ton finden** (`Find sound`).
3. Beobachte die Live-Pegel; die stärkste hörbare Quelle wird hervorgehoben.
4. Wähle diese App, um ihren Ausgang zu ändern.

Ton finden zeichnet kein Audio auf. Die Funktion liest die von Windows bereitgestellten
Sitzungs-Spitzenpegel und endet, sobald das kompakte Popup geschlossen wird.

### Das Taskleisten-Begleitfenster verwenden

1. Lass die Ziel-App mindestens einmal Ton wiedergeben, damit Windows eine Audiositzung bereitstellt.
2. Klicke mit der rechten Maustaste auf ihr Taskleistensymbol.
3. Verwende das angrenzende Adufa-Fenster zum Stummschalten, Regeln der Lautstärke oder Auswählen eines Ausgangs.
4. Die Auswahl eines Ausgangs schließt das Begleitfenster und das native Menü.

Falls kein Begleitfenster erscheint, verwende `Ctrl + Alt + A`, während der Zeiger auf der
hörbaren App liegt, oder öffne Adufa über den Infobereich.

### Sprache oder Startverhalten ändern

Öffne **Einstellungen** (`Settings`), um:

- der Windows-Sprache zu folgen oder eine unterstützte Sprache auszuwählen;
- den Start von Adufa bei der Anmeldung ein- oder auszuschalten;
- den Windows-Lautstärkemixer für Einstellungen auf Systemebene zu öffnen.

## Tastatur- und Mausreferenz

| Eingabe | Aktion |
| --- | --- |
| `Ctrl + Alt + A` | Schnellauswahl nahe am Zeiger für die hörbare App unter dem Zeiger öffnen |
| Rechtsklick auf eine hörbare Taskleisten-App | Experimentelles Adufa-Fenster neben dem nativen Menü öffnen |
| `Tab` oder `↓` | Zum nächsten Element wechseln |
| `↑` | Zum vorherigen Element wechseln |
| `Enter` oder `Space` | Fokussiertes Element aktivieren |
| `Esc` | Aktuelle Auswahl schließen oder aus den Einstellungen zurückkehren |
| Außerhalb klicken | Temporäre Adufa-Fenster schließen |

Das globale Tastenkürzel kann nicht verfügbar sein, wenn eine andere Anwendung es bereits
registriert hat. Adufa läuft weiter und der Ablauf über den Infobereich bleibt nutzbar.

## Plattformstatus

| Plattform | Status | Geplantes Backend |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **Beta verfügbar** | Windows Core Audio / WASAPI und native Win32-Oberfläche |
| macOS 14.2+ | Geplant | Core Audio mit nativer Menüleistenoberfläche |
| Linux, Wayland und X11 | Geplant | PipeWire mit nativer Desktopintegration |

Plattformübergreifende Unterstützung ist die Produktrichtung, keine Behauptung heutiger
Funktionsgleichheit. Jedes Backend meldet seine Fähigkeiten ausdrücklich, damit Adufa niemals
den Erfolg einer nicht unterstützten Routing-Operation vortäuscht. Symbol und grundlegende
Interaktionssprache werden geteilt; Verhalten und Materialien bleiben plattformnativ.

## Datenschutz

Adufa ist für rein lokale Nutzung ausgelegt.

- Keine Telemetrie oder Analysen.
- Kein Benutzerkonto.
- Keine Audioaufnahme oder -übertragung.
- Kein Netzwerkdienst für das Routing erforderlich.
- Routen und Einstellungen werden auf deinem Computer gespeichert.

Unter Windows liegt die Konfiguration im lokalen Anwendungsdatenverzeichnis des aktuellen
Benutzers. Das Entfernen der portablen Programmdatei löscht diese Einstellungsdatei nicht automatisch.

## Roadmap

Es werden keine Termine versprochen, bevor die jeweilige Plattformimplementierung nachweislich funktioniert.

### In Entwicklung

- Windows-Beta auf unterstützten Windows-10- und Windows-11-Versionen weiter stabilisieren.
- Signiertes Installationsprogramm und Prüfsummen für portable Releases.
- Barrierefreie Beschriftungen, Prüfung bei hohem Kontrast und verbesserte Tastaturabläufe.
- Prüfung der Übersetzungen durch Muttersprachler.
- Zuverlässige, optionale und datenschutzfreundliche Update-Prüfungen.

### Als Nächstes

- Suche nach Apps und Ausgängen.
- Bevorzugte Ausgänge.
- Konfigurierbare globale Tastenkürzel.
- Erweiterter Mini-Mixer.
- Profile und automatische Regeln pro App.
- Bessere Diagnose und wiederherstellbare Neuzuordnung von Routen.

### Geplante Plattformen

- macOS 14.2+ mit Core Audio, ausgeliefert für Apple Silicon und Intel.
- Linux mit PipeWire unter Wayland und X11; ein reines PulseAudio-Backend gehört nicht
  zur ersten Linux-Version.

### In Prüfung

- Tiefere native Menüintegrationen, wenn das Betriebssystem eine sichere API bereitstellt.
- Optionale Synchronisierung der Einstellungen ohne Upload von Audio- oder Aktivitätsdaten.
- Mehrere Gerätesätze und wiederverwendbare Profile für Arbeit, Gaming, Anrufe und Streaming.

## Fehlerbehebung

### Eine App fehlt

Starte die Wiedergabe und öffne Adufa erneut. Manche Apps erstellen erst dann eine Audiositzung,
wenn sie Ton ausgeben. Geschützte oder systemeigene Sitzungen stellen möglicherweise weniger
Identitätsdaten bereit und können unter einer gruppierten App erscheinen.

### Ein gespeicherter Ausgang ist nicht verfügbar

Adufa behält die gewünschte Route bei, statt sie stillschweigend durch ein ähnlich benanntes
Gerät zu ersetzen. Schließe genau dieses Gerät erneut an, wähle einen anderen Ausgang oder
`Systemstandard` (`System default`).

### Das Taskleisten-Begleitfenster wurde nicht geöffnet

Die Integration ist experimentell und erfordert eine eindeutige Zuordnung zwischen Taskleistensymbol
und hörbarer App. Probiere das globale Tastenkürzel oder das Infobereich-Popup. Windows 10 verwendet
den Ersatzablauf für Integrationen, die nur unter Windows 11 zuverlässig funktionieren.

### `Ctrl + Alt + A` bewirkt nichts

Möglicherweise hat ein anderes Programm dieses globale Tastenkürzel bereits registriert. Öffne
Adufa über den Infobereich; konfigurierbare Tastenkürzel stehen auf der Roadmap.

### Ein Terminalfenster erscheint

Wahrscheinlich läuft ein Debug-Build oder Adufa wurde über `cargo run` gestartet. Erstelle und
starte die unter [Aus dem Quellcode erstellen](#aus-dem-quellcode-erstellen) gezeigte Release-Datei.

## Entwicklung

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Ein Roundtrip-Test mit einer echten Windows-Audiositzung wird standardmäßig ignoriert, weil er
eine reale Sitzung vorübergehend verändert. Führe ihn nur auf einem Entwicklungscomputer mit
einer aktiven, entbehrlichen Audiositzung aus.

Das Repository ist aufgeteilt in:

- `crates/router-engine`: plattformunabhängige Identitäten, Befehle und Zustände;
- `platforms/windows`: Windows-Audio, Persistenz, Taskleistenintegration und native Oberfläche;
- `docs/adr`: akzeptierte Architekturentscheidungen;
- `docs/design`: Interaktions- und Identitätsforschung;
- `docs/assets`: generierte Dokumentations- und Markenressourcen.

Der CI-Workflow prüft Formatierung, Clippy und Tests und erzeugt anschließend die portable
Windows-x64-Datei. Das Pushen eines Tags wie `v0.1.0-beta.1` erstellt ein GitHub Release und
hängt `Adufa-Windows-x64.exe` an.

## Mitwirken

Fehlerberichte sollten die Windows-Version, die betroffene App, den erwarteten und beobachteten
Ausgang sowie den verwendeten Ablauf – Infobereich, Tastenkürzel oder Taskleiste – enthalten.
Hänge niemals Aufnahmen, Konfigurationsdateien oder Protokolle mit privaten Pfaden ungeprüft an.

Beiträge sollen die Grundregeln bewahren: stabile App-Identität, ereignisgesteuerte Beobachtung,
ehrliche Meldung von Fähigkeiten, native Plattformoberflächen und keine Telemetrie.

## Lizenz und Name

Der Quellcode steht unter **GPL-3.0-or-later**, wie im Cargo-Workspace angegeben. „Adufa“ und
die Projektgrafiken kennzeichnen offizielle Builds; die Open-Source-Lizenz bedeutet keine
Befürwortung veränderter Distributionen.

Der Name wurde bislang nur oberflächlich auf Kollisionen im Web und in Repositorys geprüft.
Dies ist keine rechtliche Markenfreigabe; vor der offiziellen Verteilung ist eine formale Suche
in den vorgesehenen Veröffentlichungsgebieten erforderlich.

---

<p align="center"><strong>Adufa</strong> — eine kleine Bedienoberfläche für die verborgenen Audiowege deines Desktops.</p>
