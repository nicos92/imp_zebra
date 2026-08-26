# Prompt maestro para desarrollar la aplicación de impresión de etiquetas Zebra

## 1. Rol que debes asumir

Actúa como un **arquitecto de software senior y desarrollador full-stack especializado en Rust, Tauri 2, Vue.js, TypeScript, SQLite y sistemas de impresión industrial Zebra mediante ZPL/ZPL II**.

Debes diseñar y desarrollar la aplicación siguiendo:

* Clean Architecture.
* SOLID.
* Separation of Concerns.
* Dependency Inversion.
* Domain-Driven Design aplicado de forma pragmática.
* Repository Pattern cuando corresponda.
* Result Pattern para errores de negocio.
* Inyección de dependencias.
* Código testeable.
* Bajo acoplamiento entre dominio, SQLite, Tauri y la impresora.
* Frontend Vue desacoplado de la implementación concreta del backend.
* Backend Rust como única autoridad para las reglas de negocio.
* Migraciones SQLite versionadas.
* Logging estructurado.
* Manejo explícito de errores.
* Tests unitarios e integración.
* Código preparado para futuras ampliaciones.

No quiero una implementación monolítica ni colocar toda la lógica dentro de `main.rs`, comandos Tauri o componentes Vue.

Antes de implementar, debes analizar la arquitectura y producir un plan técnico. Después debes implementarlo de forma incremental.

---

## 2. Objetivo de la aplicación

La aplicación será una aplicación de escritorio desarrollada con:

* Tauri 2.
* Backend Rust.
* Frontend Vue.js + TypeScript.
* SQLite.
* Impresoras térmicas Zebra compatibles con ZPL II.

La aplicación debe permitir configurar una impresora Zebra y generar etiquetas con códigos secuenciales.

Modelos de impresora posibles:

* Zebra ZT410.
* Zebra ZT411.
* Zebra ZT230 u otro modelo Zebra compatible.
* Otros modelos Zebra que soporten ZPL II.

La aplicación no debe quedar acoplada a un modelo específico de impresora.

---

## 3. Funcionamiento principal

El usuario debe poder indicar únicamente:

> Cantidad de etiquetas a imprimir.

Por ejemplo:

```text
Cantidad: 100
```

La aplicación debe determinar automáticamente qué códigos corresponden.

La secuencia comienza en:

```text
Z0000001
```

y continúa:

```text
Z0000002
Z0000003
Z0000004
...
Z0000100
```

Si posteriormente el usuario solicita:

```text
200
```

la aplicación debe continuar:

```text
Z0000101
Z0000102
...
Z0000300
```

El usuario **NO debe introducir manualmente el código inicial**.

---

## 4. Regla de secuencia

La secuencia válida es:

```text
Z0000001
```

hasta:

```text
Z9999999
```

Cuando se llegue a:

```text
Z9999999
```

el siguiente código debe ser:

```text
Z0000001
```

La generación de códigos debe estar implementada como una regla de dominio independiente de:

* Vue.
* Tauri.
* SQLite.
* Zebra.
* TCP.
* ZPL.

Debe ser posible probarla mediante tests unitarios sin necesidad de una base de datos ni una impresora.

Ejemplos:

```text
Z0000001 -> Z0000002
Z0000099 -> Z0000100
Z9999998 -> Z9999999
Z9999999 -> Z0000001
```

La función debe rechazar códigos inválidos.

---

## 5. Regla fundamental de persistencia

El contador debe almacenarse persistentemente en SQLite.

Nunca debe depender de:

* una variable global de Rust,
* una variable Vue,
* localStorage,
* sessionStorage,
* memoria RAM,
* un contador del frontend.

Si la aplicación se cierra:

```text
contador actual
        ↓
SQLite
        ↓
la aplicación vuelve a iniciarse
        ↓
continúa desde el último estado persistido
```

---

## 6. Consideración crítica sobre la impresión

Analiza cuidadosamente el problema de consistencia entre:

1. generar códigos,
2. persistir la secuencia,
3. enviar ZPL,
4. recibir confirmación del sistema operativo/network,
5. impresión física real de la Zebra.

No asumas que:

```text
TCP send() exitoso
```

significa necesariamente:

```text
etiqueta físicamente impresa
```

Especialmente cuando se utiliza comunicación RAW TCP hacia el puerto 9100.

Diseña el sistema de manera que pueda manejar estados de impresión.

Por ejemplo:

```text
Pending
Printing
Completed
Failed
```

o una estrategia equivalente.

Analiza también el problema:

```text
La aplicación envía la etiqueta
↓
la impresora la imprime
↓
la aplicación se cierra antes de guardar el estado
```

y el problema inverso:

```text
La aplicación guarda el estado
↓
la impresión falla
```

Debes documentar qué garantía de consistencia puede ofrecer realmente la aplicación.

No prometas "exactly once printing" si el protocolo de impresión utilizado no permite garantizarlo.

---

## 7. Impresión por lote

El usuario puede solicitar:

```text
100 etiquetas
```

La aplicación debe generar:

```text
Z0000001
Z0000002
...
Z0000100
```

Luego:

```text
200 etiquetas
```

debe generar:

```text
Z0000101
...
Z0000300
```

La arquitectura debe soportar lotes de tamaño razonable sin generar innecesariamente toda la información en el frontend.

La lógica de generación de secuencias debe estar en Rust.

---

## 8. Formato físico de las etiquetas

El rollo tiene aproximadamente:

```text
10 cm de ancho
```

y debe contener dos etiquetas por fila.

Cada etiqueta debe tener aproximadamente:

```text
5 cm de ancho
5 cm de alto
```

Por lo tanto:

```text
┌──────────────────────────────────────────┐
│                  10 cm                   │
├──────────────────────┬───────────────────┤
│                      │                   │
│      Etiqueta 1      │    Etiqueta 2    │
│                      │                   │
│      Z0000001        │    Z0000002      │
│                      │                   │
├──────────────────────┼───────────────────┤
│                      │                   │
│      Etiqueta 3      │    Etiqueta 4    │
│                      │                   │
└──────────────────────┴───────────────────┘
```

La distribución debe ser:

```text
1   2
3   4
5   6
7   8
...
```

Es decir:

* etiqueta impar → posición izquierda.
* etiqueta par → posición derecha.

No debe configurarse como una impresión vertical de una etiqueta debajo de otra.

---

## 9. Resolución de impresión

Las impresoras objetivo son principalmente de:

```text
203 DPI
```

Aproximadamente:

```text
5 cm ≈ 1.9685 pulgadas
1.9685 × 203 ≈ 400 dots
```

Por lo tanto, inicialmente considera:

```text
Etiqueta:
400 × 400 dots

Dos etiquetas:
800 × 400 dots
```

Sin embargo, **no hardcodees estos valores sin una capa de configuración**.

La configuración de la impresora debe contemplar al menos:

```text
DPI
ancho de etiqueta
alto de etiqueta
cantidad de columnas
orientación
método de conexión
```

El sistema debe permitir posteriormente soportar otras resoluciones, por ejemplo:

```text
203 DPI
300 DPI
600 DPI
```

---

## 10. ZPL

La aplicación debe generar ZPL II.

El ZPL debe ser generado exclusivamente en el backend Rust.

No quiero construir ZPL complejo directamente dentro de componentes Vue.

Crear una abstracción específica, por ejemplo:

```text
ZplLabelGenerator
```

o equivalente.

Debe recibir datos de dominio y producir:

```text
String
```

con ZPL válido.

La generación debe ser testeable sin impresora.

---

## 11. Contenido de cada etiqueta

Cada etiqueta debe contener:

### Título

Debe mostrar la fecha y hora de impresión.

Ejemplo:

```text
26/08/2026 07:15:32
```

La fecha y hora deben obtenerse en el backend Rust, no confiar en que Vue envíe la hora.

Debe definirse claramente:

* zona horaria utilizada,
* formato,
* comportamiento si cambia la configuración regional.

No hardcodear la fecha en el frontend.

---

## 12. Código de barras

El código debe imprimirse utilizando:

```text
Code 128
```

El contenido debe ser:

```text
Z0000001
```

por ejemplo.

Debajo del código de barras debe aparecer el código como texto legible:

```text
Z0000001
```

La etiqueta conceptualmente debe ser:

```text
┌──────────────────────┐
│  26/08/2026 07:15:32 │
│                      │
│      █████████       │
│      █████████       │
│      █████████       │
│                      │
│       Z0000001       │
└──────────────────────┘
```

El diseño debe dejar margen suficiente para evitar que:

* el barcode toque los bordes,
* el texto sea cortado,
* el barcode quede fuera del área imprimible,
* las dos etiquetas se superpongan.

---

## 13. Configuración de impresora

La aplicación debe permitir configurar la impresora.

Como mínimo:

```text
Nombre
Modelo
DPI
Ancho
Alto
Método de conexión
Dirección IP
Puerto
```

Para la primera versión, priorizar:

```text
TCP/IP
```

con:

```text
Puerto 9100
```

pero diseñar la arquitectura para que posteriormente pueda existir:

```text
USB
Serial
otro transporte
```

No colocar TCP directamente dentro del servicio de dominio.

Crear una abstracción como:

```text
PrinterTransport
```

o equivalente.

Por ejemplo:

```text
trait PrinterTransport {
    async fn send(&self, data: &[u8]) -> Result<(), PrinterError>;
}
```

La implementación TCP debe estar en infraestructura.

---

## 14. Arquitectura general

Utilizar una arquitectura similar a:

```text
Frontend Vue
      │
      │ invoke()
      ▼
Tauri Commands
      │
      ▼
Application Layer
      │
      ▼
Domain Layer
      │
      ├──────────────► Repository
      │
      └──────────────► Printer abstraction
                         │
                         ▼
                   Infrastructure
                    ├── SQLite
                    └── Zebra TCP
```

La dependencia debe apuntar hacia abstracciones.

El dominio no debe conocer:

```text
Tauri
SQLite
TCP
Vue
ZPL transport
```

---

## 15. Backend Rust

Separar claramente:

```text
src-tauri/
```

en capas.

Proponer una estructura similar a:

```text
src-tauri/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   │
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── printer_commands.rs
│   │   ├── print_commands.rs
│   │   └── configuration_commands.rs
│   │
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entities/
│   │   │   ├── mod.rs
│   │   │   ├── printer.rs
│   │   │   ├── print_job.rs
│   │   │   └── sequence.rs
│   │   │
│   │   ├── value_objects/
│   │   │   ├── mod.rs
│   │   │   ├── barcode.rs
│   │   │   └── printer_config.rs
│   │   │
│   │   ├── repositories/
│   │   │   ├── mod.rs
│   │   │   ├── sequence_repository.rs
│   │   │   ├── printer_repository.rs
│   │   │   └── print_job_repository.rs
│   │   │
│   │   └── services/
│   │       ├── mod.rs
│   │       ├── sequence_service.rs
│   │       └── label_service.rs
│   │
│   ├── application/
│   │   ├── mod.rs
│   │   ├── dto/
│   │   │   ├── mod.rs
│   │   │   ├── printer_dto.rs
│   │   │   └── print_dto.rs
│   │   │
│   │   └── use_cases/
│   │       ├── mod.rs
│   │       ├── configure_printer.rs
│   │       ├── get_printer_config.rs
│   │       ├── test_printer.rs
│   │       ├── preview_label.rs
│   │       └── print_labels.rs
│   │
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs
│   │   │   ├── migrations.rs
│   │   │   └── repositories/
│   │   │       ├── mod.rs
│   │   │       ├── sqlite_sequence_repository.rs
│   │   │       ├── sqlite_printer_repository.rs
│   │   │       └── sqlite_print_job_repository.rs
│   │   │
│   │   ├── printer/
│   │   │   ├── mod.rs
│   │   │   ├── tcp_transport.rs
│   │   │   └── zebra_printer.rs
│   │   │
│   │   └── zpl/
│   │       ├── mod.rs
│   │       ├── generator.rs
│   │       └── label_layout.rs
│   │
│   ├── errors/
│   │   ├── mod.rs
│   │   ├── domain_error.rs
│   │   ├── application_error.rs
│   │   └── infrastructure_error.rs
│   │
│   └── state/
│       ├── mod.rs
│       └── app_state.rs
│
├── migrations/
│   └── ...
│
└── Cargo.toml
```

Puedes modificar esta estructura si existe una mejor alternativa, pero debes justificar cualquier cambio arquitectónico importante.

No crees archivos innecesarios únicamente para cumplir una estructura artificial.

---

## 16. Frontend Vue

Utilizar:

```text
Vue 3
TypeScript
Composition API
```

Preferentemente:

```text
Pinia
```

solamente cuando realmente sea necesario para estado compartido.

Evitar un store global para datos que solamente pertenecen a una vista.

Proponer una estructura similar:

```text
src/
├── main.ts
├── App.vue
│
├── components/
│   ├── printer/
│   │   ├── PrinterForm.vue
│   │   ├── PrinterStatus.vue
│   │   └── PrinterTestButton.vue
│   │
│   ├── printing/
│   │   ├── PrintQuantityForm.vue
│   │   ├── PrintProgress.vue
│   │   └── PrintResult.vue
│   │
│   └── common/
│       ├── AppButton.vue
│       ├── AppInput.vue
│       └── AppModal.vue
│
├── views/
│   ├── DashboardView.vue
│   ├── PrinterSettingsView.vue
│   └── PrintingView.vue
│
├── application/
│   ├── printer/
│   │   ├── configurePrinter.ts
│   │   ├── getPrinterConfig.ts
│   │   └── testPrinter.ts
│   │
│   └── printing/
│       ├── printLabels.ts
│       └── previewLabel.ts
│
├── infrastructure/
│   └── tauri/
│       ├── tauriClient.ts
│       ├── printerApi.ts
│       └── printingApi.ts
│
├── domain/
│   ├── printer/
│   │   └── printer.ts
│   └── printing/
│       └── printJob.ts
│
├── types/
│   └── ...
│
└── styles/
    └── ...
```

El frontend no debe generar ZPL ni generar números secuenciales.

---

## 17. Tauri Commands

Los comandos Tauri deben ser una capa delgada.

No colocar lógica de negocio dentro de:

```rust
#[tauri::command]
```

Los comandos deben:

1. recibir DTOs,
2. validar lo mínimo necesario para deserialización,
3. invocar un caso de uso,
4. devolver DTO/resultados serializables.

Ejemplos de comandos:

```text
get_printer_config
save_printer_config
test_printer_connection
get_current_sequence
preview_label
print_labels
get_print_job
```

Considerar si realmente todos son necesarios.

---

## 18. Casos de uso

Diseñar los casos de uso de forma atómica.

Como mínimo:

### ConfigurePrinter

Configura la impresora.

### GetPrinterConfiguration

Obtiene la configuración actual.

### TestPrinterConnection

Comprueba si la impresora puede recibir datos.

### GetCurrentSequence

Obtiene información de la secuencia.

### PreviewLabel

Genera una representación previa de una etiqueta.

### PrintLabels

Recibe:

```text
quantity
```

y se encarga del proceso completo.

Debe coordinar:

```text
obtener secuencia
↓
generar códigos
↓
crear print job
↓
generar ZPL
↓
enviar a impresora
↓
actualizar estado
↓
actualizar secuencia
```

La lógica concreta debe determinarse después de analizar correctamente la consistencia transaccional.

---

## 19. SQLite

Utilizar SQLite como persistencia local.

Preferentemente utilizar:

```text
SQLx
```

con:

```text
SQLite
```

y migraciones.

La conexión debe estar encapsulada en infraestructura.

No permitir que entidades del dominio dependan de `sqlx`.

---

## 20. Esquema inicial de base de datos

Diseñar y justificar las tablas.

Como punto de partida considerar:

```text
printers
print_jobs
print_job_items
sequence_state
```

Pero no asumir que esta estructura es obligatoria.

Analizar si conviene almacenar:

```text
último código utilizado
```

o:

```text
siguiente código
```

y explicar cuál opción reduce errores.

También analizar si los códigos impresos deben registrarse individualmente o si basta con almacenar rangos:

```text
start_code
end_code
quantity
```

Para lotes grandes, considerar almacenamiento por rango en lugar de una fila por etiqueta si eso simplifica y mejora el rendimiento.

---

## 21. Concurrencia

La aplicación debe impedir que dos solicitudes de impresión simultáneas obtengan el mismo rango.

Ejemplo:

Solicitud A:

```text
100 etiquetas
```

Solicitud B:

```text
200 etiquetas
```

No puede ocurrir:

```text
A -> Z0000001...Z0000100
B -> Z0000001...Z0000200
```

Debe existir un mecanismo seguro de reserva de rangos.

Ejemplo:

```text
A -> Z0000001...Z0000100
B -> Z0000101...Z0000300
```

aunque las solicitudes lleguen prácticamente al mismo tiempo.

Investigar y utilizar correctamente las transacciones SQLite.

---

## 22. Rollover

La lógica debe soportar solicitudes que crucen:

```text
Z9999999
```

Ejemplo:

```text
último código:
Z9999998

cantidad:
3
```

resultado:

```text
Z9999999
Z0000001
Z0000002
```

Esto debe estar cubierto por tests.

---

## 23. Layout ZPL

Crear una abstracción de layout.

Por ejemplo:

```text
LabelLayout
```

que permita configurar:

```text
label_width
label_height
dpi
columns
gap
margins
barcode_height
barcode_width
font_size
```

Para 203 DPI y 5 × 5 cm utilizar inicialmente valores equivalentes a aproximadamente:

```text
400 × 400 dots
```

y dos columnas:

```text
800 × 400 dots
```

Pero validar mediante documentación de Zebra y permitir ajustes de calibración.

---

## 24. ZPL esperado

El agente debe investigar y utilizar correctamente comandos ZPL II apropiados para:

* inicio/fin de formato,
* posición de origen,
* texto,
* fuentes,
* Code 128,
* impresión de múltiples etiquetas,
* cantidad de copias si corresponde,
* orientación,
* ancho y alto.

No inventar comandos ZPL.

El generador debe producir ZPL legible.

Ejemplo conceptual:

```text
^XA

... etiqueta izquierda ...

... etiqueta derecha ...

^XZ
```

La implementación exacta debe ser validada contra la documentación de Zebra.

---

## 25. Distribución de etiquetas

Para un lote:

```text
1
2
3
4
5
6
```

el ZPL debe producir:

```text
fila 1:
1 | 2

fila 2:
3 | 4

fila 3:
5 | 6
```

Para una cantidad impar:

```text
1
2
3
```

debe producir:

```text
fila 1:
1 | 2

fila 2:
3 | vacío
```

No debe imprimirse una etiqueta inexistente en la segunda posición.

---

## 26. Configuración persistente

La configuración de impresora debe persistirse en SQLite.

Ejemplo conceptual:

```text
Printer
├── id
├── name
├── model
├── dpi
├── width
├── height
├── columns
├── connection_type
├── ip_address
├── port
├── created_at
└── updated_at
```

No guardar secretos innecesariamente.

Si en el futuro aparece autenticación, analizar cómo proteger las credenciales.

---

## 27. Errores

Crear errores específicos.

Por ejemplo:

```text
InvalidQuantity
InvalidBarcode
SequenceOverflow
PrinterNotConfigured
PrinterConnectionFailed
PrinterTimeout
PrinterUnavailable
ZplGenerationFailed
DatabaseError
PrintJobFailed
```

No devolver strings arbitrarios desde todos los lugares.

Los errores internos deben poder convertirse en errores serializables para Tauri.

El frontend debe recibir errores estructurados.

Ejemplo conceptual:

```json
{
  "code": "PRINTER_CONNECTION_FAILED",
  "message": "No fue posible conectarse con la impresora."
}
```

No mostrar stack traces al usuario.

Los detalles técnicos deben registrarse mediante logging.

---

## 28. Logging

Utilizar logging estructurado en Rust.

Considerar:

```text
tracing
tracing-subscriber
```

Registrar eventos importantes:

```text
printer configured
printer connection test
print job created
print job started
print job completed
print job failed
sequence reserved
sequence updated
database error
printer connection error
```

No registrar información sensible innecesaria.

---

## 29. Tests

Crear tests desde el principio.

Como mínimo:

## Sequence

Probar:

```text
Z0000001 -> Z0000002
Z0000099 -> Z0000100
Z9999998 -> Z9999999
Z9999999 -> Z0000001
```

## Batch generation

Probar:

```text
cantidad = 1
cantidad = 2
cantidad = 100
cantidad = 9999999
```

y cantidades inválidas.

## Rollover

Probar:

```text
Z9999998 + 3
```

resultado:

```text
Z9999999
Z0000001
Z0000002
```

## Layout

Verificar:

```text
1 -> izquierda
2 -> derecha
3 -> izquierda
4 -> derecha
```

## ZPL

Probar que el generador produzca ZPL válido para:

```text
Z0000001
```

y que contenga:

* fecha,
* hora,
* barcode Code 128,
* texto legible.

## Printer

Testear mediante mocks/fakes.

Nunca hacer que los tests unitarios dependan de una Zebra física.

---

## 30. Abstracciones para tests

Crear interfaces/traits donde exista una dependencia externa.

Ejemplo:

```text
Clock
PrinterTransport
SequenceRepository
PrinterRepository
PrintJobRepository
```

Esto permitirá utilizar:

```text
FakeClock
FakePrinterTransport
InMemorySequenceRepository
```

durante los tests.

No utilizar mocks indiscriminadamente.

Utilizar abstracciones solamente cuando aporten aislamiento o inversión de dependencias.

---

## 31. Preview

Crear una funcionalidad para generar una vista previa de la etiqueta.

El usuario debería poder visualizar conceptualmente:

```text
fecha/hora
barcode
texto
```

antes de imprimir.

No es obligatorio renderizar ZPL exactamente como lo haría una Zebra en la primera versión.

Separar:

```text
modelo de etiqueta
```

de:

```text
renderizado ZPL
```

para poder agregar posteriormente un preview más avanzado.

---

## 32. Interfaz de usuario

La aplicación debe tener como mínimo:

## Pantalla principal

Mostrar:

```text
Impresora:
Zebra ZT410

Estado:
Conectada / Desconectada

Próximo código:
Z0000101
```

Y:

```text
Cantidad de etiquetas:
[ 100 ]

[ IMPRIMIR ]
```

## Configuración

Permitir configurar:

```text
Nombre
Modelo
DPI
IP
Puerto
ancho
alto
```

y botón:

```text
Probar conexión
```

## Historial

Considerar una vista de trabajos de impresión:

```text
Fecha
Cantidad
Código inicial
Código final
Estado
Impresora
```

---

## 33. Estado de impresión

Mostrar al usuario:

```text
Preparando impresión...
Generando etiquetas...
Conectando con impresora...
Enviando datos...
Impresión enviada correctamente.
```

En caso de error:

```text
No fue posible completar la impresión.
```

El usuario debe poder consultar detalles técnicos mediante una vista de diagnóstico si corresponde.

---

## 34. Seguridad y validación

Validar en backend:

```text
quantity > 0
```

y establecer un máximo razonable configurable.

No confiar en validaciones del frontend.

Validar:

```text
IP
puerto
DPI
dimensiones
tipo de conexión
```

Evitar que una entrada mal formada termine directamente en una operación de red.

---

## 35. Dependencias

Evaluar cuidadosamente las dependencias necesarias.

Como punto de partida analizar:

```text
tauri
tokio
serde
serde_json
sqlx
chrono
thiserror
anyhow
tracing
tracing-subscriber
```

y las necesarias para comunicación TCP.

No agregar crates solamente porque son populares.

Cada dependencia debe tener una justificación.

Mantener `Cargo.toml` limpio.

---

## 36. Creación inicial del proyecto

Antes de ejecutar comandos:

1. verificar versiones instaladas;
2. comprobar Node.js;
3. comprobar pnpm;
4. comprobar Rust;
5. comprobar Cargo;
6. comprobar Tauri CLI;
7. comprobar requisitos del sistema;
8. verificar que Vue + TypeScript esté correctamente configurado.

Después crear el proyecto utilizando las herramientas oficiales actuales de Tauri 2.

No asumir comandos obsoletos.

Si la sintaxis de `create-tauri-app` cambió respecto de versiones anteriores, utilizar la sintaxis correspondiente a la versión instalada.

Documentar todos los comandos utilizados.

---

## 37. Configuración del proyecto

Preparar:

```text
package.json
tsconfig
vite
eslint
formatter
Cargo.toml
tauri.conf.json
capabilities
```

manteniendo separación entre frontend y backend.

No instalar Prettier si ya existe otra solución de formatting configurada y funcional.

Mantener consistencia con el tooling existente.

---

## 38. Orden de implementación

No desarrollar todo de una sola vez.

Utilizar estas fases:

## Fase 1 — Scaffolding

Crear:

```text
proyecto
frontend
backend
estructura de carpetas
configuración inicial
```

## Fase 2 — Dominio

Implementar:

```text
Barcode
Sequence
PrintJob
Printer configuration
```

junto con sus tests.

## Fase 3 — SQLite

Implementar:

```text
database connection
migrations
repositories
transactions
```

## Fase 4 — ZPL

Implementar:

```text
ZplGenerator
LabelLayout
Code128
two-column layout
```

con tests.

## Fase 5 — Printer Transport

Implementar:

```text
PrinterTransport
TcpPrinterTransport
```

y prueba de conexión.

## Fase 6 — Application Layer

Implementar:

```text
ConfigurePrinter
TestPrinterConnection
GetCurrentSequence
PreviewLabel
PrintLabels
```

## Fase 7 — Tauri

Crear los commands.

## Fase 8 — Vue

Crear:

```text
Printer Settings
Printing
Preview
History
Status
```

## Fase 9 — Integración

Probar:

```text
Vue
↓
Tauri
↓
Rust
↓
SQLite
↓
ZPL
↓
TCP
↓
Zebra
```

## Fase 10 — Hardening

Agregar:

```text
error handling
logging
recovery
concurrency
edge cases
tests
documentation
```

---

## 39. Regla importante sobre el agente

No quiero que simplemente generes todos los archivos inmediatamente.

Para cada fase:

1. explica qué problema se está resolviendo;
2. explica la decisión arquitectónica;
3. enumera los archivos que se crearán/modificarán;
4. explica la responsabilidad de cada archivo;
5. implementa;
6. ejecuta los tests;
7. compila;
8. corrige errores;
9. verifica que no se hayan introducido dependencias innecesarias;
10. continúa con la siguiente fase.

Si detectas una decisión arquitectónica incorrecta, detente y propón una corrección antes de continuar.

---

## 40. Regla de no sobreingeniería

La aplicación debe ser profesional, pero no quiero sobreingeniería.

No crear:

```text
factories
abstract factories
interfaces
traits
services
repositories
managers
helpers
utils
```

sin una razón concreta.

Cada abstracción debe responder a una necesidad real:

* inversión de dependencias,
* testabilidad,
* aislamiento de infraestructura,
* regla de negocio,
* reutilización,
* separación de responsabilidades.

---

## 41. Resultado esperado de la primera fase

Antes de comenzar a escribir código funcional, debes entregarme:

### A. Arquitectura

Un diagrama textual completo:

```text
Vue
 ↓
Tauri Commands
 ↓
Application
 ↓
Domain
 ↓
Infrastructure
 ├── SQLite
 ├── Zebra TCP
 └── ZPL
```

### B. Estructura completa de carpetas

Mostrar el árbol final propuesto.

### C. Lista de archivos

Para cada archivo explicar:

```text
nombre
responsabilidad
dependencias permitidas
dependencias prohibidas
```

### D. Dependencias

Mostrar:

```text
Cargo.toml
package.json
```

con explicación de cada dependencia relevante.

### E. Base de datos

Mostrar:

```text
tablas
columnas
índices
foreign keys
constraints
migraciones
```

### F. Flujo de impresión

Explicar:

```text
Usuario
 ↓
cantidad
 ↓
PrintLabels
 ↓
reserva de secuencia
 ↓
generación de códigos
 ↓
generación ZPL
 ↓
creación PrintJob
 ↓
transporte Zebra
 ↓
resultado
 ↓
persistencia
```

### G. Manejo de errores

Mostrar cómo se propagan:

```text
Infrastructure Error
↓
Application Error
↓
Tauri Error
↓
Vue Error
```

### H. Tests

Mostrar qué tests existirán y qué responsabilidad tiene cada uno.

---

## 42. Criterios de aceptación

La aplicación será considerada correctamente implementada cuando pueda:

1. Configurar una Zebra por IP.
2. Probar la conexión.
3. Persistir la configuración.
4. Mostrar la próxima secuencia.
5. Recibir una cantidad de etiquetas.
6. Generar automáticamente los códigos.
7. Generar ZPL II.
8. Imprimir dos etiquetas por fila.
9. Utilizar aproximadamente 5 × 5 cm por etiqueta a 203 DPI.
10. Imprimir Code 128.
11. Mostrar el código debajo del barcode.
12. Mostrar fecha y hora.
13. Mantener la secuencia después de cerrar la aplicación.
14. Evitar duplicar rangos ante solicitudes concurrentes.
15. Manejar el rollover `Z9999999 -> Z0000001`.
16. Manejar cantidades impares.
17. Registrar los trabajos de impresión.
18. Informar correctamente los errores.
19. Poder ejecutar tests sin una Zebra conectada.
20. Poder cambiar posteriormente el transporte de impresión sin modificar el dominio.
21. Poder cambiar el modelo de Zebra sin modificar la lógica de negocio.
22. Poder cambiar SQLite por otra persistencia en el futuro sin modificar el dominio.
23. Mantener el frontend sin conocimiento de SQLite, TCP o ZPL.
24. Mantener Tauri como una capa de transporte entre Vue y la aplicación Rust.

---

## 43. Restricción arquitectónica fundamental

La dependencia debe ser siempre:

```text
Presentation
    ↓
Application
    ↓
Domain
```

y:

```text
Infrastructure
    ↓
implementa interfaces utilizadas por Application/Domain
```

Nunca:

```text
Domain → SQLite
Domain → Tauri
Domain → TCP
Domain → Vue
Domain → ZPL transport
```

El dominio debe permanecer independiente.

---

## 44. Documentación

Crear documentación técnica suficiente para que otro desarrollador pueda entender:

```text
README.md
ARCHITECTURE.md
DATABASE.md
PRINTING.md
ZPL.md
DEVELOPMENT.md
```

Documentar especialmente:

* arquitectura;
* configuración;
* base de datos;
* secuencia;
* comunicación con Zebra;
* ZPL;
* troubleshooting;
* desarrollo local;
* ejecución de tests;
* build de producción.

---

## 45. Regla final

Antes de implementar cualquier cosa, analiza la arquitectura completa y explícame las decisiones.

No quiero una solución rápida basada en:

```text
main.rs enorme
```

ni:

```text
Vue → invoke → SQL
```

ni:

```text
Vue → genera ZPL → imprime
```

La solución debe tratar la impresión como una capacidad de infraestructura y la generación de códigos como una regla de dominio.

El resultado final debe ser una aplicación mantenible, testeable y preparada para crecer.

Comienza por:

1. analizar los requisitos;
2. identificar riesgos técnicos;
3. proponer la arquitectura;
4. proponer el árbol de carpetas;
5. proponer las entidades;
6. proponer los casos de uso;
7. proponer el esquema SQLite;
8. proponer las interfaces/traits;
9. proponer el flujo transaccional de impresión;
10. indicar los comandos exactos para crear el proyecto;
11. solamente después comenzar la implementación.
