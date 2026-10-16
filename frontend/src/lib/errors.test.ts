import { describe, expect, it } from "vitest";
import { ParseEnumError } from "./errors";

describe("ParseEnumError", () => {
  it("inherits from Error and preserves structured diagnostic payload", () => {
    const error = new ParseEnumError("TransactionSource", ["manual", "import"], "crypto");
    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(ParseEnumError);
    expect(error.name).toBe("ParseEnumError");
    expect(error.enumName).toBe("TransactionSource");
    expect(error.allowedVariants).toEqual(["manual", "import"]);
    expect(error.receivedValue).toBe("crypto");
  });

  it("safely preserves diverse received data types without throwing", () => {
    const nullErr = new ParseEnumError("TransactionStatus", ["cleared", "pending"], null);
    expect(nullErr.receivedValue).toBeNull();

    const numErr = new ParseEnumError("AccountType", ["bank_account", "credit_card"], 123);
    expect(numErr.receivedValue).toBe(123);

    const obj = { unexpected: true };
    const objErr = new ParseEnumError("ThemeMode", ["dark", "light"], obj);
    expect(objErr.receivedValue).toBe(obj);
  });
});
