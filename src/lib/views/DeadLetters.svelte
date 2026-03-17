<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import {
    deadLetterList,
    deadLetterTotal,
    selectedIds,
    loadDeadLetters,
    replaySingle,
  } from "$lib/stores/deadLetters";
  import type { DeadLetter } from "$lib/types";
  import DeadLetterTable from "$lib/components/deadletters/DeadLetterTable.svelte";
  import ReplayControls from "$lib/components/deadletters/ReplayControls.svelte";
  import MessageDetail from "$lib/components/explorer/MessageDetail.svelte";

  let page = $state(1);
  let pageSize = 25;
  let selectedItem = $state<DeadLetter | null>(null);
  let loading = $state(false);

  async function load() {
    const connId = $activeConnectionId;
    if (!connId) return;
    loading = true;
    try {
      await loadDeadLetters(connId, page, pageSize);
    } finally {
      loading = false;
    }
  }

  function handleSelect(item: DeadLetter) {
    selectedItem = item;
  }

  function handleCloseDetail() {
    selectedItem = null;
  }

  async function handleReplaySingle(item: DeadLetter) {
    const connId = $activeConnectionId;
    if (!connId) return;
    await replaySingle(connId, item.id);
    selectedItem = null;
  }

  let totalPages = $derived(Math.max(1, Math.ceil($deadLetterTotal / pageSize)));

  $effect(() => {
    if ($activeConnectionId) {
      page = 1;
      selectedIds.set(new Set());
      load();
    }
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Dead Letter Queue</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view dead letters.</p>
  {:else}
    <div class="space-y-4">
      <ReplayControls items={$deadLetterList} />

      {#if loading}
        <div class="text-center py-8 text-sm text-[var(--color-text-secondary)]">Loading...</div>
      {:else}
        <DeadLetterTable items={$deadLetterList} onSelect={handleSelect} />

        {#if $deadLetterTotal > 0}
          <div class="flex items-center justify-between text-xs text-[var(--color-text-secondary)]">
            <span>{$deadLetterTotal} total dead letters</span>
            <div class="flex items-center gap-2">
              <button
                onclick={() => { page = Math.max(1, page - 1); load(); }}
                disabled={page <= 1}
                class="px-2 py-1 rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-30"
              >
                Prev
              </button>
              <span>Page {page} of {totalPages}</span>
              <button
                onclick={() => { page = Math.min(totalPages, page + 1); load(); }}
                disabled={page >= totalPages}
                class="px-2 py-1 rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-30"
              >
                Next
              </button>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  {#if selectedItem}
    <MessageDetail item={selectedItem} onClose={handleCloseDetail} />
  {/if}
</div>
