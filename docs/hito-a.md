# Hito A: terreno isométrico por chunks

Trabajo en `feature/worldgen-2_5d`. `main` conserva la versión anterior.
Este incremento se centra en crear, ejecutar y revisar el mundo en PC.

## Auditoría del prototipo

La revisión confirmó un único campo Perlin, 289 celdas regeneradas en cada frame,
árbol y ruina en coordenadas constantes, proyección fija y ausencia de profundidad.
El atlas original ocupa 8 KiB y el buffer admitía 16.384 instancias de 48 bytes.
La ventana winit, el pipeline instanciado, el núcleo de turnos y la entrada Android
eran funcionales y se conservaron como base.

## Generación y continuidad

- `world-generation/height.rs`: cinco campos Perlin con semillas derivadas,
  ruido fractal de varias escalas, domain warping, crestas, valles encajados,
  llanuras y mesetas. No hay simulación de erosión ni sedimentación todavía.
- Las alturas quedan entre -10 y 18, en medios niveles. Cada celda conserva un
  gradiente limitado para representar pendientes dentro de esos niveles.
- `chunks.rs`: chunks de 16x16 celdas y halo de una celda. Una rejilla temporal
  de alturas de 20x20 permite calcular también los gradientes del halo.
- La generación usa coordenadas absolutas y f64; el renderizado convierte a
  coordenadas relativas al jugador antes de pasar a f32. Las coordenadas negativas
  usan división euclídea. El hash de variación no utiliza un RNG global.
- Los vecinos generan las mismas alturas, gradientes, formas y variaciones en
  sus bordes. Cargar primero otro chunk o expulsar/regenerar uno no cambia el mundo.
- La caché limita el radio a 48 celdas: como máximo 49 chunks. En la escena inicial
  hay 16 chunks, con 103.680 bytes de datos de celdas, sin contar contenedores ni
  las demás estructuras de la aplicación.

Las clases de relieve permiten distinguir visualmente tierra, roca y zonas bajas.
No representan aún un sistema de clima, biomas, ríos o materiales del Hito B.
Se retiraron los objetos decorativos en coordenadas fijas; la vegetación avanzada
y las estructuras procedurales se incorporarán en los hitos siguientes.

## Superficies, cámara y profundidad

Cada celda tiene una superficie superior inclinada y paredes hacia las caras
visibles cuando su borde supera al vecino. Las paredes llegan hasta el terreno
contiguo y se calculan también en los límites de chunks usando el halo.
Los trazos ASCII se proyectan sobre esos planos, con un fondo tenue opaco entre
trazos. Esto permite ocultar correctamente una entidad detrás de una pared.
Las caras tienen tonos distintos para hacer legible el volumen; la iluminación
direccional y las sombras del Hito C siguen pendientes.

`IsoCamera` configura tamaño de la celda, escala de altura, zoom y desplazamiento.
La profundidad ortográfica sigue el mismo rayo que la proyección. El personaje
avanza lógicamente una casilla y el desplazamiento visual interpola durante 180 ms.
La cámara sigue esa posición interpolada. Las animaciones no modifican el reloj.
No hay aún restricciones de navegación por pendiente ni simulación física de escalada.

El lienzo conserva su relación de aspecto: altura lógica de 600 y anchura derivada
de la ventana. Se probaron formatos de 16:10, 20:9 y 32:9, incluido zoom mínimo.
La interfaz es un control de prueba del terreno; el HUD RPG, inventario y áreas
seguras completas pertenecen al Hito D.

## Presupuesto e instrumentación

| Recurso | Presupuesto |
| --- | --- |
| Atlas ASCII original R8 | 8 KiB |
| Instancias GPU | 16.384 x 80 bytes = 1,25 MiB |
| Lista CPU de glifos | Capacidad limitada al mismo presupuesto |
| Profundidad | Depth16Unorm, 2 bytes por píxel |
| Profundidad a 960x600 | 1.125 KiB |
| Profundidad a 2400x1080 | 5.062,5 KiB |
| Datos de celdas, 49 chunks | 317.520 bytes |

La instancia incorpora dos ejes proyectados y profundidad por vértice. El límite
de glifos se mantiene. Hay culling por ventana y subdivisiones 3x3 cerca del jugador,
2x2 a distancia y reducción adaptativa del detalle antes de llenar el buffer.
Se reservan 512 instancias para personaje y controles; en vistas extremas se
priorizan las celdas próximas. El intento de detalle superior se interrumpe antes
de ampliar el buffer CPU.

Los chunks reutilizan sus muestras. Las instancias visibles todavía se reconstruyen
y se suben completas cada frame: los buffers estáticos e incrementales del Hito D
no se presentan como terminados. Tampoco se mide tiempo GPU con timestamps todavía.

El panel opcional y los registros muestran FPS, tiempo CPU de construcción del
mundo, glifos, celdas, chunks y tamaños de los datos. Son mediciones parciales;
no equivalen a memoria total del proceso ni a tiempo de ejecución GPU.

## Ejecutar y reproducir en PC

```sh
cargo run --locked -p glyph-demo --bin glyph-demo
# Región montañosa con la misma implementación y otra semilla
ASCII_RPG_SEED=73 cargo run --locked -p glyph-demo --bin glyph-demo
# Capturas PPM de la escena real con lectura GPU; salida, semilla, x, y, zoom
WGPU_BACKEND=vulkan cargo run --locked -p glyph-demo --example world-snapshot -- .artifacts/world-a.ppm 42 0 0 1.25
WGPU_BACKEND=vulkan cargo run --locked -p glyph-demo --example world-snapshot -- .artifacts/world-a-73.ppm 73 0 0 1.25
```

WASD/flechas mueven, espacio espera y F/Q/E conservan las acciones de prueba que
consumen un turno. Rueda o +/- cambian zoom; IJKL o arrastre con botón central
desplazan la vista; Home la centra; Tab muestra diagnóstico; Escape cierra.
Los botones O, DBG y +/- permiten probar la cámara también mediante clic/toque.
SEED, dentro del panel de diagnóstico, cambia de semilla sin consumir turnos.

VS Code incluye las tareas `Demo: ejecutar Linux`, `Terreno: montañas en PC` y
`Terreno: captura GPU`. El núcleo lógico mantiene símbolos completos y compilación
sin optimización. Terreno y gráficos usan optimización por paquete en desarrollo
para facilitar la revisión interactiva en el equipo de 4 GB.

## Verificaciones

- Formato, Clippy con advertencias como errores y diez pruebas de workspace.
- Comparación de halos en ambos ejes, tres semillas, coordenadas negativas y
  regiones lejanas; comparación con muestreo directo; expulsión y regeneración.
- Cámara, interpolación, independencia de turnos, caché y presupuesto CPU/GPU
  en vista 32:9 con zoom mínimo.
- Lectura GPU: cobertura de la A original, oclusión independiente del orden,
  fondo opaco y profundidad interpolada de un plano inclinado.
- Capturas de las semillas 42 y 73; verificación de cobertura de terreno en más
  de la mitad de la imagen. Dos capturas de la semilla 42 coinciden byte por byte.
- Ventana Linux real mediante OpenGL en Intel HD Graphics Bay Trail.
- GitHub Actions compila Linux y Android y conserva capturas y APK debug durante
  siete días. No crea Releases ni modifica automáticamente `main`.

Las primeras mediciones sin optimizar el código gráfico dieron 24,8–26,4 FPS
en Linux a 960x600, con unos 8,6 ms de CPU para la escena y 195–215 ms de
generación inicial de los 16 chunks. Ese coste inicial no se repite cada frame.
Con el perfil de desarrollo ajustado, una prueba de 180 frames a 960x600 dio
42,0 FPS, 5,90 ms de CPU media para la escena, 7.271 glifos y 828 celdas visibles.
La carga inicial alcanzó 199,41 ms. Las capturas 1280x720 contienen 7.662 glifos
para la semilla 42 y 13.041 para la 73; las dos repeticiones de la 42 son idénticas.

La ventana de la región montañosa también se abrió directamente en Debian, fuera
de Flatpak, y quedó disponible para explorar. En muestras de 300 frames durante
el desplazamiento se midieron 25,2–33,1 FPS, 5,72–7,48 ms de CPU media y picos
de aproximadamente 61 ms con carga de nuevos chunks. Son pruebas breves en
Bay Trail, sin medición de tiempo GPU ni garantía de 60 FPS en todos los paisajes.

## Android y trabajo siguiente

Se conserva NativeActivity ARM64, SDK 35, NDK r28c, el límite de resolución
del adaptador y la liberación de superficie al suspender. La versión debug del
Hito A se identifica como `0.2.0-hito-a`, versionCode 2.
La compilación ARM64 y el empaquetado debug se comprobaron localmente.
La instalación y las pruebas de esta actualización en teléfono quedan pendientes
de la revisión en PC; las pruebas físicas de la versión anterior no validan esta.

También quedan pendientes el rendimiento sostenido móvil, la generación asíncrona
para evitar pausas al cargar chunks, la pérdida del dispositivo GPU y una prueba
real en páginas de 16 KB. El Hito B añadirá vegetación y materiales; C, iluminación;
D, HUD RPG y optimización incremental.
