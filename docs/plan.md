# MiniIDE — Plan de arquitectura y desarrollo

## 1. Propósito del plan

Este documento transforma la constitución del proyecto y la especificación funcional en una estructura de módulos, un modelo de dominio, decisiones arquitectónicas y una estrategia de pruebas.

El MVP se limita a Windows y soporta únicamente:

- C# + WinForms.
- Java + Swing.

El núcleo y la interfaz del IDE se desarrollarán en Rust. La arquitectura debe permitir incorporar nuevos lenguajes y frameworks posteriormente sin rediseñar el core.

---

## 2. Principios que gobiernan el diseño

1. Rust es la tecnología principal del IDE y del core.
2. El core trabaja con conceptos y abstracciones, no con detalles concretos de C# o Java.
3. Lenguaje, framework y toolchain son responsabilidades separables.
4. El estado del documento pertenece al modelo del editor, no al componente visual.
5. El diseñador trabaja sobre un modelo visual intermedio y genera código específico del framework.
6. La compilación y ejecución son procesos externos y no deben bloquear la UI.
7. El MVP prioriza simplicidad sobre extensibilidad prematura.
8. La extensibilidad futura se logra mediante interfaces internas pequeñas antes de introducir plugins externos.

---

## 3. Módulos del sistema

### 3.1 Application

Responsabilidad:

- Ciclo de vida de la aplicación.
- Inicialización de servicios.
- Coordinación de la ventana principal.
- Cierre ordenado.

Cubre principalmente: RF-01.

---

### 3.2 UI / Presentation

Responsabilidad:

- Ventana principal.
- Menús, toolbar y paneles.
- Pestañas.
- Explorador de proyectos.
- Editor visual del código.
- Diseñador visual.
- Panel de propiedades.
- Panel de salida/errores.

No debe ser la fuente de verdad del documento ni de la configuración del proyecto.

Cubre principalmente: RF-01, RF-04, RF-05, RF-07, RF-09, RF-13, RF-16.

---

### 3.3 Core / Domain

Responsabilidad:

- Entidades y reglas centrales.
- Identidad de proyectos, documentos y recursos.
- Abstracciones de lenguaje, framework y toolchain.
- Resultados de operaciones.
- Estados y transiciones relevantes.

Este módulo no debe depender de WinForms, Swing, .NET o JDK.

Cubre transversalmente todos los RF; es la base de RF-06, RF-07, RF-09, RF-11, RF-12, RF-16.

---

### 3.4 Document / Editor

Responsabilidad:

- Contenido de documentos.
- Cursor.
- Selección.
- Undo/Redo.
- Estado modificado/no modificado.
- Edición incremental.
- Búsqueda y reemplazo.
- Pestañas y relación entre documento y vista.

Cubre: RF-02, RF-03, RF-04, RF-07, RF-09, RF-11.

---

### 3.5 Workspace / Project

Responsabilidad:

- Apertura y cierre de proyectos.
- Estructura de directorios.
- Archivos del proyecto.
- Identificación de lenguaje/framework.
- Configuración básica del proyecto.
- Creación de plantillas iniciales.

Cubre: RF-05, RF-06, RF-07, RF-09, RF-15.

---

### 3.6 Language Support

Responsabilidad:

- Identidad del lenguaje.
- Extensiones de archivo.
- Configuración de edición.
- Resaltado sintáctico inicial.
- Reglas específicas de proyecto que pertenezcan al lenguaje.

Implementaciones iniciales:

- C#.
- Java.

Cubre: RF-06, RF-07, RF-09, RF-15.

---

### 3.7 Framework Support

Responsabilidad:

- Semántica del toolkit/framework visual.
- Componentes disponibles.
- Propiedades soportadas.
- Eventos soportados.
- Reglas del diseñador.
- Generación específica de código.

Implementaciones iniciales:

- WinForms.
- Swing.

Cubre: RF-06, RF-08, RF-10, RF-15.

---

### 3.8 Designer

Responsabilidad:

- Modelo visual.
- Selección y manipulación de componentes.
- Propiedades.
- Posicionamiento y tamaño.
- Persistencia del modelo visual.
- Coordinación con el generador de código.

El diseñador no debe editar directamente todo el archivo fuente; debe operar mediante su modelo y reglas de generación.

Cubre: RF-08, RF-10, RF-11, RF-12.

---

### 3.9 Code Generation

Responsabilidad:

- Convertir el modelo visual en código correspondiente al framework.
- Actualizar únicamente las regiones controladas por el diseñador.
- Preservar código manual no administrado por el generador.

Cubre: RF-08, RF-10, RF-11.

---

### 3.10 Toolchain

Responsabilidad:

- Detectar herramientas externas.
- Conocer requisitos de la plataforma.
- Preparar invocaciones de build/run.
- Exponer resultados normalizados al core.

Implementaciones iniciales:

- .NET SDK.
- JDK.

Cubre: RF-07, RF-09, RF-11, RF-12, RF-16.

---

### 3.11 Build / Runtime

Responsabilidad:

- Lanzar compilaciones.
- Capturar stdout/stderr.
- Gestionar código de salida.
- Ejecutar aplicaciones.
- Detener procesos.
- Evitar bloqueo de la UI.

Cubre: RF-11, RF-12, RF-13, RF-16.

---

### 3.12 Diagnostics

Responsabilidad:

- Normalizar errores y advertencias.
- Asociar diagnóstico con archivo, línea y columna cuando sea posible.
- Mostrar el resultado de build y ejecución.

Cubre: RF-11, RF-12, RF-13, RF-16.

---

### 3.13 Commands

Responsabilidad:

- Representar acciones del IDE de forma independiente de botones y menús.
- Centralizar acciones como abrir, guardar, compilar, ejecutar y detener.

Cubre: RF-01, RF-02, RF-03, RF-06, RF-11, RF-12, RF-14.

---

### 3.14 Configuration

Responsabilidad:

- Configuración global del IDE.
- Preferencias básicas.
- Configuración persistente del proyecto cuando corresponda.

Cubre: RF-20 y parcialmente RF-21, que puede mantenerse fuera del MVP.

---

## 4. Modelo de datos del dominio

El modelo debe mantenerse pequeño y expresivo. Las entidades principales son:

### Project

Representa un proyecto abierto o creado por el usuario.

Información esencial:

- Identidad.
- Nombre.
- Ruta.
- Lenguaje.
- Framework.
- Configuración de compilación.
- Archivos.

Relaciones:

`Project -> Documents/Files`

`Project -> LanguageSupport`

`Project -> FrameworkSupport`

`Project -> Toolchain`

---

### LanguageId

Identificador abstracto de lenguaje.

Valores iniciales:

- C#.
- Java.

Debe permitir añadir nuevos identificadores en el futuro.

---

### FrameworkId

Identificador abstracto del toolkit/framework.

Valores iniciales:

- WinForms.
- Swing.

---

### ProjectType

Representa la combinación de tipo de aplicación que MiniIDE sabe crear.

Ejemplos iniciales:

- C# + WinForms.
- Java + Swing.

---

### ProjectFile

Representa un archivo o recurso del proyecto.

Información relevante:

- Ruta relativa.
- Tipo de archivo.
- Estado físico/lógico.
- Relación opcional con un documento abierto.

---

### Document

Representa el contenido editable de un archivo.

Información relevante:

- Identidad.
- Ruta asociada.
- Contenido.
- Modificado/no modificado.
- Metadatos de edición.

Es la fuente de verdad del contenido editado.

---

### EditorState

Representa el estado de edición de un documento.

Incluye:

- Cursor.
- Selección.
- Historial de undo.
- Historial de redo.
- Vista/pestaña asociada.

---

### DesignerModel

Representa una interfaz gráfica independientemente de su código generado.

Incluye:

- Ventana/formulario.
- Componentes.
- Identidad de cada componente.
- Posición.
- Tamaño.
- Propiedades.
- Eventos soportados.

---

### DesignerComponent

Representa un control visual del modelo.

Debe contener información común y permitir propiedades específicas del framework.

Ejemplos conceptuales:

- Button / JButton.
- Label / JLabel.
- TextBox / JTextField.
- Panel / JPanel.

---

### BuildConfiguration

Representa las opciones necesarias para construir el proyecto.

Debe permanecer independiente de la herramienta concreta siempre que sea posible.

---

### ToolchainInfo

Describe una herramienta externa detectada:

- Identidad.
- Versión.
- Ruta.
- Disponibilidad.
- Capacidades.

---

### BuildResult

Resultado normalizado de una compilación:

- Éxito/fallo.
- Código de salida.
- Salida estándar.
- Salida de error.
- Diagnósticos.

---

### RunResult / ProcessState

Representa una aplicación lanzada por el IDE:

- Proceso.
- Estado.
- Código de salida.
- Salida.
- Error.

---

### Diagnostic

Representa un error, advertencia o información relevante.

Campos esperados:

- Nivel.
- Mensaje.
- Archivo opcional.
- Línea opcional.
- Columna opcional.
- Origen.

---

### Command

Representa una operación ejecutable por la interfaz y otras superficies futuras.

Ejemplos:

- NewProject.
- OpenProject.
- Save.
- Build.
- Run.
- Stop.
- Undo.
- Redo.

---

## 5. Relaciones principales del modelo

```text
Project
 ├── ProjectType
 ├── LanguageId
 ├── FrameworkId
 ├── BuildConfiguration
 └── ProjectFile[*]
       └── Document [0..1]

Document
 └── EditorState

Project
 └── DesignerModel [0..*]
       └── DesignerComponent[*]

Project
 └── ToolchainInfo

Build
 └── BuildResult
       └── Diagnostic[*]

Run
 └── ProcessState
```

---

## 6. Decisiones arquitectónicas

### D-01 — Rust para todo el IDE

**Decisión:** Rust será la tecnología principal del IDE y del core.

**Alternativa descartada:** WinForms/C# como frontend del IDE.

**Motivo:** La constitución establece Rust como stack del IDE; además, mantener el frontend en Rust evita que el propio producto dependa de una de las plataformas que pretende soportar.

---

### D-02 — C# y Java como soportes, no como núcleo

**Decisión:** C# y Java serán proveedores de lenguaje/framework/toolchain.

**Alternativa descartada:** introducir condiciones específicas de C# y Java dentro del core.

**Motivo:** reduce acoplamiento y facilita añadir otros lenguajes en el futuro.

---

### D-03 — Separar lenguaje, framework y toolchain

**Decisión:** representar estas tres responsabilidades por separado.

**Alternativa descartada:** crear módulos por combinación, por ejemplo `CSharpWinForms` y `JavaSwing` como únicas unidades de extensión.

**Motivo:** evita que cada nueva combinación obligue a duplicar arquitectura y permite reutilizar soporte de lenguaje y toolchain.

---

### D-04 — Abstracciones internas antes de plugins externos

**Decisión:** el MVP tendrá interfaces/proveedores internos, pero no un sistema completo de plugins externos.

**Alternativa descartada:** implementar desde el inicio carga dinámica de plugins y marketplace.

**Motivo:** la constitución prioriza minimalismo y evita infraestructura innecesaria antes de tener el producto funcional.

---

### D-05 — Modelo visual intermedio para diseñadores

**Decisión:** WinForms y Swing utilizarán un modelo visual intermedio compartido, con adaptadores específicos para generar código.

**Alternativa descartada:** editar directamente el código fuente como representación única del diseñador.

**Motivo:** la edición directa complica selección, propiedades, layout y preservación de código; el modelo intermedio permite reutilizar la infraestructura del diseñador.

---

### D-06 — El documento es la fuente de verdad

**Decisión:** el estado lógico del documento pertenece al core/editor.

**Alternativa descartada:** usar el widget visual como fuente de verdad.

**Motivo:** evita inconsistencias entre modelo y UI y facilita pruebas unitarias del editor.

---

### D-07 — Toolchains externas

**Decisión:** MiniIDE orquestará .NET SDK y JDK en lugar de implementar compiladores propios.

**Alternativa descartada:** construir un compilador o runtime propio para cada lenguaje.

**Motivo:** no forma parte del objetivo del producto y multiplicaría drásticamente el alcance.

---

### D-08 — Procesos fuera del hilo de UI

**Decisión:** build/run y otras tareas lentas serán asíncronas respecto de la interfaz.

**Alternativa descartada:** ejecución bloqueante desde la UI.

**Motivo:** el requisito de responsividad y la naturaleza externa de las toolchains requieren aislamiento del trabajo largo.

---

### D-09 — Comandos independientes de la UI

**Decisión:** acciones del IDE se representarán como comandos.

**Alternativa descartada:** implementar acciones exclusivamente dentro de handlers visuales.

**Motivo:** mejora testabilidad, reutilización y futura extensión mediante atajos, menús y plugins.

---

### D-10 — Generación controlada de código

**Decisión:** el diseñador solo administrará las regiones de código que le correspondan.

**Alternativa descartada:** regenerar el archivo completo cada vez que cambia el diseño.

**Motivo:** protege el código manual del usuario y reduce el riesgo de pérdida de trabajo.

---

## 7. Estrategia de pruebas

La estrategia debe cubrir principalmente la lógica del core y las integraciones que producen resultados observables.

### 7.1 Unit tests

Usar unit tests para:

- Documento.
- Inserción/eliminación.
- Cursor y selección.
- Undo/Redo.
- Búsqueda y reemplazo.
- Modelo de proyecto.
- Identificación de lenguaje/framework.
- Validaciones.
- Modelo visual.
- Propiedades de controles.
- Comandos.
- Parseo de diagnósticos.

Objetivo: alta cobertura de lógica determinista y rápida.

---

### 7.2 Integration tests

Usar integration tests para:

- Crear proyectos.
- Abrir/cerrar proyectos.
- Interacción con el filesystem.
- Detección de .NET SDK.
- Detección de JDK.
- Invocación de compilaciones.
- Captura de stdout/stderr.
- Ejecución y detención de procesos.
- Integración entre diseñador y generación de código.

Los tests de toolchain deben poder detectar entornos faltantes y reportar el motivo sin generar falsos éxitos.

---

### 7.3 Golden / snapshot tests para generación de código

La generación de código de WinForms y Swing debe validarse comparando el resultado esperado con una referencia controlada.

Objetivo:

- Detectar cambios inesperados en el código generado.
- Proteger el contrato entre DesignerModel y generadores.

---

### 7.4 End-to-end smoke tests

Debe existir al menos un flujo completo por plataforma:

#### C# / WinForms

Crear proyecto → diseñar formulario mínimo → generar código → compilar → ejecutar.

#### Java / Swing

Crear proyecto → diseñar ventana mínima → generar código → compilar → ejecutar.

Estos tests validan el flujo de extremo a extremo y no sustituyen a los unit tests.

---

### 7.5 Tests de regresión

Cada bug corregido debe, cuando sea razonable, generar un test que reproduzca el problema antes de la corrección.

---

## 8. Cobertura de requisitos funcionales

| Requisito | Módulo(s) principal(es) |
|---|---|
| RF-01 Gestión de aplicación | Application, UI |
| RF-02 Gestión de documentos | Document, Editor, UI |
| RF-03 Edición de texto | Editor, Document |
| RF-04 Pestañas | UI, Editor |
| RF-05 Explorador de proyectos | Project, Workspace, UI |
| RF-06 Creación de proyectos | Project, Language Support, Framework Support |
| RF-07 Soporte C# | Language/C#, Toolchain/.NET, Build, Runtime |
| RF-08 Soporte WinForms | Framework/WinForms, Designer, Code Generation |
| RF-09 Soporte Java | Language/Java, Toolchain/JDK, Build, Runtime |
| RF-10 Soporte Swing | Framework/Swing, Designer, Code Generation |
| RF-11 Compilación | Toolchain, Build, Diagnostics, Commands |
| RF-12 Ejecución | Runtime, Toolchain, Process Management |
| RF-13 Terminal/salida | Runtime, Diagnostics, UI |
| RF-14 Sistema de comandos | Commands, Core, UI |
| RF-15 Abstracción de lenguajes/frameworks | Core, Language Support, Framework Support |
| RF-16 Detección de toolchains | Toolchain, Diagnostics, UI |

---

## 9. Cobertura de requisitos no funcionales

| Requisito | Decisiones/Módulos que lo soportan |
|---|---|
| RNF-01 Rust | Toda la arquitectura base |
| RNF-02 Arquitectura modular | Separación por módulos y proveedores |
| RNF-03 Extensibilidad | Abstracciones Language/Framework/Toolchain |
| RNF-04 Rendimiento | Editor eficiente, procesos aislados |
| RNF-05 Responsividad | Build/Runtime fuera de UI |
| RNF-06 Estabilidad | Manejo de errores de procesos y toolchains |
| RNF-07 Mantenibilidad | Módulos pequeños y responsabilidades definidas |
| RNF-08 Bajo acoplamiento | Core dependiente de abstracciones |
| RNF-09 Portabilidad arquitectónica | Aislamiento de dependencias específicas de Windows |
| RNF-10 Seguridad | Procesos explícitos y controlados |
| RNF-11 Usabilidad | UI, Commands, atajos y paneles consistentes |
| RNF-12 Consistencia | Modelo común para proyectos y operaciones |

---

## 10. Orden de implementación recomendado

### Fase 1 — Foundation

- Application.
- Core/domain.
- Project model.
- Document model.
- Command model.

RF relacionados: RF-01, RF-02, RF-03, RF-05, RF-06, RF-14, RF-15.

### Fase 2 — Editor funcional

- Edición de texto.
- Cursor/selección.
- Undo/Redo.
- Pestañas.
- Búsqueda/reemplazo.
- Guardado.

RF relacionados: RF-02, RF-03, RF-04, RF-11.

### Fase 3 — UI de proyecto

- Explorador.
- Creación/apertura de proyecto.
- Gestión básica de archivos.

RF relacionados: RF-05, RF-06.

### Fase 4 — C# / WinForms

- C# language support.
- .NET toolchain.
- WinForms provider.
- Designer model.
- Code generation.
- Build/run.

RF relacionados: RF-07, RF-08, RF-11, RF-12, RF-13, RF-16.

### Fase 5 — Java / Swing

- Java language support.
- JDK toolchain.
- Swing provider.
- Designer model.
- Code generation.
- Build/run.

RF relacionados: RF-09, RF-10, RF-11, RF-12, RF-13, RF-16.

### Fase 6 — Hardening

- Integration tests.
- End-to-end smoke tests.
- Regression tests.
- Error handling.
- Mejoras de rendimiento y estabilidad.

Cubre transversalmente los RNF y consolida todos los RF del MVP.

---

## 11. Criterio de arquitectura terminada para el MVP

La arquitectura del MVP se considera suficientemente consolidada cuando:

1. El core no depende directamente de C# o Java.
2. C# y Java implementan contratos comunes donde comparten responsabilidades.
3. WinForms y Swing utilizan el subsistema de diseñador sin duplicar el núcleo de edición.
4. Build y Run están aislados de la UI.
5. Los errores de toolchain se convierten en diagnósticos comprensibles.
6. Los flujos principales de ambas plataformas tienen integration/end-to-end tests.
7. Añadir un tercer lenguaje no exige reescribir el core.
8. No existe infraestructura de plugins externos innecesaria para el MVP.

---

## 12. Regla de evolución futura

Cuando se incorpore un nuevo lenguaje o framework, primero se deberá comprobar si puede implementarse utilizando las abstracciones existentes.

Solo se modificará el core cuando el nuevo requisito revele una limitación real de la abstracción actual.

La extensibilidad debe surgir de necesidades demostradas y no de especulación arquitectónica.
