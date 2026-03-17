<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import ToastContainer from "$lib/components/layout/ToastContainer.svelte";
  import Dashboard from "$lib/views/Dashboard.svelte";
  import Explorer from "$lib/views/Explorer.svelte";
  import DeadLetters from "$lib/views/DeadLetters.svelte";
  import Nodes from "$lib/views/Nodes.svelte";
  import Connections from "$lib/views/Connections.svelte";
  import { currentRoute } from "$lib/stores/router";
  import { loadConnections, connections, activeConnectionId } from "$lib/stores/connections";
  import { onAlert } from "$lib/tauri";
  import { toasts } from "$lib/stores/toasts";

  onMount(() => {
    let unsubscribe: (() => void) | undefined;
    let unlistenAlert: (() => void) | undefined;

    loadConnections().then(() => {
      // Auto-select the first connection if only one exists
      unsubscribe = connections.subscribe((conns) => {
        if (conns.length === 1) {
          activeConnectionId.set(conns[0].config.id);
        }
      });
    });

    onAlert((event) => {
      toasts.add(event.message, "warning");
    }).then((fn) => {
      unlistenAlert = fn;
    });

    return () => {
      unsubscribe?.();
      unlistenAlert?.();
    };
  });
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
