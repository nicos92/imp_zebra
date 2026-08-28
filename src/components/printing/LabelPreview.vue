<script setup lang="ts">
import type { LabelPreview } from "../../types";

defineProps<{
  preview: LabelPreview | null;
  loading: boolean;
}>();

defineEmits<{ close: [] }>();
</script>

<template>
  <div class="label-preview">
    <h3 class="label-preview__title">Vista previa</h3>
    <p v-if="loading" class="label-preview__muted">Generando vista previa...</p>
    <template v-else-if="preview">
      <p class="label-preview__code">
        <strong>{{ preview.code }}</strong>
      </p>
      <p class="label-preview__muted">Timestamp: {{ preview.timestamp }}</p>
    </template>
    <p v-else class="label-preview__muted">La vista previa se genera al configurar la impresora.</p>

    <slot />
  </div>
</template>

<style scoped>
.label-preview {
  padding: 1rem;
  background: var(--color-surface);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius);
}

.label-preview__title {
  margin: 0 0 0.5rem;
  font-size: 1rem;
}

.label-preview__code {
  margin: 0.25rem 0;
  font-size: 1.25rem;
}

.label-preview__muted {
  margin: 0.25rem 0;
  color: var(--color-muted);
}
</style>
