<script setup lang="ts">
import type { PrintStage } from "../../composables/usePrintProgress";

defineProps<{ stage: PrintStage }>();

const stageText: Record<PrintStage, string> = {
  idle: "",
  preparing: "Preparando impresión...",
  connecting: "Conectando con la impresora...",
  sending: "Enviando datos...",
  done: "Impresión enviada correctamente.",
  error: "",
};
</script>

<template>
  <div
    v-if="stage !== 'idle' && stage !== 'error'"
    class="print-progress"
    role="status"
    aria-live="polite"
  >
    <span v-if="stage !== 'done'" class="print-progress__spinner" aria-hidden="true" />
    <span>{{ stageText[stage] }}</span>
  </div>
</template>

<style scoped>
.print-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
}

.print-progress__spinner {
  width: 1rem;
  height: 1rem;
  border: 2px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
