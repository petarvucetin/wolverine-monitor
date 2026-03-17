import { writable } from "svelte/store";
import type { WolverineNode } from "$lib/types";
import { getNodes } from "$lib/tauri";
import { toasts } from "./toasts";

export const nodeList = writable<WolverineNode[]>([]);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export async function refreshNodes(connectionId: string): Promise<void> {
  try {
    const nodes = await getNodes(connectionId);
    nodeList.set(nodes);
  } catch (e) {
    toasts.add(`Failed to load nodes: ${e}`, "error");
  }
}

export function startNodePolling(connectionId: string, intervalSecs = 10): void {
  stopNodePolling();
  refreshNodes(connectionId);
  pollInterval = setInterval(() => {
    refreshNodes(connectionId);
  }, intervalSecs * 1000);
}

export function stopNodePolling(): void {
  if (pollInterval !== null) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
}
