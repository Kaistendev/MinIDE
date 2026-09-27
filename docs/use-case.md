# Casos de Uso — MiniIDE

## 1. Actores

### ACT-01 — Usuario / Desarrollador

Es la persona que utiliza MiniIDE para crear, editar, diseñar, compilar y ejecutar aplicaciones.

### ACT-02 — Toolchain .NET

Conjunto de herramientas externas utilizadas para compilar y ejecutar proyectos C# / WinForms.

### ACT-03 — JDK

Conjunto de herramientas externas utilizadas para compilar y ejecutar proyectos Java / Swing.

---

# 2. Diagrama general de casos de uso

```text
                            ┌─────────────────────┐
                            │       MiniIDE       │
                            │                     │
 Usuario ──────────────────┤ CU-01 Abrir IDE      │
     │                     │ CU-02 Crear proyecto│
     │                     │ CU-03 Abrir proyecto │
     │                     │ CU-04 Editar código  │
     │                     │ CU-05 Gestionar files│
     │                     │ CU-06 Diseñar UI     │
     │                     │ CU-07 Compilar       │
     │                     │ CU-08 Ejecutar       │
     │                     │ CU-09 Detener        │
     │                     │ CU-10 Buscar         │
     │                     │ CU-11 Guardar        │
     │                     │ CU-12 Gestionar tabs │
     │                     │ CU-13 Ver errores    │
     │                     │ CU-14 Configurar IDE │
     │                     │ CU-15 Gestionar tools│
     │                     └─────────────────────┘
     │
     └───────────────────────────────┐
                                     │
                         ┌───────────▼───────────┐
                         │    Proyecto C#        │
                         │       WinForms        │
                         └───────────────────────┘

                         ┌───────────────────────┐
                         │    Proyecto Java      │
                         │        Swing          │
                         └───────────────────────┘
```

---

# 3. Lista de casos de uso

| ID    | Caso de uso                        | Actor principal | Prioridad |
| ----- | ---------------------------------- | --------------- | --------- |
| CU-01 | Iniciar MiniIDE                    | Usuario         | Alta      |
| CU-02 | Crear proyecto                     | Usuario         | Alta      |
| CU-03 | Abrir proyecto                     | Usuario         | Alta      |
| CU-04 | Cerrar proyecto                    | Usuario         | Alta      |
| CU-05 | Explorar proyecto                  | Usuario         | Alta      |
| CU-06 | Crear archivo o directorio         | Usuario         | Alta      |
| CU-07 | Abrir documento                    | Usuario         | Alta      |
| CU-08 | Editar código                      | Usuario         | Alta      |
| CU-09 | Guardar documento                  | Usuario         | Alta      |
| CU-10 | Gestionar documentos abiertos      | Usuario         | Alta      |
| CU-11 | Buscar y reemplazar                | Usuario         | Media     |
| CU-12 | Diseñar interfaz gráfica           | Usuario         | Alta      |
| CU-13 | Modificar propiedades de controles | Usuario         | Alta      |
| CU-14 | Generar código desde diseñador     | Usuario         | Alta      |
| CU-15 | Compilar proyecto                  | Usuario         | Alta      |
| CU-16 | Ver resultados de compilación      | Usuario         | Alta      |
| CU-17 | Ejecutar proyecto                  | Usuario         | Alta      |
| CU-18 | Detener aplicación                 | Usuario         | Alta      |
| CU-19 | Detectar toolchain                 | Usuario         | Alta      |
| CU-20 | Configurar proyecto                | Usuario         | Media     |
| CU-21 | Configurar IDE                     | Usuario         | Baja      |

---

# 4. CU-01 — Iniciar MiniIDE

### Objetivo

Permitir que el usuario inicie el IDE y disponga del entorno de trabajo.

### Actor principal

Usuario.

### Precondiciones

* MiniIDE está instalado.

### Flujo principal

1. El usuario inicia MiniIDE.
2. El sistema inicializa el núcleo de la aplicación.
3. El sistema carga la configuración disponible.
4. El sistema inicializa los servicios internos.
5. El sistema muestra la ventana principal.
6. El usuario puede crear o abrir un proyecto.

### Flujos alternativos

* Si existe una configuración inválida, el sistema utilizará valores predeterminados.
* Si una toolchain no está disponible, el IDE podrá iniciarse igualmente y mostrará posteriormente la advertencia correspondiente.

### Postcondiciones

El entorno de desarrollo queda disponible.

---

# 5. CU-02 — Crear proyecto

### Objetivo

Crear un nuevo proyecto utilizando un lenguaje y framework soportados.

### Actor principal

Usuario.

### Precondiciones

* MiniIDE está iniciado.

### Flujo principal

1. El usuario selecciona "Nuevo proyecto".
2. El sistema muestra las plataformas disponibles.
3. El usuario selecciona el lenguaje.
4. El usuario selecciona el framework.
5. El usuario introduce nombre y ubicación.
6. El usuario confirma la creación.
7. El sistema valida la configuración.
8. El sistema genera la estructura del proyecto.
9. El sistema abre el proyecto.

### Opciones iniciales

```text
C# + WinForms
Java + Swing
```

### Flujos alternativos

* El nombre del proyecto es inválido.
* La ubicación no está disponible.
* El proyecto ya existe.
* La toolchain requerida no está instalada.

### Postcondiciones

Existe un proyecto válido y abierto en MiniIDE.

---

# 6. CU-03 — Abrir proyecto

### Objetivo

Cargar un proyecto existente.

### Flujo principal

1. El usuario selecciona "Abrir proyecto".
2. El sistema solicita la ubicación del proyecto.
3. El usuario selecciona el proyecto.
4. El sistema identifica su tipo.
5. El sistema carga la estructura.
6. El sistema carga la configuración.
7. El sistema muestra el proyecto en el explorador.

### Postcondiciones

El proyecto queda disponible para edición.

---

# 7. CU-04 — Cerrar proyecto

### Objetivo

Cerrar el proyecto actual.

### Flujo principal

1. El usuario selecciona "Cerrar proyecto".
2. El sistema verifica si existen documentos modificados.
3. Si existen, solicita guardar los cambios.
4. El sistema cierra los documentos.
5. El sistema libera los recursos del proyecto.
6. El sistema vuelve al estado sin proyecto.

### Resultado

El proyecto deja de estar activo.

---

# 8. CU-05 — Explorar proyecto

### Objetivo

Permitir al usuario navegar por los archivos del proyecto.

### Flujo principal

1. El usuario observa el explorador de proyectos.
2. El sistema muestra directorios y archivos.
3. El usuario expande o contrae directorios.
4. El usuario selecciona un archivo.
5. El sistema permite abrirlo.

### Resultado

El usuario puede navegar por la estructura del proyecto.

---

# 9. CU-06 — Crear archivo o directorio

### Objetivo

Modificar la estructura física del proyecto.

### Flujo principal

1. El usuario selecciona un directorio.
2. Selecciona "Nuevo archivo" o "Nuevo directorio".
3. Introduce el nombre.
4. El sistema valida el nombre.
5. El sistema crea el elemento.
6. El explorador se actualiza.

### Flujos alternativos

* El nombre ya existe.
* El nombre contiene caracteres no válidos.

---

# 10. CU-07 — Abrir documento

### Objetivo

Mostrar un archivo del proyecto en el editor.

### Flujo principal

1. El usuario selecciona un archivo.
2. El sistema carga su contenido.
3. El sistema identifica el tipo de archivo.
4. El sistema aplica la configuración de edición correspondiente.
5. El documento aparece en una pestaña.

---

# 11. CU-08 — Editar código

### Objetivo

Permitir modificar el contenido de un documento.

### Flujo principal

1. El usuario escribe o elimina texto.
2. El editor actualiza el documento.
3. El sistema actualiza la posición del cursor.
4. El sistema marca el documento como modificado.
5. El sistema actualiza el renderizado.

### Operaciones incluidas

* Insertar texto.
* Eliminar texto.
* Mover cursor.
* Seleccionar texto.
* Copiar.
* Cortar.
* Pegar.
* Deshacer.
* Rehacer.

---

# 12. CU-09 — Guardar documento

### Objetivo

Persistir los cambios realizados en un archivo.

### Flujo principal

1. El usuario selecciona "Guardar".
2. El sistema comprueba si el documento tiene cambios.
3. El sistema escribe el contenido.
4. El sistema confirma el guardado.
5. El documento deja de estar marcado como modificado.

### Alternativa

Si el documento no tiene una ruta asociada:

1. El sistema solicita una ubicación.
2. El usuario selecciona la ubicación.
3. El sistema guarda el archivo.

---

# 13. CU-10 — Gestionar documentos abiertos

### Objetivo

Permitir trabajar simultáneamente con varios documentos.

### Operaciones

* Abrir documento.
* Cambiar de pestaña.
* Cerrar pestaña.
* Reordenar pestañas.
* Guardar documento.
* Guardar todos.
* Detectar documentos modificados.

### Regla

Cerrar un documento modificado deberá solicitar confirmación antes de descartar cambios.

---

# 14. CU-11 — Buscar y reemplazar

### Objetivo

Localizar y reemplazar texto dentro de un documento.

### Flujo principal

1. El usuario abre la herramienta de búsqueda.
2. Introduce el texto.
3. El sistema localiza las coincidencias.
4. El usuario puede navegar entre resultados.
5. El usuario puede reemplazar una coincidencia o todas.

### Extensiones futuras

* Búsqueda en todo el proyecto.
* Expresiones regulares.
* Búsqueda por archivos.

---

# 15. CU-12 — Diseñar interfaz gráfica

### Objetivo

Permitir crear visualmente una interfaz para un proyecto WinForms o Swing.

### Flujo principal

1. El usuario abre un formulario mediante el diseñador.
2. El sistema carga el modelo visual.
3. El usuario selecciona un componente.
4. El usuario coloca el componente en el formulario.
5. El sistema actualiza el modelo visual.
6. El diseñador actualiza la representación visual.
7. El usuario guarda el diseño.

### Componentes iniciales

#### WinForms

* Form.
* Button.
* Label.
* TextBox.
* Panel.

#### Swing

* JFrame.
* JButton.
* JLabel.
* JTextField.
* JPanel.

---

# 16. CU-13 — Modificar propiedades de controles

### Objetivo

Permitir modificar las propiedades principales de un elemento visual.

### Flujo principal

1. El usuario selecciona un control.
2. El sistema muestra sus propiedades.
3. El usuario modifica una propiedad.
4. El sistema valida el nuevo valor.
5. El modelo visual se actualiza.
6. El diseñador refleja el cambio.

### Propiedades iniciales

* Nombre.
* Texto.
* Posición.
* Tamaño.
* Visible.
* Habilitado.

---

# 17. CU-14 — Generar código desde diseñador

### Objetivo

Convertir el modelo visual en código fuente correspondiente al framework.

### Flujo principal

1. El usuario modifica el diseño.
2. El usuario guarda el formulario.
3. El sistema transforma el modelo visual.
4. El generador correspondiente produce código.
5. El sistema actualiza el archivo generado.

### Ejemplo conceptual

```text
Modelo visual
      ↓
WinForms Generator
      ↓
C# Designer Code
```

o:

```text
Modelo visual
      ↓
Swing Generator
      ↓
Java Source Code
```

### Restricción

El generador deberá evitar sobrescribir código escrito manualmente fuera de la sección administrada por el diseñador.

---

# 18. CU-15 — Compilar proyecto

### Objetivo

Construir una aplicación a partir del código fuente.

### Flujo principal

1. El usuario selecciona "Compilar".
2. El sistema identifica el tipo de proyecto.
3. El sistema localiza la toolchain correspondiente.
4. El sistema prepara el proceso.
5. El sistema ejecuta la compilación.
6. El sistema captura la salida.
7. El sistema procesa el resultado.
8. El sistema informa si la compilación fue exitosa o falló.

### Dependencias

```text
C# → .NET SDK
Java → JDK
```

---

# 19. CU-16 — Ver resultados de compilación

### Objetivo

Mostrar al usuario el resultado de la compilación.

### El sistema deberá mostrar

* Mensajes informativos.
* Advertencias.
* Errores.
* Código de salida.
* Estado final.

Cuando sea posible, los errores deberán indicar:

```text
archivo
línea
columna
mensaje
```

### Resultado

El usuario puede identificar los problemas de compilación desde el IDE.

---

# 20. CU-17 — Ejecutar proyecto

### Objetivo

Ejecutar la aplicación desarrollada.

### Flujo principal

1. El usuario selecciona "Ejecutar".
2. El sistema verifica el estado del proyecto.
3. El sistema compila si es necesario.
4. El sistema inicia el proceso.
5. El sistema registra la salida.
6. El sistema informa que la aplicación está ejecutándose.

### C# / WinForms

El resultado será la aplicación Windows generada por .NET.

### Java / Swing

El resultado será el proceso Java correspondiente.

---

# 21. CU-18 — Detener aplicación

### Objetivo

Finalizar la aplicación ejecutada desde MiniIDE.

### Flujo principal

1. El usuario selecciona "Detener".
2. El sistema identifica el proceso asociado.
3. El sistema solicita su finalización.
4. El proceso termina.
5. El sistema actualiza el estado de ejecución.

### Resultado

La aplicación deja de ejecutarse.

---

# 22. CU-19 — Detectar toolchain

### Objetivo

Determinar si las herramientas necesarias están disponibles.

### Flujo principal

1. MiniIDE detecta el tipo de proyecto.
2. El sistema identifica la toolchain requerida.
3. El sistema busca la herramienta correspondiente.
4. El sistema verifica su disponibilidad.
5. El sistema registra la información.

### Ejemplo

```text
C# / WinForms
      ↓
.NET SDK
      ↓
Disponible
```

```text
Java / Swing
      ↓
JDK
      ↓
Disponible
```

### Alternativa

Si la herramienta no está disponible:

* El IDE muestra un diagnóstico.
* No inicia la compilación.
* Explica qué dependencia falta.

---

# 23. CU-20 — Configurar proyecto

### Objetivo

Modificar las opciones básicas del proyecto.

### Posibles configuraciones iniciales

* Nombre.
* Ruta.
* Configuración de compilación.
* Archivo de inicio.
* Directorio de salida.

### Resultado

La configuración queda persistida en el proyecto.

---

# 24. CU-21 — Configurar IDE

### Objetivo

Permitir modificar configuraciones generales de MiniIDE.

### Configuraciones iniciales

* Tema.
* Tamaño de fuente.
* Fuente del editor.
* Comportamiento de pestañas.
* Ajustes básicos del editor.

Este caso de uso puede quedar fuera del MVP si se desea reducir todavía más el alcance.

---

# 25. Relaciones principales entre casos de uso

## Crear y ejecutar proyecto

```text
CU-02 Crear proyecto
      ↓
CU-05 Explorar proyecto
      ↓
CU-07 Abrir documento
      ↓
CU-08 Editar código
      ↓
CU-09 Guardar
      ↓
CU-15 Compilar
      ↓
CU-16 Ver resultado
      ↓
CU-17 Ejecutar
      ↓
CU-18 Detener
```

## Desarrollo visual

```text
CU-02 Crear proyecto
      ↓
CU-12 Diseñar interfaz
      ↓
CU-13 Modificar propiedades
      ↓
CU-14 Generar código
      ↓
CU-15 Compilar
      ↓
CU-17 Ejecutar
```

---

# 26. Casos de uso específicos por plataforma

## C# / WinForms

```text
CU-02 Crear proyecto
        │
        ▼
C# + WinForms
        │
        ├── CU-08 Editar código
        ├── CU-12 Diseñar interfaz
        ├── CU-13 Modificar propiedades
        ├── CU-14 Generar código
        ├── CU-15 Compilar
        └── CU-17 Ejecutar
```

## Java / Swing

```text
CU-02 Crear proyecto
        │
        ▼
Java + Swing
        │
        ├── CU-08 Editar código
        ├── CU-12 Diseñar interfaz
        ├── CU-13 Modificar propiedades
        ├── CU-14 Generar código
        ├── CU-15 Compilar
        └── CU-17 Ejecutar
```

---

# 27. Casos de uso que quedan preparados para futuras versiones

La arquitectura deberá permitir incorporar posteriormente casos como:

```text
CU-22 — Instalar extensión
CU-23 — Administrar plugins
CU-24 — Crear proyecto Python
CU-25 — Crear proyecto Electron
CU-26 — Crear proyecto Tauri
CU-27 — Depurar aplicación
CU-28 — Integrar Git
CU-29 — Autocompletar código
CU-30 — Refactorizar código
```

Estos casos no forman parte del MVP.

---

# 28. Flujo principal del producto

El flujo fundamental que debe resolver MiniIDE es:

```text
                INICIAR IDE
                     │
                     ▼
              CREAR PROYECTO
                     │
              ┌──────┴──────┐
              │             │
             C#           Java
              │             │
           WinForms        Swing
              │             │
              └──────┬──────┘
                     ▼
               EDITAR CÓDIGO
                     │
                     ▼
               DISEÑAR UI
                     │
                     ▼
              GENERAR CÓDIGO
                     │
                     ▼
                 COMPILAR
                     │
              ┌──────┴──────┐
              │             │
            Éxito          Error
              │             │
              ▼             ▼
           EJECUTAR     VER ERRORES
              │
              ▼
            DETENER
```

Este flujo representa el **núcleo funcional del MVP** y debería ser posible completarlo de principio a fin tanto para C# WinForms como para Java Swing.
