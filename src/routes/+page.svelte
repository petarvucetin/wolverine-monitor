<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import StatusBar from "$lib/components/layout/StatusBar.svelte";
  import ToastContainer from "$lib/components/layout/ToastContainer.svelte";
  import Dashboard from "$lib/views/Dashboard.svelte";
  import Explorer from "$lib/views/Explorer.svelte";
  import DeadLetters from "$lib/views/DeadLetters.svelte";
  import Nodes from "$lib/views/Nodes.svelte";
  import Queues from "$lib/views/Queues.svelte";
  import Connections from "$lib/views/Connections.svelte";
  import { currentRoute, navigate } from "$lib/stores/router";
  import { loadConnections } from "$lib/stores/connections";
  import { onAlert } from "$lib/tauri";
  import { toasts } from "$lib/stores/toasts";

  onMount(() => {
    let unlistenAlert: (() => void) | undefined;

    loadConnections().then(() => {
      // Trigger auto-select for the initial route
      navigate("dashboard");
    }).catch((e) => {
      toasts.add(`Failed to load connections: ${e}`, "error");
    });

    onAlert((event) => {
      toasts.add(event.message, "warning");
    }).then((fn) => {
      unlistenAlert = fn;
    }).catch(() => {});

    return () => {
      unlistenAlert?.();
    };
  });
</script>

<div class="flex flex-col h-screen overflow-hidden">
  <div class="flex flex-1 overflow-hidden">
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
      {:else if $currentRoute === "queues"}
        <Queues />
      {:else if $currentRoute === "connections"}
        <Connections />
      {/if}
    </main>
  </div>
  <StatusBar />
</div>

<ToastContainer />
