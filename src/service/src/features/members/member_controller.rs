// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context as _};
use tokio::sync::mpsc::Receiver;
use tracing::Instrument as _;

use crate::{
    auth::AuthToken,
    models::{Paginated, Pagination, PaginationForm},
    util::{either::Either, ConcurrentTaskRunner},
    xmpp::{BareJid, XmppServiceContext},
};

use super::{errors::*, EnrichedMember, Member, MemberService};

// MARK: Get one

pub async fn get_member(
    jid: &BareJid,
    member_service: &MemberService,
    ctx: &XmppServiceContext,
) -> Result<EnrichedMember, Either<MemberNotFound, anyhow::Error>> {
    match member_service.enrich_jid(jid, ctx).await {
        Ok(Some(member)) => Ok(member),
        Ok(None) => Err(Either::E1(MemberNotFound(jid.to_string()))),
        Err(err) => Err(Either::E2(anyhow!(err).context("Enriching error"))),
    }
}

// MARK: Delete one

pub async fn delete_member(
    jid: &BareJid,
    member_service: &MemberService,
    auth: &AuthToken,
) -> Result<(), UserDeleteError> {
    member_service.delete_user(jid, auth).await
}

// MARK: Get many

pub async fn head_members(
    member_service: &MemberService,
    auth: &AuthToken,
) -> Result<Paginated<Member>, anyhow::Error> {
    let (metadata, _) = member_service.get_members(1, 1, auth).await?;
    Ok(Paginated::new(vec![], 1, 1, metadata))
}

impl Pagination {
    fn members(
        PaginationForm {
            page_number,
            page_size,
            until,
        }: PaginationForm,
    ) -> Self {
        Self {
            page_number: page_number.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
            until,
        }
    }
}

pub async fn get_members(
    member_service: &MemberService,
    pagination: PaginationForm,
    auth: &AuthToken,
) -> Result<Paginated<Member>, anyhow::Error> {
    let Pagination {
        page_number,
        page_size,
        ..
    } = Pagination::members(pagination);

    let (pages_metadata, members) = member_service
        .get_members(page_number, page_size, auth)
        .await
        .context("Database error")?;

    Ok(Paginated::new(
        members,
        page_number,
        page_size,
        pages_metadata,
    ))
}

pub async fn get_members_filtered(
    member_service: &MemberService,
    pagination: PaginationForm,
    query: &str,
    ctx: &XmppServiceContext,
) -> Result<Paginated<EnrichedMember>, anyhow::Error> {
    let Pagination {
        page_number,
        page_size,
        ..
    } = Pagination::members(pagination);

    let (pages_metadata, members) = member_service
        .search_members(query, page_number, page_size, ctx)
        .await
        .context("Database error")?;

    Ok(Paginated::new(
        members,
        page_number,
        page_size,
        pages_metadata,
    ))
}

// MARK: Enriching

pub async fn enrich_members(
    member_service: MemberService,
    jids: Vec<BareJid>,
    ctx: XmppServiceContext,
) -> HashMap<BareJid, EnrichedMember> {
    let jids_count = jids.len();
    let runner = ConcurrentTaskRunner::default();

    let cancellation_token = member_service.cancellation_token.clone();
    let run_data = jids
        .into_iter()
        .map(|jid| (jid, ctx.clone()))
        .collect::<Vec<_>>();
    let mut rx = runner.run(
        run_data,
        move |(jid, ctx)| {
            let member_service = member_service.clone();
            Box::pin(async move { member_service.enrich_jid(&jid, &ctx).await }.in_current_span())
        },
        move || cancellation_token.cancel(),
    );

    let mut res = HashMap::with_capacity(jids_count);
    while let Some(Ok(Some(member))) = rx.recv().await {
        res.insert(member.jid.clone(), EnrichedMember::from(member).into());
    }
    res
}

pub fn enrich_members_stream(
    member_service: MemberService,
    jids: Vec<BareJid>,
    ctx: XmppServiceContext,
) -> Receiver<Result<Option<EnrichedMember>, anyhow::Error>> {
    let member_service = Arc::new(member_service);
    let runner = ConcurrentTaskRunner::default();

    let cancellation_token = member_service.cancellation_token.clone();
    let run_data = jids
        .into_iter()
        .map(|jid| (jid, ctx.clone()))
        .collect::<Vec<_>>();
    runner.run(
        run_data,
        move |(jid, ctx)| {
            let member_service = member_service.clone();
            Box::pin(async move { member_service.enrich_jid(&jid, &ctx).await }.in_current_span())
        },
        move || cancellation_token.cancel(),
    )
}
