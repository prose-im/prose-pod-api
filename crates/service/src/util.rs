// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{cmp::min, future::Future};

use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::{
    models::jid::{self, BareJid, NodePart, JID},
    server_config::ServerConfig,
};

pub fn to_bare_jid(jid: &JID) -> Result<BareJid, jid::Error> {
    BareJid::new(jid.to_string().as_str())
}

pub fn bare_jid_from_username(
    username: &str,
    server_config: &ServerConfig,
) -> Result<BareJid, String> {
    Ok(BareJid::from_parts(
        Some(&NodePart::new(username).map_err(|err| format!("Invalid username: {err}"))?),
        &server_config.domain,
    ))
}

#[macro_export]
macro_rules! wrapper_type {
    ($wrapper:ident, $t:ty) => {
        #[derive(std::fmt::Debug, Clone, Eq, PartialEq, Hash)]
        #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
        #[repr(transparent)]
        pub struct $wrapper($t);

        impl std::ops::Deref for $wrapper {
            type Target = $t;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $wrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl std::str::FromStr for $wrapper {
            type Err = <$t as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$t>::from_str(s).map(Self)
            }
        }

        impl From<$t> for $wrapper {
            fn from(bare_jid: $t) -> Self {
                Self(bare_jid)
            }
        }
    };
}

#[macro_export]
macro_rules! sea_orm_try_get_by_string {
    () => {
        fn try_get_by<I: sea_orm::ColIdx>(
            res: &sea_orm::QueryResult,
            index: I,
        ) -> Result<Self, sea_orm::TryGetError> {
            // https://github.com/SeaQL/sea-orm/discussions/1176#discussioncomment-4024088
            let value = res
                .try_get_by(index)
                .map_err(sea_orm::TryGetError::DbErr)
                .and_then(|opt: Option<String>| {
                    opt.ok_or(sea_orm::TryGetError::Null(format!("{index:?}")))
                })?;
            <Self as std::str::FromStr>::from_str(value.as_str())
                // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
                .map_err(|e| sea_orm::TryGetError::Null(format!("{:?}", e)))
        }
    };
}

#[macro_export]
macro_rules! sea_orm_string_ {
    ($t:ty, $length:expr) => {
        impl From<$t> for sea_query::Value {
            fn from(value: $t) -> Self {
                Self::String(Some(Box::new(value.to_string())))
            }
        }

        impl sea_orm::TryGetable for $t {
            crate::sea_orm_try_get_by_string!();
        }

        impl sea_query::ValueType for $t {
            fn try_from(v: sea_query::Value) -> Result<Self, sea_query::ValueTypeErr> {
                match v {
                    sea_query::Value::String(Some(value)) => {
                        <Self as std::str::FromStr>::from_str(value.as_str())
                            .map_err(|_| sea_query::ValueTypeErr)
                    }
                    _ => Err(sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($t).to_string()
            }

            fn array_type() -> sea_query::ArrayType {
                sea_query::ArrayType::String
            }

            fn column_type() -> sea_query::ColumnType {
                sea_query::ColumnType::string($length)
            }
        }

        impl sea_query::Nullable for $t {
            fn null() -> sea_orm::Value {
                sea_orm::Value::String(None)
            }
        }
    };
}

#[macro_export]
macro_rules! sea_orm_string {
    ($t:ty) => {
        crate::sea_orm_string_!($t, None);
    };
}

#[macro_export]
macro_rules! sea_orm_string_enum {
    ($t:ty) => {
        crate::sea_orm_string_!(
            $t,
            Some(Self::iter().map(|v| v.to_string().len()).max().unwrap() as u32)
        );
    };
}

enum Futures<F: Future> {
    Ordered(FuturesOrdered<F>),
    Unordered(FuturesUnordered<F>),
}

impl<F: Future> Futures<F> {
    fn new(iter: impl Iterator<Item = F>, ordered: bool) -> Self {
        if ordered {
            Self::Ordered(iter.collect())
        } else {
            Self::Unordered(iter.collect())
        }
    }
    async fn next(&mut self) -> Option<F::Output> {
        match self {
            Futures::Ordered(futures) => futures.next().await,
            Futures::Unordered(futures) => futures.next().await,
        }
    }
}

impl<F: Future> From<FuturesOrdered<F>> for Futures<F> {
    fn from(futures: FuturesOrdered<F>) -> Self {
        Self::Ordered(futures)
    }
}
impl<F: Future> From<FuturesUnordered<F>> for Futures<F> {
    fn from(futures: FuturesUnordered<F>) -> Self {
        Self::Unordered(futures)
    }
}

pub fn run_parallel_tasks<F, R>(
    futures: Vec<F>,
    on_cancel: impl FnOnce() -> () + Send + 'static,
    timeout: std::time::Duration,
    ordered: bool,
) -> mpsc::Receiver<R>
where
    F: Future<Output = R> + Send + Unpin + 'static,
    R: Send + 'static,
{
    let (tx, rx) = mpsc::channel::<R>(min(futures.len(), 32));
    tokio::spawn(async move {
        let cancellation_token = CancellationToken::new();
        let mut tasks: Futures<JoinHandle<Option<R>>> = Futures::new(
            futures.into_iter().map(|future| {
                let cancellation_token = cancellation_token.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        res = future => { Some(res) },
                        _ = cancellation_token.cancelled() => { None },
                    }
                })
            }),
            ordered,
        );

        tokio::select! {
            _ = async {
                // NOTE: If `futures.len() == 0` then this `tokio::select!` ends instantly.
                while let Some(Ok(Some(msg))) = tasks.next().await {
                    if let Err(err) = tx.send(msg).await {
                        if tx.is_closed() {
                            debug!("Cannot send task result: Task aborted.");
                        } else {
                            error!("Cannot send task result: {err}");
                        }
                    }
                }
            } => {}
            _ = sleep(timeout) => {
                debug!("Timed out. Cancelling all tasks…");

                cancellation_token.cancel();
                on_cancel();
            }
        };
    });

    rx
}
