import { describe, expect, it } from "vitest";
import { usePrintProgress } from "./usePrintProgress";

describe("usePrintProgress", () => {
  it("starts idle and not busy", () => {
    const progress = usePrintProgress();
    expect(progress.stage.value).toBe("idle");
    expect(progress.isBusy.value).toBe(false);
    expect(progress.errorMessage.value).toBeNull();
  });

  it("prepares a job and keeps busy", () => {
    const progress = usePrintProgress();
    progress.startPreparing();
    expect(progress.stage.value).toBe("preparing");
    expect(progress.isBusy.value).toBe(true);
  });

  it("transitions through connecting and sending", () => {
    const progress = usePrintProgress();
    progress.startPreparing();
    progress.markConnecting();
    expect(progress.stage.value).toBe("connecting");
    progress.markSending();
    expect(progress.stage.value).toBe("sending");
  });

  it("finishes and clears busy + error", () => {
    const progress = usePrintProgress();
    progress.fail("boom");
    expect(progress.stage.value).toBe("error");
    expect(progress.isBusy.value).toBe(false);

    progress.startPreparing();
    progress.finish();
    expect(progress.stage.value).toBe("done");
    expect(progress.isBusy.value).toBe(false);
    expect(progress.errorMessage.value).toBeNull();
  });

  it("fails with a message", () => {
    const progress = usePrintProgress();
    progress.startPreparing();
    progress.fail("PRINTER_CONNECTION_FAILED");
    expect(progress.stage.value).toBe("error");
    expect(progress.isBusy.value).toBe(false);
    expect(progress.errorMessage.value).toBe("PRINTER_CONNECTION_FAILED");
  });

  it("resets to idle", () => {
    const progress = usePrintProgress();
    progress.startPreparing();
    progress.fail("boom");
    progress.reset();
    expect(progress.stage.value).toBe("idle");
    expect(progress.isBusy.value).toBe(false);
    expect(progress.errorMessage.value).toBeNull();
  });
});
