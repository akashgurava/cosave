<script lang="ts">
  import {
    INITIAL_MOCK_TRANSACTIONS,
    TransactionsView,
    type Transaction,
    type TransactionId,
  } from "$lib/features/transactions";

  // Reactive transaction dataset in local state for interaction
  let mockTransactions = $state<Transaction[]>([...INITIAL_MOCK_TRANSACTIONS]);

  function handleUpdateTransaction(id: TransactionId, updates: Partial<Transaction>) {
    mockTransactions = mockTransactions.map((tx) => {
      if (tx.id === id) {
        return { ...tx, ...updates };
      }
      return tx;
    });
  }

  function handleDeleteTransaction(id: TransactionId) {
    mockTransactions = mockTransactions.filter((tx) => tx.id !== id);
  }

  let nextId = 100;
  function handleAddTransaction(newTx: Omit<Transaction, "id">) {
    const created: Transaction = {
      ...newTx,
      id: nextId++ as TransactionId,
    };
    mockTransactions = [created, ...mockTransactions];
  }
</script>

<svelte:head>
  <title>Transactions — CoSave</title>
</svelte:head>

<div class="container mx-auto max-w-7xl px-4 py-6">
  <TransactionsView
    transactions={mockTransactions}
    onAddTransaction={handleAddTransaction}
    onUpdateTransaction={handleUpdateTransaction}
    onDeleteTransaction={handleDeleteTransaction}
  />
</div>
