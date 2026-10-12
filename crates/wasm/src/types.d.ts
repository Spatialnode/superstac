export interface SuperSTACConfig {
  catalogs: {
    id: string;
    url: string;
    collection_aliases?: Record<string, string>;
    asset_aliases?: Record<string, Record<string, string>>;
  }[];
  settings?: {
    deduplicate_items?: boolean;
    unify_response?: boolean;
    max_concurrent_catalogs?: number;
    per_catalog_timeout_seconds?: number;
    max_retry_attempts?: number;
    max_items_per_catalog?: number;
  };
}
export interface SearchQuery {
  collections?: string[];
  ids?: string[];
  bbox?: [number, number, number, number] | [number, number, number, number, number, number];
  intersects?: { type: string; coordinates?: unknown; geometries?: unknown[] };
  datetime?: string;
  limit?: number;
  sortby?: string[];
}
export interface StacItem {
  type: "Feature";
  id: string;
  collection?: string;
  geometry: Record<string, unknown> | null;
  properties: Record<string, unknown>;
  assets: Record<string, { href: string; [key: string]: unknown }>;
  links: Record<string, unknown>[];
  [key: string]: unknown;
}
export interface SearchResponse {
  items: { catalog_id: string; seen_in: string[]; item: StacItem }[];
  metadata: {
    superstac_version: string;
    catalogs_queried: number;
    catalogs_succeeded: number;
    catalogs_failed: number;
    total_items: number;
    duplicates_removed: number;
    failures: { catalog_id: string; reason: string }[];
    unsupported_collections: string[];
  };
}
export interface CatalogCollections {
  catalog_id: string;
  collections: { canonical_id: string; collection: Record<string, unknown> }[];
  error: string | null;
}
