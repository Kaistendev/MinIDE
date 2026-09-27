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

- [ ] **FE-001 — Fijar stack del frontend**
  - RF: RNF-01, RNF-02
  - Definir egui/eframe como stack oficial del frontend.
  - Hecho cuando: la documentación del frontend declara egui/eframe y no existe otro toolkit de UI para el MVP.

- [ ] **FE-002 — Crear módulo de frontend**
  - RF: RF-01, RNF-02
  - Crear el módulo/crate/carpeta destinado a la UI según la estructura real del proyecto.
  - Hecho cuando: el proyecto compila y el frontend tiene un punto de entrada separado del core.

- [ ] **FE-003 — Crear aplicación mínima eframe**
  - RF: RF-01
  - Crear la ventana mínima de eframe.
  - Hecho cuando: MiniIDE abre una ventana y puede cerrarse sin error.

- [ ] **FE-004 — Definir `UiState` mínimo**
  - RF: RF-01, RNF-08
  - Separar estado visual de estado de dominio.
  - Hecho cuando: existe un tipo de estado UI que no contiene el contenido completo del proyecto ni la lógica de negocio.

## Fase 1 — Ventana principal

- [ ] **FE-005 — Crear layout raíz**
  - RF: RF-01
  - Definir menú, área central, panel inferior y barra de estado.
  - Hecho cuando: la ventana muestra claramente esas cuatro zonas.

- [ ] **FE-006 — Crear menú principal**
  - RF: RF-01, RF-14
  - Añadir File/Edit/View/Build o equivalente mínimo.
  - Hecho cuando: cada menú se abre y contiene acciones placeholder sin duplicar comandos.

- [ ] **FE-007 — Crear barra de herramientas mínima**
  - RF: RF-14
  - Añadir botones para abrir, guardar, build, run y stop.
  - Hecho cuando: cada botón dispara o registra el comando correspondiente.

- [ ] **FE-008 — Crear barra de estado**
  - RF: RF-13
  - Mostrar estado general y documento actual.
  - Hecho cuando: la barra refleja al menos proyecto/documento/estado de operación.

- [ ] **FE-009 — Definir ciclo `UI → Command → Core`**
  - RF: RF-14, RNF-02, RNF-08
  - Introducir el punto único para emitir comandos desde la UI.
  - Hecho cuando: un botón y un menú pueden invocar la misma operación sin duplicar lógica.

## Fase 2 — Project Explorer

- [ ] **FE-010 — Crear `ProjectView`**
  - RF: RF-05
  - Crear panel izquierdo para el proyecto.
  - Hecho cuando: existe un panel dedicado que puede renderizar un árbol vacío.

- [ ] **FE-011 — Renderizar árbol de directorios**
  - RF: RF-05
  - Mostrar carpetas y archivos desde un modelo proporcionado por el core.
  - Hecho cuando: un proyecto de prueba puede visualizarse jerárquicamente.

- [ ] **FE-012 — Expandir/contraer carpetas**
  - RF: RF-05
  - Añadir estado visual para expansión.
  - Hecho cuando: el usuario puede expandir y contraer cualquier carpeta visible.

- [ ] **FE-013 — Seleccionar archivo**
  - RF: RF-05, RF-07, RF-09
  - Emitir `OpenDocument` al seleccionar un archivo editable.
  - Hecho cuando: seleccionar un archivo produce exactamente un comando de apertura.

- [ ] **FE-014 — Menú contextual básico del proyecto**
  - RF: RF-05, RF-06
  - Preparar acciones Nuevo archivo/Nuevo directorio.
  - Hecho cuando: el menú contextual aparece y dispara los comandos correspondientes.

## Fase 3 — Tabs y documentos

- [ ] **FE-015 — Crear modelo visual de tabs**
  - RF: RF-04
  - Añadir colección de pestañas visuales asociadas a documentos del core.
  - Hecho cuando: múltiples documentos pueden representarse sin copiar su contenido en `UiState`.

- [ ] **FE-016 — Renderizar tabs**
  - RF: RF-04
  - Mostrar nombre y estado modificado.
  - Hecho cuando: cada documento abierto tiene una pestaña identificable y el modificado se distingue.

- [ ] **FE-017 — Cambiar documento activo**
  - RF: RF-04
  - Emitir cambio de documento activo.
  - Hecho cuando: hacer click en otra pestaña cambia el documento mostrado.

- [ ] **FE-018 — Cerrar tab**
  - RF: RF-04, RF-09
  - Implementar cierre visual y delegación al core.
  - Hecho cuando: una pestaña puede cerrarse y los documentos modificados no se descartan silenciosamente.

## Fase 4 — Editor base

- [ ] **FE-019 — Crear `EditorView`**
  - RF: RF-03
  - Crear el área central del editor.
  - Hecho cuando: existe un editor visible con un documento de prueba.

- [ ] **FE-020 — Renderizar texto**
  - RF: RF-03
  - Mostrar contenido proveniente del `Document` del core.
  - Hecho cuando: el texto del modelo aparece completo y con scroll básico.

- [ ] **FE-021 — Cursor visual**
  - RF: RF-03
  - Dibujar el cursor según la posición del modelo.
  - Hecho cuando: el cursor se muestra en la posición indicada por el core.

- [ ] **FE-022 — Selección visual básica**
  - RF: RF-03
  - Representar rango seleccionado.
  - Hecho cuando: una selección del modelo puede visualizarse de principio a fin.

- [ ] **FE-023 — Entrada de teclado**
  - RF: RF-03
  - Convertir entrada de texto en comandos de edición.
  - Hecho cuando: una pulsación inserta texto mediante el core y no mediante un buffer independiente del frontend.

- [ ] **FE-024 — Backspace/Delete**
  - RF: RF-03
  - Mapear teclas de eliminación a comandos del core.
  - Hecho cuando: borrar un carácter modifica el documento real y actualiza la vista.

- [ ] **FE-025 — Navegación del cursor**
  - RF: RF-03
  - Integrar izquierda/derecha/arriba/abajo.
  - Hecho cuando: las cuatro direcciones actualizan el cursor del core y la UI lo refleja.

- [ ] **FE-026 — Scroll vertical**
  - RF: RF-03, RNF-04
  - Añadir scroll al documento.
  - Hecho cuando: documentos de varias pantallas pueden recorrerse sin bloquear la UI.

- [ ] **FE-027 — Scroll horizontal básico**
  - RF: RF-03
  - Permitir visualizar líneas largas.
  - Hecho cuando: una línea que supera el viewport puede recorrerse horizontalmente.

- [ ] **FE-028 — Números de línea**
  - RF: RF-03
  - Mostrar números de línea en un gutter.
  - Hecho cuando: cada línea visible presenta su número correcto.

- [ ] **FE-081 — Resaltado de sintaxis básico**
  - RF: RF-03, RF-07, RF-09
  - Colorear palabras clave, comentarios, cadenas y números según el lenguaje.
  - Hecho cuando: un archivo `.cs` y un `.java` se distinguen visualmente al
    menos en comentarios, cadenas y palabras clave.
  - Nota: task añadida al mapear con `tasks.md`. La T-045 pedía "resaltado
    básico" y no había ninguna tarea FE que lo cubriera, así que T-045 no era
    cerrable. El identificador es FE-081 y no sigue la numeración del resto de
    la fase porque se añadió más tarde.

## Fase 5 — Atajos y comandos de edición

- [ ] **FE-029 — Ctrl+S**
  - RF: RF-02, RF-14
  - Conectar atajo con `Save`.
  - Hecho cuando: Ctrl+S ejecuta exactamente el mismo comando que el menú Guardar.

- [ ] **FE-030 — Ctrl+Z / Ctrl+Y**
  - RF: RF-03
  - Conectar undo/redo.
  - Hecho cuando: los atajos llaman al core y el documento refleja el resultado.

- [ ] **FE-031 — Copiar/cortar/pegar**
  - RF: RF-03
  - Integrar clipboard con el modelo del editor.
  - Hecho cuando: copiar/cortar/pegar funciona sobre una selección real.

- [ ] **FE-032 — Ctrl+F**
  - RF: RF-03, RF-14
  - Mostrar búsqueda.
  - Hecho cuando: Ctrl+F abre la UI de búsqueda y puede enviar la consulta al core.

- [ ] **FE-033 — Ctrl+H**
  - RF: RF-03, RF-14
  - Mostrar búsqueda/reemplazo.
  - Hecho cuando: Ctrl+H abre la UI y puede solicitar reemplazo al core.

## Fase 6 — Integración del output y diagnósticos

- [ ] **FE-034 — Crear `OutputView`**
  - RF: RF-13
  - Crear panel inferior para salida de procesos.
  - Hecho cuando: el panel puede mostrar una secuencia de líneas recibidas del core.

- [ ] **FE-035 — Mostrar stdout/stderr**
  - RF: RF-13
  - Diferenciar visualmente salida normal y error sin mezclar la fuente del dato.
  - Hecho cuando: una prueba de proceso muestra ambos flujos completos.

- [ ] **FE-036 — Crear `DiagnosticsView`**
  - RF: RF-13
  - Mostrar lista de diagnósticos.
  - Hecho cuando: un diagnóstico con archivo/línea/mensaje aparece en la lista.

- [ ] **FE-037 — Seleccionar diagnóstico**
  - RF: RF-13
  - Permitir navegar al archivo/línea cuando exista posición.
  - Hecho cuando: seleccionar un diagnóstico con ubicación solicita la navegación correspondiente.

## Fase 7 — Build / Run / Stop en UI

- [ ] **FE-038 — Estado de Build**
  - RF: RF-11, RF-13
  - Mostrar Idle/Building/Success/Failed.
  - Hecho cuando: una operación de build actualiza el estado correctamente.

- [ ] **FE-039 — Estado de Run**
  - RF: RF-12, RF-13
  - Mostrar Idle/Running/Stopping/Exited.
  - Hecho cuando: el estado refleja el proceso real.

- [ ] **FE-040 — Conectar Build**
  - RF: RF-11
  - Conectar menú, toolbar y atajo Ctrl+B.
  - Hecho cuando: las tres superficies disparan el mismo comando Build.

- [ ] **FE-041 — Conectar Run**
  - RF: RF-12
  - Conectar F5 y botón Ejecutar.
  - Hecho cuando: ambas superficies disparan el mismo comando Run.

- [ ] **FE-042 — Conectar Stop**
  - RF: RF-12
  - Conectar Shift+F5 y botón Detener.
  - Hecho cuando: ambas superficies disparan el mismo comando Stop.

- [ ] **FE-043 — Evitar bloqueo de UI durante build/run**
  - RF: RF-11, RF-12, RNF-05
  - Validar visualmente e integrar estados de trabajo asíncrono.
  - Hecho cuando: durante una compilación o ejecución el editor sigue aceptando interacción básica.

## Fase 8 — Diseñador visual

- [ ] **FE-044 — Crear `DesignerView`**
  - RF: RF-08, RF-10
  - Crear área central del diseñador.
  - Hecho cuando: un `DesignerModel` vacío puede renderizarse como canvas.

- [ ] **FE-045 — Renderizar formulario**
  - RF: RF-08, RF-10
  - Dibujar ventana/formulario base desde el modelo.
  - Hecho cuando: el tamaño del formulario coincide con el modelo.

- [ ] **FE-046 — Renderizar controles**
  - RF: RF-08, RF-10
  - Dibujar componentes básicos.
  - Hecho cuando: Button/Label/TextBox/Panel o equivalentes visibles aparecen según el modelo.

- [ ] **FE-047 — Seleccionar control**
  - RF: RF-08, RF-10
  - Seleccionar un componente con click.
  - Hecho cuando: solo el control clicado queda seleccionado y el modelo de selección se actualiza.

- [ ] **FE-048 — Mover control**
  - RF: RF-08, RF-10
  - Convertir drag en comando de movimiento.
  - Hecho cuando: mover un control cambia su posición en el `DesignerModel`.

- [ ] **FE-049 — Redimensionar control**
  - RF: RF-08, RF-10
  - Añadir handles mínimos de resize.
  - Hecho cuando: el tamaño del control cambia en el modelo al arrastrar un handle.

- [ ] **FE-050 — Añadir control desde toolbox**
  - RF: RF-08, RF-10
  - Crear toolbox mínima.
  - Hecho cuando: seleccionar un control del toolbox y colocarlo crea un componente en el modelo.

- [ ] **FE-051 — Crear `PropertiesView`**
  - RF: RF-08, RF-10
  - Mostrar propiedades del elemento seleccionado.
  - Hecho cuando: seleccionar un control muestra nombre, texto, posición y tamaño.

- [ ] **FE-052 — Editar propiedades**
  - RF: RF-08, RF-10
  - Convertir cambios de propiedades en comandos.
  - Hecho cuando: modificar una propiedad actualiza el modelo y el canvas.

## Fase 9 — Menús contextuales y UX mínima

- [ ] **FE-053 — Menú contextual del editor**
  - RF: RF-03
  - Añadir acciones básicas de edición.
  - Hecho cuando: click derecho ofrece acciones pertinentes y ejecuta los mismos comandos que el menú principal.

- [ ] **FE-054 — Menú contextual del diseñador**
  - RF: RF-08, RF-10
  - Añadir duplicar/eliminar si esas operaciones existen en el core.
  - Hecho cuando: las acciones disponibles no duplican lógica en la UI.

- [ ] **FE-055 — Diálogo de confirmación al cerrar documento modificado**
  - RF: RF-02, RF-04
  - Implementar modal Guardar/Descartar/Cancelar.
  - Hecho cuando: cerrar un documento modificado siempre requiere una decisión explícita.

- [ ] **FE-056 — Diálogo de errores de toolchain**
  - RF: RF-16
  - Mostrar dependencia faltante de forma clara.
  - Hecho cuando: un .NET SDK/JDK ausente genera un mensaje útil y no un panic en UI.

## Fase 10 — Integración C# WinForms

- [ ] **FE-057 — Detectar proyecto WinForms en UI**
  - RF: RF-06, RF-07, RF-08
  - Seleccionar correctamente editor/designer disponibles.
  - Hecho cuando: abrir un proyecto C# WinForms habilita sus vistas correspondientes.

- [ ] **FE-058 — Abrir `.cs` en editor**
  - RF: RF-07
  - Conectar Project Explorer con EditorView.
  - Hecho cuando: un `.cs` se abre en una pestaña y el contenido es editable.

- [ ] **FE-059 — Abrir diseñador WinForms**
  - RF: RF-08
  - Detectar recurso diseñable.
  - Hecho cuando: el usuario puede cambiar de Code a Designer para un formulario WinForms.

- [ ] **FE-060 — Mostrar errores de compilación C#**
  - RF: RF-07, RF-11, RF-13
  - Presentar diagnósticos de .NET en la UI.
  - Hecho cuando: un error de compilación aparece con mensaje y ubicación si existe.

## Fase 11 — Integración Java Swing

- [ ] **FE-061 — Detectar proyecto Swing en UI**
  - RF: RF-06, RF-09, RF-10
  - Habilitar vistas de código/diseñador apropiadas.
  - Hecho cuando: abrir un proyecto Java Swing muestra soporte de Java/Swing.

- [ ] **FE-062 — Abrir `.java` en editor**
  - RF: RF-09
  - Conectar Project Explorer con EditorView.
  - Hecho cuando: un `.java` se abre y es editable.

- [ ] **FE-063 — Abrir diseñador Swing**
  - RF: RF-10
  - Detectar recurso diseñable.
  - Hecho cuando: una ventana Swing puede abrirse en Designer.

- [ ] **FE-064 — Mostrar errores de compilación Java**
  - RF: RF-09, RF-11, RF-13
  - Presentar diagnósticos de javac/JDK.
  - Hecho cuando: un error Java aparece con mensaje y ubicación si existe.

## Fase 12 — Accesibilidad y estabilidad mínima

- [ ] **FE-065 — Revisar foco de teclado**
  - RF: RF-03, RNF-11
  - Revisar foco entre editor, árbol, propiedades y output.
  - Hecho cuando: Tab/Shift+Tab y click permiten recuperar foco de cada área sin comportamientos erráticos.

- [ ] **FE-066 — Revisar resize de paneles**
  - RF: RF-11, RNF-11
  - Validar que los paneles soportan tamaños razonables.
  - Hecho cuando: cambiar tamaño de ventana no oculta definitivamente áreas esenciales.

- [ ] **FE-067 — Evitar panic por estado de UI incompleto**
  - RF: RF-01, RNF-06
  - Revisar `Option`/estados de transición.
  - Hecho cuando: abrir/cerrar proyecto/documento mientras no hay selección no provoca panic en pruebas relevantes.

- [ ] **FE-068 — Revisar mensajes vacíos/error**
  - RF: RF-13, RF-16
  - Asegurar estados claros para idle, sin proyecto, sin archivo y toolchain ausente.
  - Hecho cuando: cada estado básico tiene una representación visible comprensible.

## Fase 13 — Tests del frontend

- [ ] **FE-069 — Tests de mapeo evento → comando**
  - RF: RF-14
  - Probar atajos/acciones principales.
  - Hecho cuando: los tests verifican que cada interacción produce el comando esperado.

- [ ] **FE-070 — Tests de estado de tabs**
  - RF: RF-04
  - Probar selección/cierre/estado modificado.
  - Hecho cuando: los casos principales de tabs están cubiertos automáticamente.

- [ ] **FE-071 — Tests de estados Build/Run**
  - RF: RF-11, RF-12
  - Probar transiciones de UI ante resultados del core.
  - Hecho cuando: los estados Idle/Running/Failed/Exited tienen cobertura.

- [ ] **FE-072 — Tests de navegación desde diagnóstico**
  - RF: RF-13
  - Probar mapping de archivo/línea a comando de navegación.
  - Hecho cuando: un diagnóstico con ubicación produce el destino esperado.

- [ ] **FE-073 — Integration test UI → Core para Save**
  - RF: RF-02, RF-14
  - Verificar que la UI usa el mismo comando de guardado que el core.
  - Hecho cuando: el test confirma persistencia sin depender de un botón concreto.

- [ ] **FE-074 — Integration test UI → Core para Build/Run/Stop**
  - RF: RF-11, RF-12, RF-14
  - Verificar comandos y actualización de estados.
  - Hecho cuando: el flujo completo pasa sin bloqueo de UI.

- [ ] **FE-075 — Smoke test del frontend**
  - RF: RF-01 a RF-16
  - Ejecutar flujo principal mínimo.
  - Hecho cuando: Start → Open/Create → Edit → Save → Build → Run → Stop es reproducible sin fallos conocidos.

## Fase 14 — Pulido MVP

- [ ] **FE-076 — Revisar navegación y nomenclatura**
  - RF: RNF-11, RNF-12
  - Unificar textos y nombres de comandos visibles.
  - Hecho cuando: acciones equivalentes usan el mismo nombre en menú, toolbar y mensajes.

- [ ] **FE-077 — Reducir UI innecesaria**
  - RF: RNF-11
  - Eliminar controles o paneles sin función MVP.
  - Hecho cuando: cada elemento visible tiene una función definida.

- [ ] **FE-078 — Revisar rendimiento visual básico**
  - RF: RF-03, RNF-04, RNF-05
  - Revisar editor, output y designer con contenido de prueba razonable.
  - Hecho cuando: ninguna vista principal provoca congelación visible durante interacción normal.

- [ ] **FE-079 — Revisión final de arquitectura frontend**
  - RF: RNF-02, RNF-03, RNF-08
  - Verificar separación UI/core.
  - Hecho cuando: ningún widget contiene lógica de build, toolchain o generación de código específica que debería pertenecer al core/proveedor.

- [ ] **FE-080 — Cerrar frontend MVP**
  - RF: RF-01 a RF-16
  - Ejecutar checklist final y documentación.
  - Hecho cuando: las funciones frontend previstas están operativas, los tests pasan y no hay tareas fuera de alcance mezcladas en el MVP.
