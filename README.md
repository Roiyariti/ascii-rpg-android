# ASCII RPG 2.5D

Prototipo independiente de RPG dark fantasy para Android ARM64 y Linux, escrito en Rust.
La visión es un mundo elevado 2.5D dibujado con numerosos glifos ASCII mediante GPU,
personajes chibi compactos y detallados, criaturas de tamaños diversos y combate táctico.

## Estado: Hito A

La rama `feature/worldgen-2_5d` incorpora relieve fractal con domain warping, chunks
deterministas, superficies inclinadas y paredes ASCII, profundidad GPU e interpolación
de movimiento. La cámara permite zoom y desplazamiento sin deformar la relación de aspecto.
El núcleo lógico conserva los turnos independientemente de los frames.
No hay todavía combate, inventario, sombras ni un mundo jugable completo.
La revisión se centra primero en PC. Véase el [informe del Hito A](docs/hito-a.md).

Repositorio público: [Roiyariti/ascii-rpg-android](https://github.com/Roiyariti/ascii-rpg-android).
Primer commit: `ad0da12cb923a5d2490636eece54518c98c7a4d2`.

## Desarrollo

Instalar Rust mediante rustup y abrir esta carpeta con `code .`. El archivo
`rust-toolchain.toml` selecciona Rust/Cargo 1.95.0 y los componentes necesarios.
VS Code utiliza rust-analyzer para análisis, pruebas y navegación; CodeLLDB permite
depurar la aplicación. Git y la vista de control de versiones están integrados en VS Code.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
cargo run --locked -p glyph-demo --bin glyph-demo
```

En Debian, las bibliotecas de desarrollo se pueden instalar con
`sudo apt install build-essential pkg-config libwayland-dev libxkbcommon-dev libvulkan1 mesa-vulkan-drivers`.
La ventana usa Vulkan o GLES, no una terminal. Para seleccionar un backend utilizar
`WGPU_BACKEND=vulkan` o `WGPU_BACKEND=gl`.

`F5` inicia la depuración con CodeLLDB. Las tareas de VS Code incluyen pruebas,
compilación Linux, biblioteca Android ARM64 y APK debug. En terminales nuevas de esta
carpeta también se incorpora `.tools/bin` al PATH para el GitHub CLI local instalado.
Los crates del juego conservan símbolos completos para inspeccionar variables; las
dependencias utilizan símbolos limitados para reducir el consumo durante la compilación.
El núcleo conserva compilación sin optimización; terreno y gráficos usan optimización
por paquete para la prueba interactiva. Hay una tarea `Terreno: montañas en PC`.

## Prueba gráfica y Android

```sh
# Shader, atlas, cobertura y coordenadas mediante lectura de píxeles GPU
cargo run --locked -p ascii-renderer --example gpu-smoke
# Ventana real que termina tras presentar cinco frames
ASCII_RPG_SMOKE_FRAMES=5 cargo run --locked -p glyph-demo --bin glyph-demo
# Región montañosa en PC
ASCII_RPG_SEED=73 cargo run --locked -p glyph-demo --bin glyph-demo
# Captura reproducible del mundo mediante GPU, sin ventana (PPM)
WGPU_BACKEND=vulkan cargo run --locked -p glyph-demo --example world-snapshot -- .artifacts/world-a.ppm 73
# Biblioteca nativa ARM64 o APK de prueba
sh scripts/build-android.sh --library-only
sh scripts/build-android.sh
```

La APK se genera en `android/app/build/outputs/apk/debug/app-debug.apk` y se mantiene
fuera del repositorio. El script reconoce VS Code Flatpak y ejecuta las herramientas
Android del anfitrión con `flatpak-spawn --host`. Véase [Android](docs/android.md).

Controles: WASD/flechas mueven, espacio espera, F registra un ataque, Q registra un
cambio de arma y E registra equipar. Estos tres últimos solo demuestran el consumo
de turnos. Los botones inferiores admiten clic y toque. Escape cierra la ventana Linux.
Rueda o +/- ajustan el zoom; IJKL o arrastre con botón central desplazan la vista;
Home la centra y Tab muestra el diagnóstico. Los controles de cámara no consumen turnos.

El primer `cargo build` descarga dependencias; `Cargo.lock` se versiona para conservar
su resolución. Las acciones de mover, esperar, atacar, cambiar arma y equipar consumen
un turno cuando se confirman. Una entrada inválida no consume turnos. Atacar/equipar
son por ahora tipos de acción, sin sistemas RPG implementados.

## Estructura y documentación

- `crates/game-core`: simulación y proyección elevada, sin GPU.
- `crates/turn-system`: acciones y reloj de turnos.
- `crates/world-generation`: relieve multiescala, chunks con halo y caché acotada.
- `crates/ascii-renderer`: atlas R8, superficies ASCII instanciadas y profundidad Z16.
- `crates/game-assets`, `assets`, `shaders`: recursos incorporados al binario.
- `apps/glyph-demo`: ventana Linux y entrada Android `android_main` compartidas.
- `android`: NativeActivity y empaquetado Gradle para `arm64-v8a`.
- `.vscode`: tareas, análisis y extensiones mínimas.
- `.github/workflows`: formato, análisis, pruebas y compilación.
- [Arquitectura y fases](docs/arquitectura.md).
- [Versiones y fuentes oficiales](docs/tecnologias.md).
- [Flujo Git y privacidad](docs/desarrollo.md).
- [Entorno e incidencias](docs/entorno.md).
- [Auditoría, pruebas y límites del Hito A](docs/hito-a.md).

## Licencia y recursos

Licencia pendiente de elección. No se incluye una licencia de código abierto por defecto.
No se copian código, personajes ni recursos de otros videojuegos. Las dependencias
conservan sus propias licencias.
