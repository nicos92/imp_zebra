import { ref } from "vue";
import type { Ref } from "vue";

export type PrintStage = "idle" | "preparing" | "connecting" | "sending" | "done" | "error";

export interface UsePrintProgress {
  stage: Ref<PrintStage>;
  isBusy: Ref<boolean>;
  errorMessage: Ref<string | null>;
  reset: () => void;
  startPreparing: () => void;
  markConnecting: () => void;
  markSending: () => void;
  finish: () => void;
  fail: (message: string) => void;
}

export function usePrintProgress(): UsePrintProgress {
  const stage = ref<PrintStage>("idle");
  const isBusy = ref(false);
  const errorMessage = ref<string | null>(null);

  function reset(): void {
    stage.value = "idle";
    isBusy.value = false;
    errorMessage.value = null;
  }

  function startPreparing(): void {
    stage.value = "preparing";
    isBusy.value = true;
    errorMessage.value = null;
  }

  function markConnecting(): void {
    stage.value = "connecting";
  }

  function markSending(): void {
    stage.value = "sending";
  }

  function finish(): void {
    stage.value = "done";
    isBusy.value = false;
    errorMessage.value = null;
  }

  function fail(message: string): void {
    stage.value = "error";
    isBusy.value = false;
    errorMessage.value = message;
  }

  return {
    stage,
    isBusy,
    errorMessage,
    reset,
    startPreparing,
    markConnecting,
    markSending,
    finish,
    fail,
  };
}
