// prose-pod-api
//
// Copyright: 2023–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context};
use sea_orm::DatabaseConnection;
use tokio::sync::mpsc::Receiver;
use tracing::Instrument as _;

use crate::{
    models::{Paginated, Pagination, PaginationForm},
    util::{ConcurrentTaskRunner, Either},
    xmpp::BareJid,
    AppConfig,
};

use super::{EnrichedMember, Member, MemberRepository, MemberService, UserDeleteError};

// GET ONE

#[derive(Debug, thiserror::Error)]
#[error("No member with id '{jid}'")]
pub struct MemberNotFound {
    jid: BareJid,
}

pub async fn get_member(
    jid: &BareJid,
    member_service: &MemberService,
) -> Result<EnrichedMember, Either<MemberNotFound, anyhow::Error>> {
    match member_service.enrich_jid(&jid).await {
        Ok(Some(member)) => Ok(member),
        Ok(None) => Err(Either::E1(MemberNotFound { jid: jid.clone() })),
        Err(err) => Err(Either::E2(anyhow!(err).context("Enriching error"))),
    }
}

// DELETE ONE

pub async fn delete_member(
    db: &DatabaseConnection,
    jid: &BareJid,
    member_service: &MemberService,
) -> Result<(), UserDeleteError> {
    member_service.delete_user(db, jid).await
}

// GET MANY

pub async fn head_members(db: &DatabaseConnection) -> Result<Paginated<Member>, anyhow::Error> {
    let (metadata, _) = MemberRepository::get_page(db, 1, 1, None)
        .await
        .context("Database error")?;
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
) -> Result<Paginated<Member>, anyhow::Error> {
    let Pagination {
        page_number,
        page_size,
        until,
    } = Pagination::members(pagination);

    let (pages_metadata, members) = member_service
        .get_members(page_number, page_size, until)
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
) -> Result<Paginated<EnrichedMember>, anyhow::Error> {
    let Pagination {
        page_number,
        page_size,
        until,
    } = Pagination::members(pagination);

    let (pages_metadata, members) = member_service
        .search_members(query, page_number, page_size, until)
        .await
        .context("Database error")?;

    Ok(Paginated::new(
        members,
        page_number,
        page_size,
        pages_metadata,
    ))
}

// ENRICHING

pub async fn enrich_members(
    member_service: MemberService,
    jids: Vec<BareJid>,
    app_config: &AppConfig,
) -> HashMap<BareJid, EnrichedMember> {
    let jids_count = jids.len();
    let runner = ConcurrentTaskRunner::default(app_config);

    let cancellation_token = member_service.cancellation_token.clone();
    let mut rx = runner.run(
        jids,
        move |jid| {
            let member_service = member_service.clone();
            Box::pin(async move { member_service.enrich_jid(&jid).await }.in_current_span())
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
    app_config: &AppConfig,
) -> Receiver<Result<Option<EnrichedMember>, anyhow::Error>> {
    let member_service = Arc::new(member_service);
    let runner = ConcurrentTaskRunner::default(&app_config);

    let cancellation_token = member_service.cancellation_token.clone();
    runner.run(
        jids,
        move |jid| {
            let member_service = member_service.clone();
            Box::pin(async move { member_service.enrich_jid(&jid).await }.in_current_span())
        },
        move || cancellation_token.cancel(),
    )
}
