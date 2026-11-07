# Arquitectura y fases

El núcleo lógico desconoce ventanas, Android y GPU. El renderizador consumirá una lista
de instancias de glifos con posición, tamaño y color. La aplicación compondrá ambas capas
y traducirá teclado/toques a acciones. La frecuencia visual será independiente de los turnos.

La proyección inicial usa x/y como suelo y z como altura. Es una base 2.5D elevada;
la primera demo no equivale a un motor 3D completo con oclusión, sombras o navegación.
Las figuras se compondrán de muchos glifos por personaje, con materiales y capas propios;
el tamaño del personaje no limitará su detalle a una sola celda.

## Incrementos

1. Base: workspace, pruebas lógicas, documentación, Git y primera publicación.
2. Demo: ventana nativa Linux, atlas ASCII original, instancias GPU, entrada táctil,
   ciclo de vida Android, biblioteca ARM64 y APK debug cuando el entorno lo permita.
3. Medición en dispositivo: densidad de glifos, memoria, tiempos GPU, pausa/reanudación.
4. Personajes/animación: capas compactas, materiales, interpolación y criaturas grandes.
5. Terreno: chunks, bosques, ríos, montañas, ruinas, mazmorras e iluminación/sombras.
6. RPG: combate táctico, equipamiento e inventario; cada acción confirmada consume turnos.

No se crean crates vacíos para sistemas futuros. Personajes, animación y combate se
extraerán cuando tengan responsabilidades y pruebas reales. Se priorizan instancias
agrupadas, atlas compartido, buffers reutilizados y presupuestos medidos en ARM64.
