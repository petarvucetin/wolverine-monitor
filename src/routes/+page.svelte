<script lang="ts">
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import ToastContainer from "$lib/components/layout/ToastContainer.svelte";
  import Dashboard from "$lib/views/Dashboard.svelte";
  import Explorer from "$lib/views/Explorer.svelte";
  import DeadLetters from "$lib/views/DeadLetters.svelte";
  import Nodes from "$lib/views/Nodes.svelte";
  import Connections from "$lib/views/Connections.svelte";
  import { currentRoute } from "$lib/stores/router";
</script>

<div class="flex h-screen overflow-hidden">
  <Sidebar />

  <main class="flex-1 overflow-y-auto">
    {#if $currentRoute === "dashboard"}
      <Dashboard />
    {:else if $currentRoute === "explorer"}
      <Explorer />
    {:else if $currentRoute === "deadletters"}
      <DeadLetters />
    {:else if $currentRoute === "nodes"}
      <Nodes />
    {:else if $currentRoute === "connections"}
      <Connections />
    {/if}
  </main>
</div>

<ToastContainer />
