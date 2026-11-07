# Desarrollo, GitHub y privacidad

`main` conserva incrementos verificados. Los cambios importantes se desarrollan en ramas
`feat/...` y se integran después de pasar formato, clippy, pruebas y compilación.
Cada etapa tiene un commit descriptivo. No se utilizan pushes forzados.

El repositorio previsto es público y se llama `ascii-rpg-android`. Antes de crearlo hay que
comprobar con `gh auth status` y `gh api user` la cuenta personal activa y verificar que
el nombre no esté ocupado. Nunca se imprimen tokens ni se guardan en el repositorio.

Antes de cada publicación revisar `git diff --cached --check`, la lista de archivos
versionados y las exclusiones. `.tools`, `.local`, `.env`, rutas locales de Android,
target, cachés, APK y claves de firma permanecen fuera de Git.

Si falta autenticación, ejecutar GitHub CLI con `auth login --hostname github.com
--git-protocol https --web` y completar el navegador. Las credenciales pertenecen a la
configuración del usuario, no al código. Para los commits públicos se prefiere el correo
`noreply` de GitHub, configurado solamente en este repositorio.

Las APK debug se probarán sin presentarlas como versiones estables. Las futuras Releases
requerirán pruebas completas y una decisión explícita de publicar una versión.
