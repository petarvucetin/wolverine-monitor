<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import { queueList, startQueuePolling, stopQueuePolling } from "$lib/stores/queues";
  import QueueTable from "$lib/components/queues/QueueTable.svelte";
  import QueueDetail from "$lib/components/queues/QueueDetail.svelte";
  import type { QueueInfo } from "$lib/types";

  let selectedQueue = $state<QueueInfo | null>(null);

  function handleSelect(queue: QueueInfo) {
    selectedQueue = queue;
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
      <div class="flex gap-4 text-sm text-[var(--color-text-secondary)]">
        <span>{$queueList.length} queues</span>
        <span>{totalMessages} total messages</span>
      </div>
    {/if}
  </div>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view queues.</p>
  {:else}
    <div class="space-y-4">
      <div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
        <QueueTable queues={$queueList} selectedQueue={selectedQueue?.name ?? null} onSelect={handleSelect} />
      </div>

      {#if selectedQueue}
        <QueueDetail queue={selectedQueue} />
      {/if}
    </div>
  {/if}
</div>
