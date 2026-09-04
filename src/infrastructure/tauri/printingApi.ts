import type { LabelPreview, PrintJob, PrintRequest, PrintResult, WindowsPrinter } from "../../types";
import { invokeCommand } from "./tauriClient";

export function printLabels(request: PrintRequest): Promise<PrintResult> {
  return invokeCommand<PrintResult>("print_labels", { request });
}

export function listWindowsPrinters(): Promise<WindowsPrinter[]> {
  return invokeCommand<WindowsPrinter[]>("list_windows_printers");
}

export function previewLabel(
  labelWidthMm: number,
  labelHeightMm: number,
  columns: number,
  dpi: number,
): Promise<LabelPreview> {
  return invokeCommand<LabelPreview>("preview_label", {
    labelWidthMm,
    labelHeightMm,
    columns,
    dpi,
  });
}

export function listPrintJobs(limit?: number): Promise<PrintJob[]> {
  return invokeCommand<PrintJob[]>("list_print_jobs", { limit });
}

export function getPrintJob(jobId: string): Promise<PrintJob | null> {
  return invokeCommand<PrintJob | null>("get_print_job", { jobId });
}
