import { invoke } from "@tauri-apps/api/core";
import type { TauriError } from "../../types";

export function isTauriError(value: unknown): value is TauriError {
  return !!value && typeof value === "object" && "code" in value && "message" in value;
}

export function commandErrorMessage(error: unknown): string {
  if (isTauriError(error)) return error.message;
  if (error instanceof Error) return error.message;
  return String(error);
}

export function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}
