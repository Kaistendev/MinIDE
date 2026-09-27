# MiniIDE — Estrategia de tests

> Definida en la tarea T-004. Complementa a `docs/quality.md`, que define formato y lint.
> La estrategia conceptual está en `plan.md` sección 7; este documento la concreta para el código.

## 1. Objetivo

- La lógica del core se prueba con unit tests, sin UI y sin procesos externos.
- Las integraciones que producen efectos observables en el sistema se prueban con integration tests.
- Ningún test puede pasar sin haber ejecutado realmente lo que dice comprobar.

---

## 2. Dónde vive cada tipo de test

| Tipo | Ubicación | Qué cubre | Dependencias externas |
|---|---|---|---|
| Unitario | `src/**/*.rs`, bloque `#[cfg(test)] mod tests` | Modelo de dominio, documento, editor, validación, parseo de diagnósticos | Ninguna |
| Ejemplo compilado | Documentación del crate en `src/lib.rs` | API pública tal como la ve un consumidor | Ninguna |
| Arquitectura | `tests/module_boundaries.rs` | Que los módulos base sean públicos y que dependan en la dirección correcta | Ninguna |
| Conformidad | `tests/quality_policy.rs`, `tests/test_strategy.rs` | Que la política de calidad y esta estrategia sigan existiendo | Ninguna |
| Integración local | `tests/*.rs` | Flujos de editor y documento, filesystem, workspace, ciclo de proyecto | Ninguna o solo un directorio temporal |
| Integración toolchain | `tests/*.rs` con `#[ignore]` | Build y run reales con .NET SDK o JDK | Sí, toolchain del sistema |
| Golden | `tests/golden/**` | Salida exacta de los generadores de código | Ninguna |

Regla estructural verificada por `tests/test_strategy.rs`: todo módulo de `src/` distinto de `lib.rs` y `main.rs` debe contener un bloque `#[cfg(test)]`. Un módulo sin comportamiento que no lo justifique se elimina, no se documenta la excepción.

---

## 3. Cómo se ejecutan

```text
cargo test
```

Ejecuta unitarios, ejemplos compilados e integración local. Los tests que dependen de una toolchain externa se marcan con `#[ignore]` y no se ejecutan en este comando.

```text
cargo test -- --ignored
```

Ejecuta únicamente los tests de toolchain. Deben iniciarse solo cuando la toolchain está disponible.

```text
cargo test -- --include-ignored
```

Ejecuta todo. Es el comando de la verificación final, junto con `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings` definidos en `docs/quality.md`.

---

## 4. Tests que dependen de toolchains externas

Reglas obligatorias:

1. Se marcan con `#[ignore = "motivo"]` indicando qué falta, por ejemplo `requires .NET SDK`.
2. Detectan la disponibilidad de la herramienta antes de actuar.
3. Si la herramienta no está, el test falla o se omite de forma explícita. Nunca se considera un falso éxito.
4. No modifican el sistema fuera de un directorio temporal.
5. No dejan procesos activos al terminar.

```rust
#[test]
#[ignore = "requires .NET SDK"]
fn csharp_project_builds() {
    // ...
}
```

---

## 5. Golden tests

Los generadores de WinForms y Swing se validan comparando su salida con una referencia versionada.

- referencias: `tests/golden/<framework>/<caso>.txt`
- comparación: `assert_eq!(generado, esperado_leído)`
- regla: el archivo golden se actualiza solo cuando el cambio del generador es intencionado, y esa actualización se revisa como parte de la tarea

No se usa una librería de snapshots. El archivo golden es texto plano y la comparación es explícita.

---

## 6. Tests de regresión

Todo bug corregido del documento, editor o proyecto deja un test que reproduce el fallo original. El test se escribe antes de la corrección cuando es viable y permanece en la suite.

Los bugs de la suite se registran en `docs/tasks.md` junto a la tarea que los cubre, para que el caso siga siendo rastreable.

---

## 7. Nombres

El nombre del test describe el comportamiento esperado, no la función que se prueba.

```text
rejects_parent_directory_escape
contains_excludes_the_end_position
status_can_be_set_and_cleared
```

Evitar `test1`, `works` o nombres que repitan el nombre de la función.

---

## 8. Trazabilidad

| Requisito | Tests previstos | Tarea |
|---|---|---|
| RF-02 Gestión de documentos | Unitarios de buffer, estado modificado y persistencia; integración de guardado | T-016, T-028, T-037 |
| RF-03 Edición de texto | Unitarios de inserción, borrado, cursor, selección, undo/redo, búsqueda | T-028 |
| RF-06 Creación de proyectos | Unitarios de validación de nombre, ruta y combinación de plataforma | T-016 |
| RF-11 Compilación | Unitarios de `BuildResult` y parseo de diagnósticos; integración de build | T-016, T-057, T-070 |
| RF-15 Abstracciones | Unitarios de contratos; arquitectura de `tests/module_boundaries.rs` | T-052, T-092 |
| Constitución 11 | Build, run, generación y toolchains tienen integration tests | T-065, T-078 |
| Constitución 12 | Una funcionalidad nueva no rompe casos existentes | Fase 9 |

---

## 9. Regla de cierre

Una tarea está terminada cuando `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test` terminan correctamente, y los tests relevantes del requisito indicado están en verde.
