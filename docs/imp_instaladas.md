### TAREA: Corregir impresión de código de barras - Impresora Zebra

**CONTEXTO ACTUAL / BUG REPORTADO:**
La aplicación actualmente imprime enviando comandos directos a la impresora (modo RAW / passthrough directo a Zebra).
Esto está causando que el código de barras se genere más ancho / deformado.

Cuando el mismo contenido se imprime desde un archivo .TXT eligiendo una impresora instalada en Windows (usando el driver de Windows, no el directo de Zebra), el código de barras sale con el ancho correcto.

**OBJETIVO:**
Deshabilitar la configuración de impresora directa dentro del programa. La aplicación NO debe tener una impresora hardcodeada ni usar modo RAW directo a Zebra.

**REQUERIMIENTO FUNCIONAL:**

1. Antes de mandar a imprimir, la aplicación debe mostrar el diálogo de selección de impresoras instaladas en el sistema operativo (usar `PrintDialog` / impresoras de Windows).
2. El usuario debe elegir manualmente la impresora instalada en su computadora.
3. La impresión debe enviarse usando el driver de Windows de esa impresora seleccionada, NO como comando ZPL directo.
4. El archivo a imprimir es un archivo TXT (que ya contiene el formato). Debe imprimirse tal cual lo haría el Bloc de Notas de Windows.

**CRITERIO DE ACEPTACIÓN:**

- Puedo seleccionar cualquier impresora de Windows antes de imprimir.
- El código de barras impreso desde la app tiene el mismo ancho que el impreso abriendo el TXT y dándole Ctrl+P.
- La app no tiene ninguna impresora configurada por defecto en el código.

Implementa el cambio y muéstrame el diff de `PrintService.cs` / `handler de impresión`.
