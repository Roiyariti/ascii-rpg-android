#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ -f /.flatpak-info ] && command -v flatpak-spawn >/dev/null 2>&1; then
    exec flatpak-spawn --host sh "$project_dir/scripts/build-android.sh" "$@"
fi
cd "$project_dir"
export PATH="$HOME/.cargo/bin:$PATH"
export ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/28.2.13676358}"
export GRADLE_USER_HOME="${GRADLE_USER_HOME:-$project_dir/.local/gradle}"
ndk_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
if [ ! -x "$ndk_bin/aarch64-linux-android26-clang" ]; then
    printf '%s\n' 'Falta NDK r28c. Instalar ndk;28.2.13676358 o configurar ANDROID_NDK_HOME.' >&2
    exit 1
fi
if [ "${1:-}" = '--library-only' ]; then
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ndk_bin/aarch64-linux-android26-clang"
    export CC_aarch64_linux_android="$ndk_bin/aarch64-linux-android26-clang"
    export AR_aarch64_linux_android="$ndk_bin/llvm-ar"
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="${CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS:-} -C link-arg=-Wl,-z,max-page-size=16384"
    cargo build --locked --release --target aarch64-linux-android -p glyph-demo --lib
    mkdir -p android/app/src/main/jniLibs/arm64-v8a
    cp target/aarch64-linux-android/release/libascii_rpg.so android/app/src/main/jniLibs/arm64-v8a/
elif [ "$#" -eq 0 ]; then
    command -v java >/dev/null 2>&1 || { printf '%s\n' 'Falta Java 17 o 21.' >&2; exit 1; }
    mkdir -p .local
    if [ ! -f .local/debug.keystore ]; then
        (umask 077; keytool -genkeypair -keystore .local/debug.keystore -storepass android -keypass android -alias androiddebugkey -keyalg RSA -keysize 2048 -validity 10000 -dname 'CN=Android Debug,O=Android,C=US')
    fi
    exec sh android/gradlew -p android --no-daemon :app:assembleDebug
else
    printf '%s\n' 'Uso: scripts/build-android.sh [--library-only]' >&2
    exit 2
fi
