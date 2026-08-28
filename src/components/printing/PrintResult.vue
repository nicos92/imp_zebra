<script setup lang="ts">
import type { PrintResult } from "../../types";

defineProps<{ result: PrintResult | null }>();
</script>

<template>
  <div v-if="result" class="print-result">
    <h3 class="print-result__title">Impresión enviada</h3>
    <p class="print-result__line">
      <strong>{{ result.quantity }}</strong> etiqueta(s)
    </p>
    <p class="print-result__line">
      Códigos: <code>{{ result.start_code }}</code> → <code>{{ result.end_code }}</code>
    </p>
    <p class="print-result__line">
      Estado: <span class="print-result__status">{{ result.status }}</span>
    </p>
    <p class="print-result__meta">Trabajo: {{ result.job_id }}</p>
  </div>
</template>

<style scoped>
.print-result {
  padding: 1rem;
  background: var(--color-surface);
  border: 1px solid var(--color-success);
  border-radius: var(--radius);
}

.print-result__title {
  margin: 0 0 0.5rem;
  font-size: 1rem;
  color: var(--color-success);
}

.print-result__line {
  margin: 0.25rem 0;
}

.print-result__status {
  font-weight: 600;
}

.print-result__meta {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  color: var(--color-muted);
}
</style>
