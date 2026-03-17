<script lang="ts">
  import { activeConnectionId } from "$lib/stores/connections";
  import { toasts } from "$lib/stores/toasts";
  import { statusBar } from "$lib/stores/statusBar";
  import { getIncomingEnvelopes, getOutgoingEnvelopes, getDeadLetters } from "$lib/tauri";
  import type { IncomingEnvelope, OutgoingEnvelope, DeadLetter } from "$lib/types";
  import FilterBar from "$lib/components/explorer/FilterBar.svelte";
  import EnvelopeTable from "$lib/components/explorer/EnvelopeTable.svelte";
  import MessageDetail from "$lib/components/explorer/MessageDetail.svelte";

  type AnyEnvelope = IncomingEnvelope | OutgoingEnvelope | DeadLetter;

  let items = $state<AnyEnvelope[]>([]);
  let total = $state(0);
  let page = $state(1);
  let pageSize = 25;
  let currentTable = $state("incoming");
  let currentFilters = $state<{ table: string; status: string; messageType: string; dateFrom: string; dateTo: string }>({
    table: "incoming",
    status: "",
    messageType: "",
    dateFrom: "",
    dateTo: "",
  });
  let selectedItem = $state<AnyEnvelope | null>(null);
  let loading = $state(false);
  let lastConnId = $state<string | null>(null);

  async function loadData() {
    const connId = $activeConnectionId;
    if (!connId) return;

    loading = true;
    statusBar.set({ status: "loading", message: "Loading envelopes..." });
    try {
      const filters = {
        status: currentFilters.status || undefined,
        message_type: currentFilters.messageType || undefined,
        date_from: currentFilters.dateFrom ? new Date(currentFilters.dateFrom).toISOString() : undefined,
        date_to: currentFilters.dateTo ? new Date(currentFilters.dateTo).toISOString() : undefined,
      };

      if (currentTable === "incoming") {
        const result = await getIncomingEnvelopes(connId, filters, page, pageSize);
        items = result.items;
        total = result.total;
      } else if (currentTable === "outgoing") {
        const result = await getOutgoingEnvelopes(connId, filters, page, pageSize);
        items = result.items;
        total = result.total;
      } else {
        const result = await getDeadLetters(connId, filters, page, pageSize);
        items = result.items;
        total = result.total;
      }
      statusBar.set({ status: "ready", message: `${total} envelopes` });
    } catch (e) {
      toasts.add(`Failed to load envelopes: ${e}`, "error");
      statusBar.set({ status: "error", message: `${e}` });
    } finally {
      loading = false;
    }
  }

  function handleFilter(filters: { table: string; status: string; messageType: string; dateFrom: string; dateTo: string }) {
    currentFilters = filters;
    if (filters.table !== currentTable) {
      currentTable = filters.table;
      selectedItem = null;
    }
    page = 1;
    loadData();
  }

  function handlePageChange(newPage: number) {
    page = newPage;
    loadData();
  }

  function handleSelect(item: AnyEnvelope) {
    selectedItem = item;
  }

  function handleCloseDetail() {
    selectedItem = null;
  }

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId && connId !== lastConnId) {
      lastConnId = connId;
      page = 1;
      loadData();
    }
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Message Explorer</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to explore messages.</p>
  {:else}
    <div class="space-y-4">
      <FilterBar onFilter={handleFilter} />

      {#if loading}
        <div class="text-center py-8 text-sm text-[var(--color-text-secondary)]">Loading...</div>
      {:else}
        <EnvelopeTable
          {items}
          table={currentTable}
          {total}
          {page}
          {pageSize}
          onPageChange={handlePageChange}
          onSelect={handleSelect}
        />
      {/if}
    </div>
  {/if}

  {#if selectedItem}
    <MessageDetail item={selectedItem} onClose={handleCloseDetail} />
  {/if}
</div>
