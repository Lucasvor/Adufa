//! Small, dependency-free localization boundary for the native Windows UI.
//!
//! Adufa ships the catalog in the executable. The only persisted value is the
//! user's language choice; `auto` resolves from the Windows UI locale on every
//! launch so changing the operating-system language keeps working naturally.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use windows::Win32::Globalization::{GetUserDefaultUILanguage, LCIDToLocaleName};

const LOCALE_BUFFER_LENGTH: usize = 85;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locale {
    En,
    PtBr,
    Es,
    Fr,
    De,
    It,
    Ja,
    ZhCn,
}

pub const SUPPORTED_LOCALES: [Locale; 8] = [
    Locale::En,
    Locale::PtBr,
    Locale::Es,
    Locale::Fr,
    Locale::De,
    Locale::It,
    Locale::Ja,
    Locale::ZhCn,
];

impl Locale {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::PtBr => "pt-BR",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::De => "de",
            Self::It => "it",
            Self::Ja => "ja",
            Self::ZhCn => "zh-CN",
        }
    }

    /// Locale names are intentionally autonyms so the language list remains
    /// usable even when the currently selected language is unfamiliar.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::PtBr => "Português (Brasil)",
            Self::Es => "Español",
            Self::Fr => "Français",
            Self::De => "Deutsch",
            Self::It => "Italiano",
            Self::Ja => "日本語",
            Self::ZhCn => "简体中文",
        }
    }

    pub fn from_language_tag(tag: &str) -> Self {
        let tag = tag.trim().replace('_', "-").to_ascii_lowercase();
        let language = tag.split('-').next().unwrap_or_default();
        match language {
            "pt" => Self::PtBr,
            "es" => Self::Es,
            "fr" => Self::Fr,
            "de" => Self::De,
            "it" => Self::It,
            "ja" => Self::Ja,
            "zh" if tag == "zh-cn" || tag == "zh-sg" || tag.contains("hans") => Self::ZhCn,
            "en" => Self::En,
            _ => Self::En,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LanguageChoice {
    #[default]
    Automatic,
    Manual(Locale),
}

impl LanguageChoice {
    pub const fn list_index(self) -> usize {
        match self {
            Self::Automatic => 0,
            Self::Manual(Locale::En) => 1,
            Self::Manual(Locale::PtBr) => 2,
            Self::Manual(Locale::Es) => 3,
            Self::Manual(Locale::Fr) => 4,
            Self::Manual(Locale::De) => 5,
            Self::Manual(Locale::It) => 6,
            Self::Manual(Locale::Ja) => 7,
            Self::Manual(Locale::ZhCn) => 8,
        }
    }

    pub const fn from_list_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Automatic),
            1 => Some(Self::Manual(Locale::En)),
            2 => Some(Self::Manual(Locale::PtBr)),
            3 => Some(Self::Manual(Locale::Es)),
            4 => Some(Self::Manual(Locale::Fr)),
            5 => Some(Self::Manual(Locale::De)),
            6 => Some(Self::Manual(Locale::It)),
            7 => Some(Self::Manual(Locale::Ja)),
            8 => Some(Self::Manual(Locale::ZhCn)),
            _ => None,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct StoredPreference {
    language: String,
}

pub struct LanguagePreference {
    path: PathBuf,
    choice: LanguageChoice,
    system_locale: Locale,
}

impl LanguagePreference {
    pub fn load_default() -> Result<Self, String> {
        let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
            "Windows did not provide the local application-data folder.".to_owned()
        })?;
        Self::load(
            PathBuf::from(local_app_data)
                .join("Adufa")
                .join("preferences.json"),
            detect_windows_locale(),
        )
    }

    fn load(path: PathBuf, system_locale: Locale) -> Result<Self, String> {
        let choice = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice::<StoredPreference>(&bytes)
                .ok()
                .and_then(|stored| choice_from_code(&stored.language))
                .unwrap_or_default(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => LanguageChoice::default(),
            Err(error) => {
                return Err(format!(
                    "Adufa could not read its language preference: {error}"
                ));
            }
        };
        Ok(Self {
            path,
            choice,
            system_locale,
        })
    }

    pub fn automatic() -> Self {
        Self {
            path: PathBuf::new(),
            choice: LanguageChoice::Automatic,
            system_locale: detect_windows_locale(),
        }
    }

    pub const fn choice(&self) -> LanguageChoice {
        self.choice
    }

    pub const fn locale(&self) -> Locale {
        match self.choice {
            LanguageChoice::Automatic => self.system_locale,
            LanguageChoice::Manual(locale) => locale,
        }
    }

    pub fn set(&mut self, choice: LanguageChoice) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            self.choice = choice;
            return Ok(());
        }
        save_choice(&self.path, choice)?;
        self.choice = choice;
        Ok(())
    }

    pub fn automatic_label(&self) -> String {
        format!(
            "{} ({})",
            text(self.locale(), Text::Automatic),
            self.system_locale.display_name()
        )
    }
}

fn choice_from_code(value: &str) -> Option<LanguageChoice> {
    if value.eq_ignore_ascii_case("auto") {
        return Some(LanguageChoice::Automatic);
    }
    SUPPORTED_LOCALES
        .into_iter()
        .find(|locale| locale.code().eq_ignore_ascii_case(value))
        .map(LanguageChoice::Manual)
}

fn save_choice(path: &Path, choice: LanguageChoice) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "The language preference path has no parent folder.".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Adufa could not create its settings folder: {error}"))?;
    let language = match choice {
        LanguageChoice::Automatic => "auto",
        LanguageChoice::Manual(locale) => locale.code(),
    };
    let bytes = serde_json::to_vec_pretty(&StoredPreference {
        language: language.to_owned(),
    })
    .map_err(|error| format!("Adufa could not encode its language preference: {error}"))?;
    let temporary = path.with_extension("next.json");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("Adufa could not save its language preference: {error}"))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("Adufa could not replace its language preference: {error}"))?;
    }
    fs::rename(&temporary, path)
        .map_err(|error| format!("Adufa could not publish its language preference: {error}"))
}

pub fn detect_windows_locale() -> Locale {
    let mut buffer = [0_u16; LOCALE_BUFFER_LENGTH];
    // SAFETY: Both functions have no borrowed inputs beyond the writable
    // locale-name buffer. A LANGID is also a valid default-sort LCID.
    let language = unsafe { GetUserDefaultUILanguage() };
    let length = unsafe { LCIDToLocaleName(u32::from(language), Some(&mut buffer), 0) };
    if length <= 1 {
        return Locale::En;
    }
    let name = String::from_utf16_lossy(&buffer[..length as usize - 1]);
    Locale::from_language_tag(&name)
}

#[derive(Clone, Copy)]
pub enum Text {
    AudioRouter,
    Listening,
    FindSound,
    NoApplications,
    SystemDefault,
    UnavailableOutput,
    SystemSounds,
    Application,
    Settings,
    Exit,
    System,
    OpenAtLogin,
    StartAtLoginDescription,
    QuickAccess,
    OpenSelectorNearCursor,
    AudibleAppUnderPointer,
    SystemAudioSettings,
    OpenWindowsMixer,
    Language,
    LanguageDescription,
    Automatic,
    ChooseLanguage,
    AudioOutput,
    Muted,
    LocatorFailed,
    DiscoveryFailed,
    StartupFailed,
    LanguageFailed,
    WindowsMixer,
    MixerOpenFailed,
    RoutingFailed,
    RouteChangeFailed,
    CouldNotStart,
}

pub const fn text(locale: Locale, key: Text) -> &'static str {
    match locale {
        Locale::En => english(key),
        Locale::PtBr => portuguese(key),
        Locale::Es => spanish(key),
        Locale::Fr => french(key),
        Locale::De => german(key),
        Locale::It => italian(key),
        Locale::Ja => japanese(key),
        Locale::ZhCn => simplified_chinese(key),
    }
}

const fn english(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "Audio router",
        Text::Listening => "Listening",
        Text::FindSound => "Find sound",
        Text::NoApplications => "No applications are playing audio",
        Text::SystemDefault => "System default",
        Text::UnavailableOutput => "Unavailable output",
        Text::SystemSounds => "System sounds",
        Text::Application => "Application",
        Text::Settings => "Settings",
        Text::Exit => "Exit",
        Text::System => "System",
        Text::OpenAtLogin => "Open at login",
        Text::StartAtLoginDescription => "Start Adufa when you sign in",
        Text::QuickAccess => "Quick access",
        Text::OpenSelectorNearCursor => "Open selector near the cursor",
        Text::AudibleAppUnderPointer => "Targets the audible app under the pointer",
        Text::SystemAudioSettings => "System audio settings",
        Text::OpenWindowsMixer => "Open Windows volume mixer",
        Text::Language => "Language",
        Text::LanguageDescription => "Choose the interface language",
        Text::Automatic => "Automatic",
        Text::ChooseLanguage => "Choose language",
        Text::AudioOutput => "Audio output",
        Text::Muted => "Muted",
        Text::LocatorFailed => "Sound Locator failed",
        Text::DiscoveryFailed => "Audio discovery failed",
        Text::StartupFailed => "Startup preference failed",
        Text::LanguageFailed => "Language preference failed",
        Text::WindowsMixer => "Windows volume mixer",
        Text::MixerOpenFailed => "Windows could not open the application volume settings.",
        Text::RoutingFailed => "Audio routing failed",
        Text::RouteChangeFailed => "Adufa could not change this application's output.",
        Text::CouldNotStart => "Adufa could not start.",
    }
}

const fn portuguese(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "Roteador de áudio",
        Text::Listening => "Ouvindo",
        Text::FindSound => "Localizar som",
        Text::NoApplications => "Nenhum aplicativo está reproduzindo áudio",
        Text::SystemDefault => "Padrão do sistema",
        Text::UnavailableOutput => "Saída indisponível",
        Text::SystemSounds => "Sons do sistema",
        Text::Application => "Aplicativo",
        Text::Settings => "Configurações",
        Text::Exit => "Sair",
        Text::System => "Sistema",
        Text::OpenAtLogin => "Abrir ao entrar",
        Text::StartAtLoginDescription => "Iniciar o Adufa ao entrar no sistema",
        Text::QuickAccess => "Acesso rápido",
        Text::OpenSelectorNearCursor => "Abrir seletor perto do cursor",
        Text::AudibleAppUnderPointer => "Encontra o app com áudio sob o ponteiro",
        Text::SystemAudioSettings => "Configurações de áudio do sistema",
        Text::OpenWindowsMixer => "Abrir o mixer de volume do Windows",
        Text::Language => "Idioma",
        Text::LanguageDescription => "Escolher o idioma da interface",
        Text::Automatic => "Automático",
        Text::ChooseLanguage => "Escolher idioma",
        Text::AudioOutput => "Saída de áudio",
        Text::Muted => "Sem som",
        Text::LocatorFailed => "Falha ao localizar o som",
        Text::DiscoveryFailed => "Falha ao detectar o áudio",
        Text::StartupFailed => "Falha na preferência de inicialização",
        Text::LanguageFailed => "Falha na preferência de idioma",
        Text::WindowsMixer => "Mixer de volume do Windows",
        Text::MixerOpenFailed => {
            "O Windows não conseguiu abrir as configurações de volume dos aplicativos."
        }
        Text::RoutingFailed => "Falha ao rotear o áudio",
        Text::RouteChangeFailed => "O Adufa não conseguiu alterar a saída deste aplicativo.",
        Text::CouldNotStart => "O Adufa não conseguiu iniciar.",
    }
}

const fn spanish(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "Enrutador de audio",
        Text::Listening => "Escuchando",
        Text::FindSound => "Buscar sonido",
        Text::NoApplications => "Ninguna aplicación está reproduciendo audio",
        Text::SystemDefault => "Predeterminado del sistema",
        Text::UnavailableOutput => "Salida no disponible",
        Text::SystemSounds => "Sonidos del sistema",
        Text::Application => "Aplicación",
        Text::Settings => "Configuración",
        Text::Exit => "Salir",
        Text::System => "Sistema",
        Text::OpenAtLogin => "Abrir al iniciar sesión",
        Text::StartAtLoginDescription => "Iniciar Adufa al iniciar sesión",
        Text::QuickAccess => "Acceso rápido",
        Text::OpenSelectorNearCursor => "Abrir selector junto al cursor",
        Text::AudibleAppUnderPointer => "Busca la app audible bajo el puntero",
        Text::SystemAudioSettings => "Configuración de audio del sistema",
        Text::OpenWindowsMixer => "Abrir el mezclador de volumen de Windows",
        Text::Language => "Idioma",
        Text::LanguageDescription => "Elegir el idioma de la interfaz",
        Text::Automatic => "Automático",
        Text::ChooseLanguage => "Elegir idioma",
        Text::AudioOutput => "Salida de audio",
        Text::Muted => "Silenciado",
        Text::LocatorFailed => "Error al localizar el sonido",
        Text::DiscoveryFailed => "Error al detectar el audio",
        Text::StartupFailed => "Error en la preferencia de inicio",
        Text::LanguageFailed => "Error en la preferencia de idioma",
        Text::WindowsMixer => "Mezclador de volumen de Windows",
        Text::MixerOpenFailed => {
            "Windows no pudo abrir la configuración de volumen de las aplicaciones."
        }
        Text::RoutingFailed => "Error de enrutamiento de audio",
        Text::RouteChangeFailed => "Adufa no pudo cambiar la salida de esta aplicación.",
        Text::CouldNotStart => "Adufa no pudo iniciarse.",
    }
}

const fn french(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "Routeur audio",
        Text::Listening => "Écoute",
        Text::FindSound => "Repérer le son",
        Text::NoApplications => "Aucune application ne lit de son",
        Text::SystemDefault => "Valeur système par défaut",
        Text::UnavailableOutput => "Sortie indisponible",
        Text::SystemSounds => "Sons système",
        Text::Application => "Application",
        Text::Settings => "Paramètres",
        Text::Exit => "Quitter",
        Text::System => "Système",
        Text::OpenAtLogin => "Ouvrir à la connexion",
        Text::StartAtLoginDescription => "Lancer Adufa à la connexion",
        Text::QuickAccess => "Accès rapide",
        Text::OpenSelectorNearCursor => "Ouvrir le sélecteur près du curseur",
        Text::AudibleAppUnderPointer => "Cible l’app audible sous le pointeur",
        Text::SystemAudioSettings => "Paramètres audio du système",
        Text::OpenWindowsMixer => "Ouvrir le mélangeur de volume Windows",
        Text::Language => "Langue",
        Text::LanguageDescription => "Choisir la langue de l’interface",
        Text::Automatic => "Automatique",
        Text::ChooseLanguage => "Choisir la langue",
        Text::AudioOutput => "Sortie audio",
        Text::Muted => "Muet",
        Text::LocatorFailed => "Échec du repérage du son",
        Text::DiscoveryFailed => "Échec de la détection audio",
        Text::StartupFailed => "Échec de la préférence de démarrage",
        Text::LanguageFailed => "Échec de la préférence de langue",
        Text::WindowsMixer => "Mélangeur de volume Windows",
        Text::MixerOpenFailed => {
            "Windows n’a pas pu ouvrir les paramètres de volume des applications."
        }
        Text::RoutingFailed => "Échec du routage audio",
        Text::RouteChangeFailed => "Adufa n’a pas pu modifier la sortie de cette application.",
        Text::CouldNotStart => "Adufa n’a pas pu démarrer.",
    }
}

const fn german(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "Audio-Router",
        Text::Listening => "Hört zu",
        Text::FindSound => "Ton finden",
        Text::NoApplications => "Keine Anwendung gibt Audio wieder",
        Text::SystemDefault => "Systemstandard",
        Text::UnavailableOutput => "Ausgabe nicht verfügbar",
        Text::SystemSounds => "Systemsounds",
        Text::Application => "Anwendung",
        Text::Settings => "Einstellungen",
        Text::Exit => "Beenden",
        Text::System => "System",
        Text::OpenAtLogin => "Bei Anmeldung öffnen",
        Text::StartAtLoginDescription => "Adufa nach der Anmeldung starten",
        Text::QuickAccess => "Schnellzugriff",
        Text::OpenSelectorNearCursor => "Auswahl am Mauszeiger öffnen",
        Text::AudibleAppUnderPointer => "Wählt die hörbare App unter dem Zeiger",
        Text::SystemAudioSettings => "System-Audioeinstellungen",
        Text::OpenWindowsMixer => "Windows-Lautstärkemixer öffnen",
        Text::Language => "Sprache",
        Text::LanguageDescription => "Sprache der Oberfläche auswählen",
        Text::Automatic => "Automatisch",
        Text::ChooseLanguage => "Sprache auswählen",
        Text::AudioOutput => "Audioausgabe",
        Text::Muted => "Stumm",
        Text::LocatorFailed => "Tonsuche fehlgeschlagen",
        Text::DiscoveryFailed => "Audioerkennung fehlgeschlagen",
        Text::StartupFailed => "Autostart-Einstellung fehlgeschlagen",
        Text::LanguageFailed => "Spracheinstellung fehlgeschlagen",
        Text::WindowsMixer => "Windows-Lautstärkemixer",
        Text::MixerOpenFailed => {
            "Windows konnte die Lautstärkeeinstellungen der Anwendungen nicht öffnen."
        }
        Text::RoutingFailed => "Audio-Routing fehlgeschlagen",
        Text::RouteChangeFailed => "Adufa konnte die Ausgabe dieser Anwendung nicht ändern.",
        Text::CouldNotStart => "Adufa konnte nicht gestartet werden.",
    }
}

const fn italian(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "Router audio",
        Text::Listening => "In ascolto",
        Text::FindSound => "Trova suono",
        Text::NoApplications => "Nessuna applicazione sta riproducendo audio",
        Text::SystemDefault => "Predefinito di sistema",
        Text::UnavailableOutput => "Uscita non disponibile",
        Text::SystemSounds => "Suoni di sistema",
        Text::Application => "Applicazione",
        Text::Settings => "Impostazioni",
        Text::Exit => "Esci",
        Text::System => "Sistema",
        Text::OpenAtLogin => "Apri all’accesso",
        Text::StartAtLoginDescription => "Avvia Adufa dopo l’accesso",
        Text::QuickAccess => "Accesso rapido",
        Text::OpenSelectorNearCursor => "Apri il selettore vicino al cursore",
        Text::AudibleAppUnderPointer => "Individua l’app udibile sotto il puntatore",
        Text::SystemAudioSettings => "Impostazioni audio di sistema",
        Text::OpenWindowsMixer => "Apri il mixer volume di Windows",
        Text::Language => "Lingua",
        Text::LanguageDescription => "Scegli la lingua dell’interfaccia",
        Text::Automatic => "Automatico",
        Text::ChooseLanguage => "Scegli lingua",
        Text::AudioOutput => "Uscita audio",
        Text::Muted => "Disattivato",
        Text::LocatorFailed => "Ricerca del suono non riuscita",
        Text::DiscoveryFailed => "Rilevamento audio non riuscito",
        Text::StartupFailed => "Preferenza di avvio non riuscita",
        Text::LanguageFailed => "Preferenza della lingua non riuscita",
        Text::WindowsMixer => "Mixer volume di Windows",
        Text::MixerOpenFailed => {
            "Windows non ha potuto aprire le impostazioni del volume delle applicazioni."
        }
        Text::RoutingFailed => "Instradamento audio non riuscito",
        Text::RouteChangeFailed => "Adufa non ha potuto cambiare l’uscita di questa applicazione.",
        Text::CouldNotStart => "Impossibile avviare Adufa.",
    }
}

const fn japanese(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "オーディオルーター",
        Text::Listening => "検出中",
        Text::FindSound => "音を探す",
        Text::NoApplications => "音声を再生中のアプリはありません",
        Text::SystemDefault => "システムの既定値",
        Text::UnavailableOutput => "利用できない出力",
        Text::SystemSounds => "システム サウンド",
        Text::Application => "アプリケーション",
        Text::Settings => "設定",
        Text::Exit => "終了",
        Text::System => "システム",
        Text::OpenAtLogin => "サインイン時に開く",
        Text::StartAtLoginDescription => "サインイン後に Adufa を起動します",
        Text::QuickAccess => "クイック アクセス",
        Text::OpenSelectorNearCursor => "カーソル付近に選択画面を開く",
        Text::AudibleAppUnderPointer => "ポインター下の再生中アプリを選びます",
        Text::SystemAudioSettings => "システムのオーディオ設定",
        Text::OpenWindowsMixer => "Windows 音量ミキサーを開く",
        Text::Language => "言語",
        Text::LanguageDescription => "インターフェイスの言語を選択",
        Text::Automatic => "自動",
        Text::ChooseLanguage => "言語を選択",
        Text::AudioOutput => "音声出力",
        Text::Muted => "ミュート",
        Text::LocatorFailed => "音の検出に失敗しました",
        Text::DiscoveryFailed => "オーディオの検出に失敗しました",
        Text::StartupFailed => "起動設定を変更できませんでした",
        Text::LanguageFailed => "言語設定を変更できませんでした",
        Text::WindowsMixer => "Windows 音量ミキサー",
        Text::MixerOpenFailed => "Windows でアプリの音量設定を開けませんでした。",
        Text::RoutingFailed => "オーディオのルーティングに失敗しました",
        Text::RouteChangeFailed => "Adufa はこのアプリの出力を変更できませんでした。",
        Text::CouldNotStart => "Adufa を起動できませんでした。",
    }
}

const fn simplified_chinese(key: Text) -> &'static str {
    match key {
        Text::AudioRouter => "音频路由器",
        Text::Listening => "正在监听",
        Text::FindSound => "查找声音",
        Text::NoApplications => "没有应用正在播放音频",
        Text::SystemDefault => "系统默认",
        Text::UnavailableOutput => "输出不可用",
        Text::SystemSounds => "系统声音",
        Text::Application => "应用",
        Text::Settings => "设置",
        Text::Exit => "退出",
        Text::System => "系统",
        Text::OpenAtLogin => "登录时打开",
        Text::StartAtLoginDescription => "登录后启动 Adufa",
        Text::QuickAccess => "快速访问",
        Text::OpenSelectorNearCursor => "在光标附近打开选择器",
        Text::AudibleAppUnderPointer => "选择指针下正在发声的应用",
        Text::SystemAudioSettings => "系统音频设置",
        Text::OpenWindowsMixer => "打开 Windows 音量混合器",
        Text::Language => "语言",
        Text::LanguageDescription => "选择界面语言",
        Text::Automatic => "自动",
        Text::ChooseLanguage => "选择语言",
        Text::AudioOutput => "音频输出",
        Text::Muted => "已静音",
        Text::LocatorFailed => "声音定位失败",
        Text::DiscoveryFailed => "音频检测失败",
        Text::StartupFailed => "启动首选项设置失败",
        Text::LanguageFailed => "语言首选项设置失败",
        Text::WindowsMixer => "Windows 音量混合器",
        Text::MixerOpenFailed => "Windows 无法打开应用音量设置。",
        Text::RoutingFailed => "音频路由失败",
        Text::RouteChangeFailed => "Adufa 无法更改此应用的输出。",
        Text::CouldNotStart => "Adufa 无法启动。",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "adufa-language-{}-{nonce}.json",
            std::process::id()
        ))
    }

    #[test]
    fn language_tags_resolve_to_supported_locales_with_english_fallback() {
        assert_eq!(Locale::from_language_tag("pt-PT"), Locale::PtBr);
        assert_eq!(Locale::from_language_tag("es-MX"), Locale::Es);
        assert_eq!(Locale::from_language_tag("zh-Hans"), Locale::ZhCn);
        assert_eq!(Locale::from_language_tag("zh-TW"), Locale::En);
        assert_eq!(Locale::from_language_tag("ko-KR"), Locale::En);
    }

    #[test]
    fn automatic_choice_uses_the_detected_system_locale() {
        let preference = LanguagePreference::load(temporary_path(), Locale::Ja)
            .expect("missing preference should use automatic mode");
        assert_eq!(preference.choice(), LanguageChoice::Automatic);
        assert_eq!(preference.locale(), Locale::Ja);
    }

    #[test]
    fn manual_choice_survives_loading_a_new_preference_instance() {
        let path = temporary_path();
        let mut first = LanguagePreference::load(path.clone(), Locale::En)
            .expect("temporary preference should load");
        first
            .set(LanguageChoice::Manual(Locale::Fr))
            .expect("manual preference should save");

        let second = LanguagePreference::load(path.clone(), Locale::Ja)
            .expect("saved preference should reload");
        assert_eq!(second.choice(), LanguageChoice::Manual(Locale::Fr));
        assert_eq!(second.locale(), Locale::Fr);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn unknown_persisted_language_falls_back_to_automatic() {
        let path = temporary_path();
        fs::write(&path, br#"{"language":"xx"}"#).expect("fixture should be writable");
        let preference = LanguagePreference::load(path.clone(), Locale::It)
            .expect("unknown language should not prevent startup");
        assert_eq!(preference.choice(), LanguageChoice::Automatic);
        assert_eq!(preference.locale(), Locale::It);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn every_supported_locale_has_every_catalog_entry() {
        let keys = [
            Text::AudioRouter,
            Text::Listening,
            Text::FindSound,
            Text::NoApplications,
            Text::SystemDefault,
            Text::UnavailableOutput,
            Text::SystemSounds,
            Text::Application,
            Text::Settings,
            Text::Exit,
            Text::System,
            Text::OpenAtLogin,
            Text::StartAtLoginDescription,
            Text::QuickAccess,
            Text::OpenSelectorNearCursor,
            Text::AudibleAppUnderPointer,
            Text::SystemAudioSettings,
            Text::OpenWindowsMixer,
            Text::Language,
            Text::LanguageDescription,
            Text::Automatic,
            Text::ChooseLanguage,
            Text::AudioOutput,
            Text::Muted,
            Text::LocatorFailed,
            Text::DiscoveryFailed,
            Text::StartupFailed,
            Text::LanguageFailed,
            Text::WindowsMixer,
            Text::MixerOpenFailed,
            Text::RoutingFailed,
            Text::RouteChangeFailed,
            Text::CouldNotStart,
        ];
        for locale in SUPPORTED_LOCALES {
            for key in keys {
                assert!(
                    !text(locale, key).trim().is_empty(),
                    "{} has an empty string",
                    locale.code()
                );
            }
        }
    }
}
