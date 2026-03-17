import { describe, it, expect } from "vitest";
import {
  decodeBody,
  formatRelativeTime,
  getNodeHealth,
  shortenMessageType,
} from "$lib/format";

describe("decodeBody", () => {
  it("decodes valid JSON body", () => {
    const json = '{"key":"value"}';
    const bytes = Array.from(new TextEncoder().encode(json));
    const result = decodeBody(bytes);
    expect(result.type).toBe("json");
    expect(result.content).toContain('"key"');
    expect(result.content).toContain('"value"');
  });

  it("returns formatted JSON with indentation", () => {
    const json = '{"a":1}';
    const bytes = Array.from(new TextEncoder().encode(json));
    const result = decodeBody(bytes);
    expect(result.type).toBe("json");
    expect(result.content).toContain("\n");
  });

  it("returns hex and base64 for non-JSON binary data", () => {
    const bytes = [0xff, 0x00, 0xab];
    const result = decodeBody(bytes);
    expect(result.type).toBe("hex");
    expect(result.content).toBe("ff 00 ab");
    if (result.type === "hex") {
      expect(result.base64).toBeTruthy();
    }
  });

  it("handles empty body", () => {
    const result = decodeBody([]);
    // Empty string is valid UTF-8 but not valid JSON
    expect(result.type).toBe("hex");
  });
});

describe("formatRelativeTime", () => {
  it("returns dash for null input", () => {
    expect(formatRelativeTime(null)).toBe("\u2014");
  });

  it("formats seconds ago", () => {
    const tenSecondsAgo = new Date(Date.now() - 10000).toISOString();
    const result = formatRelativeTime(tenSecondsAgo);
    expect(result).toMatch(/\d+s ago/);
  });

  it("formats minutes ago", () => {
    const fiveMinutesAgo = new Date(Date.now() - 300000).toISOString();
    const result = formatRelativeTime(fiveMinutesAgo);
    expect(result).toMatch(/\d+m ago/);
  });

  it("formats hours ago", () => {
    const twoHoursAgo = new Date(Date.now() - 7200000).toISOString();
    const result = formatRelativeTime(twoHoursAgo);
    expect(result).toMatch(/\d+h ago/);
  });

  it("formats days ago", () => {
    const threeDaysAgo = new Date(Date.now() - 259200000).toISOString();
    const result = formatRelativeTime(threeDaysAgo);
    expect(result).toMatch(/\d+d ago/);
  });
});

describe("getNodeHealth", () => {
  it("returns Unknown for null health check", () => {
    expect(getNodeHealth(null)).toBe("Unknown");
  });

  it("returns Healthy for recent health check", () => {
    const recent = new Date(Date.now() - 5000).toISOString();
    expect(getNodeHealth(recent)).toBe("Healthy");
  });

  it("returns Warning for stale health check", () => {
    const stale = new Date(Date.now() - 60000).toISOString();
    expect(getNodeHealth(stale)).toBe("Warning");
  });

  it("returns Critical for very stale health check", () => {
    const old = new Date(Date.now() - 180000).toISOString();
    expect(getNodeHealth(old)).toBe("Critical");
  });

  it("accepts custom thresholds", () => {
    const recent = new Date(Date.now() - 15000).toISOString();
    expect(getNodeHealth(recent, { warning: 10, critical: 20 })).toBe("Warning");
  });
});

describe("shortenMessageType", () => {
  it("returns last segment of dotted name", () => {
    expect(shortenMessageType("My.Namespace.MyCommand")).toBe("MyCommand");
  });

  it("returns the type as-is when no dots", () => {
    expect(shortenMessageType("SimpleType")).toBe("SimpleType");
  });

  it("handles single character segments", () => {
    expect(shortenMessageType("A.B.C")).toBe("C");
  });
});
