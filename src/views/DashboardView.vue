<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { usePrinterStore } from "../stores/printer";
import { usePrintProgress } from "../composables/usePrintProgress";
import { previewLabel, printLabels } from "../infrastructure/tauri/printingApi";
import { commandErrorMessage } from "../infrastructure/tauri/tauriClient";
import type { LabelPreview as LabelPreviewData, PrintResult as PrintResultData } from "../types";
import PrinterStatus from "../components/printer/PrinterStatus.vue";
import PrintQuantityForm from "../components/printing/PrintQuantityForm.vue";
import PrintProgress from "../components/printing/PrintProgress.vue";
import PrintResult from "../components/printing/PrintResult.vue";
import LabelPreview from "../components/printing/LabelPreview.vue";

const store = usePrinterStore();
const { printer, status, nextCode } = storeToRefs(store);

const progress = usePrintProgress();
const result = ref<PrintResultData | null>(null);
const preview = ref<LabelPreviewData | null>(null);
const previewLoading = ref(false);

const canPrint = computed(() => !!printer.value);

async function loadPreview(): Promise<void> {
  if (!printer.value) {
    preview.value = null;
    return;
  }
  previewLoading.value = true;
  try {
    preview.value = await previewLabel(
      printer.value.label_width_mm,
      printer.value.label_height_mm,
      printer.value.columns,
      printer.value.dpi,
    );
  } catch (e) {
    preview.value = null;
    progress.fail(commandErrorMessage(e));
  } finally {
    previewLoading.value = false;
  }
}

async function handlePrint(quantity: number): Promise<void> {
  if (!printer.value) return;
  result.value = null;
  progress.startPreparing();
  try {
    const res = await printLabels({ quantity, printer_id: printer.value.id });
    result.value = res;
    progress.finish();
    await store.refreshAfterSave();
    await loadPreview();
  } catch (e) {
    progress.fail(commandErrorMessage(e));
  }
}

onMounted(async () => {
  await store.load();
  store.setStatus(printer.value ? "disconnected" : "unknown");
  await loadPreview();
});
</script>

<template>
  <div class="dashboard">
    <PrinterStatus :printer="printer" :status="status" :next-code="nextCode" />

    <PrintQuantityForm
      :disabled="!canPrint"
      :loading="progress.isBusy.value"
      @print="handlePrint"
    />

    <PrintProgress :stage="progress.stage.value" />

    <PrintResult :result="result" />

    <p v-if="progress.errorMessage.value" class="dashboard__error" role="alert">
      {{ progress.errorMessage.value }}
    </p>

    <LabelPreview :preview="preview" :loading="previewLoading" />
  </div>
</template>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 720px;
}

.dashboard__error {
  margin: 0;
  color: var(--color-danger);
}
</style>
