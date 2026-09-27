1. **Stack:** el IDE y su núcleo se desarrollarán en **Rust** y estará orientado inicialmente a **Windows**.
2. **Soporte inicial:** únicamente **C# + WinForms** y **Java + Swing**.
3. **Extensibilidad:** lenguaje, framework y toolchain deberán integrarse mediante abstracciones pequeñas y claras.
4. **Minimalismo:** no se implementarán funcionalidades fuera del alcance sin una decisión explícita.
5. **Core:** la lógica de dominio no dependerá directamente de una tecnología concreta.
6. **UI:** la interfaz debe permanecer separada del estado y la lógica del núcleo.
7. **Procesos:** compilación y ejecución externas no bloquearán la interfaz.
8. **Calidad:** código simple, modular, legible y sin abstracciones prematuras.
9. **Dependencias:** solo se añadirán dependencias cuando aporten valor claro al MVP.
10. **Tests:** toda lógica crítica del core tendrá **unit tests**.
11. **Integración:** compilación, ejecución, generación de código y toolchains tendrán **integration tests** donde sea viable.
12. **Regresiones:** una funcionalidad nueva no deberá romper casos de uso existentes.
13. **Build:** no se considerará terminada una tarea si el proyecto no compila limpiamente.
14. **Cambios:** cada implementación deberá ser pequeña, trazable y alineada con requisitos/casos de uso.
15. **Futuro:** otros lenguajes y frameworks se podrán añadir posteriormente sin rediseñar el core.
