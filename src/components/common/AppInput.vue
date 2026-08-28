<script setup lang="ts">
withDefaults(
  defineProps<{
    label?: string;
    type?: "text" | "number";
    min?: number;
    max?: number;
    step?: number;
    placeholder?: string;
    required?: boolean;
  }>(),
  {
    label: "",
    type: "text",
    min: undefined,
    max: undefined,
    step: undefined,
    placeholder: "",
    required: false,
  },
);

const model = defineModel<string | number | undefined>();
</script>

<template>
  <label class="app-input">
    <span v-if="label" class="app-input__label">{{ label }}</span>
    <input
      v-model="model"
      class="app-input__field"
      :type="type"
      :min="min"
      :max="max"
      :step="step"
      :placeholder="placeholder"
      :required="required"
    />
  </label>
</template>

<style scoped>
.app-input {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.app-input__label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-muted);
}

.app-input__field {
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  font-size: 0.875rem;
  background: var(--color-surface);
  color: var(--color-text);
}

.app-input__field:focus {
  outline: none;
  border-color: var(--color-primary);
}
</style>
