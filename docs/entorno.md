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

El wrapper Gradle tuvo un timeout de descarga inicial; se intenta la misma distribución
oficial con validación SHA-256. No se cambia de versión para eludir verificaciones.

El equipo dispone de aproximadamente 4 GB de RAM. Compilar Linux y Android a la vez
provocó mucho intercambio de memoria. Se limitaron los jobs de Cargo a uno en
`.cargo/config.toml`, el heap de Gradle a 1 GB y las verificaciones locales se ejecutan
por plataforma. CI puede usar dos jobs mediante `CARGO_BUILD_JOBS`.

La primera comprobación gráfica en CI pasó formato, Clippy, seis pruebas y compilación
Linux. Detectó dos supuestos incorrectos del workflow: el paquete SDK retirado `tools`
en los valores por defecto de setup-android, y el nombre del archivo ICD de Lavapipe.
Se configura solo `platform-tools` y se descubre el ICD instalado mediante su patrón.

## Verificación

Base verificada localmente:

- `cargo fmt --all --check`: correcto.
- `cargo test --workspace`: cuatro pruebas correctas; ninguna fallida.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: correcto.
- `cargo build --workspace --locked`: correcto en Linux x86_64.
- `cargo build --workspace --locked --target aarch64-linux-android`: correcto para
  las bibliotecas lógicas. No equivale a enlazar un renderizador ni generar una APK.
- Revisión del índice Git: sin patrones de secretos o rutas personales detectados.
- Exclusiones comprobadas: herramientas locales, target, `.env`, keystores y APK.

Un workflow preparado no significa que se haya ejecutado en GitHub. La ventana gráfica,
la biblioteca nativa y la APK se verificarán después de la primera publicación.
