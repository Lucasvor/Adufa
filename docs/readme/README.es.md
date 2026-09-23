<div align="center">
  <img src="../assets/adufa-icon.svg" width="112" alt="Icono de Adufa: un canal de sonido redirigido hacia una salida seleccionada">
  <h1>Adufa</h1>
  <p><strong>Dirige cada aplicación al dispositivo de audio adecuado.</strong></p>
  <p>Un selector rápido y local de salida de audio por aplicación, con control de volumen.</p>
</div>

<p align="center">
  <a href="../../README.md">English</a> ·
  <a href="../../README.pt-BR.md">Português (Brasil)</a> ·
  <strong>Español</strong> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <code>Beta para Windows</code> · <code>macOS previsto</code> · <code>Linux previsto</code> · <code>GPL-3.0-or-later</code>
</p>

![Adufa — dirige cada aplicación al dispositivo de audio adecuado](../assets/adufa-hero.svg)

## ¿Por qué Adufa?

Las videollamadas deberían usar los auriculares. La música debería sonar por los altavoces.
Un navegador puede necesitar un monitor o un cable virtual. Adufa permite ver las aplicaciones
que están emitiendo audio y enviar cada una a la salida que le corresponde, sin tener que
buscarla cada vez en el mezclador de volumen de Windows.

Adufa es deliberadamente pequeño: vive en el área de notificación, se abre cerca de donde
estás trabajando, recuerda las rutas de las aplicaciones y no estorba.

## Véalo en acción

![Una demostración breve de Adufa que muestra la ventana emergente de la bandeja, el panel complementario de la barra de tareas, el control de volumen y la selección de salida](../assets/adufa-demo.gif)

La animación es una representación determinista de la interfaz nativa creada para la
documentación; no contiene capturas del escritorio ni datos personales.

## Qué funciona actualmente

La beta pública actual funciona en **Windows 10 22H2 y Windows 11**.

- Detecta automáticamente las aplicaciones con sesiones de audio activas.
- Cambia el dispositivo de salida de una aplicación sin modificar el predeterminado del sistema.
- Recuerda las rutas de las aplicaciones cuando se reinician Adufa o la aplicación.
- Devuelve una aplicación a `Predeterminado del sistema` (`System default`) con una sola selección.
- Cambia el volumen actual y el estado de silencio de cada aplicación.
- En versiones compatibles de Windows 11, haz clic con el botón derecho en una
  aplicación en ejecución de la barra de tareas para abrir controles experimentales de
  salida, volumen y silencio junto al menú nativo, incluso antes de que tenga una sesión de audio.
- Incluye **Buscar sonido** (`Find sound`), una vista temporal en directo que resalta la aplicación con mayor volumen.
- Abre un selector rápido junto al cursor con `Ctrl + Alt + A`.
- Puede iniciarse al abrir sesión; esta opción permanece desactivada hasta que la habilites.
- Admite inglés, portugués de Brasil, español, francés, alemán, italiano, japonés y chino simplificado.
- Detecta el idioma de visualización de Windows y permite elegir otro idioma manualmente.
- Funciona localmente, sin cuentas, analítica, telemetría ni subida de audio.

### Integración experimental con Windows

En versiones compatibles de Windows 11, hacer clic con el botón derecho en el icono de la
barra de tareas de una aplicación en ejecución puede abrir el panel compacto de Adufa junto
al menú nativo, incluso antes de que la aplicación emita audio. El menú nativo sigue
disponible; Adufa lo complementa con controles de volumen y salida.

Esta integración usa el AppID del botón de la barra de tareas cuando está disponible, con el
nombre accesible y la ruta del ejecutable como alternativas. Es una función beta y puede
recurrir al atajo global o a la ventana emergente de la bandeja si Windows no proporciona una
asociación fiable.

## Instalar la beta

### Descargar una compilación portátil

Las versiones etiquetadas se compilan mediante GitHub Actions. Abre la
[página de Releases](../../../releases), descarga `Adufa-Windows-x64.exe` y ejecútalo.

El ejecutable de la beta inicial es portátil y no está firmado. Windows puede mostrar una
advertencia de SmartScreen hasta que haya paquetes firmados. Verifica que el archivo proceda
del release de este repositorio antes de permitir su ejecución.

### Compilar desde el código fuente

Requisitos:

- Windows 10 22H2 o Windows 11, x64;
- [Rust](https://www.rust-lang.org/tools/install) 1.85 o posterior con la toolchain MSVC;
- Visual Studio Build Tools con **Desarrollo para el escritorio con C++** y un Windows SDK.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --locked -p router-windows --target x86_64-pc-windows-msvc
```

Ejecuta:

```powershell
.\target\x86_64-pc-windows-msvc\release\router-windows.exe
```

Utiliza una compilación release para el uso normal. Las compilaciones release son aplicaciones
con interfaz gráfica de Windows y no abren una ventana de terminal. Las compilaciones debug
mantienen intencionadamente una consola para el diagnóstico.

## Cómo utilizarlo

### Dirigir una aplicación desde la bandeja

1. Inicia la reproducción de audio en la aplicación que quieras dirigir.
2. Abre los iconos ocultos del área de notificación y selecciona Adufa.
3. Selecciona la fila de la aplicación.
4. Elige una salida o `Predeterminado del sistema` (`System default`) para eliminar la ruta guardada.
5. Haz clic fuera de la ventana emergente o pulsa `Esc` para cerrarla.

La ruta se guarda con una identidad estable de la aplicación, no con un ID de proceso temporal.
Cuando una aplicación se reinicia, Adufa restaura la opción guardada si la plataforma puede
identificarla de forma segura.

### Averiguar qué aplicación está emitiendo sonido

1. Abre Adufa.
2. Selecciona **Buscar sonido** (`Find sound`).
3. Observa los indicadores de nivel en directo; se resalta la fuente audible más intensa.
4. Selecciona esa aplicación para cambiar su salida.

Buscar sonido no graba audio. Lee los niveles de pico de sesión que Windows ya proporciona y
se detiene cuando se cierra la ventana emergente compacta.

### Usar el panel complementario de la barra de tareas

1. Mantén abierta la aplicación de destino y haz clic con el botón derecho en su icono de la
  barra de tareas. No es necesario que el audio esté reproduciéndose todavía.
2. Utiliza el panel adyacente de Adufa para silenciar, ajustar el volumen o seleccionar una salida.
3. Al seleccionar una salida se cierran tanto el panel complementario como el menú nativo.
  Windows aplicará la ruta cuando se cree la sesión de audio de la aplicación.

Si el panel no aparece, usa `Ctrl + Alt + A` mientras apuntas a la aplicación de destino o abre
Adufa desde el área de notificación.

### Cambiar el idioma o el comportamiento de inicio

Abre **Configuración** (`Settings`) para:

- seguir el idioma de Windows o elegir cualquiera de los idiomas disponibles;
- habilitar o deshabilitar la apertura de Adufa al iniciar sesión;
- abrir el mezclador de volumen de Windows para los controles del sistema.

## Referencia de teclado y ratón

| Entrada | Acción |
| --- | --- |
| `Ctrl + Alt + A` | Abrir el selector rápido junto al cursor para la aplicación audible situada bajo el puntero |
| Clic derecho en una aplicación en ejecución de la barra de tareas | Abrir el panel experimental de Adufa junto al menú nativo |
| `Tab` o `↓` | Ir al elemento siguiente |
| `↑` | Ir al elemento anterior |
| `Enter` o `Space` | Activar el elemento enfocado |
| `Esc` | Cerrar el selector actual o volver desde Configuración |
| Clic fuera | Cerrar las ventanas transitorias de Adufa |

El atajo global puede no estar disponible si otra aplicación ya lo ha registrado; Adufa sigue
funcionando y el flujo de la bandeja permanece disponible.

## Estado de las plataformas

| Plataforma | Estado | Backend previsto |
| --- | --- | --- |
| Windows 10 22H2 / Windows 11 | **Beta disponible** | Windows Core Audio / WASAPI e interfaz Win32 nativa |
| macOS 14.2+ | Previsto | Core Audio con una interfaz nativa en la barra de menús |
| Linux, Wayland y X11 | Previsto | PipeWire con integración nativa en el escritorio |

La compatibilidad multiplataforma es la dirección del producto, no una afirmación de paridad
de funciones actual. Cada backend comunica sus capacidades explícitamente para que Adufa nunca
finja que una operación de enrutamiento no compatible ha funcionado. El icono y el lenguaje
central de interacción son comunes; el comportamiento y los materiales son nativos de cada plataforma.

## Privacidad

Adufa está diseñado para funcionar de forma local.

- Sin telemetría ni analítica.
- Sin cuenta de usuario.
- Sin grabación ni subida de audio.
- No requiere ningún servicio de red para dirigir el audio.
- Las rutas y preferencias se almacenan en tu ordenador.

En Windows, la configuración se guarda en el directorio local de datos de aplicaciones del
usuario actual. Eliminar el ejecutable portátil no borra automáticamente ese archivo de preferencias.

## Hoja de ruta

No se prometen fechas hasta que la implementación de la plataforma correspondiente esté demostrada.

### En desarrollo

- Reforzar la beta para Windows en las versiones compatibles de Windows 10 y 11.
- Instalador firmado y sumas de comprobación de los releases portátiles.
- Etiquetas accesibles, validación de alto contraste y mejores flujos de teclado.
- Revisión de las traducciones por hablantes nativos.
- Comprobaciones de actualización fiables, opcionales y respetuosas con la privacidad.

### Próximamente

- Búsqueda de aplicaciones y salidas.
- Salidas favoritas.
- Atajos globales configurables.
- Mini mezclador ampliado.
- Perfiles y reglas automáticas por aplicación.
- Mejores diagnósticos y reasociación recuperable de rutas.

### Plataformas previstas

- macOS 14.2+ mediante Core Audio, distribuido para Apple Silicon e Intel.
- Linux mediante PipeWire en Wayland y X11; un backend exclusivo para PulseAudio no forma
  parte de la primera versión para Linux.

### En estudio

- Integraciones más profundas con menús nativos cuando el sistema operativo proporcione una API segura.
- Sincronización opcional de preferencias sin subir audio ni datos de actividad.
- Varios conjuntos de dispositivos y perfiles reutilizables para trabajo, juegos, llamadas y streaming.

## Solución de problemas

### Falta una aplicación

Inicia la reproducción y vuelve a abrir Adufa. Algunas aplicaciones no crean una sesión de audio
hasta que producen sonido. Las sesiones protegidas o del sistema pueden exponer menos información
de identidad y aparecer agrupadas bajo otra aplicación.

### Una salida guardada no está disponible

Adufa conserva la ruta deseada en lugar de sustituirla silenciosamente por un dispositivo con un
nombre parecido. Vuelve a conectar el dispositivo exacto, elige otra salida o selecciona
`Predeterminado del sistema` (`System default`).

### El panel complementario de la barra de tareas no se abrió

La integración es experimental y requiere una asociación única entre el icono de la barra de tareas
y una aplicación audible. Prueba el atajo global o la ventana emergente de la bandeja. Windows 10
utiliza el flujo alternativo para integraciones que solo funcionan de forma fiable en Windows 11.

### `Ctrl + Alt + A` no hace nada

Es posible que otro programa ya haya registrado ese atajo global. Abre Adufa desde la bandeja;
los atajos configurables están en la hoja de ruta.

### Aparece una ventana de terminal

Probablemente estás ejecutando una compilación debug o iniciando mediante `cargo run`. Compila e
inicia el ejecutable release indicado en [Compilar desde el código fuente](#compilar-desde-el-código-fuente).

## Desarrollo

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Una prueba de ida y vuelta con una sesión de audio real de Windows se omite de forma predeterminada
porque modifica temporalmente una sesión real. Ejecútala solo en un equipo de desarrollo con una
sesión de audio activa y prescindible.

El repositorio se divide en:

- `crates/router-engine`: identidades, comandos y estado independientes de la plataforma;
- `platforms/windows`: audio de Windows, persistencia, integración con la barra de tareas e interfaz nativa;
- `docs/adr`: decisiones de arquitectura aceptadas;
- `docs/design`: investigación de interacción e identidad;
- `docs/assets`: recursos generados de documentación y marca.

El flujo de CI valida el formato, Clippy y las pruebas, y después genera el ejecutable portátil
Windows x64. Enviar una etiqueta como `v0.1.0-beta.1` crea un GitHub Release y adjunta
`Adufa-Windows-x64.exe`.

## Contribuir

Los informes de errores deben incluir la versión de Windows, la aplicación afectada, la salida
esperada, la salida observada y si se utilizó el flujo de la bandeja, el atajo o la barra de tareas.
No adjuntes grabaciones, archivos de configuración ni registros que contengan rutas privadas sin revisarlos.

Las contribuciones deben preservar las reglas fundamentales: identidad estable de la aplicación,
observación basada en eventos, comunicación honesta de capacidades, superficies nativas de cada
plataforma y ausencia de telemetría.

## Licencia y nombre

El código fuente se distribuye bajo **GPL-3.0-or-later**, tal como declara el workspace de Cargo.
“Adufa” y el material gráfico del proyecto identifican las compilaciones oficiales; la licencia
de código abierto no implica la aprobación de distribuciones modificadas.

El nombre solo ha pasado una comprobación preliminar de coincidencias en la web y en repositorios.
Esto no constituye una autorización jurídica de marca; la distribución oficial debe completar una
búsqueda formal en los territorios de lanzamiento previstos.

---

<p align="center"><strong>Adufa</strong> — una pequeña superficie de control para las rutas de audio que oculta tu escritorio.</p>
