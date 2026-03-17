import { writable } from "svelte/store";
import type { DashboardStats, NotifyEvent } from "$lib/types";
import { getDashboardStats, onEnvelopeChange } from "$lib/tauri";
import { statusBar } from "./statusBar";
import type { UnlistenFn } from "@tauri-apps/api/event";

export interface RecentMessage {
  id: string;
  message_type: string;
  table: "incoming" | "outgoing" | "dead_letter";
  op: string;
  timestamp: number;
}

export const MAX_RECENT = 500;

export const stats = writable<DashboardStats>({
  incoming_count: 0,
  incoming_scheduled: 0,
  incoming_handled: 0,
  outgoing_count: 0,
  dead_letter_count: 0,
  throughput: [],
});

export const recentMessages = writable<RecentMessage[]>([]);

export function addRecentMessage(event: NotifyEvent): void {
  recentMessages.update((msgs) => {
    const next: RecentMessage[] = [
      {
        id: event.id,
        message_type: event.message_type,
        table: event.table,
        op: event.op,
        timestamp: Date.now(),
      },
      ...msgs,
    ];
    if (next.length > MAX_RECENT) {
      next.length = MAX_RECENT;
    }
    return next;
  });
}

export async function refreshStats(connectionId: string): Promise<void> {
  try {
    const data = await getDashboardStats(connectionId);
    stats.set(data);
    const total = data.incoming_count + data.outgoing_count + data.dead_letter_count;
    statusBar.set({ status: "ready", message: `${total} envelopes` });
  } catch (e) {
    statusBar.set({ status: "error", message: `${e}` });
  }
}

let refreshTimeout: ReturnType<typeof setTimeout> | null = null;

function debouncedRefreshStats(connectionId: string): void {
  if (refreshTimeout) clearTimeout(refreshTimeout);
  refreshTimeout = setTimeout(() => {
    refreshStats(connectionId);
  }, 1000);
}

let pollInterval: ReturnType<typeof setInterval> | null = null;
let unlistenFn: UnlistenFn | null = null;

export async function startDashboard(connectionId: string): Promise<void> {
  stopDashboard();

  refreshStats(connectionId);

  pollInterval = setInterval(() => {
    refreshStats(connectionId);
  }, 5000);

  unlistenFn = await onEnvelopeChange((event) => {
    if (event.connection_id === connectionId) {
      addRecentMessage(event);
      debouncedRefreshStats(connectionId);
    }
  });
}

export function stopDashboard(): void {
  if (refreshTimeout !== null) {
    clearTimeout(refreshTimeout);
    refreshTimeout = null;
  }
  if (pollInterval !== null) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
  if (unlistenFn !== null) {
    unlistenFn();
    unlistenFn = null;
  }
}
