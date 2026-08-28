import type { PrintJobStatus, TauriError } from "../types";

const STATUS_LABELS: Record<PrintJobStatus, string> = {
  pending: "Pendiente",
  printing: "Imprimiendo",
  completed: "Completado",
  failed: "Fallido",
};

export function formatStatus(status: PrintJobStatus): string {
  return STATUS_LABELS[status] ?? status;
}

export function formatDate(iso: string): string {
  if (!iso) return "-";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(date);
}

export function formatConnectionType(type: string): string {
  switch (type) {
    case "tcp":
      return "TCP/IP";
    case "usb":
      return "USB";
    case "serial":
      return "Serial";
    default:
      return type;
  }
}

export function toErrorMessage(error: unknown): string {
  if (error && typeof error === "object" && "message" in error) {
    return String((error as TauriError).message);
  }
  if (error instanceof Error) return error.message;
  return String(error);
}
