<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import { nodeList, startNodePolling, stopNodePolling } from "$lib/stores/nodes";
  import NodeTable from "$lib/components/nodes/NodeTable.svelte";

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
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Nodes</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view nodes.</p>
  {:else}
    <NodeTable nodes={$nodeList} />
  {/if}
</div>
