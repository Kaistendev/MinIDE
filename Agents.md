# AGENTS.md

## MiniIDE — Guía para agentes de desarrollo

## 1. Propósito del proyecto

MiniIDE es un IDE minimalista para Windows desarrollado principalmente en Rust.

Su objetivo inicial es permitir desarrollar aplicaciones de escritorio mediante:

* C# + Windows Forms.
* Java + Swing.

El núcleo del IDE debe permanecer independiente de los lenguajes y frameworks soportados, de manera que el sistema pueda ampliarse posteriormente con nuevas tecnologías.

La prioridad del proyecto es:

> **Simplicidad, modularidad, mantenibilidad y funcionalidad esencial antes que cantidad de características.**

---

# 2. Principios fundamentales

Todo agente que modifique el proyecto deberá respetar los siguientes principios.

## 2.1 Minimalismo

No implementar funcionalidades que no formen parte del alcance actual.

Antes de añadir una característica, comprobar:

1. ¿Es necesaria para cumplir un requisito existente?
2. ¿Es necesaria para completar un caso de uso?
3. ¿Es necesaria para mantener la arquitectura?
4. ¿Puede resolverse de una manera más sencilla?

Si la respuesta es negativa, no añadirla.

---

## 2.2 Rust como núcleo

El núcleo funcional del IDE debe estar escrito en Rust.

Rust debe controlar, según corresponda:

* Estado de la aplicación.
* Documentos.
* Editor.
* Proyectos.
* Workspace.
* Comandos.
* Procesos.
* Compilación.
* Ejecución.
* Diseñadores.
* Integración con toolchains.
* Sistema de extensibilidad.

No introducir C#, JavaScript, Python u otros lenguajes para implementar partes fundamentales del IDE salvo que exista una razón arquitectónica explícita y documentada.

---

## 2.3 Separación entre IDE y aplicaciones desarrolladas

Es importante distinguir:

```text
MiniIDE
└── implementado en Rust

Aplicaciones creadas por el usuario
├── C# + WinForms
└── Java + Swing
```

C# y Java son tecnologías soportadas por el IDE.

No deben utilizarse como tecnologías internas para construir el IDE.

---

## 2.4 El núcleo debe conocer conceptos, no tecnologías concretas

El core debe trabajar mediante abstracciones.

Ejemplos de conceptos que el core puede conocer:

```text
Language
Framework
Project
Toolchain
BuildProvider
RunProvider
Designer
Document
Command
Diagnostic
```

El core no debe contener lógica específica dispersa como:

```text
if language == "csharp"
if framework == "winforms"
if language == "java"
```

La lógica específica debe encontrarse en módulos de soporte apropiados.

---

# 3. Alcance actual

## 3.1 Lenguajes soportados

Actualmente:

```text
C#
Java
```

## 3.2 Frameworks/toolkits soportados

Actualmente:

```text
C#  → WinForms
Java → Swing
```

## 3.3 Plataforma objetivo

La primera versión está orientada exclusivamente a:

```text
Windows
```

## 3.4 Funcionalidades principales

El MVP contempla:

* Editor de código.
* Gestión de documentos.
* Pestañas.
* Explorador de proyectos.
* Gestión básica de archivos.
* Crear proyectos.
* Abrir proyectos.
* Guardar proyectos.
* Resaltado sintáctico básico.
* Diseñador visual básico.
* Generación de código.
* Compilación.
* Ejecución.
* Visualización de diagnósticos.
* Gestión básica de procesos.

---

# 4. Fuera del alcance actual

No implementar sin una decisión explícita del proyecto:

* Python.
* Electron.
* Tauri.
* JavaFX.
* WPF.
* Otros lenguajes.
* Otros frameworks.
* Marketplace.
* Sistema completo de plugins externos.
* Git avanzado.
* LSP completo.
* Refactorización avanzada.
* Autocompletado avanzado.
* Debugger avanzado.
* Sistema de extensiones de terceros.
* Soporte multiplataforma.

La arquitectura debe quedar preparada para estas posibilidades, pero **no deben implementarse prematuramente**.

---

# 5. Arquitectura

La arquitectura debe mantener una separación clara entre:

```text
Application
UI
Core
Editor
Document
Project
Designer
Language Support
Framework Support
Toolchain
Build
Runtime
Diagnostics
```

Una organización conceptual recomendada:

```text
src/
├── app/
├── core/
├── ui/
├── editor/
├── document/
├── workspace/
├── project/
├── designer/
├── language/
│   ├── csharp/
│   └── java/
├── framework/
│   ├── winforms/
│   └── swing/
├── toolchain/
│   ├── dotnet/
│   └── jdk/
├── build/
├── runtime/
├── diagnostics/
└── commands/
```

La organización física puede evolucionar, pero la separación de responsabilidades debe mantenerse.

---

# 6. Regla de dependencias

La dirección general de dependencias debe ser:

```text
UI
 ↓
Application/Core
 ↓
Domain abstractions
 ↓
Implementations
```

Los módulos específicos de lenguaje/framework/toolchain no deben obligar al core a conocer detalles concretos de su implementación.

Evitar dependencias circulares.

---

# 7. Lenguajes y frameworks

Los soportes tecnológicos deberán implementarse mediante abstracciones comunes.

Conceptualmente:

```rust
trait LanguageProvider {
    fn id(&self) -> &str;
    fn extensions(&self) -> &[&str];
}
```

Y, cuando corresponda:

```rust
trait BuildProvider {
    fn build(&self, project: &Project) -> BuildResult;
}
```

```rust
trait RunProvider {
    fn run(&self, project: &Project) -> RunResult;
}
```

```rust
trait DesignerProvider {
    fn load(&self, document: &Document) -> DesignerModel;
    fn generate(&self, model: &DesignerModel) -> GeneratedCode;
}
```

Los nombres concretos de traits pueden cambiar, pero la separación conceptual debe permanecer.

---

# 8. Regla para nuevas funcionalidades

Antes de implementar una nueva funcionalidad:

1. Identificar el requisito funcional correspondiente.
2. Identificar el caso de uso relacionado.
3. Determinar a qué módulo pertenece.
4. Verificar que no duplique funcionalidad existente.
5. Evaluar si debe ser una abstracción o una implementación concreta.
6. Implementar la solución más pequeña que satisfaga el requisito.
7. Añadir pruebas.
8. Actualizar documentación cuando corresponda.

No implementar funcionalidades únicamente porque sean técnicamente interesantes.

---

# 9. Regla para cambios arquitectónicos

Un agente no debe realizar cambios arquitectónicos grandes únicamente para facilitar una tarea pequeña.

Ejemplo:

```text
Correcto:

Necesito ejecutar Java
→ implementar JavaToolchain
```

No:

```text
Necesito ejecutar Java
→ rediseñar todo el sistema de plugins
→ agregar marketplace
→ crear sistema de scripting
→ cambiar la arquitectura completa
```

Las soluciones deben ser proporcionales al problema.

---

# 10. Editor

El editor debe mantener el estado del documento separado de la representación visual.

Conceptualmente:

```text
Document Model
      ↓
Editor State
      ↓
Rendering
```

El componente de UI no debe convertirse en la fuente de verdad del contenido.

El estado debe poder manejar:

* Texto.
* Cursor.
* Selección.
* Undo.
* Redo.
* Modificaciones.
* Posición del documento.

---

# 11. Diseñador visual

El diseñador debe utilizar un modelo intermedio.

Conceptualmente:

```text
Visual Model
     │
 ┌───┴────┐
 ▼        ▼
WinForms  Swing
 ▼        ▼
C# Code   Java Code
```

No acoplar directamente toda la lógica del diseñador a archivos fuente específicos.

El modelo debe representar conceptos como:

* Ventana.
* Control.
* Posición.
* Tamaño.
* Propiedad.
* Evento.

Las particularidades de cada framework deben permanecer en su proveedor.

---

# 12. Generación de código

El código generado por el diseñador debe distinguir entre:

```text
Código generado
Código escrito por el usuario
```

Nunca sobrescribir indiscriminadamente código manual del usuario.

Los generadores deberán modificar únicamente las áreas que les correspondan.

---

# 13. Toolchains externas

MiniIDE no implementa:

* Compilador de C#.
* Compilador de Java.
* Runtime de .NET.
* JVM.

El IDE debe utilizar las herramientas externas disponibles en el sistema.

Ejemplos:

```text
C#  → .NET SDK
Java → JDK
```

El acceso a estas herramientas debe estar encapsulado.

No dispersar llamadas directas a `dotnet`, `javac`, `java` u otras herramientas por todo el código fuente.

---

# 14. Procesos externos

Todo proceso externo debe:

* Ejecutarse sin bloquear la interfaz.
* Capturar stdout.
* Capturar stderr.
* Registrar el código de salida.
* Poder ser detenido cuando corresponda.
* Manejar errores de lanzamiento.
* Liberar correctamente sus recursos.

No utilizar llamadas bloqueantes en el hilo de UI para tareas largas.

---

# 15. Manejo de errores

Los errores deben clasificarse según corresponda:

```text
User Error
Configuration Error
Project Error
Toolchain Error
Build Error
Runtime Error
Internal Error
```

Los mensajes mostrados al usuario deben ser comprensibles.

Evitar mostrar directamente errores internos sin contexto.

---

# 16. Estado y concurrencia

El estado global de la aplicación debe reducirse al mínimo.

Preferir:

```text
Owned State
Explicit State Transitions
Message Passing
```

sobre variables globales mutables.

Las tareas asíncronas deberán comunicar sus resultados mediante mecanismos explícitos.

No introducir concurrencia compleja sin necesidad.

---

# 17. Interfaz de usuario

La UI debe priorizar:

* Claridad.
* Consistencia.
* Simplicidad.
* Pocos elementos innecesarios.
* Navegación rápida.

Las características avanzadas no deben aumentar innecesariamente la complejidad visual.

La interfaz debe estar organizada alrededor de:

```text
Project Explorer
Editor
Designer
Properties
Output
```

---

# 18. Atajos y comandos

Las acciones del IDE deben modelarse como comandos cuando sea razonable.

Ejemplos:

```text
NewProject
OpenProject
Save
SaveAll
Undo
Redo
Build
Run
Stop
Find
Replace
```

Los comandos no deben depender directamente de botones concretos de la UI.

---

# 19. Pruebas

Todo comportamiento importante debe tener pruebas apropiadas.

Prioridad:

```text
Core logic
Document model
Editor operations
Project model
Code generation
Toolchain integration
Build parsing
Process management
```

La lógica pura debe probarse preferentemente mediante unit tests.

Las integraciones con herramientas externas deben probarse mediante integration tests cuando sea práctico.

---

# 20. Calidad del código

Todo código nuevo debe:

* Compilar sin warnings evitables.
* Mantener responsabilidades claras.
* Evitar duplicación innecesaria.
* Utilizar nombres descriptivos.
* Evitar funciones excesivamente grandes.
* Evitar estructuras innecesariamente complejas.
* Mantener interfaces pequeñas.
* Documentar decisiones no obvias.

No introducir abstracciones únicamente para "hacerlo más genérico".

---

# 21. Dependencias

Antes de añadir una dependencia externa:

1. Comprobar si la funcionalidad puede implementarse razonablemente con la biblioteca estándar.
2. Evaluar mantenimiento y estabilidad.
3. Comprobar que la dependencia sea necesaria para el alcance actual.
4. Evitar dependencias que introduzcan una arquitectura innecesariamente grande.
5. Justificar dependencias relevantes en la documentación.

La cantidad de dependencias debe mantenerse reducida.

---

# 22. Cambios de API

Las API internas deben mantenerse pequeñas.

Antes de cambiar una abstracción:

* Buscar sus consumidores.
* Evaluar el impacto.
* Actualizar implementaciones.
* Actualizar tests.
* Actualizar documentación relacionada.

No introducir breaking changes internos sin necesidad.

---

# 23. Git

Los commits deben representar cambios coherentes.

Preferir:

```text
feat: add Java project support
feat: add WinForms designer
fix: preserve cursor after insertion
refactor: isolate toolchain execution
test: add document undo tests
```

Evitar commits que mezclen:

```text
feature + refactor + formatting + unrelated changes
```

---

# 24. Regla para agentes de IA

Un agente debe:

1. Leer los requisitos relevantes antes de modificar código.
2. Revisar la arquitectura existente.
3. Buscar implementaciones existentes antes de crear nuevas.
4. Hacer cambios pequeños y localizados.
5. Mantener compatibilidad con las decisiones existentes.
6. Ejecutar las pruebas afectadas.
7. Verificar compilación.
8. Informar claramente cualquier limitación.

El agente no debe:

* Inventar requisitos.
* Añadir funcionalidades fuera del alcance.
* Reescribir grandes partes del proyecto sin necesidad.
* Introducir frameworks innecesarios.
* Crear sistemas de plugins prematuramente.
* Cambiar decisiones arquitectónicas sin justificación.
* Ocultar errores de compilación o tests.
* Eliminar pruebas para hacer que una implementación pase.
* Reemplazar una solución simple por una solución excesivamente abstracta.

---

# 25. Orden de prioridades

Cuando existan conflictos entre objetivos, utilizar este orden:

```text
1. Correctitud
2. Requisitos funcionales
3. Arquitectura limpia
4. Mantenibilidad
5. Rendimiento
6. Extensibilidad
7. Conveniencia
```

La extensibilidad nunca debe justificar una complejidad innecesaria en el MVP.

---

# 26. Definición de terminado

Una tarea se considera terminada cuando:

* La funcionalidad solicitada está implementada.
* El código compila.
* Las pruebas relevantes pasan.
* No se introducen regresiones conocidas.
* La funcionalidad respeta la arquitectura.
* La documentación se actualiza cuando es necesario.
* No se incluyen cambios no relacionados.

---

# 27. Regla fundamental del proyecto

Ante cualquier duda arquitectónica:

> **Primero construir la solución más pequeña que cumpla correctamente el requisito actual, manteniendo una interfaz que permita evolucionar posteriormente.**

El objetivo no es construir el IDE más grande posible.

El objetivo es construir un **núcleo de IDE pequeño, sólido y extensible**.

---
