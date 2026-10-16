/**
 * Core runtime errors for frontend contract and schema validations.
 */

/**
 * Thrown when an input value does not match any allowed enum variant.
 */
export class ParseEnumError extends Error {
  public override readonly name = "ParseEnumError";

  constructor(
    public readonly enumName: string,
    public readonly allowedVariants: readonly string[],
    public readonly receivedValue: unknown,
  ) {
    const formattedVariants = allowedVariants.map((v) => `'${v}'`).join(" | ");
    super(
      `ParseEnumError. ENUM Name: ${enumName}. Allowed Variants: ${formattedVariants}. Received Value: ${JSON.stringify(receivedValue)}.`,
    );
  }
}
