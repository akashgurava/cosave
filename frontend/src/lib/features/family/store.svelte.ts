import { MOCK_FAMILY, MOCK_MEMBERS, MOCK_ACCOUNTS } from "./mock";
import type {
  Family,
  Member,
  Account,
  BankAccount,
  CreditCardAccount,
  CreateMemberInput,
  UpdateMemberInput,
  CreateBankAccountInput,
  UpdateBankAccountInput,
  CreateCreditCardInput,
  UpdateCreditCardInput,
} from "./types";

class FamilyStore {
  family = $state<Family>({ ...MOCK_FAMILY });
  members = $state<Member[]>([...MOCK_MEMBERS]);
  accounts = $state<Account[]>([...MOCK_ACCOUNTS]);
  selectedMemberId = $state<string>(MOCK_MEMBERS[0]?.id ?? "");

  // Computed aggregates
  totalCreditLimit: number = $derived(
    this.accounts
      .filter((a): a is CreditCardAccount => a.type === "credit_card")
      .reduce((sum: number, a: CreditCardAccount) => sum + a.creditLimit, 0),
  );

  totalBankAccounts: number = $derived(
    this.accounts.filter((a: Account) => a.type === "bank_account").length,
  );

  totalCreditCards: number = $derived(
    this.accounts.filter((a: Account) => a.type === "credit_card").length,
  );

  getMember(id: string): Member | null {
    return this.members.find((m: Member) => m.id === id) ?? null;
  }

  getMemberAccounts(memberId: string): Account[] {
    return this.accounts.filter((a: Account) => a.ownerMemberId === memberId);
  }

  getMemberBankAccounts(memberId: string): BankAccount[] {
    return this.accounts.filter(
      (a: Account): a is BankAccount => a.ownerMemberId === memberId && a.type === "bank_account",
    );
  }

  getMemberCreditCards(memberId: string): CreditCardAccount[] {
    return this.accounts.filter(
      (a: Account): a is CreditCardAccount =>
        a.ownerMemberId === memberId && a.type === "credit_card",
    );
  }

  getMemberCreditLimit(memberId: string): number {
    return this.getMemberCreditCards(memberId).reduce(
      (sum: number, c: CreditCardAccount) => sum + c.creditLimit,
      0,
    );
  }

  addMember(input: CreateMemberInput): Member {
    const newMember: Member = {
      id: `mem_${Date.now()}`,
      familyId: this.family.id,
      name: input.name.trim(),
      createdAt: new Date().toISOString(),
    };
    this.members.push(newMember);
    if (!this.selectedMemberId) {
      this.selectedMemberId = newMember.id;
    }
    return newMember;
  }

  updateMember(input: UpdateMemberInput): boolean {
    const m = this.members.find((item: Member) => item.id === input.id);
    if (!m) {
      return false;
    }
    m.name = input.name.trim();
    return true;
  }

  deleteMember(id: string): void {
    this.members = this.members.filter((m: Member) => m.id !== id);
    this.accounts = this.accounts.filter((a: Account) => a.ownerMemberId !== id);
    if (this.selectedMemberId === id) {
      this.selectedMemberId = this.members[0]?.id ?? "";
    }
  }

  addBankAccount(input: CreateBankAccountInput): BankAccount {
    const newAcc: BankAccount = {
      id: `acc_${Date.now()}`,
      familyId: this.family.id,
      ownerMemberId: input.ownerMemberId,
      type: "bank_account",
      bankName: input.bankName.trim(),
      last4: input.last4.trim(),
      createdAt: new Date().toISOString(),
    };
    this.accounts.push(newAcc);
    return newAcc;
  }

  updateBankAccount(input: UpdateBankAccountInput): boolean {
    const acc = this.accounts.find(
      (a: Account): a is BankAccount => a.id === input.id && a.type === "bank_account",
    );
    if (!acc) {
      return false;
    }
    acc.bankName = input.bankName.trim();
    acc.last4 = input.last4.trim();
    return true;
  }

  addCreditCard(input: CreateCreditCardInput): CreditCardAccount {
    const newCard: CreditCardAccount = {
      id: `acc_${Date.now()}`,
      familyId: this.family.id,
      ownerMemberId: input.ownerMemberId,
      type: "credit_card",
      bankName: input.bankName.trim(),
      cardName: input.cardName.trim(),
      last4: input.last4.trim(),
      creditLimit: Math.max(0, input.creditLimit),
      createdAt: new Date().toISOString(),
    };
    this.accounts.push(newCard);
    return newCard;
  }

  updateCreditCard(input: UpdateCreditCardInput): boolean {
    const card = this.accounts.find(
      (a: Account): a is CreditCardAccount => a.id === input.id && a.type === "credit_card",
    );
    if (!card) {
      return false;
    }
    card.bankName = input.bankName.trim();
    card.cardName = input.cardName.trim();
    card.last4 = input.last4.trim();
    card.creditLimit = Math.max(0, input.creditLimit);
    return true;
  }

  deleteAccount(id: string): void {
    this.accounts = this.accounts.filter((a: Account) => a.id !== id);
  }

  resetToDefaults(): void {
    this.family = { ...MOCK_FAMILY };
    this.members = [...MOCK_MEMBERS];
    this.accounts = [...MOCK_ACCOUNTS];
    this.selectedMemberId = MOCK_MEMBERS[0]?.id ?? "";
  }
}

export const familyStore = new FamilyStore();
