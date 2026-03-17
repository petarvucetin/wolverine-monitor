export function decodeBody(body: number[]): { type: "json"; content: string } | { type: "hex"; content: string; base64: string } {
  try {
    const bytes = new Uint8Array(body);
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    JSON.parse(text); // validate JSON
    return { type: "json", content: JSON.stringify(JSON.parse(text), null, 2) };
  } catch {
    const bytes = new Uint8Array(body);
    const hex = Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join(" ");
    const base64 = btoa(bytes.reduce((s, b) => s + String.fromCharCode(b), ""));
    return { type: "hex", content: hex, base64 };
  }
}

export function formatRelativeTime(isoDate: string | null): string {
  if (!isoDate) return "\u2014";
  const diff = Date.now() - new Date(isoDate).getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

export function getNodeHealth(
  healthCheck: string | null,
  thresholds = { warning: 30, critical: 120 }
): "Healthy" | "Warning" | "Critical" | "Unknown" {
  if (!healthCheck) return "Unknown";
  const ageSeconds = (Date.now() - new Date(healthCheck).getTime()) / 1000;
  if (ageSeconds < thresholds.warning) return "Healthy";
  if (ageSeconds < thresholds.critical) return "Warning";
  return "Critical";
}

export function shortenMessageType(messageType: string): string {
  const parts = messageType.split(".");
  return parts[parts.length - 1] ?? messageType;
}
