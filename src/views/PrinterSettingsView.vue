<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePrinterStore } from "../stores/printer";
import { savePrinterConfig } from "../infrastructure/tauri/printerApi";
import { commandErrorMessage } from "../infrastructure/tauri/tauriClient";
import type { Printer, PrinterConfig } from "../types";
import PrinterForm from "../components/printer/PrinterForm.vue";

const store = usePrinterStore();

const printer = ref<Printer | null>(null);
const saving = ref(false);
const message = ref<{ type: "success" | "error"; text: string } | null>(null);

async function handleSave(config: PrinterConfig): Promise<void> {
  saving.value = true;
  message.value = null;
  try {
    const saved = await savePrinterConfig(config);
    printer.value = saved;
    store.setPrinter(saved);
    store.refreshAfterSave();
    message.value = { type: "success", text: "Configuración guardada." };
  } catch (e) {
    message.value = { type: "error", text: commandErrorMessage(e) };
  } finally {
    saving.value = false;
  }
}

onMounted(async () => {
  await store.load();
  printer.value = store.printer;
});
</script>

<template>
  <div class="settings">
    <h2 class="settings__title">Configuración de impresora</h2>

    <PrinterForm
      :initial="printer"
      :submitting="saving"
      @submit="handleSave"
    />

    <p
      v-if="message"
      class="settings__message"
      :class="`settings__message--${message.type}`"
      role="status"
    >
      {{ message.text }}
    </p>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 560px;
}

.settings__title {
  margin: 0;
  font-size: 1.25rem;
}

.settings__message {
  margin: 0;
  padding: 0.75rem;
  border-radius: var(--radius);
}

.settings__message--success {
  background: var(--color-success);
  color: #fff;
}

.settings__message--error {
  background: var(--color-danger);
  color: #fff;
}
</style>
