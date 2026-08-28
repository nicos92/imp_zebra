<script setup lang="ts">
import { reactive, watch } from "vue";
import type { PrinterConfig } from "../../types";
import AppButton from "../common/AppButton.vue";
import AppInput from "../common/AppInput.vue";

const props = defineProps<{
  initial?: PrinterConfig | null;
  submitting?: boolean;
  testing?: boolean;
}>();

const emit = defineEmits<{
  submit: [config: PrinterConfig];
  test: [config: PrinterConfig];
}>();

function defaults(): PrinterConfig {
  return {
    name: "",
    model: "ZT410",
    dpi: 203,
    label_width_mm: 50,
    label_height_mm: 50,
    columns: 2,
    connection_type: "tcp",
    ip_address: "",
    port: 9100,
  };
}

const form = reactive<PrinterConfig>(defaults());

function applyInitial(value: PrinterConfig | null | undefined): void {
  Object.assign(form, value ?? defaults());
}

watch(
  () => props.initial,
  (value) => applyInitial(value),
  { immediate: true },
);

function handleSubmit(): void {
  emit("submit", { ...form });
}
</script>

<template>
  <form class="printer-form" @submit.prevent="handleSubmit">
    <AppInput
      :model-value="form.name"
      label="Nombre"
      required
      @update:model-value="form.name = String($event)"
    />
    <AppInput
      :model-value="form.model"
      label="Modelo"
      @update:model-value="form.model = String($event)"
    />

    <div class="printer-form__grid">
      <AppInput
        :model-value="form.dpi"
        label="DPI"
        type="number"
        :min="1"
        :max="600"
        @update:model-value="form.dpi = Number($event)"
      />
      <AppInput
        :model-value="form.label_width_mm"
        label="Ancho (mm)"
        type="number"
        :min="1"
        :step="0.1"
        @update:model-value="form.label_width_mm = Number($event)"
      />
      <AppInput
        :model-value="form.label_height_mm"
        label="Alto (mm)"
        type="number"
        :min="1"
        :step="0.1"
        @update:model-value="form.label_height_mm = Number($event)"
      />
      <AppInput
        :model-value="form.columns"
        label="Columnas"
        type="number"
        :min="1"
        :max="4"
        @update:model-value="form.columns = Number($event)"
      />
      <AppInput
        :model-value="form.ip_address"
        label="Dirección IP"
        placeholder="192.168.1.100"
        required
        @update:model-value="form.ip_address = String($event)"
      />
      <AppInput
        :model-value="form.port"
        label="Puerto"
        type="number"
        :min="1"
        :max="65535"
        @update:model-value="form.port = Number($event)"
      />
    </div>

    <div class="printer-form__actions">
      <AppButton variant="secondary" :loading="testing" @click="emit('test', { ...form })">
        Probar conexión
      </AppButton>
      <AppButton type="submit" variant="primary" :loading="submitting"> Guardar </AppButton>
    </div>
  </form>
</template>

<style scoped>
.printer-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.printer-form__grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.printer-form__actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
