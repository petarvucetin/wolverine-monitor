<script lang="ts">
  interface Props {
    onFilter: (filters: { table: string; status: string; messageType: string; dateFrom: string; dateTo: string }) => void;
  }

  let { onFilter }: Props = $props();

  let table = $state("incoming");
  let status = $state("");
  let messageType = $state("");
  let dateFrom = $state("");
  let dateTo = $state("");

  function emitFilter() {
    onFilter({ table, status, messageType, dateFrom, dateTo });
  }
</script>

<div class="flex flex-wrap items-center gap-3 p-4 bg-[var(--color-surface-raised)] rounded-lg border border-[var(--color-border)]">
  <label class="flex items-center gap-2 text-sm">
    <span class="text-xs text-[var(--color-text-secondary)]">Table</span>
    <select bind:value={table} onchange={emitFilter}
      class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
      <option value="incoming">Incoming</option>
      <option value="outgoing">Outgoing</option>
      <option value="dead_letter">Dead Letter</option>
    </select>
  </label>

  {#if table === "incoming"}
    <label class="flex items-center gap-2 text-sm">
      <span class="text-xs text-[var(--color-text-secondary)]">Status</span>
      <select bind:value={status} onchange={emitFilter}
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
      onkeydown={(e) => { if (e.key === 'Enter') emitFilter(); }}
      class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
  </label>

  <label class="flex items-center gap-2 text-sm">
    <span class="text-xs text-[var(--color-text-secondary)]">From</span>
    <input type="datetime-local" bind:value={dateFrom} onchange={emitFilter}
      class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
  </label>

  <label class="flex items-center gap-2 text-sm">
    <span class="text-xs text-[var(--color-text-secondary)]">To</span>
    <input type="datetime-local" bind:value={dateTo} onchange={emitFilter}
      class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
  </label>
</div>
