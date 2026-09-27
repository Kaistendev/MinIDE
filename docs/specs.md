# MiniIDE — Requerimientos, alcance y objetivos

## 1. Descripción general

MiniIDE es un entorno de desarrollo integrado minimalista para Windows, desarrollado completamente en Rust.

Su propósito inicial es permitir la creación, edición, diseño, compilación y ejecución de aplicaciones de escritorio utilizando:

* C# con Windows Forms.
* Java con Swing.

El núcleo de la aplicación será independiente de estos lenguajes y frameworks. La arquitectura deberá permitir incorporar nuevos lenguajes y frameworks posteriormente sin rediseñar el núcleo principal.

La primera versión priorizará la funcionalidad esencial de un IDE sobre características avanzadas.

---

# 2. Objetivos del proyecto

## 2.1 Objetivo general

Desarrollar un IDE minimalista para Windows, implementado en Rust, que permita crear y gestionar proyectos C# WinForms y Java Swing mediante una arquitectura modular y extensible.

## 2.2 Objetivos específicos

1. Implementar un editor de código nativo desarrollado en Rust.

2. Proporcionar gestión básica de proyectos y archivos.

3. Permitir crear proyectos C# para Windows Forms.

4. Permitir crear proyectos Java para Swing.

5. Incorporar un diseñador visual básico para interfaces gráficas WinForms y Swing.

6. Integrar las herramientas externas necesarias para compilar y ejecutar los proyectos.

7. Mostrar al usuario los errores generados durante la compilación.

8. Mantener separado el núcleo del IDE de los detalles específicos de cada lenguaje y framework.

9. Definir interfaces internas que permitan incorporar nuevos lenguajes y frameworks en versiones futuras.

10. Mantener el producto pequeño, comprensible y orientado a las capacidades fundamentales de un IDE.

---

# 3. Alcance

## 3.1 Incluido en el alcance

La primera versión del sistema incluirá:

### Editor de código

* Creación y edición de archivos.
* Apertura de archivos existentes.
* Guardado de archivos.
* Múltiples documentos mediante pestañas.
* Cursor y selección de texto.
* Copiar, cortar y pegar.
* Deshacer y rehacer.
* Buscar y reemplazar.
* Números de línea.
* Indentación básica.
* Resaltado sintáctico para C# y Java.

### Gestión de proyectos

* Creación de proyectos.
* Apertura de proyectos.
* Explorador de archivos.
* Creación, eliminación y renombrado de archivos.
* Organización básica de directorios.
* Configuración básica del proyecto.

### Soporte C# / WinForms

* Creación de proyectos WinForms.
* Edición de archivos `.cs`.
* Generación de estructura inicial de proyectos.
* Integración con la herramienta de compilación de .NET.
* Ejecución del proyecto.
* Visualización de errores de compilación.
* Diseñador visual básico.
* Controles WinForms básicos.
* Edición de propiedades básicas.
* Generación de código asociada al diseñador.

### Soporte Java / Swing

* Creación de proyectos Java.
* Edición de archivos `.java`.
* Generación de estructura inicial de proyectos.
* Integración con el JDK.
* Compilación mediante las herramientas correspondientes.
* Ejecución del proyecto.
* Visualización de errores de compilación.
* Diseñador visual básico para Swing.
* Componentes Swing básicos.
* Edición de propiedades básicas.
* Generación de código asociada al diseñador.

### Arquitectura extensible

El sistema deberá contar con interfaces internas para representar, como mínimo:

* Lenguajes.
* Frameworks o toolkits.
* Proyectos.
* Compilación.
* Ejecución.
* Diseñadores.

Estas interfaces permitirán incorporar posteriormente otras tecnologías.

---

# 4. Fuera del alcance inicial

Las siguientes características no forman parte de la primera versión:

* Sistema de plugins externos.
* Marketplace de extensiones.
* Soporte para Python.
* Soporte para Electron.
* Soporte para Tauri.
* Soporte para WPF.
* Soporte para JavaFX.
* Soporte para otros lenguajes.
* Depurador avanzado.
* Integración completa con Git.
* Refactorización avanzada.
* Autocompletado avanzado basado en análisis semántico.
* Language Server Protocol completo.
* Sistema de extensiones de terceros.
* Desarrollo multiplataforma.

Estas funcionalidades podrán incorporarse en futuras versiones sin modificar el concepto fundamental del núcleo.

---

# 5. Requerimientos funcionales

## RF-01 — Gestión de la aplicación

El sistema deberá permitir:

* Crear una nueva sesión de trabajo.
* Abrir y cerrar proyectos.
* Cerrar correctamente la aplicación.
* Mantener el estado básico de la interfaz durante la sesión.

## RF-02 — Gestión de documentos

El sistema deberá permitir:

* Crear documentos nuevos.
* Abrir documentos existentes.
* Editar documentos.
* Guardar documentos.
* Guardar documentos con una nueva ruta.
* Cerrar documentos.
* Detectar modificaciones no guardadas.

## RF-03 — Edición de texto

El editor deberá proporcionar:

* Inserción de texto.
* Eliminación de texto.
* Movimiento del cursor.
* Selección de texto.
* Copiar.
* Cortar.
* Pegar.
* Deshacer.
* Rehacer.
* Buscar.
* Reemplazar.
* Seleccionar todo.

## RF-04 — Pestañas

El sistema deberá permitir:

* Abrir múltiples archivos simultáneamente.
* Cambiar entre documentos.
* Cerrar pestañas.
* Indicar documentos modificados.

## RF-05 — Explorador de proyectos

El sistema deberá mostrar una estructura jerárquica de archivos y directorios del proyecto.

El usuario deberá poder:

* Crear archivos.
* Crear directorios.
* Eliminar archivos.
* Renombrar archivos.
* Abrir archivos desde el explorador.

## RF-06 — Creación de proyectos

El sistema deberá permitir crear proyectos seleccionando:

* Lenguaje.
* Framework o toolkit.
* Nombre del proyecto.
* Ubicación.

La primera versión deberá permitir como mínimo:

* C# + WinForms.
* Java + Swing.

## RF-07 — Soporte C#

El sistema deberá:

* Reconocer archivos C#.
* Aplicar resaltado sintáctico básico.
* Crear proyectos C# compatibles con .NET.
* Invocar el proceso de compilación.
* Capturar la salida de compilación.
* Mostrar errores y advertencias.
* Ejecutar la aplicación generada.

## RF-08 — Soporte WinForms

El sistema deberá proporcionar un diseñador visual básico que permita:

* Crear un formulario.
* Cambiar tamaño del formulario.
* Añadir controles.
* Seleccionar controles.
* Mover controles.
* Cambiar tamaño de controles.
* Modificar propiedades básicas.
* Generar o actualizar código C# asociado al diseño.

## RF-09 — Soporte Java

El sistema deberá:

* Reconocer archivos Java.
* Aplicar resaltado sintáctico básico.
* Crear proyectos Java.
* Invocar las herramientas de compilación disponibles.
* Capturar errores de compilación.
* Ejecutar aplicaciones Java.

## RF-10 — Soporte Swing

El sistema deberá proporcionar un diseñador visual básico que permita:

* Crear una ventana.
* Añadir componentes Swing.
* Seleccionar componentes.
* Posicionar componentes.
* Cambiar propiedades básicas.
* Generar o actualizar el código Java asociado al diseño.

## RF-11 — Compilación

El sistema deberá proporcionar una operación de compilación accesible desde la interfaz.

La ejecución deberá:

1. Identificar el tipo de proyecto.
2. Seleccionar el proveedor de compilación apropiado.
3. Ejecutar la herramienta externa correspondiente.
4. Capturar stdout y stderr.
5. Interpretar errores básicos.
6. Presentar el resultado al usuario.

## RF-12 — Ejecución

El usuario deberá poder ejecutar el proyecto actual.

El sistema deberá:

* Verificar que el proyecto pueda ejecutarse.
* Ejecutar el proceso correspondiente.
* Mostrar errores de ejecución.
* Permitir finalizar el proceso.

## RF-13 — Terminal / salida de procesos

El sistema deberá mostrar:

* Salida estándar.
* Errores estándar.
* Estado del proceso.
* Código de salida.

## RF-14 — Sistema de comandos

Las acciones principales del IDE deberán estar representadas como comandos internos.

Ejemplos:

* Nuevo proyecto.
* Abrir proyecto.
* Guardar.
* Guardar todo.
* Compilar.
* Ejecutar.
* Detener ejecución.
* Deshacer.
* Rehacer.
* Buscar.

Esto permitirá que futuras funciones y extensiones reutilicen las mismas operaciones.

## RF-15 — Abstracción de lenguajes y frameworks

El núcleo deberá definir una interfaz común para los soportes tecnológicos.

Como mínimo deberá poder representar:

* Identificador del lenguaje.
* Extensiones de archivo.
* Información del proyecto.
* Capacidad de compilación.
* Capacidad de ejecución.
* Soporte de diseñador, cuando exista.

Los módulos C# y Java deberán implementar dicha abstracción.

## RF-16 — Detección de toolchains

El sistema deberá poder detectar la disponibilidad de las herramientas necesarias para trabajar con cada plataforma soportada.

Como mínimo:

* .NET SDK para C# / WinForms.
* JDK para Java / Swing.

Cuando una herramienta no esté disponible, el IDE deberá informar claramente al usuario.

---

# 6. Requerimientos no funcionales

## RNF-01 — Lenguaje de implementación

El IDE deberá estar implementado en Rust.

## RNF-02 — Arquitectura

El sistema deberá utilizar una arquitectura modular con separación entre:

* Interfaz de usuario.
* Núcleo del editor.
* Gestión de documentos.
* Gestión de proyectos.
* Diseñadores.
* Integración con toolchains.
* Ejecución de procesos.
* Soporte de lenguajes y frameworks.

## RNF-03 — Extensibilidad

Agregar un nuevo lenguaje o framework no deberá requerir modificar de forma significativa el núcleo del sistema.

La arquitectura deberá permitir incorporar posteriormente nuevos proveedores de:

* Lenguaje.
* Framework.
* Compilación.
* Ejecución.
* Diseñador.

## RNF-04 — Rendimiento

Las operaciones normales del editor deberán ejecutarse de forma fluida para proyectos pequeños y medianos.

La edición de texto no deberá bloquear la interfaz durante operaciones normales.

Las tareas potencialmente lentas, como compilación y ejecución, deberán ejecutarse fuera del hilo principal de la interfaz.

## RNF-05 — Responsividad

La interfaz deberá continuar respondiendo mientras se ejecutan:

* Compilaciones.
* Procesos externos.
* Operaciones de búsqueda sobre proyectos.
* Ejecución de aplicaciones.

## RNF-06 — Estabilidad

Un error de compilación o ejecución de un proyecto no deberá provocar el cierre del IDE.

Los procesos externos deberán poder finalizarse y sus errores deberán manejarse de forma controlada.

## RNF-07 — Mantenibilidad

El código deberá estar organizado en módulos independientes y con responsabilidades claramente definidas.

La lógica relacionada con un lenguaje concreto no deberá propagarse por todo el núcleo.

## RNF-08 — Bajo acoplamiento

El núcleo deberá depender de abstracciones y no de implementaciones concretas de C# o Java.

## RNF-09 — Portabilidad arquitectónica

Aunque la primera versión estará orientada a Windows, las partes centrales del núcleo deberán evitar dependencias innecesarias de Windows cuando no sean requeridas.

## RNF-10 — Seguridad

Los proyectos y procesos externos deberán ejecutarse respetando las capacidades normales del usuario.

El IDE no deberá ejecutar automáticamente comandos arbitrarios sin una acción explícita del usuario.

## RNF-11 — Usabilidad

Las operaciones comunes deberán ser accesibles mediante:

* Menús.
* Atajos de teclado.
* Botones.
* Menús contextuales cuando corresponda.

## RNF-12 — Consistencia

Las operaciones equivalentes de C# y Java deberán presentar una experiencia de usuario consistente aunque internamente utilicen toolchains diferentes.

---

# 7. Restricciones del sistema

## RS-01

El IDE estará inicialmente orientado exclusivamente a Windows.

## RS-02

La compilación de proyectos dependerá de herramientas externas instaladas en el sistema.

## RS-03

El IDE no implementará inicialmente compiladores propios para C# o Java.

## RS-04

El IDE no implementará inicialmente un runtime propio para ejecutar las aplicaciones desarrolladas.

## RS-05

El soporte de nuevos lenguajes será diseñado como una extensión de las abstracciones existentes, no como modificaciones arbitrarias al núcleo.

---

# 8. Arquitectura conceptual

La arquitectura inicial deberá seguir este modelo:

```
             ┌──────────────────────────┐
             │          MiniIDE         │
             │          Rust            │
             └────────────┬─────────────┘
                          │
         ┌────────────────┼────────────────┐
         │                │                │
         ▼                ▼                ▼
       UI             Editor           Project
         │                │                │
         └────────────────┼────────────────┘
                          │
                   Core abstractions
                          │
          ┌───────────────┴───────────────┐
          │                               │
  CSharpSupport                     JavaSupport
          │                               │
    ┌─────┴─────┐                   ┌─────┴─────┐
    │           │                   │           │
 WinForms     .NET                Swing        JDK
 Designer    Build               Designer     Build
```

El núcleo deberá conocer las interfaces y contratos, mientras que los módulos de soporte deberán conocer los detalles particulares de cada tecnología.

---

# 9. Criterio de éxito del MVP

La primera versión podrá considerarse funcional cuando un usuario pueda:

1. Abrir el IDE.
2. Crear un proyecto C# WinForms.
3. Crear y modificar un formulario visualmente.
4. Editar el código generado.
5. Compilar el proyecto.
6. Ejecutar el programa.
7. Detectar y visualizar errores básicos.

Y también:

1. Crear un proyecto Java Swing.
2. Diseñar una interfaz básica.
3. Editar el código Java.
4. Compilar el proyecto.
5. Ejecutar el programa.
6. Visualizar errores de compilación.

La arquitectura deberá permitir que posteriormente se añadan nuevos lenguajes y frameworks sin reemplazar el núcleo del IDE.
