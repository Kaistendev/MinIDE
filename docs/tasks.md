# MiniIDE — tasks.md

> Tareas derivadas de `plan.md`. Cada tarea está pensada para completarse en menos de 30 minutos, con dependencias explícitas por orden.

## Reglas de trabajo

- Completar las tareas en orden salvo que una dependencia ya esté satisfecha.
- No implementar funcionalidades fuera del alcance del MVP.
- Cada tarea debe terminar con build/tests relevantes en verde.
- Si una tarea crece de forma material, dividirla antes de continuar.
- Mantener el core independiente de C#, Java, WinForms y Swing.

---

# Fase 0 — Preparación

- [x] **T-001 — Crear la estructura base del workspace**  
  **RF:** RF-01, RF-15  
  **Hecho cuando:** existe la estructura inicial del proyecto Rust y `cargo check` termina correctamente.

- [x] **T-002 — Definir módulos base del core**  
  **RF:** RF-06, RF-14, RF-15  
  **Hecho cuando:** existen módulos mínimos para `core`, `document`, `project`, `commands` y `ui`, sin lógica duplicada.

- [x] **T-003 — Configurar formato, lint y política de warnings**  
  **RF:** —  
  **Hecho cuando:** `cargo fmt --check` y `cargo clippy` pueden ejecutarse y la política de calidad queda documentada.

- [x] **T-004 — Configurar estrategia inicial de tests**  
  **RF:** RF-02, RF-03, RF-06, RF-11, RF-15  
  **Hecho cuando:** existe al menos un test de ejemplo y `cargo test` termina correctamente.

---

# Fase 1 — Modelo de dominio y comandos

- [x] **T-005 — Crear `LanguageId`**  
  **RF:** RF-07, RF-09, RF-15  
  **Hecho cuando:** C# y Java pueden identificarse sin usar strings dispersos por el código.

- [x] **T-006 — Crear `FrameworkId`**  
  **RF:** RF-08, RF-10, RF-15  
  **Hecho cuando:** WinForms y Swing pueden identificarse mediante un tipo común.

- [x] **T-007 — Crear `ProjectType`**  
  **RF:** RF-06, RF-15  
  **Hecho cuando:** C# + WinForms y Java + Swing pueden representarse como tipos de proyecto válidos.

- [x] **T-008 — Crear modelo `ProjectFile`**  
  **RF:** RF-05, RF-06  
  **Hecho cuando:** un archivo del proyecto puede representarse con ruta relativa y metadatos mínimos.

- [x] **T-009 — Crear modelo `Project`**  
  **RF:** RF-05, RF-06, RF-15  
  **Hecho cuando:** un proyecto puede almacenar nombre, ruta, lenguaje, framework y archivos sin depender de UI.

- [x] **T-010 — Añadir validaciones básicas de proyecto**  
  **RF:** RF-06  
  **Hecho cuando:** se rechazan nombre, ruta o combinación de plataforma inválidos mediante reglas testeadas.

- [x] **T-011 — Crear `BuildConfiguration` mínima**  
  **RF:** RF-11, RF-15  
  **Hecho cuando:** un proyecto puede expresar configuración básica de build sin conocer `dotnet` o `javac`.

- [x] **T-012 — Crear `Diagnostic`**  
  **RF:** RF-11, RF-13, RF-16  
  **Hecho cuando:** un diagnóstico puede contener nivel, mensaje y posición opcional.

- [x] **T-013 — Crear `BuildResult`**  
  **RF:** RF-11, RF-13  
  **Hecho cuando:** un resultado puede expresar éxito/fallo, código de salida, stdout, stderr y diagnósticos.

- [x] **T-014 — Crear `ProcessState` / `RunResult`**  
  **RF:** RF-12, RF-13  
  **Hecho cuando:** el sistema puede representar estado de ejecución, código de salida y salida del proceso.

- [x] **T-015 — Definir comandos principales**  
  **RF:** RF-01, RF-02, RF-03, RF-11, RF-12, RF-14  
  **Hecho cuando:** existen comandos para nuevo/abrir/cerrar proyecto, guardar, compilar, ejecutar, detener, deshacer, rehacer y buscar.

- [x] **T-016 — Probar el modelo de dominio**  
  **RF:** RF-06, RF-11, RF-12, RF-15  
  **Hecho cuando:** los tipos principales tienen unit tests para construcción, validación y estados inválidos relevantes.

---

# Fase 2 — Documento y editor

- [x] **T-017 — Crear buffer de documento mínimo**  
  **RF:** RF-02, RF-03  
  **Hecho cuando:** un documento puede almacenar y recuperar texto sin depender de la UI.

- [x] **T-018 — Implementar inserción de texto**  
  **RF:** RF-03  
  **Hecho cuando:** insertar texto modifica el documento en la posición indicada y tiene tests.

- [x] **T-019 — Implementar borrado**  
  **RF:** RF-03  
  **Hecho cuando:** borrar rangos válidos funciona y los casos de borde están cubiertos por tests.

- [x] **T-020 — Implementar cursor**  
  **RF:** RF-03  
  **Hecho cuando:** el cursor puede moverse y su posición permanece válida tras editar el documento.

- [x] **T-021 — Implementar selección**  
  **RF:** RF-03  
  **Hecho cuando:** un rango puede seleccionarse, consultarse y reemplazarse correctamente.

- [x] **T-022 — Implementar estado modified**  
  **RF:** RF-02, RF-04  
  **Hecho cuando:** cualquier modificación marca el documento y un guardado puede limpiarlo.

- [x] **T-023 — Implementar undo básico**  
  **RF:** RF-03  
  **Hecho cuando:** una edición puede deshacerse y el documento vuelve a su estado anterior.

- [x] **T-024 — Implementar redo básico**  
  **RF:** RF-03  
  **Hecho cuando:** una operación deshecha puede rehacerse correctamente.

- [x] **T-025 — Añadir búsqueda**  
  **RF:** RF-03  
  **Hecho cuando:** el editor puede localizar coincidencias dentro del documento.

- [x] **T-026 — Añadir reemplazo**  
  **RF:** RF-03  
  **Hecho cuando:** una coincidencia puede reemplazarse y existe una opción para reemplazar todas.

- [x] **T-027 — Crear modelo de pestaña abierta**  
  **RF:** RF-04  
  **Hecho cuando:** varios documentos pueden mantenerse abiertos y cada pestaña referencia un documento único.

- [x] **T-028 — Probar editor y documento**  
  **RF:** RF-02, RF-03, RF-04  
  **Hecho cuando:** existen tests para edición, cursor, selección, undo/redo, búsqueda y estado modificado.

---

# Fase 3 — Workspace y filesystem

- [x] **T-029 — Definir modelo `Workspace`**  
  **RF:** RF-05, RF-06  
  **Hecho cuando:** un workspace puede contener el proyecto activo y su raíz.

- [x] **T-030 — Implementar descubrimiento de archivos**  
  **RF:** RF-05  
  **Hecho cuando:** el proyecto puede enumerar archivos y directorios desde su raíz.

- [x] **T-031 — Implementar crear archivo**  
  **RF:** RF-06  
  **Hecho cuando:** el sistema crea un archivo dentro del workspace y actualiza el modelo.

- [x] **T-032 — Implementar crear directorio**  
  **RF:** RF-06  
  **Hecho cuando:** el sistema crea un directorio válido dentro del workspace.

- [x] **T-033 — Implementar renombrar archivo/directorio**  
  **RF:** RF-06  
  **Hecho cuando:** una entrada puede renombrarse y el modelo refleja la nueva ruta.

- [x] **T-034 — Implementar eliminar archivo/directorio**  
  **RF:** RF-06  
  **Hecho cuando:** una entrada puede eliminarse y desaparece del modelo del workspace.

- [x] **T-035 — Implementar apertura/cierre de proyecto**  
  **RF:** RF-03, RF-05, RF-06  
  **Hecho cuando:** el sistema puede abrir un proyecto existente, activar el workspace y cerrarlo limpiamente.

- [x] **T-036 — Implementar guardado de documento en filesystem**  
  **RF:** RF-02  
  **Hecho cuando:** un documento puede persistirse y recuperar exactamente su contenido.

- [x] **T-037 — Añadir tests de filesystem/workspace**  
  **RF:** RF-02, RF-05, RF-06  
  **Hecho cuando:** crear, abrir, renombrar, eliminar y guardar tienen integration tests aislados de la UI.

---

# Fase 4 — UI mínima del IDE

> **Delegada en `docs/frontend-tasks.md`.** La UI se ejecuta y se sigue allí, con
> egui/eframe como stack ya decidido (`frontend-plan.md` FD-01). Estas tareas se
> conservan por su trazabilidad de RF, no para implementarlas en paralelo: hacer
>las dos a la vez es justamente lo que duplicaba trabajo.

| Tarea de `tasks.md` | Se implementa en `frontend-tasks.md` |
|---|---|
| T-038 Ventana principal | FE-003, FE-005 |
| T-039 Menú y barra de comandos | FE-006, FE-007, FE-009 |
| T-040 Panel de proyecto | FE-010, FE-011, FE-012, FE-013 |
| T-041 Documento con editor visual | FE-019, FE-020, FE-023 |
| T-042 Pestañas visuales | FE-015, FE-016, FE-017, FE-018 |
| T-043 Acciones básicas de guardado | FE-029, FE-055, FE-073 |
| T-044 Panel de salida | FE-034, FE-035 |
| T-045 Números de línea y resaltado | FE-028, FE-081 |
| T-046 Probar el flujo UI | FE-075 |

- [ ] **T-038 — Crear ventana principal**  
  **RF:** RF-01  
  **Hecho cuando:** MiniIDE inicia y muestra una ventana principal funcional.  
  **Estado:** delegada en FE-003 y FE-005.

- [ ] **T-039 — Añadir menú y barra de comandos mínima**  
  **RF:** RF-01, RF-14  
  **Hecho cuando:** Nuevo, Abrir, Guardar, Compilar, Ejecutar y Detener son accesibles desde la UI.  
  **Estado:** delegada en FE-006, FE-007 y FE-009.

- [ ] **T-040 — Añadir panel de proyecto**  
  **RF:** RF-05  
  **Hecho cuando:** el árbol de archivos del workspace se muestra y puede seleccionarse un archivo.  
  **Estado:** delegada en FE-010 a FE-013.

- [ ] **T-041 — Conectar documento con editor visual**  
  **RF:** RF-02, RF-03, RF-04  
  **Hecho cuando:** abrir un documento muestra su contenido y las ediciones actualizan el modelo del documento.  
  **Estado:** delegada en FE-019, FE-020 y FE-023. El editor existe y lo que
  se escribe entra por el documento del core, pero todavia no hay quien le pase
  ese documento: la ventana sigue sin proyecto abierto, que es FE-058. Por eso
  "abrir un documento muestra su contenido" todavia no se puede comprobar de punta
  a punta, y esta tarea no se cierra hasta que se pueda.

- [ ] **T-042 — Implementar pestañas visuales**  
  **RF:** RF-04  
  **Hecho cuando:** el usuario puede cambiar entre documentos y cerrar pestañas sin perder el vínculo con el modelo.  
  **Estado:** delegada en FE-015 a FE-018. El concepto de documento activo es
  T-097, que es lo que desbloquea FE-017.

- [ ] **T-043 — Implementar acciones básicas de guardado**  
  **RF:** RF-02, RF-14  
  **Hecho cuando:** Guardar y Guardar todo funcionan desde menú/atajo y respetan el estado modified.  
  **Estado:** el atajo de guardar es FE-029, y el atajo pide la misma acción que el
  botón del menú, así que los dos piden `Save`. "Guardar todo" sigue pendiente: es
  FE-055 y FE-073, y el comando `SaveAll` ya existe (lo añadió T-096, que esta nota
  daba por inexistente). El atajo y el botón piden el comando, pero todavía no hay quien
  lo ejecute: el editor de la ventana es una fase posterior.

- [x] **T-044 — Implementar panel de salida**  
  **RF:** RF-13  
  **Hecho cuando:** la UI puede mostrar texto de salida y error de procesos sin bloquearse.  
  **Estado:** cerrada. El panel y la separación de flujos son FE-034 y FE-035: la
  ventana muestra la salida de un proceso línea a línea y distingue lo normal de lo
  que es error (`salida.rs`, FE-034/FE-035). La otra mitad —que la salida llegue sola
  mientras el proceso corre— la cerró T-098: el registro recoge las líneas en vivo y
  `Operaciones::salida` alimenta el panel con lo que hay en cada frame, sin bloquear.

- [x] **T-045 — Añadir números de línea y resaltado básico**  
  **RF:** RF-03, RF-07, RF-09  
  **Hecho cuando:** el editor muestra números de línea y puede distinguir al menos sintaxis básica para C# y Java.  
  **Estado:** números de línea en FE-028. El resaltado no tenía tarea en
  `frontend-tasks.md`: se ha añadido FE-081 para que esta tarea sea cerrable.  
  **Decision:** las palabras clave las dice el lenguaje, no el editor. `LanguageProvider`
  tiene `keywords()` y son sus listas las que el resaltado mira, así que añadir un
  lenguaje es añadir su lista y no tocar el editor. El editor solo sabe pintar un
  comentario, una cadena y una palabra clave; lo que es una palabra clave se lo pregunta
  a quien le pasa el lenguaje. Lo que no mira, y por eso es básico: comentarios de varias
  líneas, cadenas de varias líneas y escapes.

- [ ] **T-046 — Probar el flujo UI básico**  
  **RF:** RF-01, RF-02, RF-04, RF-05  
  **Hecho cuando:** existe un smoke test/manual checklist reproducible de iniciar, abrir, editar, guardar y cerrar un proyecto.  
  **Estado:** delegada en FE-075.

---

# Fase 5 — Abstracciones de lenguaje/framework/toolchain

- [x] **T-047 — Definir contrato de `LanguageProvider`**  
  **RF:** RF-07, RF-09, RF-15  
  **Hecho cuando:** el core puede consultar identidad, extensiones y configuración de edición mediante una interfaz común.

- [x] **T-048 — Definir contrato de `FrameworkProvider`**  
  **RF:** RF-08, RF-10, RF-15  
  **Hecho cuando:** el core puede consultar capacidades visuales y de generación mediante una interfaz común.

- [x] **T-049 — Definir contrato de `ToolchainProvider`**  
  **RF:** RF-11, RF-12, RF-16  
  **Hecho cuando:** el core puede detectar capacidades y solicitar build/run sin conocer comandos concretos.

- [x] **T-050 — Definir proveedor de generación de código**  
  **RF:** RF-08, RF-10, RF-15  
  **Hecho cuando:** el diseñador puede solicitar generación mediante un contrato independiente del lenguaje.

- [x] **T-051 — Registrar soportes iniciales**  
  **RF:** RF-07, RF-08, RF-09, RF-10, RF-15  
  **Hecho cuando:** C# + WinForms y Java + Swing aparecen como combinaciones válidas sin lógica condicional global.

- [x] **T-052 — Probar contratos de soporte**  
  **RF:** RF-15  
  **Hecho cuando:** existen tests que validan que los proveedores iniciales satisfacen las capacidades mínimas esperadas.

---

# Fase 6 — C# + WinForms

- [x] **T-053 — Implementar detección del .NET SDK**  
  **RF:** RF-07, RF-16  
  **Hecho cuando:** el IDE puede detectar disponibilidad, versión y ubicación del SDK o informar que falta.

- [x] **T-054 — Crear template C# WinForms**  
  **RF:** RF-06, RF-07, RF-08  
  **Hecho cuando:** Nuevo proyecto genera una estructura mínima válida de C# WinForms.

- [x] **T-055 — Implementar soporte de extensiones C#**  
  **RF:** RF-07  
  **Hecho cuando:** `.cs` se asocia automáticamente con C# y usa su configuración de edición.

- [x] **T-056 — Implementar build C# mediante .NET SDK**  
  **RF:** RF-07, RF-11  
  **Hecho cuando:** un proyecto C# válido puede compilar desde MiniIDE y devuelve un `BuildResult` normalizado.

- [x] **T-057 — Implementar parseo básico de errores C#**  
  **RF:** RF-11, RF-13  
  **Hecho cuando:** errores con archivo/línea/columna se transforman en `Diagnostic` cuando el formato lo permite.

- [x] **T-058 — Implementar ejecución C#**  
  **RF:** RF-07, RF-12  
  **Hecho cuando:** un proyecto C# compilable puede ejecutarse desde MiniIDE y su proceso queda registrado.

- [x] **T-059 — Implementar detención de proceso C#**  
  **RF:** RF-12, RF-13  
  **Hecho cuando:** una aplicación C# iniciada por MiniIDE puede detenerse sin cerrar el IDE.

- [x] **T-060 — Crear modelo visual WinForms mínimo**  
  **RF:** RF-08  
  **Hecho cuando:** el modelo puede representar un formulario y los controles Button, Label, TextBox y Panel.

- [x] **T-061 — Implementar selección y movimiento en diseñador WinForms**  
  **RF:** RF-08  
  **Hecho cuando:** un control puede seleccionarse, moverse y redimensionarse dentro del formulario.

- [x] **T-062 — Implementar panel de propiedades WinForms**  
  **RF:** RF-08  
  **Hecho cuando:** nombre, texto, posición, tamaño, visible y habilitado pueden editarse y reflejarse en el modelo.

- [x] **T-063 — Implementar generador WinForms mínimo**  
  **RF:** RF-08, RF-11  
  **Hecho cuando:** el modelo visual genera código C# válido para un formulario mínimo.

- [x] **T-064 — Proteger regiones de código generado WinForms**  
  **RF:** RF-08, RF-11  
  **Hecho cuando:** regenerar el diseño no elimina código manual fuera de la región administrada.

- [x] **T-065 — Integration test C# WinForms end-to-end**  
  **RF:** RF-06, RF-07, RF-08, RF-11, RF-12, RF-16  
  **Hecho cuando:** un test crea proyecto, genera UI mínima, compila y ejecuta una aplicación C# WinForms real cuando .NET está disponible.

---

# Fase 7 — Java + Swing

- [x] **T-066 — Implementar detección del JDK**  
  **RF:** RF-09, RF-16  
  **Hecho cuando:** el IDE puede detectar disponibilidad, versión y ubicación del JDK o informar que falta.

- [x] **T-067 — Crear template Java Swing**  
  **RF:** RF-06, RF-09, RF-10  
  **Hecho cuando:** Nuevo proyecto genera una estructura mínima válida de Java Swing.

- [x] **T-068 — Implementar soporte de extensiones Java**  
  **RF:** RF-09  
  **Hecho cuando:** `.java` se asocia automáticamente con Java y usa su configuración de edición.

- [x] **T-069 — Implementar build Java mediante JDK**  
  **RF:** RF-09, RF-11  
  **Hecho cuando:** un proyecto Java válido puede compilarse desde MiniIDE y devuelve un `BuildResult` normalizado.

- [x] **T-070 — Implementar parseo básico de errores Java**  
  **RF:** RF-11, RF-13  
  **Hecho cuando:** errores de compilación Java se convierten en `Diagnostic` con posición cuando sea posible.

- [x] **T-071 — Implementar ejecución Java**  
  **RF:** RF-09, RF-12  
  **Hecho cuando:** un proyecto Java compilable puede ejecutarse desde MiniIDE y su proceso queda registrado.

- [x] **T-072 — Implementar detención de proceso Java**  
  **RF:** RF-12, RF-13  
  **Hecho cuando:** una aplicación Java iniciada por MiniIDE puede detenerse sin cerrar el IDE.

- [x] **T-073 — Crear modelo visual Swing mínimo**  
  **RF:** RF-10  
  **Hecho cuando:** el modelo puede representar JFrame y los componentes JButton, JLabel, JTextField y JPanel.

- [x] **T-074 — Implementar selección y movimiento en diseñador Swing**  
  **RF:** RF-10  
  **Hecho cuando:** un componente puede seleccionarse, moverse y redimensionarse dentro de la ventana.

- [x] **T-075 — Implementar panel de propiedades Swing**  
  **RF:** RF-10  
  **Hecho cuando:** las propiedades básicas soportadas pueden editarse y reflejarse en el modelo.

- [x] **T-076 — Implementar generador Swing mínimo**  
  **RF:** RF-10, RF-11  
  **Hecho cuando:** el modelo visual genera código Java Swing válido para una ventana mínima.

- [x] **T-077 — Proteger regiones de código generado Swing**  
  **RF:** RF-10, RF-11  
  **Hecho cuando:** regenerar el diseño no elimina código manual fuera de la región administrada.

- [x] **T-078 — Integration test Java Swing end-to-end**  
  **RF:** RF-06, RF-09, RF-10, RF-11, RF-12, RF-16  
  **Hecho cuando:** un test crea proyecto, genera UI mínima, compila y ejecuta una aplicación Java Swing real cuando el JDK está disponible.

---

# Fase 8 — Diagnósticos, salida y robustez

- [x] **T-079 — Centralizar salida de procesos**  
  **RF:** RF-12, RF-13  
  **Hecho cuando:** build/run entregan stdout, stderr y código de salida mediante una estructura común.

- [x] **T-080 — Mejorar panel de diagnósticos**  
  **RF:** RF-11, RF-13, RF-16  
  **Hecho cuando:** el usuario puede ver errores y advertencias agrupados y acceder al archivo/línea cuando existe.

- [x] **T-081 — Manejar toolchain ausente**  
  **RF:** RF-11, RF-16  
  **Hecho cuando:** intentar compilar sin .NET/JDK produce un diagnóstico claro y el IDE permanece operativo.

- [x] **T-082 — Aislar tareas largas de la UI**  
  **RF:** RF-11, RF-12, RF-13  
  **Hecho cuando:** compilar o ejecutar no congela la ventana y el resultado llega de forma asíncrona.

- [x] **T-083 — Añadir cancelación/detención robusta**  
  **RF:** RF-12  
  **Hecho cuando:** un proceso activo puede detenerse y el estado del IDE vuelve correctamente a inactivo.

- [x] **T-084 — Añadir manejo de errores inesperados**  
  **RF:** RF-01, RF-11, RF-12, RF-16  
  **Hecho cuando:** fallos de filesystem, proceso o toolchain no cierran inesperadamente el IDE.

---

# Fase 9 — Puente con el frontend

> Lo que la UI necesita del core y todavía no existe. Está aquí, y no dentro de
> `frontend-tasks.md`, porque es código del core y porque mientras falte, las
> tareas FE correspondientes están bloqueadas. Ninguna de estas tareas es de UI.

| Tarea de `tasks.md` | Desbloquea en `frontend-tasks.md` |
|---|---|
| T-095 Movimiento del cursor en `Document` | FE-023, FE-024, FE-025, FE-030 |
| T-096 Comandos que la UI necesita | FE-013, FE-014, FE-018, FE-029, FE-033, FE-055 |
| T-097 Documento activo | FE-016, FE-017, FE-070 |
| T-098 Procesos sin bloquear | FE-034, FE-035, FE-038, FE-039, FE-043, FE-074 |

- [x] **T-095 — Exponer el movimiento del cursor en `Document`**  
  **RF:** RF-03  
  **Hecho cuando:** `Document` puede mover el cursor a izquierda, derecha, arriba y abajo usando la lógica de `Cursor`, sin que la UI tenga que duplicarla.  
  **Nota:** el movimiento existe en `Cursor`, pero `Document` solo expone `move_to`
  con posicion absoluta. Sin esto, FE-025 no se puede hacer sin copiar la lógica
  de salto de linea en el frontend, que `plan.md` §2.4 prohibe.  
  **Decision:** moverse descarta la selección. Mover no es alargar, y alargar es lo que
  hace `extend_to`, que es a donde va la selección mientras se pulsa Mayusculas. Si
  moverse dejara la selección, cada flecha movería el final de lo seleccionado.
  Moverse tampoco marca el documento: es lo que distingue mover de editar, y si no,
  un paseo con las flechas dejaría el archivo con cambios sin guardar.

- [x] **T-096 — Completar los comandos que la UI necesita**  
  **RF:** RF-02, RF-04, RF-05, RF-14  
  **Hecho cuando:** existen `OpenDocument`, `CloseDocument`, `SaveAll`, `NewFile`,
  `NewDirectory` y `Replace` en `Command`, con sus resultados y sus tests.  
  **Nota:** `Command` tiene hoy `NewProject`, `OpenProject`, `CloseProject`,
  `Save`, `Build`, `Run`, `Stop`, `Undo`, `Redo` y `Find`. FE-009 pide que un
  boton y un menú usen el mismo comando, y no pueden si el comando no existe.  
  **Despues:** `ActivateDocument` lo añadio T-097, que es la tarea que sabe
  cuál es el documento activo.

- [x] **T-097 — Añadir documento activo al conjunto de pestañas**  
  **RF:** RF-04  
  **Hecho cuando:** `OpenTabs` sabe cuál es la pestaña activa, se puede cambiar y
  se define qué pasa con el indice al cerrar la activa.  
  **Nota:** T-027 lo dejo fuera a proposito por ser politica. FE-017 lo necesita y
  la eleccion de que hacer al cerrar la activa es de las que hay que decidir
  explicitamente, no por defecto.  
  **Decision:** abrir un documento deja activa su pestaña, y abrir uno que ya
  esta abierto lo trae a delante sin crear una segunda. Al cerrar la activa pasa
  a estarlo la que ocupa su sitio, y si era la ultima, la nueva ultima; con no
  quedar ninguna no hay activa. Se decidio en el core y no en la ventana porque
  `OpenTabs::close` ya devuelve la pestaña para que quien la cierre pueda mirar
  si tenia cambios sin guardar, y quien decide eso tiene que ser el mismo. La
  ventana repite la misma regla en su lista de pestañas - FE-015 - y lleva el
  mismo nombre en el test para que las dos se cambien a la vez.  
  **Nota:** para que la ventana pueda pedirlo, `Command` tiene
  `ActivateDocument`, que va con los de documentos porque es de los que ha ido
  pidiendo la interfaz, como ellos (T-096).

- [x] **T-098 — Ejecutar procesos sin bloquear la interfaz**  
  **RF:** RF-11, RF-12, RF-13  
  **Hecho cuando:** un proceso externo se lanza en segundo plano, se capturan
  stdout y stderr mientras corre, se puede detener y su estado se puede consultar
  sin bloquear.  
  **Estado:** `ProcessRegistry` en `runtime.rs` lanza el proceso con `spawn` —sin
  esperar— y lo registra. La captura en vivo son dos hilos lectores, uno por flujo
  (`Stdio::piped()` + `BufReader::lines()`), que mandan cada línea por un canal con
  su origen (`OutputStream` y `OutputLine`). `RunningProcess::read_output` recoge lo
  que ya ha llegado con `try_recv` —sin bloquear, que es lo que FE-043 exige— y lo
  acumula en la salida; `result` espera a los lectores al terminar para no perder la
  última línea. El panel de salida de la ventana (`Operaciones::salida`, FE-034 y
  FE-035) se alimenta de ahí, así que la salida llega mientras corre y no solo al
  final. Tests: unitarios del core (líneas, flujos y acumulación) y de la ventana,
  y pruebas ignoradas que lanzan un proceso real, en `runtime.rs` y
  `frontend::operaciones`.

---

# Fase 10 — Tests de regresión y hardening

- [x] **T-085 — Añadir tests de regresión de documentos**  
  **RF:** RF-02, RF-03, RF-04  
  **Hecho cuando:** bugs corregidos del editor quedan cubiertos por tests reproducibles.  
  **Estado:** `tests/document_regression.rs`, 18 casos. El repositorio no tenía un
  registro de bugs, así que los casos se derivaron de las reglas ya escritas en
  `specs.md` y `plan.md` y de los unitarios existentes: dónde puede y no puede estar
  el cursor, qué cuenta como un paso de undo, qué pasa al borrar entre líneas, que
  `replace_all` termina cuando el reemplazo contiene lo buscado, y el ciclo de
  guardar, cerrar, reabrir y volver a abrir. No hizo falta cambiar el core.

- [x] **T-086 — Añadir tests de regresión de proyectos**  
  **RF:** RF-05, RF-06  
  **Hecho cuando:** creación, apertura, renombrado y cierre de proyecto tienen casos de regresión cubiertos.  
  **Estado:** `tests/project_regression.rs`, 16 casos, sobre el ciclo completo de
  proyecto en `Project` y `Workspace`: crear, abrir, renombrar, cerrar y reabrir,
  con las negativas (directorio ocupado, no-proyecto, WPF, nombre que pisaría otro,
  raíz relativa) y el cierre del espacio de trabajo. Tampoco hizo falta cambiar el
  core: lo que se corrigió al escribirlos eran expectativas del propio test.

- [x] **T-087 — Añadir snapshot/golden tests de código WinForms**  
  **RF:** RF-08  
  **Hecho cuando:** los outputs esperados del generador WinForms están versionados y un cambio inesperado rompe el test.  
  **Estado:** los tres casos de WinForms de `tests/golden_generation.rs` comparan
  contra `tests/golden/winforms/*.txt`. `.gitattributes` fija `eol=lf` en
  `tests/golden/**` para que el `autocrlf` de Windows no cambie la referencia.  
  **Bug:** al versionar el golden se vio que el generador escribía las banderas entre
  comillas (`this.class.Visible = "false";`), que no compila. Se arregló en
  `src/generation/winforms.rs` (`BOOLEAN_PROPERTIES` y `property_line`) y los
  unitarios que fijaban el texto entrecomillado se corrigieron con él.

- [x] **T-088 — Añadir snapshot/golden tests de código Swing**  
  **RF:** RF-10  
  **Hecho cuando:** los outputs esperados del generador Swing están versionados y un cambio inesperado rompe el test.  
  **Estado:** los tres casos de Swing de `tests/golden_generation.rs` contra
  `tests/golden/swing/*.txt`. El generador de Swing no tenía el bug de las banderas;
  el golden lo deja fijado.

- [x] **T-089 — Ejecutar smoke test C# completo**  
  **RF:** RF-06, RF-07, RF-08, RF-11, RF-12, RF-13, RF-16  
  **Hecho cuando:** el flujo crear → editar → diseñar → compilar → ejecutar funciona sin intervención manual inesperada.  
  **Estado:** `tests/winforms_end_to_end.rs` hace el flujo entero —crear, editar el
  archivo del diseñador y el que el usuario conserva, diseñar, compilar, ejecutar y
  detener—. El caso `#[ignore]` pasó con el SDK de .NET 10.0.301 y su runtime de
  Windows Forms: el proceso se quedó vivo y se detuvo sin cerrar el IDE.

- [x] **T-090 — Ejecutar smoke test Java completo**  
  **RF:** RF-06, RF-09, RF-10, RF-11, RF-12, RF-13, RF-16  
  **Hecho cuando:** el flujo crear → editar → diseñar → compilar → ejecutar funciona sin intervención manual inesperada.  
  **Estado:** igual que T-089, en `tests/java_swing_end_to_end.rs`. El caso
  `#[ignore]` pasó con el JDK 25.0.3: compiló, arrancó y se detuvo.

- [x] **T-091 — Revisar dependencias**  
  **RF:** —  
  **Hecho cuando:** no existen dependencias no utilizadas y cada dependencia externa relevante tiene una razón clara.  
  **Estado:** las tres dependencias declaradas se usan: `eframe` en
  `src/frontend/`, `image` solo en `src/frontend/icon.rs` para el logo, y
  `winresource` en `build.rs` para el icono del ejecutable. `tests/quality_policy.rs`
  añade el guard: cada dependencia de `Cargo.toml` se usa en el crate y lleva su
  razón escrita al lado, que la nombra.

- [x] **T-092 — Revisar acoplamiento del core**  
  **RF:** RF-15  
  **Hecho cuando:** el core no contiene lógica específica duplicada de C#, Java, WinForms o Swing fuera de sus proveedores.  
  **Estado:** revisados los módulos del core. Los nombres de lenguaje, framework y
  tipo de proyecto son vocabulario del dominio (`LanguageId`, `FrameworkId`,
  `ProjectType`), y la detección de archivos de proyecto está una sola vez, en
  `project.rs`. La lógica concreta (generación, componentes, toolchains) vive en sus
  proveedores. `tests/core_boundaries.rs` lo fija: ningún módulo del core, fuera de
  `generation/`, `framework/`, `language/`, `toolchain/`, `templates.rs`,
  `supports.rs` y el frontend, menciona `System.Windows.Forms`, `javax.swing`, un
  tipo de control concreto ni el nombre de una herramienta.

- [x] **T-093 — Ejecutar suite completa**  
  **RF:** Todos los RF del MVP  
  **Hecho cuando:** `cargo test`, `cargo fmt --check`, `cargo clippy` y los smoke/integration tests disponibles terminan correctamente.  
  **Estado:** `cargo fmt --all --check` limpio, `cargo clippy --all-targets -- -D
  warnings` limpio, `cargo test` sin fallos en ningún target, y `cargo test --
  --ignored` con los 29 casos que dependen de herramientas y entorno, todos en verde.

- [x] **T-094 — Verificar definición de terminado del MVP**  
  **RF:** Todos los RF del MVP  
  **Hecho cuando:** se puede demostrar cada flujo principal del MVP y no quedan requisitos funcionales sin trazabilidad a una tarea completada.  
  **Estado:** cada requisito tiene tarea y prueba que lo demuestra: RF-01
  `tests/frontend_integration.rs`; RF-02/03/04 `tests/document_regression.rs` y
  `tests/editor_flow.rs`; RF-05/06 `tests/project_regression.rs` y
  `tests/filesystem.rs`; RF-07/08 `tests/golden_generation.rs` y
  `tests/winforms_end_to_end.rs`; RF-09/10 `tests/golden_generation.rs`,
  `tests/java_swing_end_to_end.rs` y `tests/jdk_detection.rs`; RF-11/12/13/16 los
  smoke tests anteriores y `tests/jdk_detection.rs`; RF-14 los unitarios de
  `commands.rs`; RF-15 `tests/module_boundaries.rs` y `tests/core_boundaries.rs`.  
  **Hallazgo:** la trazabilidad está completa, pero **la definición de terminado no
  se puede cerrar del todo**: en la fase 9, **T-098** sigue sin hacer (el runtime
  solo tiene los tipos de resultado y no lanza nada) y de ella dependen T-043, T-044
  y T-046, que siguen abiertas por eso. Compilar y ejecutar se demuestran en los
  smoke tests, pero la salida en vivo que pide `AGENTS.md` §14 todavía no está en el
  core. Es lo único que separa al MVP de su definición de terminado.

---

# Orden de dependencia resumido

```text
Fase 0
  ↓
Fase 1 — Dominio
  ↓
Fase 2 — Documento/Editor
  ↓
Fase 3 — Workspace/Filesystem
  ↓
Fase 5 — Abstracciones
  ↓
Fase 6 — C# / WinForms
  ↓
Fase 7 — Java / Swing
  ↓
Fase 8 — Robustez
  ↓
Fase 9 — Puente con el frontend
  ↓
Fase 4 — UI (docs/frontend-tasks.md, stack egui/eframe ya decidido)
  ↓
Fase 10 — Hardening
```

La Fase 4 se ejecuta desde `docs/frontend-tasks.md` y va antes del hardening
final, pero despues de todo el core: la UI no tiene nada que mostrar hasta que
existen el proyecto, los documentos, las toolchains y los generadores.

## Regla de precedencia entre `tasks.md` y `frontend-tasks.md`

- El core es `tasks.md`. La UI es `frontend-tasks.md`. No se implementa una tarea
  FE sin que sus prerequisitos de `tasks.md` esten `[x]`.
- Egui/eframe esta ya decidido en `frontend-plan.md` FD-01. No se vuelve a
  abrir la decision de stack dentro de una tarea.
- `frontend-tasks.md` no anade logica de negocio, toolchains ni generacion: si
  una tarea FE necesita algo que el core no tiene, es una tarea de `tasks.md`
  primero.

# Criterio global de tarea completada

Una tarea marcada como terminada debe cumplir simultáneamente:

- Implementación mínima necesaria.
- RF indicado satisfecho o avance verificable del RF.
- `Hecho cuando:` comprobado.
- Tests relevantes en verde.
- Sin introducir funcionalidad fuera del alcance.
- Sin romper las tareas previamente completadas.
