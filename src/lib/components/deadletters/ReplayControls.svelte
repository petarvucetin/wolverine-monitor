<script lang="ts">
  import type { DeadLetter } from "$lib/types";
  import { selectedIds, replaySelected } from "$lib/stores/deadLetters";
  import { activeConnectionId } from "$lib/stores/connections";

  interface Props {
    items: DeadLetter[];
  }

  let { items }: Props = $props();
  let replaying = $state(false);

  let selectedCount = $derived($selectedIds.size);
  let replayableSelected = $derived(
    items.filter((item) => $selectedIds.has(item.id) && item.replayable)
  );
  let nonReplayableCount = $derived(selectedCount - replayableSelected.length);

  async function handleReplay() {
    const connId = $activeConnectionId;
    if (!connId || replayableSelected.length === 0) return;

    replaying = true;
    try {
      await replaySelected(
        connId,
        replayableSelected.map((item) => item.id)
      );
    } finally {
      replaying = false;
    }
  }
</script>

{#if selectedCount > 0}
  <div class="flex items-center gap-4 px-4 py-3 rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)]">
    <span class="text-sm">
      {selectedCount} selected
    </span>

    {#if nonReplayableCount > 0}
      <span class="text-xs text-yellow-400">
        {nonReplayableCount} item(s) not replayable and will be skipped
      </span>
    {/if}

    <button
      onclick={handleReplay}
      disabled={replaying || replayableSelected.length === 0}
      class="ml-auto px-4 py-1.5 text-sm rounded-md bg-blue-600 hover:bg-blue-500 text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
    >
      {#if replaying}
        Replaying...
      {:else}
        Replay Selected ({replayableSelected.length})
      {/if}
    </button>
  </div>
{/if}
