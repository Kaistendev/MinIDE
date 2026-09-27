# MiniIDE — Política de calidad

> Definida en la tarea T-003. Es la fuente de verdad para formato, lint y warnings del proyecto.

## 1. Alcance

La política aplica a todos los targets del paquete `miniide`: librería, binario, tests de integración y ejemplos.

No aplica a código generado por proveedores externos (plantillas de C# o Java), que siguen las convenciones de su lenguaje.

---

## 2. Formato

Configuración: `rustfmt.toml`

```toml
edition = "2021"
max_width = 100
```

Regla: el formato lo decide `rustfmt`, no la revisión manual.

```text
cargo fmt          # aplica el formato
cargo fmt --check  # verifica el formato sin modificar archivos
```

`cargo fmt --check` debe terminar correctamente antes de considerar terminada una tarea.

---

## 3. Lint

Configuración: `clippy.toml`

```toml
msrv = "1.95"
```

`msrv` coincide con `rust-version` de `Cargo.toml`. Clippy no debe sugerir APIs más nuevas que la versión mínima soportada.

Regla: `clippy::all` se trata como error, no como aviso.

---

## 4. Política de warnings

Configuración: tabla `[lints]` de `Cargo.toml`, aplicada a todos los targets.

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "deny"
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"
```

Implicaciones:

- `unsafe_code` está prohibido en todo el proyecto. Si una dependencia lo exige, debe aislarse detrás de un módulo propio y justificarse.
- No se admite `dbg!`, `todo!` ni `unimplemented!` en código entregable.
- Un warning de Clippy es un fallo, no una sugerencia.

Los warnings nativos de `rustc` se mantienen como `warn` por defecto. El gate de la suite los convierte en error:

```text
cargo clippy --all-targets -- -D warnings
```

---

## 5. Comandos de verificación

Toda tarea se cierra con estos comandos en verde, en este orden:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo test` incluye los tests que verifican esta misma política (`tests/quality_policy.rs`), de modo que eliminar la configuración o la documentación rompe la suite.

---

## 6. Excepciones

- Una excepción de lint debe ser local, con `#[allow(...)]` y una razón concreta en el sitio.
- No se permite `#[allow]` a nivel de crate o de módulo para silenciar la política de forma general.
- Si una regla resulta incorrecta o inaplicable, se modifica la configuración y se documenta el motivo aquí, en lugar de desactivar el lint archivo por archivo.

---

## 7. Trazabilidad

| Origen | Relación con esta política |
|---|---|
| Constitución 13 | Una tarea no está terminada si el proyecto no compila limpiamente. |
| Constitución 20 | Código simple, legible y sin abstracciones prematuras. |
| RNF-07 | Mantenibilidad: formato y lint consistentes y verificables. |
| AGENTS.md 19 | Tests obligatorios para la lógica crítica. |
| AGENTS.md 20 | Responsabilidades claras y documentación de decisiones no obvias. |
| AGENTS.md 24 | El agente debe verificar compilación y tests antes de informar. |
| T-093 | Gate final: `cargo test`, `cargo fmt --check` y `cargo clippy`. |

---

## 8. Nota sobre el toolchain

`rust-toolchain.toml` no se fija todavía. La versión activa es la que reporta `rustc --version`, y la compatibilidad mínima declarada es `1.95`, la que exige `eframe` (`docs/frontend-plan.md` FD-01). Antes de añadir el stack de la interfaz era `1.75`, y subirla fue el precio de no empezar con una versión de eframe de 2024.

Fijar el toolchain se hará cuando exista una razón concreta, por ejemplo una versión de .NET SDK o JDK condicionada por el entorno de compilación.
