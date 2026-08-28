<script setup lang="ts">
import { ref } from "vue";
import AppButton from "../common/AppButton.vue";
import AppInput from "../common/AppInput.vue";

const emit = defineEmits<{ print: [quantity: number] }>();

defineProps<{
  disabled?: boolean;
  loading?: boolean;
}>();

const quantity = ref(100);

function handleSubmit(): void {
  if (!quantity.value || quantity.value < 1) return;
  emit("print", Number(quantity.value));
}
</script>

<template>
  <form class="print-quantity" @submit.prevent="handleSubmit">
    <AppInput
      :model-value="quantity"
      label="Cantidad de etiquetas"
      type="number"
      :min="1"
      placeholder="100"
      @update:model-value="quantity = Number($event)"
    />
    <AppButton type="submit" variant="primary" :disabled="disabled" :loading="loading">
      Imprimir
    </AppButton>
  </form>
</template>

<style scoped>
.print-quantity {
  display: flex;
  align-items: flex-end;
  gap: 1rem;
}

.print-quantity > :first-child {
  flex: 1;
}
</style>
