//! `GET /api/news/award_announcements/` — federal award announcements from two sources, named on every row by `source` (Tango API 5.14.0).
//!
//! `official_release` rows are the awards of $7.5 million or more that the Department of War publishes in its daily contracts release, with the text as published.
//!
//! `makegov_generated` rows cover new civilian IDVs of $25 million or more and every new OTA and OT IDV.
//! They are written by MakeGov from FPDS data with a language model and are **not government text**.
//! A generated announcement is dropped when an official release covers the same award.
//!
//! Lists hide announcements DoD later withdrew unless [`ListAwardAnnouncementsOptions::include_withdrawn`] is `Some(true)`; a withdrawn announcement stays retrievable by its `uuid`.
//!
//! `text` can run past 16,000 characters, so a list carries it only when named in `shape`.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_award_announcements`] and [`Client::iterate_award_announcements`].
///
/// All twenty of the API's filters are named fields, plus `ordering`.
/// This endpoint pages by `page`, so there is no `cursor` option.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAwardAnnouncementsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server default 10, cap 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Comma-separated field selector. Use [`SHAPE_AWARD_ANNOUNCEMENTS_MINIMAL`](crate::SHAPE_AWARD_ANNOUNCEMENTS_MINIMAL) or roll your own. Name `text` to get the announcement body on a list.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Full-text search over the title and text, at least two characters; a match in the title ranks above a match in the text. Wrap in double quotes for a phrase.
    #[builder(into)]
    pub search: Option<String>,
    /// `official_release` for the Department of War's daily contracts release, or `makegov_generated` for announcements written by MakeGov from FPDS data with a language model, which are not government text.
    #[builder(into)]
    pub source: Option<String>,
    /// A Tango agency name, abbreviation, code or organization key for the awarding organization, e.g. `Navy`. OR several with `|`.
    #[builder(into)]
    pub agency: Option<String>,
    /// The DoD component as printed in the official release's section header, e.g. `NAVY`. Case-insensitive exact match; OR several with `|`. Generated announcements have no component.
    #[builder(into)]
    pub component: Option<String>,
    /// Published on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub published_date_after: Option<String>,
    /// Published on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub published_date_before: Option<String>,
    /// Awarded on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub award_date_after: Option<String>,
    /// Awarded on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub award_date_before: Option<String>,
    /// Amount at least, in USD.
    #[builder(into)]
    pub amount_min: Option<String>,
    /// Amount at most, in USD.
    #[builder(into)]
    pub amount_max: Option<String>,
    /// A contract number named in the announcement, e.g. `N00019-25-C-0070` or `N0001925C0070`. Dashes and case are ignored; OR several with `|`.
    #[builder(into)]
    pub piid: Option<String>,
    /// Case-insensitive substring match on the awardee names, at least three characters.
    #[builder(into)]
    pub awardee: Option<String>,
    /// An exact UEI of an awardee. Official releases print no UEI, so this matches only where one was resolved.
    #[builder(into)]
    pub uei: Option<String>,
    /// Action type, e.g. `new_award`. OR several with `|`.
    #[builder(into)]
    pub action_type: Option<String>,
    /// Announcement type. OR several with `|`.
    #[builder(into)]
    pub announcement_type: Option<String>,
    /// Announces a civilian award.
    pub civilian: Option<bool>,
    /// `Some(true)` keeps announcements with at least one small-business awardee. `Some(false)` keeps those whose awardees are stated not to be small. Generated announcements state neither, so they match neither.
    pub small_business: Option<bool>,
    /// Announces a modification to an existing contract.
    pub is_modification: Option<bool>,
    /// Names more than one awardee.
    pub is_multiple_award: Option<bool>,
    /// `Some(true)` includes announcements DoD later withdrew, which lists hide otherwise.
    pub include_withdrawn: Option<bool>,

    /// Sort key: `published_date`, `amount` or `rank`, prefixed with `-` for descending. The server default is `-published_date`, or `-rank` under a [`search`](Self::search). `rank` requires a non-empty `search`, and descending `published_date` and `amount` put rows with no value last.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct. The API answers an unknown key with a 400.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListAwardAnnouncementsOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            self.page,
            self.limit,
            None,
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        for (key, value) in [
            ("search", &self.search),
            ("source", &self.source),
            ("agency", &self.agency),
            ("component", &self.component),
            ("published_date_after", &self.published_date_after),
            ("published_date_before", &self.published_date_before),
            ("award_date_after", &self.award_date_after),
            ("award_date_before", &self.award_date_before),
            ("amount_min", &self.amount_min),
            ("amount_max", &self.amount_max),
            ("piid", &self.piid),
            ("awardee", &self.awardee),
            ("uei", &self.uei),
            ("action_type", &self.action_type),
            ("announcement_type", &self.announcement_type),
        ] {
            push_opt(&mut q, key, value.as_deref());
        }
        push_opt_bool(&mut q, "civilian", self.civilian);
        push_opt_bool(&mut q, "small_business", self.small_business);
        push_opt_bool(&mut q, "is_modification", self.is_modification);
        push_opt_bool(&mut q, "is_multiple_award", self.is_multiple_award);
        push_opt_bool(&mut q, "include_withdrawn", self.include_withdrawn);
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_award_announcement`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetAwardAnnouncementOptions {
    /// Shape selector. When empty, the server returns its detail default, which [`SHAPE_AWARD_ANNOUNCEMENTS_COMPREHENSIVE`](crate::SHAPE_AWARD_ANNOUNCEMENTS_COMPREHENSIVE) mirrors.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetAwardAnnouncementOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_opt(&mut q, "shape", self.shape.as_deref());
        if self.flat {
            q.push(("flat".into(), "true".into()));
        }
        if self.flat_lists {
            q.push(("flat_lists".into(), "true".into()));
        }
        q
    }
}

impl Client {
    /// `GET /api/news/award_announcements/` — one page of award announcements, newest publication first by default, or best match first under a `search`.
    ///
    /// `makegov_generated` rows are written by MakeGov from FPDS data with a language model and are not government text; read `source` on each row, or filter on it with [`ListAwardAnnouncementsOptions::source`].
    pub async fn list_award_announcements(
        &self,
        opts: ListAwardAnnouncementsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        self.get_page("/api/news/award_announcements/", &q).await
    }

    /// `GET /api/news/award_announcements/{uuid}/` — one award announcement by its `uuid`, including one DoD later withdrew.
    ///
    /// `makegov_generated` rows are written by MakeGov from FPDS data with a language model and are not government text.
    pub async fn get_award_announcement(
        &self,
        announcement_uuid: &str,
        opts: Option<GetAwardAnnouncementOptions>,
    ) -> Result<Record> {
        if announcement_uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_award_announcement: announcement_uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!(
            "/api/news/award_announcements/{}/",
            urlencoding(announcement_uuid)
        );
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every award announcement matching `opts`.
    pub fn iterate_award_announcements(
        &self,
        opts: ListAwardAnnouncementsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, _cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            Box::pin(async move { client.list_award_announcements(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn list_award_announcements_all_filters_emit() {
        let opts = ListAwardAnnouncementsOptions::builder()
            .search("\"hypersonic\"")
            .source("official_release")
            .agency("Navy|Army")
            .component("NAVY")
            .published_date_after("2026-09-01")
            .published_date_before("2026-09-30")
            .award_date_after("2026-08-01")
            .award_date_before("2026-08-31")
            .amount_min("100000000")
            .amount_max("500000000.50")
            .piid("N00019-25-C-0070")
            .awardee("lockheed")
            .uei("ABCDEF123456")
            .action_type("new_award")
            .announcement_type("award")
            .civilian(false)
            .small_business(false)
            .is_modification(true)
            .is_multiple_award(false)
            .include_withdrawn(true)
            .ordering("-amount")
            .build();
        let q = opts.to_query();
        for (k, v) in [
            ("search", "\"hypersonic\""),
            ("source", "official_release"),
            ("agency", "Navy|Army"),
            ("component", "NAVY"),
            ("published_date_after", "2026-09-01"),
            ("published_date_before", "2026-09-30"),
            ("award_date_after", "2026-08-01"),
            ("award_date_before", "2026-08-31"),
            ("amount_min", "100000000"),
            ("amount_max", "500000000.50"),
            ("piid", "N00019-25-C-0070"),
            ("awardee", "lockheed"),
            ("uei", "ABCDEF123456"),
            ("action_type", "new_award"),
            ("announcement_type", "award"),
            ("civilian", "false"),
            ("small_business", "false"),
            ("is_modification", "true"),
            ("is_multiple_award", "false"),
            ("include_withdrawn", "true"),
            ("ordering", "-amount"),
        ] {
            assert_eq!(get_q(&q, k).as_deref(), Some(v), "{k}");
        }
        assert_eq!(q.len(), 21);
    }

    #[test]
    fn list_award_announcements_zero_value_omitted() {
        assert!(ListAwardAnnouncementsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn only_the_detail_shape_names_text() {
        let names_text = |shape: &str| shape.split(',').any(|f| f == "text");
        assert!(!names_text(crate::SHAPE_AWARD_ANNOUNCEMENTS_MINIMAL));
        assert!(names_text(crate::SHAPE_AWARD_ANNOUNCEMENTS_COMPREHENSIVE));
    }

    #[tokio::test]
    async fn empty_uuid_is_rejected_before_any_request() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_award_announcement("", None).await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }
}
