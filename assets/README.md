# Recursos propios del prototipo

`font5x7.txt` define trazos geométricos originales para los 95 caracteres ASCII
imprimibles. Es un recurso provisional de pruebas, creado para este proyecto.
No procede de otros videojuegos. El atlas R8 se genera en memoria (128x64, 8 KiB)
y se incorpora al binario; Android no depende de rutas del equipo de desarrollo.

Los sprites iniciales se componen en `apps/glyph-demo/src/scene.rs` con numerosos
glifos y colores. La calidad y densidad final del arte siguen pendientes de desarrollo.
La licencia del proyecto y de estos recursos queda pendiente de elección.
