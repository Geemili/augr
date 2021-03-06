use crate::Tag;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

pub type PatchRef = Uuid;
type EventRef = String;
type Set<T> = std::collections::HashSet<T>;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Patch {
    pub id: Uuid,

    #[serde(default, skip_serializing_if = "Set::is_empty")]
    pub add_start: Set<AddStart>,

    #[serde(default, skip_serializing_if = "Set::is_empty")]
    pub remove_start: Set<RemoveStart>,

    #[serde(default, skip_serializing_if = "Set::is_empty")]
    pub add_tag: Set<AddTag>,

    #[serde(default, skip_serializing_if = "Set::is_empty")]
    pub remove_tag: Set<RemoveTag>,

    #[serde(default, skip_serializing_if = "Set::is_empty")]
    pub create_event: Set<CreateEvent>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddStart {
    #[serde(default)]
    pub parents: BTreeSet<PatchRef>,
    pub event: EventRef,
    pub time: DateTime<Utc>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveStart {
    #[serde(default)]
    pub parents: Option<BTreeSet<PatchRef>>,
    pub patch: PatchRef,
    pub event: EventRef,
    pub time: DateTime<Utc>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddTag {
    #[serde(default)]
    pub parents: BTreeSet<PatchRef>,
    pub event: EventRef,
    pub tag: Tag,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveTag {
    #[serde(default)]
    pub parents: Option<BTreeSet<PatchRef>>,
    pub patch: PatchRef,
    pub event: EventRef,
    pub tag: Tag,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateEvent {
    pub event: EventRef,
    pub start: DateTime<Utc>,
    pub tags: Vec<Tag>,
}

impl Patch {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            add_start: Set::new(),
            remove_start: Set::new(),
            add_tag: Set::new(),
            remove_tag: Set::new(),
            create_event: Set::new(),
        }
    }

    pub fn with_id(id: PatchRef) -> Self {
        Self {
            id,
            add_start: Set::new(),
            remove_start: Set::new(),
            add_tag: Set::new(),
            remove_tag: Set::new(),
            create_event: Set::new(),
        }
    }

    pub fn patch_ref(&self) -> &PatchRef {
        &self.id
    }

    pub fn parents(&self) -> Set<PatchRef> {
        let add_start_parents = self.add_start.iter().flat_map(|x| x.parents.iter());
        let remove_start_parents = self.remove_start.iter().map(|x| &x.patch).chain(
            self.remove_start
                .iter()
                .flat_map(|x| x.parents.iter().flat_map(|s| s.iter())),
        );
        let remove_tag_parents = self.remove_tag.iter().map(|x| &x.patch).chain(
            self.remove_tag
                .iter()
                .flat_map(|x| x.parents.iter().flat_map(|s| s.iter())),
        );
        let add_tag_parents = self.add_tag.iter().flat_map(|x| x.parents.iter());
        add_start_parents
            .chain(remove_start_parents)
            .chain(remove_tag_parents)
            .chain(add_tag_parents)
            .cloned()
            .collect()
    }

    pub fn add_start(mut self, parent: PatchRef, event: EventRef, time: DateTime<Utc>) -> Self {
        self.add_start.insert(AddStart {
            parents: {
                let mut s = BTreeSet::new();
                s.insert(parent);
                s
            },
            event,
            time,
        });
        self
    }

    pub fn remove_start(mut self, patch: PatchRef, event: EventRef, time: DateTime<Utc>) -> Self {
        self.remove_start.insert(RemoveStart {
            parents: None,
            patch,
            event,
            time,
        });
        self
    }

    pub fn add_tag(mut self, parent: PatchRef, event: EventRef, tag: String) -> Self {
        self.add_tag.insert(AddTag {
            parents: {
                let mut s = BTreeSet::new();
                s.insert(parent);
                s
            },
            event,
            tag,
        });
        self
    }

    pub fn remove_tag(mut self, patch: PatchRef, event: EventRef, tag: String) -> Self {
        self.remove_tag.insert(RemoveTag {
            parents: None,
            patch,
            event,
            tag,
        });
        self
    }

    pub fn create_event(
        mut self,
        event: EventRef,
        start: DateTime<Utc>,
        tags: Vec<String>,
    ) -> Self {
        self.create_event.insert(CreateEvent { event, start, tags });
        self
    }

    pub fn insert_add_start(&mut self, add_start: AddStart) {
        self.add_start.insert(add_start);
    }

    pub fn insert_remove_start(&mut self, remove_start: RemoveStart) {
        self.remove_start.insert(remove_start);
    }

    pub fn insert_add_tag(&mut self, add_tag: AddTag) {
        self.add_tag.insert(add_tag);
    }

    pub fn insert_remove_tag(&mut self, remove_tag: RemoveTag) {
        self.remove_tag.insert(remove_tag);
    }

    pub fn insert_create_event(&mut self, create_event: CreateEvent) {
        self.create_event.insert(create_event);
    }
}

impl Default for Patch {
    fn default() -> Self {
        Patch::new()
    }
}

impl AddStart {
    pub fn parents(&self) -> impl Iterator<Item = &PatchRef> {
        self.parents.iter()
    }
}
impl RemoveStart {
    pub fn parents(&self) -> impl Iterator<Item = &PatchRef> {
        self.parents.iter().flat_map(|s| s.iter())
    }
}
impl AddTag {
    pub fn parents(&self) -> impl Iterator<Item = &PatchRef> {
        self.parents.iter()
    }
}
impl RemoveTag {
    pub fn parents(&self) -> impl Iterator<Item = &PatchRef> {
        self.parents.iter().flat_map(|s| s.iter())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::offset::{TimeZone, Utc};

    macro_rules! s (
        { $stuff:expr } => {
            {
                $stuff.to_string()
            }
         };
    );

    #[test]
    fn read_patch_with_create_event_toml() {
        let id = Uuid::parse_str("e39076fe-6b5a-4a7f-b927-7fc1df5ba275").unwrap();
        let expected = Patch::with_id(id).create_event(
            s!("a"),
            Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
            vec![s!("work"), s!("coding")],
        );

        let toml_str = r#"
            id = "e39076fe-6b5a-4a7f-b927-7fc1df5ba275"

            [[create-event]]
            event = "a"
            start = "2019-07-24T14:00:00+00:00"
            tags = ["work", "coding"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn serialize_patch_with_add_tag_toml() {
        let id = Uuid::parse_str("e39076fe-6b5a-4a7f-b927-7fc1df5ba275").unwrap();
        let patch0 = Uuid::parse_str("fa5de1d9-aa11-49fa-b064-8128281a7d91").unwrap();
        let event0 = Uuid::parse_str("0c435b19-4504-440c-abc7-f4e4d6a7d25f").unwrap();

        let patch = Patch::with_id(id).add_start(
            patch0.clone(),
            event0.to_string(),
            Utc.ymd(2019, 07, 24).and_hms(14, 0, 0),
        );

        let toml_str = "id = \"e39076fe-6b5a-4a7f-b927-7fc1df5ba275\"\n\n[[add-start]]\nparents = [\"fa5de1d9-aa11-49fa-b064-8128281a7d91\"]\nevent = \"0c435b19-4504-440c-abc7-f4e4d6a7d25f\"\ntime = \"2019-07-24T14:00:00Z\"\n".to_string();
        let serialized = toml::ser::to_string(&patch).unwrap();
        println!("{}", serialized);
        assert_eq!(toml_str, serialized);
    }

    #[test]
    fn read_patch_with_parents() {
        let id = Uuid::parse_str("e39076fe-6b5a-4a7f-b927-7fc1df5ba275").unwrap();
        let patch0 = Uuid::parse_str("fa5de1d9-aa11-49fa-b064-8128281a7d91").unwrap();
        let patch1 = Uuid::parse_str("0c435b19-4504-440c-abc7-f4e4d6a7d25f").unwrap();

        let mut expected = Patch::with_id(id);

        let remove_start = RemoveStart {
            parents: {
                let mut s = BTreeSet::new();
                s.insert(patch0.clone());
                s.insert(patch1.clone());
                Some(s)
            },
            patch: patch0.clone(),
            event: s!("a"),
            time: Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
        };
        expected.insert_remove_start(remove_start);

        let toml_str = r#"
            id = "e39076fe-6b5a-4a7f-b927-7fc1df5ba275"

            [[remove-start]]
            parents = ["fa5de1d9-aa11-49fa-b064-8128281a7d91", "0c435b19-4504-440c-abc7-f4e4d6a7d25f"]
            patch = "fa5de1d9-aa11-49fa-b064-8128281a7d91"
            event = "a"
            time = "2019-07-24T14:00:00+00:00"
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_patch_with_all_fields_toml() {
        let patch0 = Uuid::parse_str("fa5de1d9-aa11-49fa-b064-8128281a7d91").unwrap();

        let expected =
            Patch::with_id(Uuid::parse_str("2a226f4d-60f2-493d-9e9a-d6c71d98b515").unwrap())
                .add_start(
                    patch0.clone(),
                    s!("a"),
                    Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
                )
                .remove_start(
                    patch0.clone(),
                    s!("a"),
                    Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
                )
                .add_tag(patch0.clone(), s!("a"), s!("work"))
                .remove_tag(patch0.clone(), s!("a"), s!("coding"))
                .create_event(
                    s!("a"),
                    Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
                    vec![s!("work"), s!("coding")],
                );

        let toml_str = r#"
            id = "2a226f4d-60f2-493d-9e9a-d6c71d98b515"

            [[add-start]]
            parents = ["fa5de1d9-aa11-49fa-b064-8128281a7d91"]
            event = "a"
            time = "2019-07-24T14:00:00+00:00"

            [[remove-start]]
            patch = "fa5de1d9-aa11-49fa-b064-8128281a7d91"
            event = "a"
            time = "2019-07-24T14:00:00+00:00"

            [[add-tag]]
            parents = ["fa5de1d9-aa11-49fa-b064-8128281a7d91"]
            event = "a"
            tag = "work"

            [[remove-tag]]
            patch = "fa5de1d9-aa11-49fa-b064-8128281a7d91"
            event = "a"
            tag = "coding"

            [[create-event]]
            event = "a"
            start = "2019-07-24T14:00:00+00:00"
            tags = ["work", "coding"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

}
