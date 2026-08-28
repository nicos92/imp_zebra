<script setup lang="ts">
import { computed } from "vue";
import type { Printer } from "../../types";
import type { PrinterConnectionStatus } from "../../stores/printer";
import { formatConnectionType } from "../../utils/format";

const props = defineProps<{
  printer: Printer | null;
  status: PrinterConnectionStatus;
  nextCode: string;
}>();

const statusLabel = computed(() => {
  switch (props.status) {
    case "connected":
      return "Conectada";
    case "disconnected":
      return "Desconectada";
    case "testing":
      return "Probando conexión...";
    default:
      return "Desconocido";
  }
});
</script>

<template>
  <section class="printer-status">
    <div v-if="printer" class="printer-status__info">
      <p class="printer-status__name">{{ printer.name }}</p>
      <p class="printer-status__model">{{ printer.model }}</p>
      <p class="printer-status__detail">
        {{ printer.ip_address }}:{{ printer.port }} ·
        {{ formatConnectionType(printer.connection_type) }} · {{ printer.dpi }} DPI
      </p>
    </div>
    <p v-else class="printer-status__empty">
      No hay impresora configurada.
      <RouterLink to="/settings">Configurar</RouterLink>
    </p>

    <div class="printer-status__footer">
      <span class="printer-status__next"
        >Próximo código: <strong>{{ nextCode }}</strong></span
      >
      <span class="printer-status__badge" :class="`printer-status__badge--${status}`">
        {{ statusLabel }}
      </span>
    </div>
  </section>
</template>

<style scoped>
.printer-status {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.printer-status__name {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
}

.printer-status__model,
.printer-status__detail {
  margin: 0.125rem 0 0;
  color: var(--color-muted);
  font-size: 0.875rem;
}

.printer-status__empty {
  margin: 0;
  color: var(--color-muted);
}

.printer-status__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.printer-status__next {
  font-size: 0.875rem;
}

.printer-status__badge {
  padding: 0.25rem 0.625rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
}

.printer-status__badge--connected {
  background: var(--color-success);
  color: #fff;
}

.printer-status__badge--disconnected,
.printer-status__badge--unknown {
  background: var(--color-muted);
  color: #fff;
}

.printer-status__badge--testing {
  background: var(--color-warning);
  color: #fff;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}
</style>
