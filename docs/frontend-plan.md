# MiniIDE — Plan de implementación del Frontend con egui/eframe

## 1. Propósito

Este documento define cómo implementar el frontend de MiniIDE usando **Rust + egui + eframe**.

El frontend será una capa de presentación: mostrará el estado proveniente del core, recibirá interacción del usuario y emitirá comandos. No será la fuente de verdad de documentos, proyectos, procesos ni diseños.

El frontend debe cubrir inicialmente:

- Ventana principal.
- Menú y comandos.
- Project Explorer.
- Editor con pestañas.
- Área de diseñador.
- Panel de propiedades.
- Consola/salida.
- Indicadores de estado.
- Diálogos básicos.

El objetivo es obtener una UI funcional y minimalista antes de añadir capacidades visuales avanzadas.

---

## 2. Stack del frontend

```text
Rust
├── egui      → widgets, layout, interacción y rendering de la UI
└── eframe    → aplicación/ventana y ciclo de ejecución de egui
```

El resto del sistema se comunica con el frontend mediante modelos y comandos del proyecto.

No se debe introducir otra librería de UI para el MVP.

---

## 3. Responsabilidades del frontend

El frontend será responsable de:

- Crear y mantener la ventana de la aplicación.
- Dibujar la interfaz.
- Capturar teclado y mouse.
- Mostrar documentos y estados del editor.
- Mostrar el árbol de proyectos.
- Mostrar el diseñador visual.
- Mostrar propiedades y diagnósticos.
- Convertir interacción de UI en comandos del core.
- Mostrar progreso y resultados de operaciones.
- Gestionar el estado puramente visual de la interfaz.

El frontend no será responsable de:

- Ser la fuente de verdad del texto.
- Implementar compiladores.
- Implementar el sistema de proyectos.
- Ejecutar directamente lógica específica de .NET o Java repartida entre widgets.
- Generar código WinForms/Swing directamente desde componentes de UI.
- Ejecutar procesos bloqueantes en el hilo de UI.

---

## 4. Arquitectura conceptual

```text
                         eframe
                           │
                           ▼
                    ┌─────────────┐
                    │ MiniIDE App │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  UI State   │
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
          ▼                ▼                ▼
     Main Window       Commands        View Models
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                    ┌─────────────┐
                    │    Core     │
                    └─────────────┘
```

Regla principal:

```text
Usuario → UI → Command → Core → State/Result → UI
```

La UI no debe modificar directamente el dominio salvo mediante una API claramente definida.

---

## 5. Estado de UI vs estado del dominio

### Estado del dominio

Pertenece al core:

- Proyecto activo.
- Documentos.
- Contenido.
- Cursor lógico.
- Selección lógica.
- Estado modificado.
- Modelo del diseñador.
- Estado de build.
- Estado de ejecución.
- Diagnósticos.

### Estado de UI

Pertenece al frontend:

- Panel abierto/cerrado.
- Tamaño/posición visual de paneles.
- Pestaña visual activa.
- Elemento actualmente expandido en el árbol.
- Foco visual.
- Estado temporal de diálogos.
- Estado de hover/selección visual.

Cuando un dato pueda afectar la lógica del proyecto, debe pertenecer al core y no exclusivamente a `egui`.

---

## 6. Composición principal de la ventana

La ventana principal deberá dividirse de forma simple:

```text
┌─────────────────────────────────────────────────────────┐
│ Menu / Toolbar                                          │
├──────────────┬──────────────────────────┬───────────────┤
│ Project      │ Editor / Designer        │ Properties    │
│ Explorer     │                          │               │
│              │                          │               │
│              │                          │               │
├──────────────┴──────────────────────────┴───────────────┤
│ Output / Diagnostics / Terminal                         │
├─────────────────────────────────────────────────────────┤
│ Status bar                                              │
└─────────────────────────────────────────────────────────┘
```

En el MVP no se debe intentar reproducir la cantidad de paneles de un IDE comercial.

---

## 7. Módulos de frontend

### 7.1 `app_ui`

Responsabilidad:

- Crear la aplicación eframe.
- Inicializar estado de UI.
- Coordinar actualización de frames.

### 7.2 `main_window`

Responsabilidad:

- Estructura general de la ventana.
- Menú.
- Toolbar.
- Paneles principales.
- Área central.
- Barra de estado.

### 7.3 `project_view`

Responsabilidad:

- Mostrar workspace/proyecto.
- Expandir/contraer carpetas.
- Seleccionar archivos.
- Acciones contextuales básicas.

### 7.4 `editor_view`

Responsabilidad:

- Presentación del documento.
- Cursor/selección visual.
- Interacción de teclado.
- Scroll.
- Render del contenido.
- Tabs de documentos.

El modelo de edición permanece fuera de este módulo.

### 7.5 `designer_view`

Responsabilidad:

- Canvas del diseñador.
- Selección de componentes.
- Move/resize visual.
- Grilla opcional.
- Interacción visual básica.

No contiene la lógica específica de generación de C# o Java.

### 7.6 `properties_view`

Responsabilidad:

- Mostrar propiedades del elemento seleccionado.
- Editar valores mediante comandos hacia el modelo del diseñador.

### 7.7 `output_view`

Responsabilidad:

- Mostrar stdout/stderr.
- Mostrar diagnósticos.
- Mostrar estado de build/run.

### 7.8 `dialogs`

Responsabilidad:

- Nuevo proyecto.
- Abrir proyecto.
- Confirmar guardado.
- Errores y avisos.

### 7.9 `ui_state`

Responsabilidad:

- Estado efímero y puramente visual.
- Estado de paneles.
- Estado de modales.
- Selecciones visuales.

---

## 8. Comunicación con el core

Se recomienda que la UI consuma un contexto de aplicación pequeño, conceptualmente:

```rust
struct AppContext {
    // acceso controlado a comandos/queries del core
}
```

La UI debe:

1. Leer un snapshot o vista del estado necesario.
2. Renderizarlo.
3. Capturar interacción.
4. Emitir un comando.
5. Recibir el resultado/cambio de estado.
6. Renderizar el siguiente frame.

Evitar exponer estructuras internas completas del core a todos los widgets.

---

## 9. Editor con egui

El editor es el componente de mayor riesgo técnico del frontend.

Para el MVP debe priorizar:

- Texto visible.
- Scroll vertical/horizontal.
- Cursor visible.
- Selección básica.
- Entrada de teclado.
- Click para posicionamiento.
- Pestañas.
- Números de línea.
- Resaltado sintáctico básico.

La implementación debe empezar con una representación sencilla y optimizar únicamente cuando exista evidencia de necesidad.

### Regla

No acoplar el modelo de `Document` a los widgets de egui.

---

## 10. Diseñador visual con egui

El designer utilizará el `DesignerModel` del core.

Flujo:

```text
Pointer event
    ↓
Designer View
    ↓
Designer Command
    ↓
Designer Model
    ↓
Updated State
    ↓
Designer View
```

La representación visual del control no será responsable de conocer cómo se genera C# o Java.

---

## 11. Comandos de UI

Acciones principales:

```text
NewProject
OpenProject
CloseProject
NewFile
Save
SaveAll
Undo
Redo
Find
Replace
Build
Run
Stop
OpenDesigner
```

Los menús, botones y atajos deberán disparar los mismos comandos cuando la acción sea equivalente.

---

## 12. Asincronía y operaciones largas

Las siguientes operaciones no deben bloquear el frame de egui:

- Compilación.
- Ejecución de aplicaciones.
- Detección de toolchains.
- Lecturas/escrituras potencialmente lentas.
- Operaciones futuras de búsqueda sobre grandes workspaces.

La UI mostrará estados como:

```text
Idle
Running
Building
Success
Failed
Stopping
```

La implementación concreta de concurrencia pertenece al subsistema de runtime/core, no a cada widget.

---

## 13. Integración visual con el diseñador y editor

La ventana central deberá poder alternar entre:

```text
Code
Designer
```

o presentar ambos de manera controlada cuando el recurso lo requiera.

No se debe implementar simultáneamente un sistema complejo de docking, múltiples ventanas y layouts persistentes en el MVP.

---

## 14. Tema y estilo

El MVP utilizará un tema consistente proporcionado por egui con ajustes mínimos.

Prioridades:

1. Legibilidad.
2. Contraste adecuado.
3. Consistencia.
4. Densidad apropiada para una herramienta de desarrollo.

No implementar inicialmente un sistema completo de theming.

---

## 15. Accesos de teclado

El frontend deberá reservar y manejar los atajos esenciales:

```text
Ctrl+N  Nuevo
Ctrl+O  Abrir
Ctrl+S  Guardar
Ctrl+Shift+S  Guardar como
Ctrl+Z  Deshacer
Ctrl+Y  Rehacer
Ctrl+F  Buscar
Ctrl+H  Reemplazar
Ctrl+B  Compilar
F5      Ejecutar
Shift+F5 Detener
```

La capa de comandos será la responsable de la acción; egui únicamente detecta la interacción.

---

## 16. Estrategia de pruebas del frontend

### Unit tests

Probar lógica puramente visual o de transformación:

- Conversión de estados a view models.
- Cálculos de layout no triviales.
- Mapeo de posiciones.
- Conversión de eventos a comandos.
- Estado de tabs y paneles.

### Integration tests

Probar:

- UI → command → core.
- Apertura de documento desde Project Explorer.
- Save desde menú/atajo.
- Build desde toolbar/atajo.
- Run/Stop desde UI.
- Designer → cambio de modelo → refresh.

### Smoke tests

Verificar manual o automáticamente los flujos principales:

```text
Start IDE
→ Create project
→ Open document
→ Edit
→ Save
→ Build
→ Run
→ Stop
```

y el equivalente con Designer.

---

## 17. Decisiones

### FD-01 — egui/eframe

**Decisión:** usar egui + eframe como stack de UI del MVP.

**Alternativa descartada:** WinForms/WPF.

**Motivo:** el IDE debe estar construido en Rust y no debe depender de C# para su propia interfaz.

### FD-02 — UI inmediata

**Decisión:** adoptar el modelo de UI de egui.

**Alternativa descartada:** construir un framework declarativo propio encima de Rust.

**Motivo:** sería infraestructura adicional sin valor para el MVP.

### FD-03 — Un solo stack de UI

**Decisión:** no mezclar múltiples toolkits de UI en el MVP.

**Alternativa descartada:** egui + otra biblioteca para editor/designer.

**Motivo:** reducir dependencias y complejidad de integración.

### FD-04 — Core como fuente de verdad

**Decisión:** el frontend consume estado del core y emite comandos.

**Alternativa descartada:** mantener copias completas del dominio dentro de la UI.

**Motivo:** evita inconsistencias y facilita las pruebas.

### FD-05 — Editor propio del proyecto

**Decisión:** implementar el editor como parte de MiniIDE y mantener su modelo en Rust.

**Alternativa descartada:** basar el MVP en un control externo que sea la fuente de verdad del texto.

**Motivo:** el editor es una capacidad central del producto y debe integrarse con Document/Command/Undo del core.

---

## 18. Criterios de aceptación del frontend MVP

El frontend se considera funcional cuando:

1. La aplicación abre una ventana estable con eframe.
2. Se muestran menú, explorador, editor, output y estado.
3. El usuario puede crear/abrir un proyecto.
4. El usuario puede abrir y editar un documento.
5. Guardar funciona desde comando/menú/atajo.
6. C# WinForms puede abrirse y trabajarse desde la UI.
7. Java Swing puede abrirse y trabajarse desde la UI.
8. El diseñador puede seleccionar y modificar componentes básicos.
9. Build/Run/Stop pueden activarse sin congelar la interfaz.
10. Los diagnósticos aparecen en el panel correspondiente.
