# Phase 3: SQLite — Plan de Implementación

## Objetivo
Fortalecer la capa SQLite: transacciones atómicas para reserva de secuencia, migraciones con SQLx runner, y tests de integración con SQLite in-memory.

## Estado Actual
- ✅ `connection.rs` — Pool creation con WAL, busy_timeout, foreign_keys
- ✅ `migrations.rs` — DDL hardcoded ejecuta `CREATE TABLE IF NOT EXISTS`
- ✅ 3 repos — `SqliteSequenceRepository`, `SqlitePrinterRepository`, `SqlitePrintJobRepository`
- ❌ Migraciones no usan `sqlx::migrate!()` — el archivo `.sql` no se ejecuta
- ❌ Sin transacciones atómicas en reserva de secuencia
- ❌ Sin tests de infraestructura (0 tests en database/)

---

## Tarea 1: Migraciones con SQLx Runner
**Archivo:** `src-tauri/src/infrastructure/database/migrations.rs`

**Problema:** El DDL está hardcoded en Rust. El archivo `migrations/001_initial.sql` existe pero no se usa.

**Solución:** Reemplazar `sqlx::query(DLL)` con `sqlx::migrate!()` macro.

**Cambios:**
```rust
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), InfrastructureError> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| InfrastructureError::Database(format!("Migration failed: {}", e)))?;
    Ok(())
}
```

**Notas:**
- `sqlx::migrate!("../../migrations")` — path relativo al `Cargo.toml`
- La macro incluye los archivos `.sql` en el binario en compile-time
- Requiere feature `migrate` en Cargo.toml (ya habilitado)
- Eliminar el DDL hardcoded del `run_migrations` actual

**Tests:**
- `test_migrations_run_on_empty_db` — Verificar que las tablas existen después de migrar

---

## Tarea 2: Transacción Atómica para Reserva de Secuencia
**Archivo:** `src-tauri/src/infrastructure/database/repositories/sqlite_sequence_repository.rs`

**Problema:** Actualmente hace `SELECT` + `UPDATE` separados. Sin transacción, dos concurrentes pueden leer el mismo código.

**Solución:** Agregar método `reserve_range` que ejecuta SELECT + UPDATE dentro de `BEGIN IMMEDIATE`.

**Cambios en el trait `SequenceRepository`:**
```rust
// domain/repositories/sequence_repository.rs
#[async_trait]
pub trait SequenceRepository: Send + Sync {
    async fn get_last_used_code(&self) -> Result<String, DomainError>;
    async fn update_last_used_code(&self, code: &str) -> Result<(), DomainError>;
    async fn reserve_range(&self, quantity: u64) -> Result<(String, String, String), DomainError>;
    //                                                     ↑ (start_code, end_code, new_last_used)
}
```

**Implementación en `sqlite_sequence_repository.rs`:**
```rust
async fn reserve_range(&self, quantity: u64) -> Result<(String, String, String), DomainError> {
    let mut tx = self.pool.begin().await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    let row: (String,) = sqlx::query_as(
        "SELECT last_used_code FROM sequence_state WHERE id = 1"
    )
    .fetch_one(&mut *tx).await
    .map_err(|e| DomainError::Database(e.to_string()))?;

    let current = row.0;
    let mut seq = Sequence::from_code(&current)
        .map_err(|e| DomainError::Database(e.to_string()))?;

    let (start, end, _codes) = seq.reserve_range(quantity)
        .map_err(|e| DomainError::Database(e.to_string()))?;

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE sequence_state SET last_used_code = ?, updated_at = ? WHERE id = 1")
        .bind(seq.last_used_code())
        .bind(&now)
        .execute(&mut *tx).await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    tx.commit().await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    Ok((start, end, seq.last_used_code()))
}
```

**Por qué `BEGIN IMMEDIATE`:**
- SQLite adquiere lock de escritura al BEGIN, no al primer write
- Evita `SQLITE_BUSY` en el UPDATE si hay readers concurrents
- La reserva es una operación crítica — mejor fallar rápido que esperar

**Tests:**
- `test_reserve_range_atomic` — Reserva 5 códigos, verifica secuencia
- `test_reserve_range_updates_db` — Verifica que el `last_used_code` se persiste
- `test_reserve_range_wrap` — Reserva cerca de Z9999999, verifica wrap

---

## Tarea 3: Helper de Test con SQLite In-Memory
**Archivo:** `src-tauri/src/infrastructure/database/test_helpers.rs` (nuevo)

**Problema:** Tests de repos necesitan DB real pero rápida.

**Solución:** Función helper que crea pool in-memory con schema.

```rust
#[cfg(test)]
pub async fn create_test_pool() -> SqlitePool {
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run test migrations");

    pool
}
```

**Notas:**
- `max_connections(1)` — SQLite in-memory solo permite 1 conexión
- `:memory:` — DB temporal, se destruye al cerrar el pool
- Ejecuta migraciones reales para tener schema completo

---

## Tarea 4: Tests de SqliteSequenceRepository
**Archivo:** `src-tauri/src/infrastructure/database/repositories/sqlite_sequence_repository.rs`

| Test | Descripción |
|------|-------------|
| `test_get_initial_code` | Lee `last_used_code` recién creado — espera `Z0000000` |
| `test_update_code` | Actualiza a `Z0000050`, verifica que se persiste |
| `test_reserve_range` | Reserva 5, verifica start/end y DB actualizada |
| `test_reserve_range_wrap` | Reserva desde `Z9999998`, verifica wrap a `Z0000002` |

---

## Tarea 5: Tests de SqlitePrinterRepository
**Archivo:** `src-tauri/src/infrastructure/database/repositories/sqlite_printer_repository.rs`

| Test | Descripción |
|------|-------------|
| `test_save_and_find_by_id` | Guarda printer, recupera por ID |
| `test_find_all` | Guarda 2 printers, verifica find_all retorna 2 |
| `test_update` | Guarda, actualiza config, verifica cambios |
| `test_delete` | Guarda, elimina, verifica None |
| `test_find_nonexistent` | Busca ID inexistente, retorna None |

---

## Tarea 6: Tests de SqlitePrintJobRepository
**Archivo:** `src-tauri/src/infrastructure/database/repositories/sqlite_print_job_repository.rs`

| Test | Descripción |
|------|-------------|
| `test_save_and_find_by_id` | Guarda print job, recupera por ID |
| `test_update_status` | Actualiza status a `completed`, verifica cambio |
| `test_find_recent` | Guarda 3 jobs, find_recent(2) retorna 2 más recientes |
| `test_find_recent_empty` | find_recent en DB vacía retorna Vec vacío |

---

## Archivos a Modificar

| Archivo | Acción |
|---------|--------|
| `src-tauri/src/infrastructure/database/migrations.rs` | Reemplazar DDL hardcoded con `sqlx::migrate!()` |
| `src-tauri/src/infrastructure/database/mod.rs` | Agregar `test_helpers` module |
| `src-tauri/src/infrastructure/database/test_helpers.rs` | **Nuevo** — `create_test_pool()` |
| `src-tauri/src/domain/repositories/sequence_repository.rs` | Agregar `reserve_range` al trait |
| `src-tauri/src/infrastructure/database/repositories/sqlite_sequence_repository.rs` | Implementar `reserve_range` con transacción + tests |
| `src-tauri/src/domain/services/sequence_service.rs` | Actualizar `reserve_range` para usar nuevo método del repo |
| `src-tauri/src/infrastructure/database/repositories/sqlite_printer_repository.rs` | Agregar tests |
| `src-tauri/src/infrastructure/database/repositories/sqlite_print_job_repository.rs` | Agregar tests |

---

## Tests Esperados

| Module | Tests |
|--------|-------|
| `infrastructure::database` | 1 (migrations) |
| `infrastructure::database::repositories::sqlite_sequence_repository` | 4 |
| `infrastructure::database::repositories::sqlite_printer_repository` | 5 |
| `infrastructure::database::repositories::sqlite_print_job_repository` | 4 |
| **Total nuevos** | **14** |
| **Total acumulado** | **53 + 14 = 67** |

---

## Orden de Ejecución

1. Tarea 3 — Crear test helper (sin dependencias)
2. Tarea 1 — Migraciones con sqlx::migrate!() + test migrations
3. Tarea 2 — Transacción atómica en SequenceRepository
4. Tarea 4 — Tests de SqliteSequenceRepository
5. Tarea 5 — Tests de SqlitePrinterRepository
6. Tarea 6 — Tests de SqlitePrintJobRepository
7. `cargo test` — Verificar 67 tests
8. Actualizar `DEVELOPMENT.md` con resumen Fase 3

---

## Decisiones Arquitectónicas

1. **`BEGIN IMMEDIATE`** — Mejor fallar rápido que esperar en reserva de secuencia
2. **`sqlx::migrate!()`** — Compile-time embedding del `.sql`, no DDL hardcoded
3. **`max_connections(1)` para tests** — SQLite in-memory solo permite 1 conexión
4. **`reserve_range` en trait** — La atomicidad es responsabilidad del repositorio, no del servicio
5. **Tests con DB real** — No mocks para infraestructura; la DB in-memory es rápida y confiable

---

## Riesgos

| Riesgo | Mitigación |
|--------|------------|
| `sqlx::migrate!()` path relativo puede fallar | Verificar path con `cargo test` temprano |
| SQLite in-memory y `max_connections(1)` limitan tests | Tests secuenciales dentro de cada repo |
| `BEGIN IMMEDIATE` puede causar `SQLITE_BUSY` | `busy_timeout=5000` ya configurado en connection |
