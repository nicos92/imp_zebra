import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { getConfiguredPrinter, getCurrentSequence } from "../infrastructure/tauri/printerApi";
import { commandErrorMessage } from "../infrastructure/tauri/tauriClient";
import type { Printer, SequenceInfo } from "../types";

export type PrinterConnectionStatus = "unknown" | "connected" | "disconnected" | "testing";

export const usePrinterStore = defineStore("printer", () => {
  const printer = ref<Printer | null>(null);
  const sequence = ref<SequenceInfo | null>(null);
  const status = ref<PrinterConnectionStatus>("unknown");
  const loading = ref(false);
  const error = ref<string | null>(null);

  const connected = computed(() => status.value === "connected");
  const nextCode = computed(() => sequence.value?.next_code ?? "-");
  const hasPrinter = computed(() => printer.value !== null);

  async function load(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      printer.value = await getConfiguredPrinter();
      sequence.value = await getCurrentSequence();
    } catch (e) {
      error.value = commandErrorMessage(e);
    } finally {
      loading.value = false;
    }
  }

  function setPrinter(value: Printer | null): void {
    printer.value = value;
  }

  function setSequence(value: SequenceInfo): void {
    sequence.value = value;
  }

  function setStatus(value: PrinterConnectionStatus): void {
    status.value = value;
  }

  async function refreshAfterSave(): Promise<void> {
    await load();
  }

  return {
    printer,
    sequence,
    status,
    loading,
    error,
    connected,
    nextCode,
    hasPrinter,
    load,
    setPrinter,
    setSequence,
    setStatus,
    refreshAfterSave,
  };
});
