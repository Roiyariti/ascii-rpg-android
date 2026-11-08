# Selección técnica

Consulta realizada el 7 de noviembre de 2026. Se eligen versiones compatibles y fijadas;
no se exige usar la versión más reciente de cada herramienta.

| Componente | Selección | Motivo |
| --- | --- | --- |
| Rust / Cargo | 1.95.0, edición 2024 | Toolchain ya disponible, fijado por rustup |
| wgpu | 30.0.1 | GPU nativa Vulkan/GLES; MSRV documentado 1.87 |
| winit | 0.30.13 | Ventana y ciclo de vida; raw-window-handle 0.6 |
| glam | 0.34.2 | Vectores y proyección; serde habilitado |
| serde | 1.0.229 | Estado serializable |
| noise | 0.9.0 | Perlin determinista |
| Android SDK | plataforma 35, Build Tools 35.0.0 | Instalados, compatibles con AGP 8.9 |
| Android NDK | r28c / 28.2.13676358 | Instalado, ARM64, páginas de 16 KB |
| AGP / Gradle | 8.9.2 / 8.11.1 | Pareja documentada para SDK 35 |
| Java | 21 (mínimo AGP: 17) | Instalado; soportado por Gradle 8.11.1 |

Fuentes primarias:

- [Rust 1.95](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/).
- [wgpu y backends](https://docs.rs/wgpu/30.0.1/wgpu/).
- [winit Android: NativeActivity y android-activity 0.6](https://docs.rs/winit/0.30.13/winit/platform/android/index.html).
- [glam](https://docs.rs/glam/0.34.2/glam/), [serde](https://docs.rs/serde/1.0.229/serde/), [noise](https://docs.rs/noise/0.9.0/noise/).
- [Compatibilidad AGP 8.9](https://developer.android.com/build/releases/agp-8-9-0-release-notes).
- [Compatibilidad Java/Gradle](https://docs.gradle.org/8.11.1/userguide/compatibility.html).
- [Perfiles Cargo y símbolos de depuración por paquete](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides).
- [NDK y otros sistemas de compilación](https://developer.android.com/ndk/guides/other_build_systems).

AGP 9.x está disponible, pero requiere una actualización del conjunto de herramientas.
Se conserva SDK 35 durante el prototipo local; los requisitos de publicación en tiendas
deberán revisarse antes de distribuir. SDK 35 no se presenta como política actual de Google Play.
