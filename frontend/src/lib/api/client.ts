/**
 * REST API client with URL interpolation and response envelope parsing.
 *
 * Provides typed methods (`get`, `post`, `patch`, `delete`) that execute
 * requests through a configured TransportAdapter, unwrap backend envelopes,
 * and enforce contract schemas.
 */

import {
  Status,
  type ApiResponse,
  ApiError,
  ContractViolationError,
  parseCode,
  parseStatus,
  isObject,
} from "./contracts";
import { type TransportAdapter, type TransportResponse, FetchTransportAdapter } from "./transport";

export * from "./contracts";
export * from "./transport";

export interface RequestOptions<T = unknown> {
  pathParams?: Record<string, string | number>;
  query?: Record<
    string,
    string | number | boolean | readonly (string | number | boolean)[] | null | undefined
  >;
  headers?: Record<string, string>;
  signal?: AbortSignal;
  schema?: (data: unknown) => T;
}

export function buildUrl(
  path: string,
  pathParams?: Record<string, string | number>,
  query?: Record<
    string,
    string | number | boolean | readonly (string | number | boolean)[] | null | undefined
  >,
): string {
  let url = path;
  if (pathParams !== undefined) {
    for (const [key, val] of Object.entries(pathParams)) {
      const encoded = encodeURIComponent(String(val));
      url = url.replaceAll(`:${key}`, encoded).replaceAll(`{${key}}`, encoded);
    }
  }
  if (query !== undefined) {
    const searchParams = new URLSearchParams();
    for (const [k, v] of Object.entries(query)) {
      if (v === null || v === undefined) continue;
      if (Array.isArray(v)) {
        for (const item of v) {
          if (item !== null && item !== undefined) {
            searchParams.append(k, String(item));
          }
        }
      } else {
        searchParams.set(k, String(v));
      }
    }
    const qs = searchParams.toString();
    if (qs.length > 0) {
      url += (url.includes("?") ? "&" : "?") + qs;
    }
  }
  return url;
}

let activeTransport: TransportAdapter = new FetchTransportAdapter();

async function executeRequestEnvelope<T>(
  method: "GET" | "POST" | "PATCH" | "DELETE",
  path: string,
  body?: unknown,
  options: RequestOptions<T> = {},
): Promise<ApiResponse<T>> {
  const url = buildUrl(path, options.pathParams, options.query);
  const headers: Record<string, string> = {
    Accept: "application/json",
    ...options.headers,
  };

  let bodyString: string | undefined = undefined;
  if (body !== undefined) {
    bodyString = typeof body === "string" ? body : JSON.stringify(body);
    if (headers["Content-Type"] === undefined) {
      headers["Content-Type"] = "application/json";
    }
  }

  let res: TransportResponse;
  try {
    res = await activeTransport.fetch({
      url,
      method,
      headers,
      body: bodyString,
      signal: options.signal,
    });
  } catch (err: unknown) {
    throw new ApiError(
      err instanceof Error ? err.message : "Network request failed",
      0,
      0,
      "NETWORK_ERROR",
      err,
    );
  }

  const json: unknown = await res.json().catch(() => null);

  const isObj = typeof json === "object" && json !== null;
  const rawCode = isObj && "code" in json ? Number((json as Record<string, unknown>).code) : 0;
  const rawStatus =
    isObj && "status" in json ? String((json as Record<string, unknown>).status) : "";
  const rawData = isObj && "data" in json ? (json as Record<string, unknown>).data : null;

  if (
    res.status >= 400 ||
    rawCode !== 0 ||
    (rawStatus.length > 0 && rawStatus !== "OK" && rawStatus !== "HEALTHY")
  ) {
    let errorDetails = typeof rawData === "string" ? rawData : "";
    let action: string | null = null;

    if (isObject(rawData)) {
      if (typeof rawData.message === "string") {
        errorDetails = rawData.message;
      }
      if (typeof rawData.action === "string") {
        action = rawData.action;
      }
    }

    const statusMsg =
      rawStatus.length > 0 ? rawStatus : res.statusText.length > 0 ? res.statusText : "ERROR";
    const httpStatus = res.status >= 400 ? res.status : rawCode >= 400 ? rawCode : 500;
    const finalMessage =
      errorDetails.length > 0 ? errorDetails : `API Error (${httpStatus}): ${statusMsg}`;
    throw new ApiError(
      finalMessage,
      httpStatus,
      rawCode !== 0 ? rawCode : httpStatus,
      statusMsg,
      rawData,
      action,
    );
  }

  const payload =
    (rawData !== null && rawData !== undefined) || (isObj && "data" in json) ? rawData : json;

  let validatedData: T;
  if (options.schema !== undefined) {
    try {
      validatedData = options.schema(payload);
    } catch (err) {
      if (err instanceof ApiError) throw err;
      throw new ContractViolationError(
        err instanceof Error ? err.message : "Schema validation failed",
        payload,
      );
    }
  } else {
    validatedData = payload as T;
  }

  const code = parseCode(rawCode);
  const status = rawStatus.length > 0 ? parseStatus(rawStatus) : Status.Ok;

  return {
    code,
    status,
    data: validatedData,
  };
}

async function executeRequest<T>(
  method: "GET" | "POST" | "PATCH" | "DELETE",
  path: string,
  body?: unknown,
  options: RequestOptions<T> = {},
): Promise<T> {
  const envelope = await executeRequestEnvelope<T>(method, path, body, options);
  return envelope.data;
}

/**
 * High-leverage, strongly typed API client for CoSave.
 */
export const api = {
  get<T>(path: string, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("GET", path, undefined, options);
  },
  post<T>(path: string, body?: unknown, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("POST", path, body, options);
  },
  patch<T>(path: string, body?: unknown, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("PATCH", path, body, options);
  },
  delete<T>(path: string, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("DELETE", path, undefined, options);
  },
  setTransport(adapter: TransportAdapter): () => void {
    const prev = activeTransport;
    activeTransport = adapter;
    return () => {
      activeTransport = prev;
    };
  },
};
