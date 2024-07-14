use serde::{Deserialize, Deserializer, Serialize};

use crate::pagination::Pagination;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct User(pub i64);

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Users {
    pub users: Vec<String>
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Package(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Project(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Owner(pub i64);

#[derive(Debug, Eq, PartialEq)]
pub struct Owned(pub Owner, pub Project);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Range {
    pub min: Option<i64>,
    pub max: Option<i64>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameData {
    pub title: String,
    pub title_sort_key: String,
    pub publisher: String,
    pub year: String,
    pub players: Option<Range>,
    pub length: Option<Range>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileData {
    pub version: String,
    pub filename: String,
    pub url: String,
    pub size: i64,
    pub checksum: String,
    pub published_at: String,
    pub published_by: String,
    pub requires: String,
    pub authors: Vec<String>
}

// TODO: probably needs slug
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageData {
    pub name: String,
    pub description: String,
    pub releases: Vec<FileData>,
    pub files: Vec<FileData>
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageDataPost {
// TODO: display name?
//    pub name: String,
    pub description: String
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectData {
    pub name: String,
    pub description: String,
    pub revision: i64,
    pub created_at: String,
    pub modified_at: String,
    pub tags: Vec<String>,
    pub game: GameData,
    pub readme: String,
    pub image: Option<String>,
    pub owners: Vec<String>,
    pub packages: Vec<PackageData>
}

fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>
{
    // Ensure that explicit null in the JSON is mapped to Some(None)
    Deserialize::deserialize(de).map(Some)
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RangePatch {
    #[serde(default, deserialize_with = "double_option")]
    pub min: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub max: Option<Option<i64>>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameDataPatch {
    pub title: Option<String>,
    pub title_sort_key: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub players: Option<RangePatch>,
    pub length: Option<RangePatch>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MaybeProjectDataPatch {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub game: Option<GameDataPatch>,
    pub readme: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub image: Option<Option<String>>
}

impl MaybeProjectDataPatch {
    fn empty(&self) -> bool {
        match self {
            MaybeProjectDataPatch {
                description: None,
                tags: None,
                game: None | Some(GameDataPatch {
                    title: None,
                    title_sort_key: None,
                    publisher: None,
                    year: None,
                    players: None | Some(RangePatch {
                        min: None,
                        max: None
                    }),
                    length: None | Some(RangePatch {
                        min: None,
                        max: None
                    }),
                }),
                readme: None,
                image: None
            }
            => true,
            _ => false
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "MaybeProjectDataPatch")]
pub struct ProjectDataPatch {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub game: GameDataPatch,
    pub readme: Option<String>,
    pub image: Option<Option<String>>
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("invalid data {0:?}")]
pub struct ProjectDataPatchError(MaybeProjectDataPatch);

impl TryFrom<MaybeProjectDataPatch> for ProjectDataPatch {
    type Error = ProjectDataPatchError;

    fn try_from(m: MaybeProjectDataPatch) -> Result<Self, Self::Error> {
        // at least one element must be present to be a valid request
        if m.empty() {
            Err(ProjectDataPatchError(m))
        }
        else {
            Ok(
                ProjectDataPatch {
                    description: m.description,
                    tags: m.tags,
                    game: m.game.unwrap_or_default(),
                    readme: m.readme,
                    image: m.image
                }
            )
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectDataPost {
    pub description: String,
    pub tags: Vec<String>,
    pub game: GameData,
    pub readme: String,
    pub image: Option<String>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectSummary {
    pub name: String,
    pub description: String,
    pub revision: i64,
    pub created_at: String,
    pub modified_at: String,
    pub tags: Vec<String>,
    pub game: GameData
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Projects {
    pub projects: Vec<ProjectSummary>,
    pub meta: Pagination
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maybe_project_data_patch_from_json_game_title() {
        let json = "{\"game\":{\"title\":\"foo\"}}";
        assert_eq!(
            serde_json::from_str::<MaybeProjectDataPatch>(json).unwrap(),
            MaybeProjectDataPatch {
                game: Some(GameDataPatch {
                    title: Some("foo".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn maybe_project_data_patch_from_json_image() {
        let json = "{\"image\": \"foo.png\"}";
        assert_eq!(
            serde_json::from_str::<MaybeProjectDataPatch>(json).unwrap(),
            MaybeProjectDataPatch {
                image: Some(Some("foo.png".into())),
                ..Default::default()
            }
        );
    }

    #[test]
    fn maybe_project_data_patch_from_json_image_clear() {
        let json = "{\"image\": null}";
        assert_eq!(
            serde_json::from_str::<MaybeProjectDataPatch>(json).unwrap(),
            MaybeProjectDataPatch {
                image: Some(None),
                ..Default::default()
            }
        );
    }

    #[test]
    fn maybe_project_data_patch_default_empty() {
        assert!(MaybeProjectDataPatch::default().empty());
    }

    #[test]
    fn maybe_project_data_patch_default_and_game_empty() {
        assert!(
            MaybeProjectDataPatch {
                game: Some(GameDataPatch::default()),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                readme: Some("foo".into()),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_game_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                game: Some(GameDataPatch {
                    year: Some("1979".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }.empty()
        );
    }
    #[test]
    fn try_from_project_data_patch_description() {
        assert_eq!(
            ProjectDataPatch::try_from(
                MaybeProjectDataPatch {
                    description: Some("d".into()),
                    ..Default::default()
                }
            ).unwrap(),
            ProjectDataPatch {
                description: Some("d".into()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn try_from_project_data_patch_image_clear() {
        assert_eq!(
            ProjectDataPatch::try_from(
                MaybeProjectDataPatch {
                    image: Some(None),
                    ..Default::default()
                }
            ).unwrap(),
            ProjectDataPatch {
                image: Some(None),
                ..Default::default()
            }
        );
    }

    #[test]
    fn try_from_project_data_patch_err() {
        assert_eq!(
            ProjectDataPatch::try_from(MaybeProjectDataPatch::default())
                .unwrap_err(),
            ProjectDataPatchError(MaybeProjectDataPatch::default())
        );
    }
}
