<div align="center">
  <img src="../assets/adufa-icon.svg" width="112" alt="Icône Adufa : un canal audio redirigé vers une sortie sélectionnée">
  <h1>Adufa</h1>
  <p><strong>Dirigez chaque application vers le bon périphérique audio.</strong></p>
  <p>Un sélecteur rapide et local de sortie audio par application, avec réglage du volume.</p>
</div>

<p align="center">
  <a href="../../README.md">English</a> ·
  <a href="../../README.pt-BR.md">Português (Brasil)</a> ·
  <a href="README.es.md">Español</a> ·
  <strong>Français</strong> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <code>Bêta Windows</code> · <code>macOS prévu</code> · <code>Linux prévu</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — dirigez chaque application vers le bon périphérique audio](../assets/adufa-hero.svg)

## Pourquoi Adufa ?

Les appels vidéo devraient utiliser le casque. La musique devrait sortir des enceintes.
Un navigateur peut nécessiter un moniteur ou un câble virtuel. Adufa affiche les applications
qui émettent du son et envoie chacune vers la sortie qui lui convient, sans devoir fouiller
chaque fois dans le mélangeur de volume de Windows.

Adufa est volontairement compact : il réside dans la zone de notification, s'ouvre près de
votre espace de travail, mémorise les routages des applications et sait se faire discret.

## Découvrez-le en action

![Une courte démonstration d'Adufa montrant la fenêtre de la zone de notification, le panneau complémentaire de la barre des tâches, le réglage du volume et la sélection de sortie](../assets/adufa-demo.gif)

L'animation est un rendu déterministe de l'interface native destiné à la documentation ;
elle ne contient aucune capture du bureau ni donnée personnelle.

## Ce qui fonctionne aujourd'hui

La bêta publique actuelle fonctionne sous **Windows 10 22H2 et Windows 11**.

- Détecte automatiquement les applications possédant des sessions audio actives.
- Change le périphérique de sortie d'une application sans modifier la sortie système par défaut.
- Mémorise les routages après le redémarrage d'Adufa ou de l'application.
- Rétablit une application sur `Sortie système par défaut` (`System default`) en une sélection.
- Modifie le volume actuel et l'état muet de chaque application.
- Sur les versions compatibles de Windows 11, faites un clic droit sur une
  application audible dans la barre des tâches pour ouvrir des commandes
  expérimentales de sortie, volume et muet à côté du menu natif.
- Inclut **Localiser le son** (`Find sound`), une vue temporaire en direct qui met en évidence l'application la plus forte.
- Ouvre un sélecteur rapide près du pointeur avec `Ctrl + Alt + A`.
- Peut démarrer à l'ouverture de session ; cette option reste désactivée tant que vous ne l'activez pas.
- Prend en charge l'anglais, le portugais brésilien, l'espagnol, le français, l'allemand,
  l'italien, le japonais et le chinois simplifié.
- Détecte la langue d'affichage de Windows et permet de la remplacer manuellement.
- Fonctionne localement, sans compte, statistiques, télémétrie ni transfert audio.

### Intégration Windows expérimentale

Sur les versions compatibles de Windows 11, un clic droit sur l'icône de barre des tâches
d'une application qui émet du son peut ouvrir le panneau compact d'Adufa à côté du menu natif.
Le menu natif reste disponible ; Adufa le complète avec les commandes de volume et de sortie.

Cette intégration repose sur l'association entre l'icône visible de la barre des tâches et une
application ayant une session audio active. Il s'agit d'une fonction bêta qui peut se rabattre
sur le raccourci global ou la fenêtre de la zone de notification lorsque Windows ne fournit pas
d'association fiable.

## Installer la bêta

### Télécharger une version portable

Les versions étiquetées sont compilées par GitHub Actions. Ouvrez la
[page Releases](../../../releases), téléchargez `Adufa-Windows-x64.exe`, puis exécutez-le.

L'exécutable de cette première bêta est portable et non signé. Windows peut afficher un
avertissement SmartScreen tant que des paquets signés ne sont pas disponibles. Vérifiez que
le fichier provient bien d'un release de ce dépôt avant d'autoriser son exécution.

### Compiler depuis les sources

Prérequis :

- Windows 10 22H2 ou Windows 11, x64 ;
- [Rust](https://www.rust-lang.org/tools/install) 1.85 ou ultérieur avec la toolchain MSVC ;
- Visual Studio Build Tools avec **Développement Desktop en C++** et un Windows SDK.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

Exécutez :

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

Utilisez une compilation release pour un usage normal. Les compilations release sont des
applications graphiques Windows et n'ouvrent pas de terminal. Les compilations debug conservent
volontairement une console à des fins de diagnostic.

## Utilisation

### Router une application depuis la zone de notification

1. Lancez du son dans l'application à router.
2. Ouvrez les icônes masquées de la zone de notification et sélectionnez Adufa.
3. Sélectionnez la ligne de l'application.
4. Choisissez une sortie, ou `Sortie système par défaut` (`System default`) pour supprimer le routage enregistré.
5. Cliquez en dehors de la fenêtre ou appuyez sur `Esc` pour la fermer.

Le routage est associé à une identité d'application stable plutôt qu'à un ID de processus
temporaire. Après le redémarrage d'une application, Adufa restaure le choix enregistré lorsque
la plateforme peut l'identifier de façon sûre.

### Identifier l'application qui émet du son

1. Ouvrez Adufa.
2. Sélectionnez **Localiser le son** (`Find sound`).
3. Observez les indicateurs de niveau en direct ; la source audible la plus forte est mise en évidence.
4. Sélectionnez cette application pour changer sa sortie.

Localiser le son n'enregistre pas l'audio. Cette fonction lit les niveaux de crête des sessions
que Windows fournit déjà et s'arrête à la fermeture de la fenêtre compacte.

### Utiliser le panneau complémentaire de la barre des tâches

1. Laissez l'application cible produire du son au moins une fois afin que Windows expose une session audio.
2. Faites un clic droit sur son icône dans la barre des tâches.
3. Utilisez le panneau adjacent d'Adufa pour couper le son, régler le volume ou choisir une sortie.
4. La sélection d'une sortie ferme à la fois le panneau complémentaire et le menu natif.

Si le panneau n'apparaît pas, utilisez `Ctrl + Alt + A` en pointant l'application audible, ou
ouvrez Adufa depuis la zone de notification.

### Changer la langue ou le comportement au démarrage

Ouvrez **Paramètres** (`Settings`) pour :

- suivre la langue de Windows ou choisir l'une des langues prises en charge ;
- activer ou désactiver le lancement d'Adufa à l'ouverture de session ;
- ouvrir le mélangeur de volume Windows pour les réglages système.

## Référence du clavier et de la souris

| Entrée | Action |
| --- | --- |
| `Ctrl + Alt + A` | Ouvrir le sélecteur rapide près du pointeur pour l'application audible sous celui-ci |
| Clic droit sur une application audible de la barre des tâches | Ouvrir le panneau expérimental d'Adufa à côté du menu natif |
| `Tab` ou `↓` | Passer à l'élément suivant |
| `↑` | Revenir à l'élément précédent |
| `Enter` ou `Space` | Activer l'élément sélectionné |
| `Esc` | Fermer le sélecteur actuel ou quitter les Paramètres |
| Clic en dehors | Fermer les fenêtres temporaires d'Adufa |

Le raccourci global peut être indisponible si une autre application l'a déjà enregistré ;
Adufa continue de fonctionner et le flux depuis la zone de notification reste utilisable.

## État des plateformes

| Plateforme | État | Backend prévu |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **Bêta disponible** | Windows Core Audio / WASAPI et interface Win32 native |
| macOS 14.2+ | Prévu | Core Audio avec une interface native dans la barre des menus |
| Linux, Wayland et X11 | Prévu | PipeWire avec intégration native au bureau |

Le multiplateforme est la direction du produit, et non une promesse de parité fonctionnelle
aujourd'hui. Chaque backend communique explicitement ses capacités afin qu'Adufa ne prétende
jamais qu'une opération de routage non prise en charge a réussi. L'icône et le langage principal
d'interaction sont communs ; les comportements et matériaux restent natifs à chaque plateforme.

## Confidentialité

Adufa est conçu pour fonctionner uniquement en local.

- Aucune télémétrie ni statistique.
- Aucun compte utilisateur.
- Aucun enregistrement ni transfert audio.
- Aucun service réseau nécessaire au routage.
- Les routages et préférences sont enregistrés sur votre ordinateur.

Sous Windows, la configuration est stockée dans le dossier local de données d'application de
l'utilisateur actuel. Supprimer l'exécutable portable ne supprime pas automatiquement ce fichier.

## Feuille de route

Aucune date n'est promise tant que l'implémentation de la plateforme concernée n'est pas éprouvée.

### En développement

- Renforcer la bêta Windows sur les versions prises en charge de Windows 10 et 11.
- Programme d'installation signé et sommes de contrôle pour les releases portables.
- Libellés accessibles, validation du contraste élevé et parcours clavier améliorés.
- Révision des traductions par des locuteurs natifs.
- Recherche de mises à jour fiable, facultative et respectueuse de la vie privée.

### Prochainement

- Recherche d'applications et de sorties.
- Sorties favorites.
- Raccourcis globaux configurables.
- Mini-mélangeur étendu.
- Profils et règles automatiques par application.
- Diagnostics améliorés et réassociation récupérable des routages.

### Plateformes prévues

- macOS 14.2+ avec Core Audio, distribué pour Apple Silicon et Intel.
- Linux avec PipeWire sous Wayland et X11 ; un backend uniquement PulseAudio ne fait pas
  partie de la première version Linux.

### À l'étude

- Intégrations plus poussées aux menus natifs lorsque le système fournit une API sûre.
- Synchronisation facultative des préférences sans transférer d'audio ni de données d'activité.
- Plusieurs ensembles de périphériques et des profils réutilisables pour le travail, les jeux, les appels et le streaming.

## Dépannage

### Une application est absente

Lancez la lecture puis rouvrez Adufa. Certaines applications ne créent une session audio qu'au
moment de produire du son. Les sessions protégées ou appartenant au système peuvent exposer moins
d'informations d'identité et apparaître groupées sous une autre application.

### Une sortie enregistrée est indisponible

Adufa conserve le routage souhaité au lieu de le remplacer silencieusement par un périphérique
au nom similaire. Reconnectez exactement le même périphérique, choisissez une autre sortie ou
sélectionnez `Sortie système par défaut` (`System default`).

### Le panneau complémentaire de la barre des tâches ne s'est pas ouvert

L'intégration est expérimentale et exige une association unique entre l'icône de la barre des
tâches et une application audible. Essayez le raccourci global ou la fenêtre de la zone de
notification. Windows 10 utilise le flux de secours pour les intégrations qui ne sont fiables que sous Windows 11.

### `Ctrl + Alt + A` ne fait rien

Un autre programme a peut-être déjà enregistré ce raccourci global. Ouvrez Adufa depuis la zone
de notification ; les raccourcis configurables figurent sur la feuille de route.

### Une fenêtre de terminal apparaît

Vous exécutez probablement une compilation debug ou un lancement via `cargo run`. Compilez et
lancez l'exécutable release indiqué dans [Compiler depuis les sources](#compiler-depuis-les-sources).

## Développement

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Un test aller-retour sur une session audio Windows réelle est ignoré par défaut, car il modifie
temporairement une vraie session. Ne l'exécutez que sur une machine de développement disposant
d'une session audio active et non essentielle.

Le dépôt est organisé comme suit :

- `crates/router-engine` : identités, commandes et état indépendants de la plateforme ;
- `platforms/windows` : audio Windows, persistance, intégration à la barre des tâches et interface native ;
- `docs/adr` : décisions architecturales acceptées ;
- `docs/design` : recherche sur l'interaction et l'identité ;
- `docs/assets` : ressources générées de documentation et de marque.

Le workflow de CI valide le formatage, Clippy et les tests, puis produit l'exécutable portable
Windows x64. L'envoi d'une étiquette telle que `v0.1.0-beta.1` crée un GitHub Release et joint
`Adufa-Windows-x64.exe`.

## Contribuer

Les rapports de bogues doivent préciser la version de Windows, l'application concernée, la sortie
attendue, la sortie observée et le parcours utilisé : zone de notification, raccourci ou barre des
tâches. Ne joignez jamais d'enregistrements, de fichiers de configuration ou de journaux contenant
des chemins privés sans les avoir vérifiés.

Les contributions doivent préserver les principes fondamentaux : identité d'application stable,
observation pilotée par événements, communication honnête des capacités, surfaces natives de la
plateforme et absence de télémétrie.

## Licence et nom

Le code source est placé sous licence **GPL-3.0-or-later**, comme l'indique le workspace Cargo.
« Adufa » et les illustrations du projet identifient les compilations officielles ; la licence
open source n'implique pas l'approbation des distributions modifiées.

Le nom n'a fait l'objet que d'une recherche préliminaire de collisions sur le Web et dans les
dépôts. Cela ne constitue pas une validation juridique de marque ; la distribution officielle
devra effectuer une recherche formelle dans les territoires de lancement visés.

---

<p align="center"><strong>Adufa</strong> — une petite surface de contrôle pour les chemins audio que votre bureau dissimule.</p>
