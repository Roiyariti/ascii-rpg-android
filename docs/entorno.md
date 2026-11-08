# Entorno y estado de la primera etapa

## Detección

El anfitrión es Debian 13 x86_64. La terminal del agente está dentro del Flatpak de
VS Code (runtime Freedesktop 25.08), por lo que algunas herramientas del anfitrión
no aparecen en su PATH. `flatpak-spawn --host` permite consultar Java y Android.

- VS Code 1.139.1; `code .` ejecutado para abrir el proyecto.
- rust-analyzer instalado; CodeLLDB 1.12.3 instalado para depuración.
- Rust estable del usuario: 1.99.1; el proyecto fija la herramienta disponible 1.95.0.
- Git 2.56.0 dentro de Flatpak.
- GitHub CLI 2.103.3 instalado en `.tools/bin/gh`, con SHA-256 oficial verificado.
- Java OpenJDK 21, SDK/plataforma 35, Build Tools 35.0.0 y NDK r28c en el anfitrión.
- Destino Rust `aarch64-linux-android` ya instalado.

## Incidencias

El aislamiento adicional de ejecución falla al crear namespaces (`bwrap`). Las operaciones
locales se realizan mediante el mecanismo de ejecución autorizado disponible.
Esto no requiere cambiar opciones del kernel ni modificar otros proyectos.

GitHub CLI no tenía sesión inicialmente. Se completó el acceso mediante navegador y se
verificó la cuenta personal `Roiyariti`. Se comprobó que el nombre estaba disponible y
se creó el repositorio público `Roiyariti/ascii-rpg-android`, con primer commit
`ad0da12cb923a5d2490636eece54518c98c7a4d2`. El hash remoto de `main` coincidió con el local.
GitHub Actions pasó para ese primer commit antes de avanzar a la fase gráfica.

El wrapper Gradle tuvo un timeout de descarga inicial. Se descargó la misma distribución
oficial mediante Python, se verificó su SHA-256 y se colocó en la caché del wrapper.
El empaquetado local terminó correctamente con esa distribución.

El equipo dispone de aproximadamente 4 GB de RAM. Compilar Linux y Android a la vez
provocó mucho intercambio de memoria. Se limitaron los jobs de Cargo a uno en
`.cargo/config.toml`, el heap de Gradle a 1 GB y las verificaciones locales se ejecutan
por plataforma. CI puede usar dos jobs mediante `CARGO_BUILD_JOBS`.
VS Code excluye también las cachés de herramientas de sus observadores de archivos.
Los seis crates del juego tienen símbolos completos para depurar variables; las
dependencias conservan símbolos limitados y sus herramientas de compilación mantienen
el perfil ligero. Al añadir crates, consultar los overrides de `Cargo.toml`.

AGP emite localmente un aviso sobre XML de SDK versión 4 frente a su lector versión 3,
debido a la diferencia de antigüedad con las herramientas de línea de comandos. No
impidió configurar Gradle ni generar la APK. Se revisará al actualizar el conjunto SDK/AGP.

La primera comprobación gráfica en CI pasó formato, Clippy, seis pruebas y compilación
Linux. Detectó dos supuestos incorrectos del workflow: el paquete SDK retirado `tools`
en los valores por defecto de setup-android, y el nombre del archivo ICD de Lavapipe.
Se configura solo `platform-tools` y se descubre el ICD instalado mediante su patrón.

La primera ejecución por ADB detectó que `Limits::downlevel_defaults()` limitaba las
texturas a 2048 píxeles, insuficientes para la pantalla de 2400x1080 del teléfono.
Se corrigió la petición del dispositivo con `using_resolution(adapter.limits())`,
se recompiló la APK y se actualizó la instalación mediante `adb install -r`.

## Verificación de la demo

- Formato, Clippy con advertencias como errores, seis pruebas y compilación Linux:
  correctos tanto localmente como en CI.
- Prueba de lectura GPU: 256 píxeles de cobertura de la A original, posición y fondo
  correctos. Localmente usa Vulkan por software (llvmpipe), y CI usa Lavapipe.
- Ventana Linux real: cinco frames presentados con 1.694 glifos por frame en la GPU
  Intel HD Graphics (Bay Trail), backend OpenGL. Mesa indica soporte Vulkan incompleto
  para esa GPU; la selección automática de backend permitió usar el controlador GL.
  Comprobado tanto dentro de VS Code Flatpak como directamente en Debian anfitrión.
- CodeLLDB: adaptador nativo iniciado, breakpoint real en `GameState.apply` y variables
  `self` y `action` accesibles. `launch.json` conserva ASLR para depurar en Flatpak sin
  solicitar permisos adicionales para cambiar la disposición de memoria del proceso.
- ARM64: biblioteca gráfica completa compilada y enlazada localmente con NDK r28c;
  primera compilación release con un job: 16 min 55 s. La compilación incremental
  siguiente tardó 1,72 s. Gradle local generó la APK debug con `BUILD SUCCESSFUL`.
- APK local: `android/app/build/outputs/apk/debug/app-debug.apk`, 10.489.982 bytes,
  versión `0.1.0-dev`, API mínima 26, destino 35 y ABI `arm64-v8a`.
- Firma v2 válida, `zipalign -c -P 16 4` correcto, segmentos ELF LOAD alineados a
  16 KB y una única biblioteca `lib/arm64-v8a/libascii_rpg.so`. Exporta
  `ANativeActivity_onCreate` y `android_main`; solo depende de bibliotecas del sistema
  Android (`liblog`, `libandroid`, `libdl`, `libm` y `libc`).
- SHA-256 de la APK local verificada:
  `b6c31255798b7e4477867ac236357f9d0d2f178380e3524cfc67fafe8f606cdb`.
- Teléfono ARM64 con Android 16 / API 36: instalación por ADB correcta, actividad
  nativa en primer plano y escena visible a 2400x1080 mediante Vulkan en Mali-G57 MC2.
  Se observó respuesta de los controles en el contador de turnos. La captura de
  comprobación permanece local en `.artifacts`, fuera de Git.
- [CI del código y perfil de depuración](https://github.com/Roiyariti/ascii-rpg-android/actions/runs/42431483610):
  trabajos Linux y Android correctos.
- Revisión del índice Git: 49 archivos sin patrones detectados de secretos o rutas
  personales; comprobadas las exclusiones de herramientas, target, `.env`, keystores y APK.

## Pendientes

Ampliar las pruebas del teléfono ARM64: pausa/reanudación, consumo, memoria GPU,
otros controladores y páginas de 16 KB. La instalación, el arranque gráfico y la
respuesta del contador a los controles ya se comprobaron en un dispositivo real.
No se publica una Release ni se presenta el prototipo como una versión estable jugable.
