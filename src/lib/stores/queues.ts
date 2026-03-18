import { writable } from "svelte/store";
import type { QueueInfo } from "$lib/types";
import { getQueues } from "$lib/tauri";
import { statusBar } from "./statusBar";

export const queueList = writable<QueueInfo[]>([]);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export async function refreshQueues(connectionId: string): Promise<void> {
  try {
    const queues = await getQueues(connectionId);
    queueList.set(queues);
    const totalMessages = queues.reduce((s, q) => s + q.count + q.scheduled_count, 0);
    statusBar.set({ status: "ready", message: `${queues.length} queues, ${totalMessages} messages` });
  } catch (e) {
    statusBar.set({ status: "error", message: `${e}` });
  }
}

export function startQueuePolling(connectionId: string, intervalSecs = 5): void {
  stopQueuePolling();
  refreshQueues(connectionId);
  pollInterval = setInterval(() => {
    refreshQueues(connectionId);
  }, intervalSecs * 1000);
}

export function stopQueuePolling(): void {
  if (pollInterval !== null) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
}
