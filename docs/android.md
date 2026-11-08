# Android nativo ARM64

El paquete `io.github.roiyariti.asciirpg` inicia `android.app.NativeActivity` y carga
`libascii_rpg.so`. `android_main` utiliza el `AndroidApp` reexportado por winit, evitando
versiones incompatibles de android-activity. No requiere Java/Kotlin de aplicación.

## Requisitos fijados

- Rust/Cargo 1.95.0 y destino `aarch64-linux-android`.
- Java 17 o 21; probado con el Java del anfitrión.
- SDK: `platforms;android-35`, `build-tools;35.0.0` y `platform-tools`.
- NDK: `ndk;28.2.13676358` (r28c).
- AGP 8.9.2, wrapper Gradle 8.11.1 y checksum de distribución.
- Android mínimo 8.0 / API 26, ARM64, Vulkan o GLES 3.0.

Instalar paquetes faltantes con el `sdkmanager` oficial y revisar sus licencias:

```sh
sdkmanager 'platforms;android-35' 'build-tools;35.0.0' 'platform-tools' 'ndk;28.2.13676358'
rustup target add --toolchain 1.95.0 aarch64-linux-android
```

El script usa `ANDROID_HOME`, `ANDROID_SDK_ROOT` y `ANDROID_NDK_HOME`, con valores
convencionales si no se definen. No se versiona `local.properties` ni ninguna ruta personal.

```sh
sh scripts/build-android.sh --library-only
sh scripts/build-android.sh
```

La biblioteca se compila en release para limitar tamaño y memoria. La APK es debug:
se firma con una clave de desarrollo generada en `.local/debug.keystore`. Esta clave,
los caches de Gradle y las bibliotecas copiadas a `jniLibs` están excluidos de Git.
La variante release de Gradle no tiene firma de distribución configurada.

Se usan NDK r28c y enlace con `max-page-size=16384`; AGP 8.9.2 alinea las bibliotecas
nativas sin comprimir. Esto prepara compatibilidad con páginas de 16 KB, que además
deberá probarse en un dispositivo o emulador adecuado.

## Ciclo de vida y pruebas

La ventana/superficie se crean en `resumed` y se liberan en `suspended`. El estado lógico
permanece en memoria durante esa pausa; aún no hay guardado persistente tras muerte del
proceso. Las superficies perdidas se recrean y un tamaño cero evita dibujar.

La escena usa un lienzo virtual 960x600 que se adapta al tamaño disponible; la tipografía
puede deformarse en otras relaciones de aspecto. Safe areas, accesibilidad, pérdida del
dispositivo GPU, materiales avanzados y rendimiento en hardware móvil son trabajo futuro.

Con un dispositivo propio conectado:

```sh
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n io.github.roiyariti.asciirpg/android.app.NativeActivity
adb logcat -s AsciiRpg
```

Compilar una APK no prueba la ejecución táctil, la pausa/reanudación ni la compatibilidad
del controlador de un teléfono. GitHub Actions conserva la APK debug como artefacto
temporal de pruebas, sin crear Releases ni versiones estables.

La APK debug se instaló por ADB y se abrió en un teléfono ARM64 con Android 16
(API 36), pantalla de 2400x1080 y GPU Mali-G57 MC2. Se comprobó el renderizado
Vulkan de la escena y la respuesta del contador de turnos a los controles.
Esta prueba detectó un límite de textura solicitado de 2048 píxeles que impedía
crear la superficie. El dispositivo wgpu ahora solicita los límites de resolución
del adaptador mediante `using_resolution`, conservando los demás límites básicos.
Siguen pendientes las pruebas de pausa/reanudación, consumo, memoria y páginas de 16 KB.
