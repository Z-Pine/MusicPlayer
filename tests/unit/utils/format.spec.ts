import { describe, it, expect } from "vitest";
import { formatDuration } from "@/utils/format";

describe("formatDuration", () => {
  it("formats 0 seconds", () => {
    expect(formatDuration(0)).toBe("0:00");
  });

  it("formats positive seconds", () => {
    expect(formatDuration(65)).toBe("1:05");
    expect(formatDuration(125)).toBe("2:05");
    expect(formatDuration(3600)).toBe("60:00");
  });

  it("formats negative input as 0:00", () => {
    expect(formatDuration(-10)).toBe("0:00");
  });

  it("handles undefined-like falsy values", () => {
    expect(formatDuration(NaN)).toBe("0:00");
  });
});
