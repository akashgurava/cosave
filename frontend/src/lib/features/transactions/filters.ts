import type { DatePreset, Transaction, TransactionFilters } from "./types";

/**
 * Resolves a DatePreset relative to a reference date into concrete startDate and endDate strings (YYYY-MM-DD).
 */
export function resolveDatePresetToRange(
  preset: DatePreset,
  referenceDateStr = "2026-10-05",
): { startDate?: string; endDate?: string } {
  if (preset === "all" || preset === "custom") {
    return {};
  }

  const parts = referenceDateStr.split("-");
  const year = Number(parts[0]);
  const month = Number(parts[1]);
  const day = Number(parts[2]);
  if (!year || !month || !day) return {};

  const endDate = referenceDateStr;
  const d = new Date(year, month - 1, day);

  switch (preset) {
    case "1d":
      // Today (anchor date)
      break;
    case "3d":
      d.setDate(d.getDate() - 2); // 3 days inclusive: day-2, day-1, day
      break;
    case "7d":
      d.setDate(d.getDate() - 6); // 7 days inclusive: day-6 .. day
      break;
    case "1m":
      d.setMonth(d.getMonth() - 1);
      break;
    case "3m":
      d.setMonth(d.getMonth() - 3);
      break;
    case "6m":
      d.setMonth(d.getMonth() - 6);
      break;
    case "1y":
      d.setFullYear(d.getFullYear() - 1);
      break;
  }

  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const dt = String(d.getDate()).padStart(2, "0");
  const startDate = `${y}-${m}-${dt}`;

  return { startDate, endDate };
}

/**
 * Returns the YYYY-MM-DD cutoff date for a given DatePreset relative to an anchor date.
 */
export function getDatePresetCutoff(
  preset: DatePreset,
  referenceDateStr = "2026-10-05",
): string | null {
  const range = resolveDatePresetToRange(preset, referenceDateStr);
  return range.startDate ?? null;
}

/**
 * Pure filter evaluator applying all criteria in TransactionFilters.
 */
export function applyFilters(
  transactions: readonly Transaction[],
  filters: TransactionFilters,
  referenceDateStr?: string,
): readonly Transaction[] {
  const maxTxDate = transactions.reduce((max, t) => (t.date > max ? t.date : max), "");
  const baseRefDate = referenceDateStr ?? (maxTxDate || "2026-10-05");

  return transactions.filter((tx) => {
    // 1. Text Search Query
    if (filters.searchQuery.trim().length > 0) {
      const q = filters.searchQuery.trim().toLowerCase();
      const matchDesc = (tx.description ?? "").toLowerCase().includes(q);
      const matchPayee = tx.payee.toLowerCase().includes(q);
      const matchNotes = (tx.notes ?? "").toLowerCase().includes(q);
      if (!matchDesc && !matchPayee && !matchNotes) return false;
    }

    // 2. Date Range Filter
    if (filters.startDate !== undefined && tx.date < filters.startDate) {
      return false;
    }
    if (filters.endDate !== undefined && tx.date > filters.endDate) {
      return false;
    }

    // 3. Amount Filter
    const absAmount = Math.abs(tx.amount);
    if (filters.amountPreset !== "all") {
      switch (filters.amountPreset) {
        case "lt100":
          if (absAmount >= 100 * 100) return false;
          break;
        case "lt500":
          if (absAmount >= 500 * 100) return false;
          break;
        case "lt1000":
          if (absAmount >= 1000 * 100) return false;
          break;
        case "lt2000":
          if (absAmount >= 2000 * 100) return false;
          break;
        case "gte2000":
          if (absAmount < 2000 * 100) return false;
          break;
        case "custom":
          if (filters.customAmountMin !== undefined && absAmount < filters.customAmountMin) {
            return false;
          }
          if (filters.customAmountMax !== undefined && absAmount > filters.customAmountMax) {
            return false;
          }
          break;
      }
    }

    // 4. Member filter
    if (filters.selectedMemberIds.length > 0) {
      if (!filters.selectedMemberIds.includes(tx.memberId)) return false;
    }

    // 5. Account filter
    if (filters.selectedAccountIds.length > 0) {
      if (!filters.selectedAccountIds.includes(tx.accountId)) return false;
    }

    // 6. Type filter (by typeId)
    if (filters.selectedTypeIds.length > 0) {
      if (!filters.selectedTypeIds.includes(tx.typeId)) return false;
    }

    // 7. Category filter
    if (filters.selectedCategoryIds.length > 0) {
      if (!filters.selectedCategoryIds.includes(tx.categoryId)) return false;
    }

    // 8. Subcategory filter
    if (filters.selectedSubcategoryIds.length > 0) {
      const matchUndefined = filters.selectedSubcategoryIds.some((s) => Number(s) <= 0);
      const regularIds = filters.selectedSubcategoryIds.filter((s) => Number(s) > 0);
      const matchesNone = matchUndefined && tx.subcategoryId === undefined;
      const matchesRegular =
        tx.subcategoryId !== undefined && regularIds.includes(tx.subcategoryId);
      if (!matchesNone && !matchesRegular) {
        return false;
      }
    }

    // 9. Status filter
    if (filters.selectedStatuses.length > 0) {
      if (!filters.selectedStatuses.includes(tx.status)) return false;
    }

    return true;
  });
}

/**
 * Parses user input currency string ($XX.XX or XX.XX) into MinorUnits integer.
 */
export function parseCurrencyInput(value: string): number | null {
  const cleaned = value.replace(/[^0-9.-]/g, "").trim();
  if (cleaned.length === 0) return null;
  const num = parseFloat(cleaned);
  if (Number.isNaN(num)) return null;
  return Math.round(num * 100);
}
