import { writable, get } from "svelte/store";
import type { DeadLetter, BulkReplayResult } from "$lib/types";
import { getDeadLetters, replayDeadLetter, replayDeadLettersBulk } from "$lib/tauri";
import { toasts } from "./toasts";

export const deadLetterList = writable<DeadLetter[]>([]);
export const deadLetterTotal = writable<number>(0);
export const selectedIds = writable<Set<string>>(new Set());

export async function loadDeadLetters(
  connectionId: string,
  page: number,
  pageSize: number,
  messageType?: string
): Promise<void> {
  try {
    const filters = messageType ? { message_type: messageType } : {};
    const result = await getDeadLetters(connectionId, filters, page, pageSize);
    deadLetterList.set(result.items);
    deadLetterTotal.set(result.total);
  } catch (e) {
    toasts.add(`Failed to load dead letters: ${e}`, "error");
  }
}

export async function replaySingle(connectionId: string, id: string): Promise<void> {
  try {
    await replayDeadLetter(connectionId, id);
    toasts.add("Dead letter replayed successfully", "success");
    deadLetterList.update((items) => items.filter((item) => item.id !== id));
    deadLetterTotal.update((t) => Math.max(0, t - 1));
    selectedIds.update((ids) => {
      const next = new Set(ids);
      next.delete(id);
      return next;
    });
  } catch (e) {
    toasts.add(`Failed to replay dead letter: ${e}`, "error");
  }
}

export async function replaySelected(connectionId: string, ids: string[]): Promise<void> {
  try {
    const result: BulkReplayResult = await replayDeadLettersBulk(connectionId, ids);
    if (result.succeeded > 0) {
      toasts.add(`Replayed ${result.succeeded} dead letter(s)`, "success");
    }
    if (result.failed > 0) {
      toasts.add(`Failed to replay ${result.failed} dead letter(s)`, "warning");
    }

    // Remove succeeded items from the list
    const failedIds = new Set(result.errors.map((e) => e.id));
    const succeededIds = ids.filter((id) => !failedIds.has(id));
    deadLetterList.update((items) =>
      items.filter((item) => !succeededIds.includes(item.id))
    );
    deadLetterTotal.update((t) => Math.max(0, t - result.succeeded));
    selectedIds.set(new Set());
  } catch (e) {
    toasts.add(`Failed to replay dead letters: ${e}`, "error");
  }
}
