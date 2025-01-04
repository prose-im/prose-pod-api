// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{borrow::Cow, collections::HashSet};

use axum::{
    body::Body,
    extract::{FromRequestParts, Query as AxumQuery},
    http::{request::Parts, uri, Request},
};

use super::strict_qs_query::{QsQuery as StrictQsQuery, QsQueryRejection};

/// Like [`serde_qs::axum::QsQuery`], but also supports non-bracketed parameters for arrays.
pub struct QsQuery<T>(pub T);

/// The Prose Pod API should support repeated query parameters
/// (e.g. `jids[]=a@example.org&jids[]=b@example.org`)
pub fn rename_repeated_query_param_names(req: Request<Body>) -> Request<Body> {
    let (mut head, body) = req.into_parts();
    let uri = head.uri.clone();

    let query = uri.query().unwrap_or_default();
    let new_query = rename_repeated_query_param_names_(query);

    head.uri = uri::Builder::from(uri.clone())
        .path_and_query(format!("{path}?{new_query}", path = uri.clone().path()))
        .build()
        .unwrap();

    Request::from_parts(head, body)
}

fn rename_repeated_query_param_names_(query: &str) -> String {
    // Find repeated keys.
    let query_pairs = form_urlencoded::parse(query.as_bytes());
    let mut keys = HashSet::<Cow<'_, str>>::new();
    let mut repeated_keys = HashSet::<Cow<'_, str>>::new();
    for (key, _) in query_pairs {
        // Support mixed keys (e.g. `"jids=" == "jid[]="`).
        let key = key
            .strip_suffix("[]")
            .map(|k| Cow::Owned(k.to_owned()))
            .unwrap_or(key);

        // Store key if repeated.
        if !keys.insert(key.clone()) {
            repeated_keys.insert(key);
        }
    }

    // Replace repeated keys by their bracketed equivalent.
    let mut new_query = query.to_string();
    for key in repeated_keys {
        // NOTE: Constructing regular expressions for every repeated key of every request
        //   would be highly inefficient, therefore we opted for the far more efficient
        //   manual string replacement. The use case is so simple that the code remains readable.

        // Skip key if it's already bracketed.
        if key.ends_with("[]") {
            continue;
        }

        // Replace first match.
        if query.starts_with(&format!("{key}=")) {
            new_query.insert_str(key.len(), "[]");
        }
        // Replace remaining matches.
        new_query = new_query.replace(&format!("&{key}="), &format!("&{key}[]="));
    }

    new_query
}

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for QsQuery<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = QsQueryRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match StrictQsQuery::<T>::from_request_parts(parts, state).await {
            Ok(StrictQsQuery(res)) => Ok(Self(res)),
            // NOTE: In most cases, `StrictQsQuery` will work because `rename_repeated_query_param_names`
            //   renames repeated keys to add brackets. However, in the case of a single non-bracketed key,
            //   `StrictQsQuery` will fail to decode the array. `AxumQuery` doesn't work with repeated keys,
            //   but since in that precise case there is only one value, we can use it.
            //
            // WARN: This is a trick, which doesn't work if the type to deserialize contains multiple array.
            //   In that case, a workaround could be to split it into two different types, which would be
            //   individually deserialized.
            //
            //   NOTE: Another solution, more unstable, could be to read the error message (which looks like
            //     `Failed to deserialize query string: invalid type: string \"whatever\", expected a sequence`),
            //     to find and replace the key associated to that value. It depends on an implementation detail,
            //     could lead to errors (imagine a value being present for multiple keys) and would require us
            //     to write code for all types (string, int, bool…), which is why we will not do it.
            Err(err) => match AxumQuery::from_request_parts(parts, state).await {
                Ok(AxumQuery(res)) => Ok(Self(res)),
                Err(_) => Err(err),
            },
        }
    }
}

// #[axum::async_trait]
// impl<T, S> FromRequestParts<S> for StrictQsQuery<T>
// where
//     T: serde::de::DeserializeOwned,
// {
//     type Rejection = QsQueryRejection;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         // TODO: error handling
//         let query = parts.uri.query().unwrap();
//         Ok(Self(serde_qs::from_str(query).unwrap()))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_bracketed_get_brackets() {
        let example_map = vec![
            ("foo=a", "foo=a"),
            ("foo=a&foo=b", "foo[]=a&foo[]=b"),
            ("foo=a&foo=b&foo=c", "foo[]=a&foo[]=b&foo[]=c"),
        ];
        for (query, expected) in example_map {
            assert_eq!(rename_repeated_query_param_names_(query).as_str(), expected);
        }
    }

    #[test]
    fn test_bracketed_dont_get_double_brackets() {
        let example_map = vec![
            ("foo[]=a", "foo[]=a"),
            ("foo[]=a&foo[]=b", "foo[]=a&foo[]=b"),
            ("foo[]=a&foo[]=b&foo[]=c", "foo[]=a&foo[]=b&foo[]=c"),
        ];
        for (query, expected) in example_map {
            assert_eq!(rename_repeated_query_param_names_(query).as_str(), expected);
        }
    }

    #[test]
    fn test_works_on_multiple_keys() {
        let example_map = vec![
            ("foo=a&bar=a&foo=b&bar=b", "foo[]=a&bar[]=a&foo[]=b&bar[]=b"),
            ("foo=a&bar=a&bar=b", "foo=a&bar[]=a&bar[]=b"),
        ];
        for (query, expected) in example_map {
            assert_eq!(rename_repeated_query_param_names_(query).as_str(), expected);
        }
    }

    #[test]
    fn test_mixed_brackets_work_too() {
        let example_map = vec![
            ("foo=a&foo[]=b", "foo[]=a&foo[]=b"),
            ("foo[]=a&foo=b", "foo[]=a&foo[]=b"),
        ];
        for (query, expected) in example_map {
            assert_eq!(rename_repeated_query_param_names_(query).as_str(), expected);
        }
    }
}
