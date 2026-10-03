//! Browser STAC transport. Pagination is polled directly without Tokio tasks.
use reqwest::{Client, Method, Url};
use serde::Deserialize;
use serde_json::{Map, Value};
use superstac_core::errors::SuperSTACError;

fn error(e: impl std::fmt::Display) -> SuperSTACError {
    SuperSTACError::SearchFailed(e.to_string())
}

#[derive(Deserialize)]
struct Link {
    rel: String,
    href: String,
    method: Option<String>,
    body: Option<Value>,
    #[serde(default)]
    headers: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    merge: bool,
}

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    links: Vec<Link>,
    #[serde(flatten)]
    fields: Map<String, Value>,
}

/// Read paginated collections, bounding the result to avoid unbounded allocation.
pub async fn collections(
    client: &Client,
    base: &str,
    cap: usize,
) -> Result<Vec<Value>, SuperSTACError> {
    pages(
        client,
        endpoint(base, "collections")?,
        Method::GET,
        None,
        "collections",
        cap,
    )
    .await
}

pub async fn search(
    client: &Client,
    base: &str,
    search: &stac::api::Search,
    cap: usize,
) -> Result<Vec<stac::api::Item>, SuperSTACError> {
    pages(
        client,
        endpoint(base, "search")?,
        Method::POST,
        Some(serde_json::to_value(search).map_err(error)?),
        "features",
        cap,
    )
    .await?
    .into_iter()
    .map(|item| serde_json::from_value(item).map_err(error))
    .collect()
}

fn endpoint(base: &str, path: &str) -> Result<Url, SuperSTACError> {
    Url::parse(&format!("{}/{}", base.trim_end_matches('/'), path)).map_err(error)
}

async fn pages(
    client: &Client,
    mut url: Url,
    mut method: Method,
    mut body: Option<Value>,
    field: &str,
    cap: usize,
) -> Result<Vec<Value>, SuperSTACError> {
    let mut items = Vec::new();
    let mut headers = std::collections::BTreeMap::<String, String>::new();
    let mut visited = std::collections::HashSet::new();
    while items.len() < cap {
        let signature = format!("{method} {url} {body:?} {headers:?}");
        if !visited.insert(signature) || visited.len() > 10_000 {
            return Err(error("pagination did not terminate"));
        }
        let mut request = client.request(method.clone(), url.clone());
        if method == Method::POST {
            if let Some(value) = &body {
                request = request.json(value);
            }
        }
        for (key, value) in &headers {
            request = request.header(key, value);
        }
        let response = request
            .send()
            .await
            .map_err(error)?
            .error_for_status()
            .map_err(error)?;
        let page_url = response.url().clone();
        let mut page: Page = response.json().await.map_err(error)?;
        let values = page
            .fields
            .remove(field)
            .ok_or_else(|| error(format!("missing {field}")))?;
        let values = values
            .as_array()
            .ok_or_else(|| error(format!("{field} must be an array")))?;
        items.extend(values.iter().take(cap - items.len()).cloned());
        if items.len() == cap {
            break;
        }
        let Some(next) = page.links.into_iter().find(|link| link.rel == "next") else {
            break;
        };
        url = page_url.join(&next.href).map_err(error)?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(error("pagination URL must use HTTP or HTTPS"));
        }
        method = Method::from_bytes(next.method.as_deref().unwrap_or("GET").as_bytes())
            .map_err(error)?;
        if method != Method::GET && method != Method::POST {
            return Err(error("pagination supports only GET and POST"));
        }
        if next.merge && method == Method::POST {
            let mut merged = body.take().unwrap_or_else(|| Value::Object(Map::new()));
            let target = merged
                .as_object_mut()
                .ok_or_else(|| error("pagination body must be an object"))?;
            if let Some(extra) = next.body {
                target.extend(
                    extra
                        .as_object()
                        .ok_or_else(|| error("pagination body must be an object"))?
                        .clone(),
                );
            }
            body = Some(merged);
        } else {
            body = next.body;
        }
        headers = next.headers;
    }
    Ok(items)
}
