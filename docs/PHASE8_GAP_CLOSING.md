# Phase 8 — Cierre de Gaps — Plan

## Contexto

La auditoría de `PHASE8_PLAN.md` detectó 5 gaps entre lo planeado y lo implementado.
El 90% de la fase está completo (router, store, composable, APIs, types, utils, styles,
3 vistas, 8/9 componentes, `main.ts`, `App.vue`, `index.html`, `package.json`).

## Gaps

| # | Gap | Descripción |
|---|-----|-------------|
| 1 | `AppModal.vue` ausente | Componente listado en el plan pero no existe |
| 2 | `tauriClient` sin tests directos | `invokeCommand`/`isTauriError` solo se cubren indirectamente |
| 3 | Smoke tests de componentes ausentes | Ningún `.spec.ts` bajo `src/components/` |
| 4 | `docs/DEVELOPMENT.md` sin actualizar | Phase 8 sin marcar ✅, sin resumen |
| 5 | `docs/ARCHITECTURE.md` desalineada | Árbol `src/`, diagrama y tabla Frontend desactualizados |

## Decisiones

1. **`AppModal.vue`**: se crea standalone sin importarlo en vistas (decisión del usuario).
   Listo para uso futuro conforme al plan original ("Modal base (preview)").
2. **`tauriClient.spec.ts`**: nuevo archivo dedicado; se mantiene el bloque
   `commandErrorMessage` existente en `printerApi.spec.ts` (no duplicar).
3. **Smoke tests**: 1 `it("mounts")` por componente (10, incluyendo `AppModal.vue`).
   Sin asserts lógicos, solo verificar que monta sin crashear. Patrón del plan: §Tests.
4. **Docs**: se añade la sección 19 a `DEVELOPMENT.md` alineada con las secciones 11-18
   existentes. En `ARCHITECTURE.md` se alinean §3, §4, §5 y §6 al estado real.

## Archivos

### Nuevos
| Archivo | Responsabilidad |
|---------|-----------------|
| `docs/PHASE8_GAP_CLOSING.md` | Plan de cierre de gaps |
| `src/components/common/AppModal.vue` | Modal base reutilizable |
| `src/infrastructure/tauri/tauriClient.spec.ts` | Tests de `invokeCommand`/`isTauriError` |
| `src/components/common/AppButton.spec.ts` | Smoke test |
| `src/components/common/AppInput.spec.ts` | Smoke test |
| `src/components/common/AppModal.spec.ts` | Smoke test |
| `src/components/printer/PrinterStatus.spec.ts` | Smoke test |
| `src/components/printer/PrinterForm.spec.ts` | Smoke test |
| `src/components/printing/PrintQuantityForm.spec.ts` | Smoke test |
| `src/components/printing/PrintProgress.spec.ts` | Smoke test |
| `src/components/printing/PrintResult.spec.ts` | Smoke test |
| `src/components/printing/LabelPreview.spec.ts` | Smoke test |

### Modificados
| Archivo | Cambio |
|---------|--------|
| `docs/DEVELOPMENT.md` | Phase 8 ✅ + resumen de implementación (sección 19) |
| `docs/ARCHITECTURE.md` | §3 diagrama, §4 árbol `src/`, §5 tabla Frontend, §6 versión pinia |
| `vitest.config.ts` | **Nuevo** — entorno jsdom para tests de componentes |
| `package.json` | + devDependency `jsdom` (requerido por `mount` de `@vue/test-utils`) |

## Tests

- `tauriClient.spec.ts`: `invokeCommand` delega a `invoke` con cmd+args; `isTauriError`
  discrimina envelopes `{ code, message }`; `commandErrorMessage` ya cubierto
  (printerApi.spec.ts).
- Smoke tests: mount de cada componente con las props mínimas requeridas.
- Entorno: `jsdom` vía `vitest.config.ts` (los `mount` de componentes necesitan DOM).

## Verificación (orden)

1. `pnpm test`
2. `pnpm build`
3. `pnpm lint` + `pnpm format`

## Notas

- `jsdom` es la única dependencia nueva, y es de desarrollo (no afecta el bundle de la app).
- `AppModal.vue` usa `<Teleport to="body">`: sus tests consultan `document.body`, no el wrapper.