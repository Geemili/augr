use crate::{Event, PatchRef, Tag};
use chrono::{DateTime, Utc};
use snafu::{ensure, Snafu};
use std::collections::BTreeSet;

#[derive(Default, Clone, Debug)]
pub struct PatchedEvent {
    starts_added: BTreeSet<(PatchRef, DateTime<Utc>)>,
    starts_removed: BTreeSet<(PatchRef, DateTime<Utc>)>,
    tags_added: BTreeSet<(PatchRef, String)>,
    tags_removed: BTreeSet<(PatchRef, String)>,

    /// Stores the latest patches that have been applied. Will generally be a
    /// single patch, but if multiple patches were created asynchronously, there
    /// may be multiple patches. Essentially, it stores every patch that has not
    /// been referenced by another patch applied to it.
    latest_patches: BTreeSet<PatchRef>,
}

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum Error {
    #[snafu(display("Event has multiple start times"))]
    MultipleStartTimes,

    #[snafu(display("Event has no start times"))]
    NoStartTimes,
}

impl PatchedEvent {
    pub fn new() -> Self {
        Self {
            starts_added: BTreeSet::new(),
            starts_removed: BTreeSet::new(),
            tags_added: BTreeSet::new(),
            tags_removed: BTreeSet::new(),
            latest_patches: BTreeSet::new(),
        }
    }

    /// Remove patch from latest_patches, meaning that it has been referenced by another
    /// patch.
    pub fn remove_patch_from_latest(&mut self, patch: &PatchRef) {
        self.latest_patches.remove(patch);
    }

    /// Add patch to latest_patches, meaning that it has just been applied to this event.
    pub fn add_patch_to_latest(&mut self, patch: PatchRef) {
        self.latest_patches.insert(patch);
    }

    pub fn add_start(&mut self, patch: PatchRef, datetime: DateTime<Utc>) {
        self.starts_added.insert((patch, datetime));
    }

    pub fn remove_start(&mut self, patch: PatchRef, datetime: DateTime<Utc>) {
        self.starts_removed.insert((patch, datetime));
    }

    pub fn starts(&self) -> BTreeSet<(PatchRef, DateTime<Utc>)> {
        self.starts_added
            .difference(&self.starts_removed)
            .cloned()
            .collect()
    }

    pub fn add_tag(&mut self, patch: PatchRef, tag: Tag) {
        self.tags_added.insert((patch, tag));
    }

    pub fn remove_tag(&mut self, patch: PatchRef, tag: Tag) {
        self.tags_removed.insert((patch, tag));
    }

    pub fn tags(&self) -> BTreeSet<(PatchRef, Tag)> {
        self.tags_added
            .difference(&self.tags_removed)
            .cloned()
            .collect()
    }

    pub fn latest_patches(&self) -> BTreeSet<PatchRef> {
        self.latest_patches.clone()
    }

    pub fn flatten(&self) -> Result<Event, Error> {
        let starts = self.starts();
        ensure!(starts.len() < 2, MultipleStartTimes);
        ensure!(!starts.is_empty(), NoStartTimes);
        let start = starts
            .iter()
            .map(|patch_and_dt| patch_and_dt.1)
            .next()
            .expect("should be exactly one start");
        let tags = self
            .tags_added
            .difference(&self.tags_removed)
            .cloned()
            .map(|patch_and_tag| patch_and_tag.1)
            .collect();
        Ok(Event::new(start, tags))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use uuid::Uuid;

    #[test]
    fn remove_start_from_event() {
        let dt0 = Utc.ymd(2019, 07, 23).and_hms(12, 0, 0);
        let dt1 = Utc.ymd(2019, 07, 23).and_hms(12, 30, 0);
        let patch_ref_a = Uuid::parse_str("81790c38-96dd-4577-8b85-9f7c8bd6802b").unwrap();

        let mut event = PatchedEvent::new();
        event.add_start(patch_ref_a.clone(), dt0);
        event.add_start(patch_ref_a.clone(), dt1);
        event.remove_start(patch_ref_a.clone(), dt0);

        assert_eq!(
            event.starts(),
            [(patch_ref_a.clone(), dt1)].into_iter().cloned().collect()
        );
    }

    #[test]
    fn remove_tag_from_event() {
        let patch_ref_a = Uuid::parse_str("81790c38-96dd-4577-8b85-9f7c8bd6802b").unwrap();

        let mut event = PatchedEvent::new();
        event.add_tag(patch_ref_a.clone(), "hello".into());
        event.add_tag(patch_ref_a.clone(), "world".into());
        event.remove_tag(patch_ref_a.clone(), "world".into());

        assert_eq!(
            event.tags(),
            [(patch_ref_a.clone(), "hello".into())]
                .into_iter()
                .cloned()
                .collect()
        );
    }
}
