export type ConnectionType = "tcp" | "usb" | "serial";

export type PrintJobStatus = "pending" | "printing" | "completed" | "failed";

export interface PrinterConfig {
  id?: string;
  name: string;
  model: string;
  dpi: number;
  label_width_mm: number;
  label_height_mm: number;
  columns: number;
  connection_type: ConnectionType;
  ip_address: string;
  port: number;
}

export interface Printer extends PrinterConfig {
  id: string;
  created_at: string;
  updated_at: string;
}

export interface SequenceInfo {
  last_used_code: string;
  next_code: string;
}

export interface PrintRequest {
  quantity: number;
  printer_id: string;
}

export interface PrintResult {
  job_id: string;
  start_code: string;
  end_code: string;
  quantity: number;
  status: string;
}

export interface PrintJob {
  id: string;
  printer_id: string;
  start_code: string;
  end_code: string;
  quantity: number;
  status: PrintJobStatus;
  created_at: string;
  completed_at: string | null;
}

export interface LabelPreview {
  code: string;
  timestamp: string;
  zpl: string;
}

export interface TauriError {
  code: string;
  message: string;
}
