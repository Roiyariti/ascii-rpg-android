# Arquitectura y fases

El núcleo lógico desconoce ventanas, Android y GPU. El renderizador consume instancias
de superficies de glifos con ejes proyectados, profundidad, atlas y color. La aplicación
compone ambas capas y traduce teclado/toques a acciones. La frecuencia visual es
independiente de los turnos.

La proyección usa x/y como suelo y z como altura. `IsoCamera` incorpora seguimiento,
zoom y desplazamiento. El Hito A proyecta superficies inclinadas y paredes continuas
entre celdas, con profundidad por vértice y buffer Z16 para la oclusión. Los chunks
conservan muestras de terreno y halo; la escena reconstruye sus instancias visibles.
Las sombras, la navegación por pendientes y el guardado persistente siguen pendientes.
Las figuras se compondrán de muchos glifos por personaje, con materiales y capas propios;
el tamaño del personaje no limitará su detalle a una sola celda.

## Incrementos

1. Base: workspace, pruebas lógicas, documentación, Git y primera publicación.
2. Demo: ventana nativa Linux, atlas ASCII original, instancias GPU, entrada táctil,
   ciclo de vida Android, biblioteca ARM64 y APK debug cuando el entorno lo permita.
3. Hito A: relieve multiescala, chunks deterministas, cámara, volumen ASCII y pruebas en PC.
4. Hito B: vegetación volumétrica, materiales y variaciones de bioma.
5. Hito C: iluminación, sombras y oclusión ambiental escalable.
6. Hito D: HUD RPG, controles configurables, cachés visuales y buffers incrementales.
7. RPG: combate táctico, equipamiento e inventario; cada acción confirmada consume turnos.

El informe de [Hito A](hito-a.md) detalla reproducibilidad, continuidad, presupuestos
de memoria y comprobaciones gráficas. Cada plataforma se valida por separado.

No se crean crates vacíos para sistemas futuros. Personajes, animación y combate se
extraerán cuando tengan responsabilidades y pruebas reales. Se priorizan instancias
agrupadas, atlas compartido, buffers reutilizados y presupuestos medidos en ARM64.
