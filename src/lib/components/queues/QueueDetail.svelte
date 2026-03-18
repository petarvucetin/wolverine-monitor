<script lang="ts">
  import type { QueueInfo } from "$lib/types";
  import { activeConnectionId } from "$lib/stores/connections";
  import { getQueueMessages } from "$lib/tauri";
  import { toasts } from "$lib/stores/toasts";

  interface Props {
    queue: QueueInfo;
  }

  let { queue }: Props = $props();

  let scheduled = $state(false);
  let page = $state(1);
  let pageSize = 25;
  let items = $state<Record<string, unknown>[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let columns = $state<string[]>([]);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  async function load() {
    const connId = $activeConnectionId;
    if (!connId) return;
    loading = true;
    try {
      const result = await getQueueMessages(connId, queue.name, scheduled, page, pageSize);
      items = result.items;
      total = result.total;
      if (result.items.length > 0) {
        columns = Object.keys(result.items[0]);
      }
    } catch (e) {
      toasts.add(`Failed to load queue messages: ${e}`, "error");
    } finally {
      loading = false;
    }
  }

  function startAutoRefresh() {
    stopAutoRefresh();
    load();
    refreshInterval = setInterval(load, 5000);
  }

  function stopAutoRefresh() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }

  function handleToggle() {
    page = 1;
    startAutoRefresh();
  }

  let totalPages = $derived(Math.max(1, Math.ceil(total / pageSize)));

  $effect(() => {
    void queue.name;
    void $activeConnectionId;
    page = 1;
    scheduled = false;
    startAutoRefresh();
    return () => stopAutoRefresh();
  });

  function formatCell(value: unknown): string {
    if (value === null || value === undefined) return "-";
    if (typeof value === "object") return JSON.stringify(value).slice(0, 120);
    const s = String(value);
    return s.length > 80 ? s.slice(0, 80) + "..." : s;
  }
</script>

<div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
  <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
    <h3 class="text-sm font-semibold font-mono">{queue.name}</h3>
    <div class="flex items-center gap-3">
      {#if queue.has_scheduled_table}
        <label class="flex items-center gap-1.5 text-xs cursor-pointer">
          <input type="checkbox" bind:checked={scheduled} onchange={handleToggle} />
          Scheduled
        </label>
      {/if}
      <span class="text-xs text-[var(--color-text-secondary)]">{total} rows</span>
    </div>
  </div>

  {#if loading && items.length === 0}
    <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">Loading...</div>
  {:else if items.length === 0}
    <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">Queue is empty.</div>
  {:else}
    <div class="overflow-x-auto max-h-96">
      <table class="w-full text-xs">
        <thead class="sticky top-0 bg-[var(--color-surface-raised)]">
          <tr class="text-left text-[var(--color-text-secondary)] border-b border-[var(--color-border)]">
            {#each columns as col}
              <th class="px-3 py-2 whitespace-nowrap">{col}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each items as row, i (i)}
            <tr class="border-b border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)]">
              {#each columns as col}
                <td class="px-3 py-1.5 whitespace-nowrap font-mono max-w-xs truncate">{formatCell(row[col])}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    {#if total > pageSize}
      <div class="flex items-center justify-between px-4 py-2 border-t border-[var(--color-border)] text-xs text-[var(--color-text-secondary)]">
        <span>Page {page} of {totalPages}</span>
        <div class="flex gap-2">
          <button onclick={() => { page = Math.max(1, page - 1); load(); }} disabled={page <= 1}
            class="px-2 py-1 rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-30">Prev</button>
          <button onclick={() => { page = Math.min(totalPages, page + 1); load(); }} disabled={page >= totalPages}
            class="px-2 py-1 rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-30">Next</button>
        </div>
      </div>
    {/if}
  {/if}
</div>
