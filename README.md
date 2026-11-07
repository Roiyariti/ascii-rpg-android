# ASCII RPG 2.5D

Prototipo independiente de RPG dark fantasy para Android ARM64 y Linux, escrito en Rust.
La visión es un mundo elevado 2.5D dibujado con numerosos glifos ASCII mediante GPU,
personajes chibi compactos y detallados, criaturas de tamaños diversos y combate táctico.

## Estado inicial

Workspace funcional con núcleo lógico, reloj de turnos y una muestra determinista de
elevaciones. El renderizador GPU y la aplicación nativa Android constituyen el siguiente
incremento. No hay todavía combate, inventario ni un mundo jugable.

## Desarrollo

Instalar Rust mediante rustup y abrir esta carpeta con `code .`. El archivo
`rust-toolchain.toml` selecciona Rust/Cargo 1.95.0 y los componentes necesarios.
VS Code utiliza rust-analyzer para análisis, pruebas y navegación; CodeLLDB permite
depurar la futura aplicación. Git y la vista de control de versiones están integrados en VS Code.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
```

El primer `cargo build` descarga dependencias; `Cargo.lock` se versiona para conservar
su resolución. Las acciones de mover, esperar, atacar, cambiar arma y equipar consumen
un turno cuando se confirman. Una entrada inválida no consume turnos. Atacar/equipar
son por ahora tipos de acción, sin sistemas RPG implementados.

## Estructura y documentación

- `crates/game-core`: simulación y proyección elevada, sin GPU.
- `crates/turn-system`: acciones y reloj de turnos.
- `crates/world-generation`: muestra limitada de elevaciones por semilla.
- `.vscode`: tareas, análisis y extensiones mínimas.
- `.github/workflows`: formato, análisis, pruebas y compilación.
- [Arquitectura y fases](docs/arquitectura.md).
- [Versiones y fuentes oficiales](docs/tecnologias.md).
- [Flujo Git y privacidad](docs/desarrollo.md).
- [Entorno e incidencias](docs/entorno.md).

## Licencia y recursos

Licencia pendiente de elección. No se incluye una licencia de código abierto por defecto.
No se copian código, personajes ni recursos de otros videojuegos. Las dependencias
conservan sus propias licencias.
