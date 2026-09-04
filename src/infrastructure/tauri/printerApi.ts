import type { Printer, PrinterConfig, SequenceInfo } from "../../types";
import { invokeCommand } from "./tauriClient";

export function getConfiguredPrinter(): Promise<Printer | null> {
  return invokeCommand<Printer | null>("get_configured_printer");
}

export function getCurrentSequence(): Promise<SequenceInfo> {
  return invokeCommand<SequenceInfo>("get_current_sequence");
}

export function getPrinterConfig(id: string): Promise<Printer | null> {
  return invokeCommand<Printer | null>("get_printer_config", { id });
}

export function savePrinterConfig(config: PrinterConfig): Promise<Printer> {
  return invokeCommand<Printer>("save_printer_config", { config });
}
