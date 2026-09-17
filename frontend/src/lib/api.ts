/**
 * Standard API error codes matching the Rust backend.
 */
export const Code = {
  Zero: 0,
  BadRequest: 400,
  Unauthorized: 401,
  Conflict: 409,
  InternalError: 500,
  zero: (): 0 => 0,
  badRequest: (): 400 => 400,
  unauthorized: (): 401 => 401,
  conflict: (): 409 => 409,
  internalError: (): 500 => 500,
} as const;

export type Code = 0 | 400 | 401 | 409 | 500;

/**
 * Standard status strings returned in API response envelopes.
 */
export const Status = {
  Healthy: "HEALTHY",
  Ok: "OK",
  BadRequest: "BAD_REQUEST",
  Unauthenticated: "UNAUTHENTICATED",
  InvalidCredentials: "INVALID_CREDENTIALS",
  UserExists: "USER_EXISTS",
  InternalError: "INTERNAL_ERROR",
  healthy: (): "HEALTHY" => "HEALTHY",
  ok: (): "OK" => "OK",
  badRequest: (): "BAD_REQUEST" => "BAD_REQUEST",
  unauthenticated: (): "UNAUTHENTICATED" => "UNAUTHENTICATED",
  invalidCredentials: (): "INVALID_CREDENTIALS" => "INVALID_CREDENTIALS",
  userExists: (): "USER_EXISTS" => "USER_EXISTS",
  internalError: (): "INTERNAL_ERROR" => "INTERNAL_ERROR",
} as const;

export type Status =
  | "HEALTHY"
  | "OK"
  | "BAD_REQUEST"
  | "UNAUTHENTICATED"
  | "INVALID_CREDENTIALS"
  | "USER_EXISTS"
  | "INTERNAL_ERROR";

/**
 * Canonical JSON response envelope matching backend `ApiResponse<T>`.
 */
export interface ApiResponse<T> {
  code: Code;
  status: Status;
  data: T;
}

/**
 * System roles available for user accounts.
 */
export type Role = "admin" | "member";

/**
 * Public user representation returned by auth endpoints.
 */
export interface UserDto {
  id: string;
  name: string;
  role: Role;
  created_at: number;
}

/**
 * Registration request payload.
 */
export interface RegisterPayload {
  name: string;
  password: string;
}

/**
 * Login request payload.
 */
export interface LoginPayload {
  name: string;
  password: string;
}

/**
 * Base error for API communication failures or contract mismatches.
 */
export class ApiError extends Error {
  public readonly details: unknown;
  public readonly httpStatus: number;
  public readonly apiStatus: Status | null;
  public readonly code: Code | null;

  constructor(
    message: string,
    details: unknown = null,
    httpStatus = 500,
    apiStatus: Status | null = null,
    code: Code | null = null,
  ) {
    super(message);
    this.name = "ApiError";
    this.details = details;
    this.httpStatus = httpStatus;
    this.apiStatus = apiStatus;
    this.code = code;
  }
}

/**
 * Thrown when an endpoint returns an unexpected numeric response code.
 */
export class UnanticipatedCodeError extends ApiError {
  constructor(public readonly rawCode: unknown) {
    super(`Unanticipated API response code: ${JSON.stringify(rawCode)}`);
    this.name = "UnanticipatedCodeError";
  }
}

/**
 * Thrown when an endpoint returns an unrecognized status string.
 */
export class UnanticipatedStatusError extends ApiError {
  constructor(public readonly rawStatus: unknown) {
    super(`Unanticipated API response status: ${JSON.stringify(rawStatus)}`);
    this.name = "UnanticipatedStatusError";
  }
}

/**
 * Thrown when an API response payload violates its expected schema contract.
 */
export class ContractViolationError extends ApiError {
  constructor(message: string, details: unknown = null) {
    super(`API Contract Violation: ${message}`, details);
    this.name = "ContractViolationError";
  }
}

/**
 * Type guard verifying whether a value is a non-null, non-array object record.
 */
export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * Validates and narrows an unknown value to a known API Code.
 */
export function parseCode(rawCode: unknown): Code {
  switch (rawCode) {
    case 0:
    case Code.Zero:
      return Code.Zero;
    case 400:
    case Code.BadRequest:
      return Code.BadRequest;
    case 401:
    case Code.Unauthorized:
      return Code.Unauthorized;
    case 409:
    case Code.Conflict:
      return Code.Conflict;
    case 500:
    case Code.InternalError:
      return Code.InternalError;
    default:
      throw new UnanticipatedCodeError(rawCode);
  }
}

/**
 * Validates and narrows an unknown value to a known API Status.
 */
export function parseStatus(rawStatus: unknown): Status {
  switch (rawStatus) {
    case "HEALTHY":
    case Status.Healthy:
      return Status.Healthy;
    case "OK":
    case Status.Ok:
      return Status.Ok;
    case "BAD_REQUEST":
    case Status.BadRequest:
      return Status.BadRequest;
    case "UNAUTHENTICATED":
    case Status.Unauthenticated:
      return Status.Unauthenticated;
    case "INVALID_CREDENTIALS":
    case Status.InvalidCredentials:
      return Status.InvalidCredentials;
    case "USER_EXISTS":
    case Status.UserExists:
      return Status.UserExists;
    case "INTERNAL_ERROR":
    case Status.InternalError:
      return Status.InternalError;
    default:
      throw new UnanticipatedStatusError(rawStatus);
  }
}

/**
 * Validates and narrows an unknown value to a Role enum.
 */
export function parseRole(raw: unknown): Role {
  if (raw === "admin" || raw === "member") {
    return raw;
  }
  throw new ContractViolationError(`Invalid user role: ${JSON.stringify(raw)}`);
}

/**
 * Validates and narrows raw JSON data to a strongly-typed UserDto.
 */
export function parseUserDto(raw: unknown): UserDto {
  if (!isObject(raw)) {
    throw new ContractViolationError("UserDto payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("UserDto.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("UserDto.name must be a string", raw);
  }
  if (typeof raw.created_at !== "number") {
    throw new ContractViolationError("UserDto.created_at must be a number", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    role: parseRole(raw.role),
    created_at: raw.created_at,
  };
}

/**
 * Validates and narrows nullable raw JSON data to UserDto or null.
 */
export function parseNullableUserDto(raw: unknown): UserDto | null {
  if (raw === null || raw === undefined) {
    return null;
  }
  return parseUserDto(raw);
}

/**
 * Validates and narrows raw JSON data to null.
 */
export function parseNull(raw: unknown): null {
  if (raw === null || raw === undefined) {
    return null;
  }
  throw new ContractViolationError(`Expected null response data, got: ${JSON.stringify(raw)}`);
}

/**
 * Extracts and validates the numeric code from an API response object.
 */
export function extractCode(response: unknown): Code {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  return parseCode((response as Record<string, unknown>).code);
}

/**
 * Extracts and validates the status string from an API response object.
 */
export function extractStatus(response: unknown): Status {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  return parseStatus((response as Record<string, unknown>).status);
}

/**
 * Extracts the payload data from an API response, verifying code and status first.
 */
export function extractData<T>(response: unknown, parser?: (data: unknown) => T): T {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  const res = response as Record<string, unknown>;
  parseCode(res.code);
  parseStatus(res.status);
  return parser ? parser(res.data) : (res.data as T);
}

/**
 * Validates and extracts a typed ApiResponse envelope from an unknown response object.
 * When a parser is provided, response data is strictly validated and narrowed at runtime.
 */
export function extractApiResponse<T>(
  response: unknown,
  parser?: (data: unknown) => T,
): ApiResponse<T> {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  const res = response as Record<string, unknown>;
  const code = parseCode(res.code);
  const status = parseStatus(res.status);
  const data = parser ? parser(res.data) : (res.data as T);
  return {
    code,
    status,
    data,
  };
}

/**
 * Typed wrapper around fetch that parses and validates the standardized API response envelope.
 */
export async function apiFetch<T>(
  url: string,
  init: RequestInit = {},
  parser?: (data: unknown) => T,
): Promise<ApiResponse<T>> {
  const headers = new Headers(init.headers || {});
  if (!headers.has("Content-Type") && init.body) {
    headers.set("Content-Type", "application/json");
  }

  const res = await fetch(url, {
    credentials: "same-origin",
    ...init,
    headers,
  });

  const json: unknown = await res.json().catch(() => null);

  if (!res.ok) {
    if (json && typeof json === "object") {
      try {
        const envelope = extractApiResponse<unknown>(json);
        const apiErr = new ApiError(
          `API Error: ${envelope.status}`,
          envelope.data,
          res.status,
          envelope.status,
          envelope.code,
        );
        console.warn(
          `[cosave:api] ${init.method || "GET"} ${url} failed (${res.status}):`,
          envelope.status,
        );
        throw apiErr;
      } catch (err) {
        if (err instanceof ApiError) {
          throw err;
        }
      }
    }
    const httpErr = new ApiError(`HTTP ${res.status}: ${res.statusText}`, null, res.status);
    console.warn(`[cosave:api] ${init.method || "GET"} ${url} HTTP failure:`, httpErr.message);
    throw httpErr;
  }

  try {
    return extractApiResponse<T>(json, parser);
  } catch (err) {
    console.error(`[cosave:api] ${init.method || "GET"} ${url} contract violation:`, err);
    throw err;
  }
}

/**
 * Auth API service functions.
 */
export const authApi = {
  async register(payload: RegisterPayload): Promise<ApiResponse<UserDto | null>> {
    return apiFetch<UserDto | null>(
      "/api/v1/auth/register",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseNullableUserDto,
    );
  },

  async login(payload: LoginPayload): Promise<ApiResponse<UserDto | null>> {
    return apiFetch<UserDto | null>(
      "/api/v1/auth/login",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseNullableUserDto,
    );
  },

  async logout(): Promise<ApiResponse<null>> {
    return apiFetch<null>(
      "/api/v1/auth/logout",
      {
        method: "POST",
      },
      parseNull,
    );
  },

  async me(): Promise<ApiResponse<UserDto>> {
    return apiFetch<UserDto>("/api/v1/auth/me", {}, parseUserDto);
  },
};

export interface TransactionTypeItem {
  id: string;
  name: string;
  color: string;
}

export interface SubcategoryItem {
  id: string;
  name: string;
}

export interface CategoryItem {
  id: string;
  name: string;
  type: string;
  subcategories: SubcategoryItem[];
}

export interface CategoryHierarchyResponse {
  types: TransactionTypeItem[];
  categories: CategoryItem[];
}

/**
 * Request payload to create a new transaction type.
 */
export interface CreateTypePayload {
  name: string;
  color: string;
}

/**
 * Request payload to update the display color of a transaction type.
 */
export interface UpdateTypeColorPayload {
  color: string;
}

/**
 * Request payload to create a new category under a transaction type.
 */
export interface CreateCategoryPayload {
  type_name: string;
  name: string;
}

/**
 * Generic request payload to rename an entity (category or subcategory).
 */
export interface UpdateNamePayload {
  name: string;
}

/**
 * Request payload to create a new subcategory under an existing category.
 */
export interface CreateSubcategoryPayload {
  category_id: string;
  name: string;
}

/**
 * Validates and narrows raw JSON data to a strongly-typed TransactionTypeItem.
 */
export function parseTransactionTypeItem(raw: unknown): TransactionTypeItem {
  if (!isObject(raw)) {
    throw new ContractViolationError("TransactionTypeItem payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("TransactionTypeItem.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("TransactionTypeItem.name must be a string", raw);
  }
  if (typeof raw.color !== "string") {
    throw new ContractViolationError("TransactionTypeItem.color must be a string", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    color: raw.color,
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed SubcategoryItem.
 */
export function parseSubcategoryItem(raw: unknown): SubcategoryItem {
  if (!isObject(raw)) {
    throw new ContractViolationError("SubcategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("SubcategoryItem.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("SubcategoryItem.name must be a string", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CategoryItem.
 */
export function parseCategoryItem(raw: unknown): CategoryItem {
  if (!isObject(raw)) {
    throw new ContractViolationError("CategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("CategoryItem.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("CategoryItem.name must be a string", raw);
  }
  if (typeof raw.type !== "string") {
    throw new ContractViolationError("CategoryItem.type must be a string", raw);
  }
  if (!Array.isArray(raw.subcategories)) {
    throw new ContractViolationError("CategoryItem.subcategories must be an array", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    type: raw.type,
    subcategories: raw.subcategories.map(parseSubcategoryItem),
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CategoryHierarchyResponse.
 */
export function parseCategoryHierarchyResponse(raw: unknown): CategoryHierarchyResponse {
  if (!isObject(raw)) {
    throw new ContractViolationError("CategoryHierarchyResponse payload must be an object", raw);
  }
  if (!Array.isArray(raw.types)) {
    throw new ContractViolationError("CategoryHierarchyResponse.types must be an array", raw);
  }
  if (!Array.isArray(raw.categories)) {
    throw new ContractViolationError("CategoryHierarchyResponse.categories must be an array", raw);
  }
  return {
    types: raw.types.map(parseTransactionTypeItem),
    categories: raw.categories.map(parseCategoryItem),
  };
}

/**
 * Category & Transaction Hierarchy API service functions.
 * The Rust backend is the authoritative Single Source of Truth (SSOT).
 */
export const categoriesApi = {
  async getHierarchy(): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      "/api/v1/categories",
      {},
      parseCategoryHierarchyResponse,
    );
  },

  async createType(payload: CreateTypePayload): Promise<ApiResponse<TransactionTypeItem>> {
    return apiFetch<TransactionTypeItem>(
      "/api/v1/categories/types",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseTransactionTypeItem,
    );
  },

  async updateTypeColor(id: string, color: string): Promise<ApiResponse<null>> {
    const payload: UpdateTypeColorPayload = { color };
    return apiFetch<null>(
      `/api/v1/categories/types/${encodeURIComponent(id)}/color`,
      {
        method: "PATCH",
        body: JSON.stringify(payload),
      },
      parseNull,
    );
  },

  async deleteType(id: string): Promise<ApiResponse<null>> {
    return apiFetch<null>(
      `/api/v1/categories/types/${encodeURIComponent(id)}`,
      {
        method: "DELETE",
      },
      parseNull,
    );
  },

  async createCategory(payload: CreateCategoryPayload): Promise<ApiResponse<CategoryItem>> {
    return apiFetch<CategoryItem>(
      "/api/v1/categories",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseCategoryItem,
    );
  },

  async updateCategory(id: string, name: string): Promise<ApiResponse<null>> {
    const payload: UpdateNamePayload = { name };
    return apiFetch<null>(
      `/api/v1/categories/${encodeURIComponent(id)}`,
      {
        method: "PATCH",
        body: JSON.stringify(payload),
      },
      parseNull,
    );
  },

  async deleteCategory(id: string): Promise<ApiResponse<null>> {
    return apiFetch<null>(
      `/api/v1/categories/${encodeURIComponent(id)}`,
      {
        method: "DELETE",
      },
      parseNull,
    );
  },

  async createSubcategory(
    payload: CreateSubcategoryPayload,
  ): Promise<ApiResponse<SubcategoryItem>> {
    return apiFetch<SubcategoryItem>(
      "/api/v1/categories/subcategories",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseSubcategoryItem,
    );
  },

  async updateSubcategory(id: string, name: string): Promise<ApiResponse<null>> {
    const payload: UpdateNamePayload = { name };
    return apiFetch<null>(
      `/api/v1/categories/subcategories/${encodeURIComponent(id)}`,
      {
        method: "PATCH",
        body: JSON.stringify(payload),
      },
      parseNull,
    );
  },

  async deleteSubcategory(id: string): Promise<ApiResponse<null>> {
    return apiFetch<null>(
      `/api/v1/categories/subcategories/${encodeURIComponent(id)}`,
      {
        method: "DELETE",
      },
      parseNull,
    );
  },

  async resetDefaults(): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      "/api/v1/categories/reset",
      {
        method: "POST",
      },
      parseCategoryHierarchyResponse,
    );
  },
};
