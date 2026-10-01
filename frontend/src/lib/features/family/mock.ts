import type { Family, Member, Account, CurrencyOption, CurrencyCode } from "./types";
import { getBrowserRegion } from "./currency";

export const MOCK_SUPPORTED_CURRENCIES: readonly CurrencyOption[] = [
  { code: "INR", symbol: "₹", name: "Indian Rupee", scale: 2 },
  { code: "USD", symbol: "$", name: "US Dollar", scale: 2 },
  { code: "EUR", symbol: "€", name: "Euro", scale: 2 },
  { code: "GBP", symbol: "£", name: "British Pound", scale: 2 },
  { code: "CAD", symbol: "CA$", name: "Canadian Dollar", scale: 2 },
  { code: "AUD", symbol: "A$", name: "Australian Dollar", scale: 2 },
  { code: "JPY", symbol: "¥", name: "Japanese Yen", scale: 0 },
  { code: "CHF", symbol: "CHF", name: "Swiss Franc", scale: 2 },
  { code: "SGD", symbol: "S$", name: "Singapore Dollar", scale: 2 },
  { code: "NZD", symbol: "NZ$", name: "New Zealand Dollar", scale: 2 },
  { code: "AED", symbol: "AED", name: "UAE Dirham", scale: 2 },
  { code: "CNY", symbol: "¥", name: "Chinese Yuan", scale: 2 },
  { code: "BRL", symbol: "R$", name: "Brazilian Real", scale: 2 },
  { code: "MXN", symbol: "Mex$", name: "Mexican Peso", scale: 2 },
  { code: "KRW", symbol: "₩", name: "South Korean Won", scale: 0 },
  { code: "SEK", symbol: "kr", name: "Swedish Krona", scale: 2 },
  { code: "NOK", symbol: "kr", name: "Norwegian Krone", scale: 2 },
  { code: "DKK", symbol: "kr", name: "Danish Krone", scale: 2 },
  { code: "ZAR", symbol: "R", name: "South African Rand", scale: 2 },
  { code: "HKD", symbol: "HK$", name: "Hong Kong Dollar", scale: 2 },
] as const;

export const SUPPORTED_CURRENCIES = MOCK_SUPPORTED_CURRENCIES;

export const MOCK_REGION_TO_CURRENCY: Record<string, CurrencyCode> = {
  IN: "INR",
  US: "USD",
  GB: "GBP",
  UK: "GBP",
  DE: "EUR",
  FR: "EUR",
  IT: "EUR",
  ES: "EUR",
  NL: "EUR",
  BE: "EUR",
  IE: "EUR",
  PT: "EUR",
  AT: "EUR",
  FI: "EUR",
  GR: "EUR",
  CA: "CAD",
  AU: "AUD",
  JP: "JPY",
  CH: "CHF",
  SG: "SGD",
  NZ: "NZD",
  AE: "AED",
  CN: "CNY",
  BR: "BRL",
  MX: "MXN",
  KR: "KRW",
  SE: "SEK",
  NO: "NOK",
  DK: "DKK",
  ZA: "ZAR",
  HK: "HKD",
};

export const MOCK_TIMEZONE_TO_CURRENCY: Record<string, CurrencyCode> = {
  "Asia/Kolkata": "INR",
  "Asia/Calcutta": "INR",
  "Asia/Tokyo": "JPY",
  "Asia/Singapore": "SGD",
  "Asia/Hong_Kong": "HKD",
  "Asia/Seoul": "KRW",
  "Asia/Dubai": "AED",
  "Asia/Shanghai": "CNY",
  "Europe/London": "GBP",
  "Europe/Paris": "EUR",
  "Europe/Berlin": "EUR",
  "Europe/Rome": "EUR",
  "Europe/Madrid": "EUR",
  "Europe/Amsterdam": "EUR",
  "Europe/Brussels": "EUR",
  "Europe/Zurich": "CHF",
  "Europe/Stockholm": "SEK",
  "Europe/Oslo": "NOK",
  "Europe/Copenhagen": "DKK",
  "Australia/Sydney": "AUD",
  "Australia/Melbourne": "AUD",
  "Pacific/Auckland": "NZD",
  "America/Toronto": "CAD",
  "America/Vancouver": "CAD",
  "America/Sao_Paulo": "BRL",
  "America/Mexico_City": "MXN",
  "Africa/Johannesburg": "ZAR",
};

/**
 * Simulates backend default currency resolution (e.g. GET /api/v1/currencies/default?region=...).
 * Backend checks region, falls back to server/browser timezone, and never defaults to USD.
 */
export function resolveMockDefaultCurrency(region?: string): CurrencyCode {
  if (region && MOCK_REGION_TO_CURRENCY[region]) {
    return MOCK_REGION_TO_CURRENCY[region];
  }
  if (typeof Intl !== "undefined") {
    try {
      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
      if (tz && MOCK_TIMEZONE_TO_CURRENCY[tz]) {
        return MOCK_TIMEZONE_TO_CURRENCY[tz];
      }
      for (const [prefix, curr] of Object.entries(MOCK_TIMEZONE_TO_CURRENCY)) {
        if (tz && tz.startsWith(prefix)) {
          return curr;
        }
      }
    } catch {
      // continue
    }
  }
  return MOCK_SUPPORTED_CURRENCIES[0]?.code ?? "INR";
}

const defaultCurrency = resolveMockDefaultCurrency(getBrowserRegion());

export const MOCK_FAMILY: Family = {
  id: 1,
  name: "The Miller Family",
  currency: defaultCurrency,
  created_at: 1705276800,
};

export const MOCK_MEMBERS: Member[] = [
  {
    id: 1,
    family_id: 1,
    name: "Sarah Miller",
    created_at: 1705276800,
  },
  {
    id: 2,
    family_id: 1,
    name: "David Miller",
    created_at: 1705276800,
  },
  {
    id: 3,
    family_id: 1,
    name: "Emma Miller",
    created_at: 1709251200,
  },
];

export const MOCK_ACCOUNTS: Account[] = [
  {
    id: 101,
    family_id: 1,
    owner_member_id: 1,
    type: "bank_account",
    currency: defaultCurrency,
    bank_name: "Chase",
    account_name: "Total Checking",
    last4: "4821",
    available_balance_cents: 845025,
    created_at: 1705363200,
  },
  {
    id: 102,
    family_id: 1,
    owner_member_id: 1,
    type: "bank_account",
    currency: defaultCurrency,
    bank_name: "Ally Bank",
    account_name: "Savings Bucket",
    last4: "9102",
    available_balance_cents: 2540050,
    created_at: 1705536000,
  },
  {
    id: 201,
    family_id: 1,
    owner_member_id: 1,
    type: "credit_card",
    currency: defaultCurrency,
    bank_name: "American Express",
    card_name: "Gold Card",
    last4: "1004",
    credit_limit_cents: 1500000,
    available_cents: 1325000,
    outstanding_cents: 175000,
    created_at: 1705708800,
  },
  {
    id: 202,
    family_id: 1,
    owner_member_id: 1,
    type: "credit_card",
    currency: defaultCurrency,
    bank_name: "Chase",
    card_name: "Sapphire Preferred",
    last4: "5561",
    credit_limit_cents: 2000000,
    available_cents: 1785000,
    outstanding_cents: 215000,
    created_at: 1707523200,
  },
  {
    id: 103,
    family_id: 1,
    owner_member_id: 2,
    type: "bank_account",
    currency: defaultCurrency,
    bank_name: "Wells Fargo",
    account_name: "Everyday Checking",
    last4: "1140",
    available_balance_cents: 320000,
    created_at: 1705449600,
  },
  {
    id: 104,
    family_id: 1,
    owner_member_id: 2,
    type: "bank_account",
    currency: defaultCurrency,
    bank_name: "Capital One",
    account_name: "360 Performance Savings",
    last4: "7723",
    available_balance_cents: 1850000,
    created_at: 1706745600,
  },
  {
    id: 203,
    family_id: 1,
    owner_member_id: 2,
    type: "credit_card",
    currency: defaultCurrency,
    bank_name: "Goldman Sachs",
    card_name: "Apple Card",
    last4: "8820",
    credit_limit_cents: 1000000,
    available_cents: 920000,
    outstanding_cents: 80000,
    created_at: 1707696000,
  },
  {
    id: 204,
    family_id: 1,
    owner_member_id: 2,
    type: "credit_card",
    currency: defaultCurrency,
    bank_name: "Citi",
    card_name: "Double Cash",
    last4: "4392",
    credit_limit_cents: 1200000,
    available_cents: 1050000,
    outstanding_cents: 150000,
    created_at: 1707955200,
  },
  {
    id: 105,
    family_id: 1,
    owner_member_id: 3,
    type: "bank_account",
    currency: defaultCurrency,
    bank_name: "Charles Schwab",
    account_name: "Investor Checking",
    last4: "3309",
    available_balance_cents: 125000,
    created_at: 1709337600,
  },
  {
    id: 205,
    family_id: 1,
    owner_member_id: 3,
    type: "credit_card",
    currency: defaultCurrency,
    bank_name: "Discover",
    card_name: "Discover it",
    last4: "6210",
    credit_limit_cents: 350000,
    available_cents: 310000,
    outstanding_cents: 40000,
    created_at: 1709596800,
  },
];
