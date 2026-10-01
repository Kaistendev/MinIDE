# MiniIDE — Tasks de implementación del Frontend (egui/eframe)

> Todas las tareas están diseñadas para durar menos de 30 minutos. Si una tarea crece, dividirla antes de continuar.

## Cómo se trabaja este documento

- **La UI es de aquí.** `docs/tasks.md` lleva el core y su Fase 4 (T-038 a
  T-046) está delegada en este documento; mapping tarea a tarea en la tabla de
  la Fase 4 de `tasks.md`.
- **El stack ya está decidido:** egui/eframe, en `frontend-plan.md` FD-01. No
  se vuelve a abrir dentro de una tarea.
- **Antes de empezar una tarea FE, sus prerequisitos de `tasks.md` tienen que
  estar `[x]`.** Los que hoy bloquean algo están en la Fase 9 de `tasks.md`
  (T-095 a T-098) y son: movimiento del cursor en `Document`, los comandos que
  faltan, el documento activo y lanzar procesos sin bloquear.
- **Lo que falte en el core no se resuelve aquí.** Si una tarea FE necesita una
  capacidad que el core no tiene, se añade como tarea en `tasks.md` y se sigue
  por el core. El frontend no implementa toolchains, generación de código ni
  lógica de documento.
- **La numeración de fases de este documento es propia** y no coincide con la de
  `plan.md` ni con la de `tasks.md`. Las ids `FE-xxx` sí son globales.

## Fase 0 — Preparación

- [x] **FE-001 — Fijar stack del frontend**
  - RF: RNF-01, RNF-02
  - Definir egui/eframe como stack oficial del frontend.
  - Hecho cuando: la documentación del frontend declara egui/eframe y no existe otro toolkit de UI para el MVP.

- [x] **FE-002 — Crear módulo de frontend**
  - RF: RF-01, RNF-02
  - Crear el módulo/crate/carpeta destinado a la UI según la estructura real del proyecto.
  - Hecho cuando: el proyecto compila y el frontend tiene un punto de entrada separado del core.

- [x] **FE-003 — Crear aplicación mínima eframe**
  - RF: RF-01
  - Crear la ventana mínima de eframe.
  - Hecho cuando: MiniIDE abre una ventana y puede cerrarse sin error.

- [x] **FE-004 — Definir `UiState` mínimo**
  - RF: RF-01, RNF-08
  - Separar estado visual de estado de dominio.
  - Hecho cuando: existe un tipo de estado UI que no contiene el contenido completo del proyecto ni la lógica de negocio.

## Fase 1 — Ventana principal

- [x] **FE-005 — Crear layout raíz**
  - RF: RF-01
  - Definir menú, área central, panel inferior y barra de estado.
  - Hecho cuando: la ventana muestra claramente esas cuatro zonas.

- [x] **FE-006 — Crear menú principal**
  - RF: RF-01, RF-14
  - Añadir File/Edit/View/Build o equivalente mínimo.
  - Hecho cuando: cada menú se abre y contiene acciones placeholder sin duplicar comandos.

- [x] **FE-007 — Crear barra de herramientas mínima**
  - RF: RF-14
  - Añadir botones para abrir, guardar, build, run y stop.
  - Hecho cuando: cada botón dispara o registra el comando correspondiente.

- [x] **FE-008 — Crear barra de estado**
  - RF: RF-13
  - Mostrar estado general y documento actual.
  - Hecho cuando: la barra refleja al menos proyecto/documento/estado de operación.

- [x] **FE-009 — Definir ciclo `UI → Command → Core`**
  - RF: RF-14, RNF-02, RNF-08
  - Introducir el punto único para emitir comandos desde la UI.
  - Hecho cuando: un botón y un menú pueden invocar la misma operación sin duplicar lógica.

## Fase 2 — Project Explorer

- [x] **FE-010 — Crear `ProjectView`**
  - RF: RF-05
  - Crear panel izquierdo para el proyecto.
  - Hecho cuando: existe un panel dedicado que puede renderizar un árbol vacío.

- [x] **FE-011 — Renderizar árbol de directorios**
  - RF: RF-05
  - Mostrar carpetas y archivos desde un modelo proporcionado por el core.
  - Hecho cuando: un proyecto de prueba puede visualizarse jerárquicamente.

- [x] **FE-012 — Expandir/contraer carpetas**
  - RF: RF-05
  - Añadir estado visual para expansión.
  - Hecho cuando: el usuario puede expandir y contraer cualquier carpeta visible.

- [x] **FE-013 — Seleccionar archivo**
  - RF: RF-05, RF-07, RF-09
  - Emitir `OpenDocument` al seleccionar un archivo editable.
  - Hecho cuando: seleccionar un archivo produce exactamente un comando de apertura.

- [x] **FE-014 — Menú contextual básico del proyecto**
  - RF: RF-05, RF-06
  - Preparar acciones Nuevo archivo/Nuevo directorio.
  - Hecho cuando: el menú contextual aparece y dispara los comandos correspondientes.

## Fase 3 — Tabs y documentos

- [x] **FE-015 — Crear modelo visual de tabs**
  - RF: RF-04
  - Añadir colección de pestañas visuales asociadas a documentos del core.
  - Hecho cuando: múltiples documentos pueden representarse sin copiar su contenido en `UiState`.

- [x] **FE-016 — Renderizar tabs**
  - RF: RF-04
  - Mostrar nombre y estado modificado.
  - Hecho cuando: cada documento abierto tiene una pestaña identificable y el modificado se distingue.

- [x] **FE-017 — Cambiar documento activo**
  - RF: RF-04
  - Emitir cambio de documento activo.
  - Hecho cuando: hacer click en otra pestaña cambia el documento mostrado.

- [x] **FE-018 — Cerrar tab**
  - RF: RF-04, RF-09
  - Implementar cierre visual y delegación al core.
  - Hecho cuando: una pestaña puede cerrarse y los documentos modificados no se descartan silenciosamente.

## Fase 4 — Editor base

- [x] **FE-019 — Crear `EditorView`**
  - RF: RF-03
  - Crear el área central del editor.
  - Hecho cuando: existe un editor visible con un documento de prueba.

- [x] **FE-020 — Renderizar texto**
  - RF: RF-03
  - Mostrar contenido proveniente del `Document` del core.
  - Hecho cuando: el texto del modelo aparece completo y con scroll básico.

- [x] **FE-021 — Cursor visual**
  - RF: RF-03
  - Dibujar el cursor según la posición del modelo.
  - Hecho cuando: el cursor se muestra en la posición indicada por el core.

- [x] **FE-022 — Selección visual básica**
  - RF: RF-03
  - Representar rango seleccionado.
  - Hecho cuando: una selección del modelo puede visualizarse de principio a fin.

- [x] **FE-023 — Entrada de teclado**
  - RF: RF-03
  - Convertir entrada de texto en comandos de edición.
  - Hecho cuando: una pulsación inserta texto mediante el core y no mediante un buffer independiente del frontend.

- [x] **FE-024 — Backspace/Delete**
  - RF: RF-03
  - Mapear teclas de eliminación a comandos del core.
  - Hecho cuando: borrar un carácter modifica el documento real y actualiza la vista.

- [x] **FE-025 — Navegación del cursor**
  - RF: RF-03
  - Integrar izquierda/derecha/arriba/abajo.
  - Hecho cuando: las cuatro direcciones actualizan el cursor del core y la UI lo refleja.

- [x] **FE-026 — Scroll vertical**
  - RF: RF-03, RNF-04
  - Añadir scroll al documento.
  - Hecho cuando: documentos de varias pantallas pueden recorrerse sin bloquear la UI.

- [x] **FE-027 — Scroll horizontal básico**
  - RF: RF-03
  - Permitir visualizar líneas largas.
  - Hecho cuando: una línea que supera el viewport puede recorrerse horizontalmente.

- [x] **FE-028 — Números de línea**
  - RF: RF-03
  - Mostrar números de línea en un gutter.
  - Hecho cuando: cada línea visible presenta su número correcto.

- [x] **FE-081 — Resaltado de sintaxis básico**
  - RF: RF-03, RF-07, RF-09
  - Colorear palabras clave, comentarios, cadenas y números según el lenguaje.
  - Hecho cuando: un archivo `.cs` y un `.java` se distinguen visualmente al
    menos en comentarios, cadenas y palabras clave.
  - Nota: task añadida al mapear con `tasks.md`. La T-045 pedía "resaltado
    básico" y no había ninguna tarea FE que lo cubriera, así que T-045 no era
    cerrable. El identificador es FE-081 y no sigue la numeración del resto de
    la fase porque se añadió más tarde.

## Fase 5 — Atajos y comandos de edición

- [x] **FE-029 — Ctrl+S**
  - RF: RF-02, RF-14
  - Conectar atajo con `Save`.
  - Hecho cuando: Ctrl+S ejecuta exactamente el mismo comando que el menú Guardar.

- [x] **FE-030 — Ctrl+Z / Ctrl+Y**
  - RF: RF-03
  - Conectar undo/redo.
  - Hecho cuando: los atajos llaman al core y el documento refleja el resultado.

- [x] **FE-031 — Copiar/cortar/pegar**
  - RF: RF-03
  - Integrar clipboard con el modelo del editor.
  - Hecho cuando: copiar/cortar/pegar funciona sobre una selección real.

- [x] **FE-032 — Ctrl+F**
  - RF: RF-03, RF-14
  - Mostrar búsqueda.
  - Hecho cuando: Ctrl+F abre la UI de búsqueda y puede enviar la consulta al core.

- [x] **FE-033 — Ctrl+H**
  - RF: RF-03, RF-14
  - Mostrar búsqueda/reemplazo.
  - Hecho cuando: Ctrl+H abre la UI y puede solicitar reemplazo al core.

## Fase 6 — Integración del output y diagnósticos

- [x] **FE-034 — Crear `OutputView`**
  - RF: RF-13
  - Crear panel inferior para salida de procesos.
  - Hecho cuando: el panel puede mostrar una secuencia de líneas recibidas del core.

- [x] **FE-035 — Mostrar stdout/stderr**
  - RF: RF-13
  - Diferenciar visualmente salida normal y error sin mezclar la fuente del dato.
  - Hecho cuando: una prueba de proceso muestra ambos flujos completos.

- [x] **FE-036 — Crear `DiagnosticsView`**
  - RF: RF-13
  - Mostrar lista de diagnósticos.
  - Hecho cuando: un diagnóstico con archivo/línea/mensaje aparece en la lista.

- [x] **FE-037 — Seleccionar diagnóstico**
  - RF: RF-13
  - Permitir navegar al archivo/línea cuando exista posición.
  - Hecho cuando: seleccionar un diagnóstico con ubicación solicita la navegación correspondiente.

## Fase 7 — Build / Run / Stop en UI

- [x] **FE-038 — Estado de Build**
  - RF: RF-11, RF-13
  - Mostrar Idle/Building/Success/Failed.
  - Hecho cuando: una operación de build actualiza el estado correctamente.
  - Nota: los cuatro estados viven en `src/frontend/operaciones.rs` y se enseñan en su
    propio campo de la barra de estado. Decide el core (`BuildResult::succeeded`, que mira
    el código de salida) y la ventana solo lo traduce.

- [x] **FE-039 — Estado de Run**
  - RF: RF-12, RF-13
  - Mostrar Idle/Running/Stopping/Exited.
  - Hecho cuando: el estado refleja el proceso real.
  - Nota: el estado sale de `ProcessState` y no de lo que se pulsó. `Deteniendo` existe
    porque pedir la parada no es haber parado, y solo se pasa a `Terminado` cuando el
    sistema deja de ver el proceso.

- [x] **FE-040 — Conectar Build**
  - RF: RF-11
  - Conectar menú, toolbar y atajo Ctrl+B.
  - Hecho cuando: las tres superficies disparan el mismo comando Build.
  - Nota: las tres piden la misma `Accion`, `acciones::COMPILAR`, y `App` la ejecuta una
    sola vez por pulsación.

- [x] **FE-041 — Conectar Run**
  - RF: RF-12
  - Conectar F5 y botón Ejecutar.
  - Hecho cuando: ambas superficies disparan el mismo comando Run.

- [x] **FE-042 — Conectar Stop**
  - RF: RF-12
  - Conectar Shift+F5 y botón Detener.
  - Hecho cuando: ambas superficies disparan el mismo comando Stop.

- [x] **FE-043 — Evitar bloqueo de UI durante build/run**
  - RF: RF-11, RF-12, RNF-05
  - Validar visualmente e integrar estados de trabajo asíncrono.
  - Hecho cuando: durante una compilación o ejecución el editor sigue aceptando interacción básica.
  - Nota: la ventana ejecuta los comandos en `App::avanzar`, que es `eframe::App::logic`.
    Compilar arranca en otro hilo con `build::start_build` y se recoge con `try_recv`;
    ejecutar usa `runtime::ProcessRegistry`, que además es lo que permite parar de verdad.
    Mientras hay trabajo en marcha se pide otro repintado, porque egui solo dibuja cuando
    le llega algo.
  - Lo que sigue sin cerrar: T-098 sigue abierta porque la salida del proceso se recoge al
    terminar y no mientras corre, que es lo que le falta.

## Fase 8 — Diseñador visual

- [x] **FE-044 — Crear `DesignerView`**
  - RF: RF-08, RF-10
  - Crear área central del diseñador.
  - Hecho cuando: un `DesignerModel` vacío puede renderizarse como canvas.
  - Nota: `src/frontend/diseniador.rs`. Es la segunda vista del área central, junto al
    editor: cuál de las dos se ve lo dice `UiState::vista_central`. Abrir un diseñador de
    verdad es FE-059 y FE-063.

- [x] **FE-045 — Renderizar formulario**
  - RF: RF-08, RF-10
  - Dibujar ventana/formulario base desde el modelo.
  - Hecho cuando: el tamaño del formulario coincide con el modelo.
  - Nota: un punto del modelo es un píxel del canvas. Sin esa equivalencia, un formulario
    pegado al borde del panel podría no caber, y lo que se ve dejaría de ser lo que se
    genera.

- [x] **FE-046 — Renderizar controles**
  - RF: RF-08, RF-10
  - Dibujar componentes básicos.
  - Hecho cuando: Button/Label/TextBox/Panel o equivalentes visibles aparecen según el modelo.
  - Nota: se dibujan en el orden del modelo y el último queda encima. Es el mismo criterio
    que usa el golpe de selección, así que lo que se ve y lo que se puede pulsar coinciden.

- [x] **FE-047 — Seleccionar control**
  - RF: RF-08, RF-10
  - Seleccionar un componente con click.
  - Hecho cuando: solo el control clicado queda seleccionado y el modelo de selección se actualiza.
  - Nota: la selección la guarda `Diseniador`, no el `DesignerModel`. El modelo es lo que se
    convierte en código y a la hora de generar da igual qué control esté resaltado.

- [x] **FE-048 — Mover control**
  - RF: RF-08, RF-10
  - Convertir drag en comando de movimiento.
  - Hecho cuando: mover un control cambia su posición en el `DesignerModel`.
  - Nota: el drag se convierte en `diseniador::Comando::Mover` y entra por `App::pedir`. Un
    control no se puede dejar fuera del formulario, porque el canvas enseña el formulario y
    no lo que hay más allá.

- [x] **FE-049 — Redimensionar control**
  - RF: RF-08, RF-10
  - Añadir handles mínimos de resize.
  - Hecho cuando: el tamaño del control cambia en el modelo al arrastrar un handle.
  - Nota: cuatro esquinas, que es lo que "handles mínimos" pide. El tamaño no baja del
    mínimo porque un control sin tamaño no se puede agarrar.

- [x] **FE-050 — Añadir control desde toolbox**
  - RF: RF-08, RF-10
  - Crear toolbox mínima.
  - Hecho cuando: seleccionar un control del toolbox y colocarlo crea un componente en el modelo.
  - Nota: los controles del toolbox los pone quien abre el diseñador, con los que declara su
    framework. Aquí no hay lista fija porque eso sería escribir "Button" o "JButton" en la
    ventana; FE-057 y FE-061 son las que la rellenan.

- [x] **FE-051 — Crear `PropertiesView`**
  - RF: RF-08, RF-10
  - Mostrar propiedades del elemento seleccionado.
  - Hecho cuando: seleccionar un control muestra nombre, texto, posición y tamaño.
  - Nota: `src/frontend/propiedades.rs`, en una columna a la derecha. Se enseñan todas las
    propiedades del control y no solo la del texto porque su nombre depende del framework:
    `Text` en WinForms y `text` en Swing.

- [x] **FE-052 — Editar propiedades**
  - RF: RF-08, RF-10
  - Convertir cambios de propiedades en comandos.
  - Hecho cuando: modificar una propiedad actualiza el modelo y el canvas.
  - Nota: el panel no guarda copia de ningún valor: manda un `diseniador::Comando` por
    `App::pedir`, el mismo camino que el canvas. Las posiciones y los tamaños se cambian
    con `DragValue` y el nombre solo al perder el foco, porque mientras se teclea un nombre
    casi nunca es un nombre válido.

## Fase 9 — Menús contextuales y UX mínima

- [x] **FE-053 — Menú contextual del editor**
  - RF: RF-03
  - Añadir acciones básicas de edición.
  - Hecho cuando: click derecho ofrece acciones pertinentes y ejecuta los mismos comandos que el menú principal.
  - Nota: `editor::menu_contextual`, sobre el área del editor y no sobre la ventana entera,
    para que el clic derecho en el explorador no abra los dos menús. La lista es la misma
    que la del menú Editar y está escrita con las mismas `Accion`, así que un clic pide lo
    mismo en los dos sitios.

- [x] **FE-054 — Menú contextual del diseñador**
  - RF: RF-08, RF-10
  - Añadir duplicar/eliminar si esas operaciones existen en el core.
  - Hecho cuando: las acciones disponibles no duplican lógica en la UI.
  - Nota: solo hay eliminar. `DesignerModel` quita controles y no los duplica, así que un
    botón de duplicar sería un botón que acepta el clic y no hace nada. Eliminar es un
    `diseniador::Comando::Borrar` más, por el mismo `App::pedir` que el resto de los gestos.

- [x] **FE-055 — Diálogo de confirmación al cerrar documento modificado**
  - RF: RF-02, RF-04
  - Implementar modal Guardar/Descartar/Cancelar.
  - Hecho cuando: cerrar un documento modificado siempre requiere una decisión explícita.
  - Nota: `src/frontend/dialogos.rs`. La pregunta la hace `tabs` al pulsar la cruz, antes de
    cerrar nada, y la contesta el diálogo: Guardar emite `Save` y luego `CloseDocument`,
    Descartar solo `CloseDocument`, y Cancelar no hace nada. Lo que el diálogo no hace es
    guardar por su cuenta, porque entonces habría dos caminos para guardar y no habría forma
    de saber cuál falló.

- [x] **FE-056 — Diálogo de errores de toolchain**
  - RF: RF-16
  - Mostrar dependencia faltante de forma clara.
  - Hecho cuando: un .NET SDK/JDK ausente genera un mensaje útil y no un panic en UI.
  - Nota: antes de compilar o ejecutar se pregunta al proveedor si está disponible. El
    mensaje es su `missing_message()`, no uno escrito aquí: `.NET SDK` y `JDK` se nombran en
    la toolchain, no en la ventana. Con la herramienta ausente no se lanza nada y se avisa;
    sin el aviso, el fallo llegaría como un error de compilación cualquiera, que es lo que
    RF-16 dice que no tiene que pasar.

## Fase 10 — Integración C# WinForms

- [x] **FE-057 — Detectar proyecto WinForms en UI**
  - RF: RF-06, RF-07, RF-08
  - Seleccionar correctamente editor/designer disponibles.
  - Hecho cuando: abrir un proyecto C# WinForms habilita sus vistas correspondientes.
  - Nota: `src/frontend/vistas.rs`. `Vistas` se calcula al abrir el proyecto preguntando al
    framework que lo declara —`Supports::framework(project_type.framework())`— y no mirando
    el lenguaje, así que el módulo no menciona ninguna tecnología y añadir un framework es
    añadir su soporte en `supports.rs`. Da dos respuestas: si hay vista de diseño —un
    framework sin generación de código no la tiene— y qué controles ofrece su toolbox, con
    el nombre que tienen en ese framework. Hay un test que lee el código del módulo y falla
    si aparece el nombre de un lenguaje o de un framework (AGENTS.md §2.4).

- [x] **FE-058 — Abrir `.cs` en editor**
  - RF: RF-07
  - Conectar Project Explorer con EditorView.
  - Hecho cuando: un `.cs` se abre en una pestaña y el contenido es editable.
  - Nota: `App` guarda los documentos con `OpenTabs` del core y los archivos del proyecto con
    `discover_files`, y `ejecutar_peticiones` resuelve `OpenDocument`, `ActivateDocument`,
    `Save` y `CloseDocument`: los cuatro hacen falta para que abrir un documento de verdad no
    deje la ventana y el core diciendo cosas distintas, y el diálogo de cerrar de FE-055
    depende de los dos últimos. El archivo se busca en la lista del proyecto y no en lo que
    se ha pulsado, porque un comando no lleva datos. Guardar y cerrar actuán sobre el
    documento que el *core* tiene activo, que no siempre es el que la ventana tiene
    marcado: el diálogo quita la pestaña y pide guardar en el mismo clic, y cuando se ejecuta
    la ventana ya está enseñando la de al lado.
  - Nota: al abrir un documento de verdad aparecieron dos fallos del editor que sus propios
    tests no veían porque siempre lo dibujaban en un hueco con su esquina en cero. `ui.painter()`
    pinta en coordenadas de la ventana y el editor pintaba en las suyas, así que el texto
    salía en la esquina de arriba, encima del menú. Y egui suelta el foco de todo widget que
    no se ha dibujado en el frame, así que el editor pedía el foco sin registrarse como
    widget y lo perdía en el frame siguiente: de cada dos pulsaciones de teclado solo llegaba
    una. Los dos están corregidos y cubiertos por `tests/winforms_frontend.rs`, que escribe
    en un proyecto de verdad.

- [x] **FE-059 — Abrir diseñador WinForms**
  - RF: RF-08
  - Detectar recurso diseñable.
  - Hecho cuando: el usuario puede cambiar de Code a Designer para un formulario WinForms.
  - Nota: el recurso diseñable se busca por el marcador con el que los generadores del core
    cierran la zona que escriben —`generation::MARKER_BEGIN`—, que es lo mismo para todos los
    frameworks: un archivo con esa zona tiene un diseño detrás y ningún otro lo tiene, diga su
    framework lo que diga. Se busca una vez, al abrir el proyecto, y se para en el primero.
    El interruptor `Code` / `Designer` se dibuja encima del área central y solo cuando se
    puede pasar de una vista a otra: framework con diseñador, diseño en el proyecto y una
    clase abierta a la que pertenezca ese diseño —FE-063 precisa este último punto—. Volver a
    pulsarlo con el diseñador ya delante no rehace el modelo, que es lo que perdería los
    controles colocados.
  - Lo que sigue sin cerrar: el modelo se abre vacío. El core tiene el generador pero no un
    lector que convierta un archivo generado en modelo, así que todavía no se recupera el
    diseño que ya había escrito. Cargar el diseño del archivo es capacidad del core y queda
    como tarea en `tasks.md`, no aquí.

- [x] **FE-060 — Mostrar errores de compilación C#**
  - RF: RF-07, RF-11, RF-13
  - Presentar diagnósticos de .NET en la UI.
  - Hecho cuando: un error de compilación aparece con mensaje y ubicación si existe.
  - Nota: `Operaciones` guarda los diagnósticos de la última compilación y los borra al
    pedir la siguiente, porque son los de la anterior y el código ya puede haber cambiado.
    `aplicar_el_resultado` es un paso aparte de `recoger` para que lo que el core dice se
    pueda comprobar sin lanzar el SDK de .NET, que es lo que hacen sus tests y los del
    layout. Pulsar un diagnóstico marca su fila en el explorador y pide `OpenDocument`: el
    comando no lleva datos, así que el destino se dice marcando la fila, con la misma clave
    que usa el explorador. Saltar a la línea es FE-072.

## Fase 11 — Integración Java Swing

- [x] **FE-061 — Detectar proyecto Swing en UI**
  - RF: RF-06, RF-09, RF-10
  - Habilitar vistas de código/diseñador apropiadas.
  - Hecho cuando: abrir un proyecto Java Swing muestra soporte de Java/Swing.
  - Nota: no hizo falta código nuevo para esto, y esa es la comprobación de FE-057. Lo mismo
    que habilita las vistas de un proyecto de C# las habilita las de uno de Java: `Vistas` se
    calcula con `Supports`, sin mirar el lenguaje, así que Swing trae `JButton`, `JLabel` y
    compañía en su toolbox y no los controles del otro framework. El árbol sí es distinto —
    los fuentes están en `src/main/java`, tres carpetas más abajo— y se llega a ellos
    desplegando. Todo está en `vistas.rs`, en `app.rs` y en `tests/java_swing_frontend.rs`.

- [x] **FE-062 — Abrir `.java` en editor**
  - RF: RF-09
  - Conectar Project Explorer con EditorView.
  - Hecho cuando: un `.java` se abre y es editable.
  - Nota: mismo camino que FE-058 y por lo mismo no hay un comando por lenguaje: el explorador
    marca la fila, pide `OpenDocument` y la ventana abre el archivo con el lenguaje que el core
    le da por su extensión. Lo que se comprueba aquí es lo propio de Java —que el archivo se
    abre con su ruta entera dentro del proyecto, que se puede escribir y que lo escrito llega
    al archivo— porque una ruta de tres niveles no es lo mismo que un archivo en la raíz.

- [x] **FE-063 — Abrir diseñador Swing**
  - RF: RF-10
  - Detectar recurso diseñable.
  - Hecho cuando: una ventana Swing puede abrirse en Designer.
  - Nota: aquí el diseño vive en el mismo archivo que el código —no hay un archivo generado al
    lado como en el otro framework—, y eso obliga a precisar lo que era FE-059. `puede_diseñar`
    ya no basta con que el proyecto tenga un diseño: tiene que ser **el diseño de la clase que
    se está viendo**, comparando el nombre del archivo del diseño con el de la clase. Con
    cualquier otra clase —`Main.java`, la que lanza la aplicación— el interruptor no aparece, y
    con el criterio anterior sí y habría dejado diseñar un formulario de una clase que no es
    ventana. Sigue sin nombres de framework: el archivo del diseño es el de la clase o el que
    se generó al lado.
  - Lo que sigue sin cerrar: igual que en FE-059, el modelo se abre vacío porque el core no
    tiene un lector que convierta un archivo generado en modelo.

- [x] **FE-064 — Mostrar errores de compilación Java**
  - RF: RF-09, RF-11, RF-13
  - Presentar diagnósticos de javac/JDK.
  - Hecho cuando: un error Java aparece con mensaje y ubicación si existe.
  - Nota: la salida de `javac` entra por el mismo camino que la de `dotnet` —FE-060— porque los
    diagnósticos los da el proveedor de la toolchain y la ventana no los distingue. Los tests
    parsean con `JdkToolchain::parse_diagnostics` y una salida real de `javac`, así que
    comprueban el parser del core y la ventana a la vez sin necesitar el JDK instalado.

## Fase 12 — Accesibilidad y estabilidad mínima

- [x] **FE-065 — Revisar foco de teclado**
  - RF: RF-03, RNF-11
  - Revisar foco entre editor, árbol, propiedades y output.
  - Hecho cuando: Tab/Shift+Tab y click permiten recuperar foco de cada área sin comportamientos erráticos.
  - Nota: la revisión encontró un fallo real: el editor pedía el foco cada vez que el ratón
    estaba encima, así que cualquier clic en un botón —en la barra, en el menú, en una fila del
    árbol— se perdía en cuanto el puntero volvía al editor, y con él la tecla que iba a
    activar ese botón. Ahora el foco se pide **con un clic dentro del editor**, no solo con pasar
    por encima, y el editor se registra como widget enfocable para que egui no se lo suelte
    (FE-058). Con eso, Tab mueve el foco a otra zona y lo que se escriba después ya no entra en
    el documento, y un clic dentro del editor lo recupera sin cerrar nada. Es lo que comprueban
    `escribir_solo_escribe_mientras_el_editor_tiene_el_foco` y
    `el_tab_mueve_el_foco_y_el_texto_deja_de_llegar_al_editor`.
  - Nota: los tests de las fases 10 y 11 escribían con el ratón encima sin clicar. Se
    cambiaron para clicar antes, que es lo que hace un usuario, y así los tests de escritura
    cubren también el foco.

- [x] **FE-066 — Revisar resize de paneles**
  - RF: RF-11, RNF-11
  - Validar que los paneles soportan tamaños razonables.
  - Hecho cuando: cambiar tamaño de ventana no oculta definitivamente áreas esenciales.
  - Nota: en el tamaño mínimo que declara la ventana (640×400) siguen viéndose el menú, la
    barra de herramientas, el explorador, las propiedades, el editor y la barra de estado: se
    comprueba con lo que hay escrito en cada zona, que es lo que el usuario ve, y no con los
    rectángulos de los paneles —el área central no pinta fondo, así que buscarla por su
    rectángulo no la encuentra—.
  - Nota: la revisión encontró el segundo fallo real: una línea más ancha que la zona se
    pintaba entera y se metía encima de la columna de propiedades. El editor pinta ahora
    recortado a su hueco, que es lo que deja el desplazamiento horizontal sin cambios: lo que
    no cabe no se ve, y con la rueda se ve el resto.

- [x] **FE-067 — Evitar panic por estado de UI incompleto**
  - RF: RF-01, RNF-06
  - Revisar `Option`/estados de transición.
  - Hecho cuando: abrir/cerrar proyecto/documento mientras no hay selección no provoca panic en pruebas relevantes.
  - Nota: los cuatro comandos de documento —abrir, activar, guardar y cerrar— se ejecutan
    contra una ventana sin proyecto, sin selección y sin documentos; se abre un documento que
    no está en el proyecto; se cambia de proyecto con documentos del anterior abiertos; se
    escribe con todos los documentos cerrados; se abre un proyecto sin archivos; y se pide el
    diseñador sin ningún documento. Ninguno se cae, y los que no pueden hacer su cosa dicen
    por qué en la barra en vez de perder la petición en silencio, que es lo que hace parecer un
    IDE que no responde.

- [x] **FE-068 — Revisar mensajes vacíos/error**
  - RF: RF-13, RF-16
  - Asegurar estados claros para idle, sin proyecto, sin archivo y toolchain ausente.
  - Hecho cuando: cada estado básico tiene una representación visible comprensible.
  - Nota: la barra decía "Sin proyecto" y "Sin documento" siempre, porque en FE-005 todavía no
    había forma de saber qué había abierto. Desde FE-058 y FE-062 sí lo hay, así que ahora dice
    el nombre del proyecto y el del documento que se está viendo, y solo dice "Sin proyecto" y
    "Sin documento" cuando no hay. Los cuatro estados básicos —nada abierto, con proyecto, con
    documento y con la herramienta que falta— tienen cada uno su texto, comprobados en
    `status.rs` y mirando la barra de verdad en `layout.rs`.

## Fase 13 — Tests del frontend

- [x] **FE-069 — Tests de mapeo evento → comando**
  - RF: RF-14
  - Probar atajos/acciones principales.
  - Hecho cuando: los tests verifican que cada interacción produce el comando esperado.
  - Nota: cada atajo de `ATAJOS` tiene su test en `frontend/atajos.rs` con pulsaciones de
    verdad, y cada acción de `frontend/acciones.rs` se comprueba con un clic que acaba
    pidiendo su comando.

- [x] **FE-070 — Tests de estado de tabs**
  - RF: RF-04
  - Probar selección/cierre/estado modificado.
  - Hecho cuando: los casos principales de tabs están cubiertos automáticamente.
  - Nota: cubren la pestaña que se marca, el asterisco al escribir, el cierre con diálogo y el
    descarte en `frontend/tabs.rs`, `frontend/dialogos.rs` y `frontend/layout.rs`. Al
    escribirlos apareció un fallo real: el área interactiva del editor arrancaba en
    `ui.min_rect().min`, tapaba la tira de pestañas y se comía los clics de las etiquetas y de
    la `×`. Ahora empieza en `ui.cursor().min`.

- [x] **FE-071 — Tests de estados Build/Run**
  - RF: RF-11, RF-12
  - Probar transiciones de UI ante resultados del core.
  - Hecho cuando: los estados Idle/Running/Failed/Exited tienen cobertura.
  - Nota: los cuatro estados de compilación y los cuatro de ejecución se recorren uno detrás de
    otro mirando la barra de estado de verdad, en `frontend/layout.rs`. Los de ejecución lanzan
    un proceso real y van marcados con `#[ignore]`, como los del core.

- [x] **FE-072 — Tests de navegación desde diagnóstico**
  - RF: RF-13
  - Probar mapping de archivo/línea a comando de navegación.
  - Hecho cuando: un diagnóstico con ubicación produce el destino esperado.
  - Nota: al pulsar un diagnóstico con ubicación se anota el destino —archivo y línea— en el
    estado visual y se pide abrir el documento; `App` lo consume al abrirlo y mueve el cursor
    a esa línea. Está en `frontend/diagnosticos.rs` (el destino) y en `frontend/app.rs` (el
    cursor, incluso con el archivo ya abierto y sin tocar otro documento).

- [x] **FE-073 — Integration test UI → Core para Save**
  - RF: RF-02, RF-14
  - Verificar que la UI usa el mismo comando de guardado que el core.
  - Hecho cuando: el test confirma persistencia sin depender de un botón concreto.
  - Nota: en `tests/frontend_integration.rs`. Se guarda con Ctrl+S y con el comando, nunca
    pulsando el botón, y lo que se mira es el archivo del disco: es el mismo camino para el
    atajo, el menú y el botón porque los tres piden `Command::Save`.

- [x] **FE-074 — Integration test UI → Core para Build/Run/Stop**
  - RF: RF-11, RF-12, RF-14
  - Verificar comandos y actualización de estados.
  - Hecho cuando: el flujo completo pasa sin bloqueo de UI.
  - Nota: en `tests/frontend_integration.rs`, pulsando los botones de la barra y mirando lo que
    se ve en ella. El flujo completo usa un toolchain de prueba que lanza `cmd`, porque sin
    lanzar un proceso no se puede recorrer el camino entero, y va marcado con `#[ignore]`. El
    caso de que la ventana no se congela no necesita procesos y corre siempre.

- [x] **FE-075 — Smoke test del frontend**
  - RF: RF-01 a RF-16
  - Ejecutar flujo principal mínimo.
  - Hecho cuando: Start → Open/Create → Edit → Save → Build → Run → Stop es reproducible sin fallos conocidos.
  - Nota: `el_flujo_principal_se_puede_repetir_de_principio_a_fin` recorre el flujo entero con
    un proyecto de verdad, abre un archivo del árbol, escribe, guarda con el atajo, compila,
    ejecuta y para, y comprueba que el archivo del disco tiene lo escrito y que la ventana
    sigue en pie. Va marcado con `#[ignore]` porque compilar y ejecutar lanzan procesos de
    verdad.

## Fase 14 — Pulido MVP

- [x] **FE-076 — Revisar navegación y nomenclatura**
  - RF: RNF-11, RNF-12
  - Unificar textos y nombres de comandos visibles.
  - Hecho cuando: acciones equivalentes usan el mismo nombre en menú, toolbar y mensajes.
  - Nota: los menús y la barra ya compartían las mismas `Accion`; lo que se unifica aquí es
    lo que estaba escrito a mano. El botón de guardar del diálogo de cerrar y el texto del
    campo de búsqueda salen ahora de `GUARDAR.nombre` y `BUSCAR.nombre`, y se ha quitado el
    `nombre()` de `VistaCentral`, que decía "Editor" y "Diseñador" mientras el conmutador dice
    "Code" y "Designer". `tests/frontend_boundaries.rs` lo comprueba: ningún módulo del
    frontend puede escribir un nombre de acción por su cuenta. Los títulos de los menús se
    dejan fuera porque son nombres de sección y no de acción.

- [x] **FE-077 — Reducir UI innecesaria**
  - RF: RNF-11
  - Eliminar controles o paneles sin función MVP.
  - Hecho cuando: cada elemento visible tiene una función definida.
  - Nota: el menú Editar ofrecía copiar, cortar y pegar, y el menú Ver tenía "mostrar panel
    de proyecto" y "mostrar panel de propiedades"; los cinco estaban apagados porque el core
    no tiene esos comandos, y los dos paneles siempre están en la ventana. Se han quitado el
    menú Ver entero y las tres acciones del portapapeles, también del menú contextual del
    editor -copiar, cortar y pegar siguen funcionando con las teclas del sistema, que es de
    donde los trae egui-. `Accion` ya no puede llevar un comando que no existe, así que un
    botón apagado no puede volver a entrar ni por descuido.

- [x] **FE-078 — Revisar rendimiento visual básico**
  - RF: RF-03, RNF-04, RNF-05
  - Revisar editor, output y designer con contenido de prueba razonable.
  - Hecho cuando: ninguna vista principal provoca congelación visible durante interacción normal.
  - Nota: el editor ya pintaba solo las líneas que caben, y ahora está comprobado con un
    archivo de veinte mil líneas. La salida no: pintaba un widget por línea, así que una
    compilación de cinco mil líneas medía cinco mil textos en cada frame. Ahora usa un scroll
    area que solo construye las líneas visibles y que se queda pegado al final, que es donde
    el proceso falla y lo dice. El diseñador no se puede virtualizar -es un lienzo con
    posiciones absolutas-, pero sí se ha comprobado que mil controles cuestan dos o tres
    veces lo mismo que cuatro.

- [x] **FE-079 — Revisión final de arquitectura frontend**
  - RF: RNF-02, RNF-03, RNF-08
  - Verificar separación UI/core.
  - Hecho cuando: ningún widget contiene lógica de build, toolchain o generación de código específica que debería pertenecer al core/proveedor.
  - Nota: la ventana pide y enseña; compilar, ejecutar, arrancar procesos y escribir archivos
    son de `operaciones` y del core. `tests/frontend_boundaries.rs` lo deja anotado para que
    no se deshaga: ningún módulo del frontend lanza procesos ni nombra una herramienta,
    solo `app` y `operaciones` conocen el build y el toolchain, solo `operaciones` arranca una
    compilación y decide si ha ido bien, y nadie escribe archivos desde la ventana.

- [x] **FE-080 — Cerrar frontend MVP**
  - RF: RF-01 a RF-16
  - Ejecutar checklist final y documentación.
  - Hecho cuando: las funciones frontend previstas están operativas, los tests pasan y no hay tareas fuera de alcance mezcladas en el MVP.
  - Nota: el checklist de cierre está al final de este documento. La suite completa pasa, con
    los tests que lanzan procesos de verdad marcados con `#[ignore]` y ejecutados aparte.

## Cierre del frontend MVP

Los diez criterios de aceptación de `docs/frontend-plan.md` §18, con dónde se mira cada uno:

| Criterio | Dónde |
| --- | --- |
| Ventana estable con eframe | `app::tests::the_window_content_can_be_drawn_without_a_window` y `frontend_integration.rs`, que escribe en la ventana |
| Menú, explorador, editor, output y estado | `layout::tests::la_ventana_aguanta_el_tamano_minimo_sin_que_desaparezca_nada` |
| Crear y abrir proyecto | `explorador::tests` para el árbol y `app::tests` para abrir, en `tests/winforms_frontend.rs` y `tests/java_swing_frontend.rs` |
| Abrir y editar un documento | `explorador::tests::seleccionar_un_archivo_pide_abrirlo_exactamente_una_vez` y `editor::tests` |
| Guardar desde comando, menú y atajo | `frontend_integration.rs::guardar_dos_veces_por_caminos_distintos_llega_al_mismo_core` y `acciones::tests` |
| C# WinForms desde la UI | `tests/winforms_frontend.rs` y `tests/winforms_end_to_end.rs` |
| Java Swing desde la UI | `tests/java_swing_frontend.rs` y `tests/java_swing_end_to_end.rs` |
| Diseñador con componentes básicos | `diseniador::tests` y `propiedades::tests` |
| Build, Run y Stop sin congelar | `frontend_integration.rs::la_ventana_se_sigue_dibujando_mientras_compila` y `operaciones::tests` |
| Diagnósticos en su panel | `diagnosticos::tests` y `layout::tests::los_errores_de_la_compilacion_se_ensenan_abajo_con_donde_estan` |

Fuera de alcance: el frontend no ha crecido más allá de las fases 1 a 14. Lo que el MVP no
prevía -docking, más de dos vistas, refactorización avanzada, atajos configurables- sigue sin
estar, y los dos menús y los botones que sí están son los que el plan pide.

Con esto el frontend queda cerrado. Lo que viene después -todo lo anterior al core, que ya
existe- no se toca desde aquí: `AGENTS.md` §9 dice que un cambio grande para resolver una tarea
pequeña no es un cambio de arquitectura, es otra cosa.
