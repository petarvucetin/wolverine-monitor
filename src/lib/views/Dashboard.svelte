<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import { stats, startDashboard, stopDashboard } from "$lib/stores/dashboard";
  import CounterCard from "$lib/components/dashboard/CounterCard.svelte";
  import ThroughputChart from "$lib/components/dashboard/ThroughputChart.svelte";
  import LiveFeed from "$lib/components/dashboard/LiveFeed.svelte";

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId) {
      startDashboard(connId);
    } else {
      stopDashboard();
    }
    return () => {
      stopDashboard();
    };
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Dashboard</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view dashboard.</p>
  {:else}
    <div class="grid grid-cols-3 gap-4 mb-6">
      <CounterCard
        label="Incoming"
        value={$stats.incoming_count}
        color="#22c55e"
        rate="{$stats.incoming_scheduled} scheduled"
      />
      <CounterCard
        label="Outgoing"
        value={$stats.outgoing_count}
        color="#3b82f6"
      />
      <CounterCard
        label="Dead Letters"
        value={$stats.dead_letter_count}
        color="#ef4444"
      />
    </div>

    <div class="mb-6">
      <ThroughputChart data={$stats.throughput} />
    </div>

    <LiveFeed />
  {/if}
</div>
