/**
 * Core runtime errors for frontend contract and schema validations.
 */

import type { z } from "zod";

/**
 * Normalized issue codes wrapping Zod's internal error taxonomy (`z.ZodIssue`).
 */
class ModelIssueCode {
  public static readonly InvalidType = new ModelIssueCode("invalid_type");
  public static readonly InvalidEnum = new ModelIssueCode("invalid_enum");
  public static readonly MissingField = new ModelIssueCode("missing_field");
  public static readonly ConstraintViolation = new ModelIssueCode("constraint_violation");
  public static readonly Custom = new ModelIssueCode("custom");

  public readonly value: string;

  private constructor(value: string) {
    this.value = value;
  }

  isInvalidType(): boolean {
    return this === ModelIssueCode.InvalidType;
  }

  isInvalidEnum(): boolean {
    return this === ModelIssueCode.InvalidEnum;
  }

  isMissingField(): boolean {
    return this === ModelIssueCode.MissingField;
  }

  isConstraintViolation(): boolean {
    return this === ModelIssueCode.ConstraintViolation;
  }

  isCustom(): boolean {
    return this === ModelIssueCode.Custom;
  }

  toString(): string {
    return this.value;
  }
}

/**
 * Diagnostic issue extracted from a Zod schema validation failure.
 */
interface ModelIssue {
  readonly path: string;
  readonly code: ModelIssueCode;
  readonly message: string;
}

function getPathValue(obj: unknown, path: readonly PropertyKey[]): unknown {
  let curr = obj;
  for (const segment of path) {
    if (curr === null || typeof curr !== "object") return undefined;
    curr = (curr as Record<PropertyKey, unknown>)[segment];
  }
  return curr;
}

/**
 * Translates a Zod validation issue into a ModelIssue with normalized code and path.
 */
export function normalizeZodIssue(issue: z.ZodIssue, raw: unknown, modelName: string): ModelIssue {
  const path = issue.path.length > 0 ? issue.path.join(".") : modelName;

  let code: ModelIssueCode = ModelIssueCode.Custom;
  if (issue.code === "invalid_type") {
    const rawVal = getPathValue(raw, issue.path);
    code = rawVal === undefined ? ModelIssueCode.MissingField : ModelIssueCode.InvalidType;
  } else if (issue.code === "invalid_value") {
    code = ModelIssueCode.InvalidEnum;
  } else if (issue.code === "too_small" || issue.code === "too_big") {
    code = ModelIssueCode.ConstraintViolation;
  }

  return {
    path,
    code,
    message: issue.message,
  };
}

/**
 * Thrown when an input payload fails domain model schema validation.
 */
export class ParseModelError extends Error {
  public override readonly name = "ParseModelError";
  public readonly modelName: string;
  public readonly issues: readonly ModelIssue[];
  public readonly raw: unknown;

  constructor(modelName: string, issues: readonly ModelIssue[], raw: unknown) {
    const details = issues.map((i) => `${i.path}: [${i.code}] ${i.message}`).join("; ");
    super(`[${modelName}] Validation failed: ${details}`);
    this.modelName = modelName;
    this.issues = issues;
    this.raw = raw;
  }
}

/**
 * Thrown when an input value does not match any allowed enum variant.
 */
export class ParseEnumError extends Error {
  public override readonly name = "ParseEnumError";
  public readonly enumName: string;
  public readonly allowedVariants: readonly string[];
  public readonly receivedValue: unknown;

  constructor(enumName: string, allowedVariants: readonly string[], receivedValue: unknown) {
    const formattedVariants = allowedVariants.map((v) => `'${v}'`).join(" | ");
    super(
      `ParseEnumError. ENUM Name: ${enumName}. Allowed Variants: ${formattedVariants}. Received Value: ${JSON.stringify(receivedValue)}.`,
    );
    this.enumName = enumName;
    this.allowedVariants = allowedVariants;
    this.receivedValue = receivedValue;
  }
}

/**
 * Thrown when a value cannot be converted or validated for a branded ID or type.
 */
export class IdCreationError extends Error {
  public override readonly name: string = "IdCreationError";
  public readonly action: string;
  public readonly targetType: string;
  public readonly idName: string;
  public readonly raw: unknown;

  constructor(targetType: string, idName: string, raw: unknown) {
    super(`Type: ${targetType}. ID Name: ${idName} Value: ${String(raw)}. Failed to create ID.`);
    this.action = `UI.ID_CREATION.TYPE_CONVERSION.${idName}`;
    this.targetType = targetType;
    this.idName = idName;
    this.raw = raw;
  }
}

/**
 * Thrown when an internal rule that must always be true is broken (e.g. data that
 * is guaranteed to exist by system design is missing).
 *
 * This indicates an internal programming bug or corrupted application state,
 * rather than a user mistake. Includes a unique action code pinpointing exactly
 * where the failure occurred.
 */
export class InvariantViolationError extends Error {
  public override readonly name: string = "InvariantViolationError";
  public readonly action: string;

  constructor(action: string, message: string) {
    super(`[${action}] InvariantViolation: ${message}`);
    this.action = action;
  }
}
