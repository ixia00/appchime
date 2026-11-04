import type { AppError, ErrorCode } from "./types";

const CODES: readonly ErrorCode[] = [
  "duplicateApp",
  "notFound",
  "unsupportedAudio",
  "fileTooLarge",
  "invalidApp",
  "invalidShortcut",
  "invalidInput",
  "io",
  "other",
];

export function toAppError(e: unknown): AppError {
  if (e && typeof e === "object" && "code" in e) {
    const code = (e as { code: unknown }).code;
    const message = String((e as { message?: unknown }).message ?? "");
    if (CODES.includes(code as ErrorCode)) return { code: code as ErrorCode, message };
  }
  return { code: "other", message: e instanceof Error ? e.message : String(e) };
}
