<script lang="ts">
  interface Props {
    onFilter: (filters: { table: string; status: string; messageType: string }) => void;
  }

  let { onFilter }: Props = $props();

  let table = $state("incoming");
  let status = $state("");
  let messageType = $state("");

  function emitFilter() {
    onFilter({ table, status, messageType });
  }

  $effect(() => {
    // Re-emit when any filter changes
    void table;
    void status;
    void messageType;
    emitFilter();
  });
</script>

<div class="flex items-center gap-3 p-4 bg-[var(--color-surface-raised)] rounded-lg border border-[var(--color-border)]">
  <label class="flex items-center gap-2 text-sm">
    <span class="text-xs text-[var(--color-text-secondary)]">Table</span>
    <select bind:value={table}
      class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
      <option value="incoming">Incoming</option>
      <option value="outgoing">Outgoing</option>
      <option value="dead_letter">Dead Letter</option>
    </select>
  </label>

  {#if table === "incoming"}
    <label class="flex items-center gap-2 text-sm">
      <span class="text-xs text-[var(--color-text-secondary)]">Status</span>
      <select bind:value={status}
        class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
        <option value="">All</option>
        <option value="Incoming">Incoming</option>
        <option value="Scheduled">Scheduled</option>
        <option value="Handled">Handled</option>
      </select>
    </label>
  {/if}

  <label class="flex items-center gap-2 text-sm flex-1">
    <span class="text-xs text-[var(--color-text-secondary)]">Type</span>
    <input bind:value={messageType} placeholder="Filter by message type..."
      class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
  </label>
</div>
