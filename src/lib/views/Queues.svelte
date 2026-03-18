<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import { queueList, startQueuePolling, stopQueuePolling, refreshQueues, purgeQueue, purgeAllQueues } from "$lib/stores/queues";
  import QueueTable from "$lib/components/queues/QueueTable.svelte";
  import QueueDetail from "$lib/components/queues/QueueDetail.svelte";
  import type { QueueInfo } from "$lib/types";

  let selectedQueue = $state<QueueInfo | null>(null);
  let confirmPurgeAll = $state(false);

  function handleSelect(queue: QueueInfo) {
    selectedQueue = queue;
  }

  async function handlePurge(queueName: string) {
    const connId = $activeConnectionId;
    if (!connId) return;
    await purgeQueue(connId, queueName);
    if (selectedQueue?.name === queueName) {
      selectedQueue = null;
    }
  }

  async function handlePurgeAll() {
    const connId = $activeConnectionId;
    if (!connId) return;
    confirmPurgeAll = false;
    await purgeAllQueues(connId);
    selectedQueue = null;
  }

  function handleRefresh() {
    const connId = $activeConnectionId;
    if (connId) refreshQueues(connId);
  }

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId) {
      startQueuePolling(connId);
    } else {
      stopQueuePolling();
    }
    return () => {
      stopQueuePolling();
    };
  });

  let totalMessages = $derived($queueList.reduce((s, q) => s + q.count + q.scheduled_count, 0));
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-xl font-semibold">Queues</h1>
    {#if $activeConnectionId && $queueList.length > 0}
      <div class="flex items-center gap-4">
        <span class="text-sm text-[var(--color-text-secondary)]">{$queueList.length} queues</span>
        <span class="text-sm text-[var(--color-text-secondary)]">{totalMessages} total messages</span>
        <div class="flex items-center gap-2">
          <button
            onclick={handleRefresh}
            class="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] transition-colors"
            title="Refresh queues"
          >Refresh</button>
          {#if confirmPurgeAll}
            <span class="text-xs text-[var(--color-error,#ef4444)]">Are you sure?</span>
            <button
              onclick={handlePurgeAll}
              class="px-3 py-1.5 text-xs rounded bg-[var(--color-error,#ef4444)] text-white hover:opacity-90 transition-colors"
            >Yes, Purge All</button>
            <button
              onclick={() => confirmPurgeAll = false}
              class="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] transition-colors"
            >Cancel</button>
          {:else}
            <button
              onclick={() => confirmPurgeAll = true}
              class="px-3 py-1.5 text-xs rounded border border-[var(--color-error,#ef4444)] text-[var(--color-error,#ef4444)] hover:bg-[var(--color-error,#ef4444)] hover:text-white transition-colors"
            >Purge All</button>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view queues.</p>
  {:else}
    <div class="space-y-4">
      <div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
        <QueueTable queues={$queueList} selectedQueue={selectedQueue?.name ?? null} onSelect={handleSelect} onPurge={handlePurge} />
      </div>

      {#if selectedQueue}
        <QueueDetail queue={selectedQueue} />
      {/if}
    </div>
  {/if}
</div>
