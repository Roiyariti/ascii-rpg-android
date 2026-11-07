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

GitHub CLI no tenía una sesión iniciada en la primera comprobación. La creación del
repositorio remoto y el primer push requieren completar `gh auth login` mediante navegador.
La fase gráfica espera a esa primera publicación, siguiendo el orden solicitado.

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
