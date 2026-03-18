<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import { nodeList, nodeAssignments, nodeRecords, startNodePolling, stopNodePolling, refreshNodes } from "$lib/stores/nodes";
  import NodeTable from "$lib/components/nodes/NodeTable.svelte";
  import GenericRowTable from "$lib/components/nodes/GenericRowTable.svelte";

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId) {
      startNodePolling(connId);
    } else {
      stopNodePolling();
    }
    return () => {
      stopNodePolling();
    };
  });

  function handleRefresh() {
    const connId = $activeConnectionId;
    if (connId) refreshNodes(connId);
  }
</script>

<div class="p-6">
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-xl font-semibold">Nodes</h1>
    {#if $activeConnectionId}
      <button
        onclick={handleRefresh}
        class="px-3 py-1.5 text-xs rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] transition-colors"
      >Refresh</button>
    {/if}
  </div>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view nodes.</p>
  {:else}
    <div class="space-y-6">
      <div>
        <h2 class="text-lg font-semibold mb-3">Nodes</h2>
        <NodeTable nodes={$nodeList} />
      </div>

      <GenericRowTable title="Node Assignments" rows={$nodeAssignments} />

      <GenericRowTable title="Node Records" rows={$nodeRecords} />
    </div>
  {/if}
</div>
