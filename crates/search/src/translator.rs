use stac::api::Search;
use crate::query::SearchQuery;

/// Convert a SuperSTAC `SearchQuery` into the typed `stac::api::Search` the
/// stac-io client expects. Field-by-field copy with no canonicalization —
/// alias rewriting happens upstream in the STAC API backend.
pub fn to_stac_search(
    query: SearchQuery,
) -> Search {
    let mut search = Search::new();

    if !query.collections.is_empty() {
        search = search.collections(
            query.collections,
        );
    }

    if let Some(ids) = query.ids {
        search = search.ids(ids);
    }

    if let Some(intersects) =
        query.intersects
    {
        search = search.intersects(intersects);
    }

    if let Some(limit) = query.limit {
        search = search.limit(limit as u64);
    }

    if let Some(datetime) =
        query.datetime
    {
        search = search.datetime(datetime);
    }
    if let Some(bbox) = query.bbox {
    search = search.bbox(bbox);
}
    search
}