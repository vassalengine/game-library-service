use serde::{Deserialize, Deserializer, Serialize};

use crate::pagination::Pagination;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct User(pub i64);

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Users {
    pub users: Vec<String>
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Release(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Package(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Project(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Owner(pub i64);

#[derive(Debug, Eq, PartialEq)]
pub struct Owned(pub Owner, pub Project);

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Range {
    pub min: Option<i64>,
    pub max: Option<i64>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameData {
    pub title: String,
    pub title_sort_key: String,
    pub publisher: String,
    pub year: String,
    pub players: Range,
    pub length: Range
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileData {
    pub filename: String,
    pub url: String,
    pub size: i64,
    pub sha256: String,
    pub published_at: String,
    pub published_by: String,
    pub requires: Option<String>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReleaseData {
    pub version: String,
    pub files: Vec<FileData>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageData {
    pub name: String,
    pub sort_key: i64,
    pub description: String,
    pub releases: Vec<ReleaseData>
}

#[derive(Clone, Debug, Deserialize, Default, Eq, PartialEq, Serialize)]
pub struct MaybePackageDataPost {
    pub sort_key: i64,
    pub description: String
}

impl MaybePackageDataPost {
    fn overlong(&self) -> bool {
        self.description.len() > DESCRIPTION_MAX_LENGTH
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "MaybePackageDataPost")]
pub struct PackageDataPost {
// TODO: display name?
//    pub name: String,
    pub sort_key: i64,
    pub description: String
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("invalid data {0:?}")]
pub struct PackageDataPostError(MaybePackageDataPost);

impl TryFrom<MaybePackageDataPost> for PackageDataPost {
    type Error = PackageDataPostError;

    fn try_from(m: MaybePackageDataPost) -> Result<Self, Self::Error> {
        if m.overlong() {
            Err(PackageDataPostError(m))
        }
        else {
            Ok(
                PackageDataPost {
                    sort_key: m.sort_key,
                    description: m.description
                }
            )
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GalleryImage {
    pub filename: String,
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
    pub packages: Vec<PackageData>,
    pub gallery: Vec<GalleryImage>
}

fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>
{
    // Ensure that explicit null in the JSON is mapped to Some(None)
    Deserialize::deserialize(de).map(Some)
}

// maximum field lengths
const DESCRIPTION_MAX_LENGTH: usize = 1024;
const GAME_TITLE_MAX_LENGTH: usize = 256;
const GAME_PUBLISHER_MAX_LENGTH: usize = 256;
const GAME_YEAR_MAX_LENGTH: usize = 32;
const README_MAX_LENGTH: usize = 65536;
const IMAGE_MAX_LENGTH: usize = 256;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RangePatch {
    #[serde(default, deserialize_with = "double_option")]
    pub min: Option<Option<u32>>,
    #[serde(default, deserialize_with = "double_option")]
    pub max: Option<Option<u32>>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MaybeGameDataPatch {
    pub title: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub players: Option<RangePatch>,
    pub length: Option<RangePatch>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(from = "MaybeGameDataPatch")]
pub struct GameDataPatch {
    pub title: Option<String>,
    pub title_sort_key: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub players: RangePatch,
    pub length: RangePatch
}

impl From<MaybeGameDataPatch> for GameDataPatch {
    fn from(m: MaybeGameDataPatch) -> Self {
        GameDataPatch {
            title: m.title,
            title_sort_key: None,
            publisher: m.publisher,
            year: m.year,
            players: m.players.unwrap_or_default(),
            length: m.length.unwrap_or_default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MaybeProjectDataPatch {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub game: Option<MaybeGameDataPatch>,
    pub readme: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub image: Option<Option<String>>
}

impl MaybeProjectDataPatch {
    fn empty(&self) -> bool {
        matches!(
            self,
            MaybeProjectDataPatch {
                description: None,
                tags: None,
                game: None | Some(MaybeGameDataPatch {
                    title: None,
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
        )
    }

    fn overlong(&self) -> bool {
        matches!(
            &self.description,
            Some(s) if s.len() > DESCRIPTION_MAX_LENGTH
        ) ||
        matches!(
            &self.readme,
            Some(s) if s.len() > README_MAX_LENGTH
        ) ||
        matches!(
            &self.image,
            Some(Some(s)) if s.len() > IMAGE_MAX_LENGTH
        ) ||
        if let Some(game) = &self.game {
            matches!(
                &game.title,
                Some(s) if s.len() > GAME_TITLE_MAX_LENGTH
            ) ||
            matches!(
                &game.publisher,
                Some(s) if s.len() > GAME_PUBLISHER_MAX_LENGTH
            ) ||
            matches!(
                &game.year,
                Some(s) if s.len() > GAME_YEAR_MAX_LENGTH
            )
        }
        else {
            false
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
        // and field lengths must be within bounds
        if m.empty() || m.overlong() {
            Err(ProjectDataPatchError(m))
        }
        else {
            Ok(
                ProjectDataPatch {
                    description: m.description,
                    tags: m.tags,
                    game: m.game.unwrap_or_default().into(),
                    readme: m.readme,
                    image: m.image
                }
            )
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MaybeRangePost {
    pub min: Option<u32>,
    pub max: Option<u32>
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "MaybeRangePost")]
pub struct RangePost {
    pub min: Option<u32>,
    pub max: Option<u32>
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("min > max: {0:?}")]
pub struct RangePostError(MaybeRangePost);

impl TryFrom<MaybeRangePost> for RangePost {
    type Error = RangePostError;

    fn try_from(m: MaybeRangePost) -> Result<Self, Self::Error> {
        match (m.min, m.max) {
            (None, _) | (_, None) => Ok(RangePost{ min: m.min, max: m.max }),
            (Some(min), Some(max)) if min <= max => Ok(RangePost{ min: m.min, max: m.max }),
            _ => Err(RangePostError(m))
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GameDataPost {
    pub title: String,
    #[serde(skip)]
    pub title_sort_key: String,
    pub publisher: String,
    pub year: String,
    pub players: RangePost,
    pub length: RangePost
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MaybeProjectDataPost {
    pub description: String,
    pub tags: Vec<String>,
    pub game: GameDataPost,
    pub readme: String,
    pub image: Option<String>
}

impl MaybeProjectDataPost {
    fn overlong(&self) -> bool {
        self.description.len() > DESCRIPTION_MAX_LENGTH ||
        self.game.title.len() > GAME_TITLE_MAX_LENGTH ||
        self.game.publisher.len() > GAME_PUBLISHER_MAX_LENGTH ||
        self.game.year.len() > GAME_YEAR_MAX_LENGTH ||
        self.readme.len() > README_MAX_LENGTH ||
        self.image.as_ref().is_some_and(|i| i.len() > IMAGE_MAX_LENGTH)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "MaybeProjectDataPost")]
pub struct ProjectDataPost {
    pub description: String,
    pub tags: Vec<String>,
    pub game: GameDataPost,
    pub readme: String,
    pub image: Option<String>
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("invalid data {0:?}")]
pub struct ProjectDataPostError(MaybeProjectDataPost);

impl TryFrom<MaybeProjectDataPost> for ProjectDataPost {
    type Error = ProjectDataPostError;

    fn try_from(m: MaybeProjectDataPost) -> Result<Self, Self::Error> {
        // field lengths must be within bounds
        if m.overlong() {
            Err(ProjectDataPostError(m))
        }
        else {
            Ok(
                ProjectDataPost{
                    description: m.description,
                    tags: m.tags,
                    game: m.game,
                    readme: m.readme,
                    image: m.image
                }
            )
        }
    }
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

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("flag tag {0} unknown")]
pub struct FlagTagError(String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub enum FlagTag {
    Inappropriate,
    Spam,
    Illegal,
    Other
}

impl TryFrom<&str> for FlagTag {
    type Error = FlagTagError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "inappropriate" => Ok(FlagTag::Inappropriate),
            "spam" => Ok(FlagTag::Spam),
            "illegal" => Ok(FlagTag::Illegal),
            "other" => Ok(FlagTag::Other),
            _ => Err(FlagTagError(value.into()))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlagData {
    flag: FlagTag,
    message: Option<String>
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "FlagData")]
pub enum Flag {
    Inappropriate,
    Spam,
    Illegal(String),
    Other(String)
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("flag {0:?} invalid")]
pub struct FlagError(FlagData);

impl TryFrom<FlagData> for Flag {
    type Error = FlagError;

    fn try_from(fd: FlagData) -> Result<Self, Self::Error> {
        match fd {
            FlagData {
                flag: FlagTag::Inappropriate,
                message: None
            } => Ok(Flag::Inappropriate),
            FlagData {
                flag: FlagTag::Spam,
                message: None
            } => Ok(Flag::Spam),
            FlagData {
                flag: FlagTag::Illegal,
                message: Some(msg)
            } => Ok(Flag::Illegal(msg)),
            FlagData {
                flag: FlagTag::Other,
                message: Some(msg)
            } => Ok(Flag::Other(msg)),
            _ => Err(FlagError(fd))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_from_package_data_post_ok() {
        assert_eq!(
            PackageDataPost::try_from(
                MaybePackageDataPost {
                    sort_key: 3,
                    description: "desc".into()
                }
            ).unwrap(),
            PackageDataPost {
                sort_key: 3,
                description: "desc".into()
            }
        );
    }

    #[test]
    fn try_from_package_data_post_overlong_description() {
        let mpdp = MaybePackageDataPost {
            description: "x".repeat(DESCRIPTION_MAX_LENGTH + 1),
            ..Default::default()
        };

        assert_eq!(
            PackageDataPost::try_from(mpdp.clone()).unwrap_err(),
            PackageDataPostError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_post_ok() {
        assert_eq!(
            ProjectDataPost::try_from(
                MaybeProjectDataPost {
                    description: "description".into(),
                    tags: vec![],
                    game: GameDataPost {
                        title: "the title".into(),
                        title_sort_key: "title, the".into(),
                        publisher: "publisher".into(),
                        year: "1979".into(),
                        players: RangePost::default(),
                        length: RangePost::default()
                    },
                    readme: "readme".into(),
                    image: None
                }
            ).unwrap(),
            ProjectDataPost {
                description: "description".into(),
                tags: vec![],
                game: GameDataPost {
                    title: "the title".into(),
                    title_sort_key: "title, the".into(),
                    publisher: "publisher".into(),
                    year: "1979".into(),
                    players: RangePost::default(),
                    length: RangePost::default()
                },
                readme: "readme".into(),
                image: None
            }
        );
    }

    #[test]
    fn try_from_project_data_post_overlong_description() {
        let mpdp = MaybeProjectDataPost {
            description: "x".repeat(DESCRIPTION_MAX_LENGTH + 1),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPost::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPostError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_post_overlong_title() {
        let mpdp = MaybeProjectDataPost {
            game: GameDataPost {
                title: "x".repeat(GAME_TITLE_MAX_LENGTH + 1),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPost::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPostError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_post_overlong_publisher() {
        let mpdp = MaybeProjectDataPost {
            game: GameDataPost {
                publisher: "x".repeat(GAME_PUBLISHER_MAX_LENGTH + 1),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPost::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPostError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_post_overlong_year() {
        let mpdp = MaybeProjectDataPost {
            game: GameDataPost {
                year: "x".repeat(GAME_YEAR_MAX_LENGTH + 1),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPost::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPostError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_post_overlong_readme() {
        let mpdp = MaybeProjectDataPost {
            readme: "x".repeat(README_MAX_LENGTH + 1),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPost::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPostError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_post_overlong_image() {
        let mpdp = MaybeProjectDataPost {
            image: Some("x".repeat(IMAGE_MAX_LENGTH + 1)),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPost::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPostError(mpdp)
        );
    }

    #[test]
    fn maybe_project_data_post_from_json_minimal() {
    let json = r#"
        {
            "game": {
                "title": "foo",
                "publisher": "",
                "year": "",
                "players": { "min": null, "max": null },
                "length": { "min": null, "max": null }
            },
            "description": "",
            "tags": [],
            "readme": ""
        }"#;

        assert_eq!(
            serde_json::from_str::<MaybeProjectDataPost>(json).unwrap(),
            MaybeProjectDataPost {
                description: "".into(),
                tags: vec![],
                game: GameDataPost {
                    title: "foo".into(),
                    title_sort_key: "".into(),
                    publisher: "".into(),
                    year: "".into(),
                    ..Default::default()
                },
                readme: "".into(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn maybe_project_data_patch_from_json_game_title() {
        let json = r#"{"game":{"title":"foo"}}"#;
        assert_eq!(
            serde_json::from_str::<MaybeProjectDataPatch>(json).unwrap(),
            MaybeProjectDataPatch {
                game: Some(MaybeGameDataPatch {
                    title: Some("foo".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn maybe_project_data_patch_from_json_image() {
        let json = r#"{"image": "foo.png"}"#;
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
        let json = r#"{"image": null}"#;
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
                game: Some(MaybeGameDataPatch::default()),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_description_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                description: Some("description".into()),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_readme_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                readme: Some("readme".into()),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_image_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                image: Some(None),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_image_not_empty_none() {
        assert!(
            !MaybeProjectDataPatch {
                image: Some(Some("image".into())),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_title_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                game: Some(MaybeGameDataPatch {
                    title: Some("title".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_publisher_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                game: Some(MaybeGameDataPatch {
                    publisher: Some("publisher".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_year_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                game: Some(MaybeGameDataPatch {
                    year: Some("1979".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }.empty()
        );
    }

    #[test]
    fn maybe_project_data_patch_plyers_not_empty() {
        assert!(
            !MaybeProjectDataPatch {
                game: Some(MaybeGameDataPatch {
                    players: Some(RangePatch {
                        min: Some(Some(1)),
                        max: Some(Some(2))
                     }),
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
                    description: Some("description".into()),
                    ..Default::default()
                }
            ).unwrap(),
            ProjectDataPatch {
                description: Some("description".into()),
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
    fn try_from_project_data_patch_empty() {
        assert_eq!(
            ProjectDataPatch::try_from(MaybeProjectDataPatch::default())
                .unwrap_err(),
            ProjectDataPatchError(MaybeProjectDataPatch::default())
        );
    }

    #[test]
    fn try_from_project_data_patch_overlong_description() {
        let mpdp = MaybeProjectDataPatch {
            description: Some("x".repeat(DESCRIPTION_MAX_LENGTH + 1)),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPatch::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPatchError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_patch_overlong_title() {
        let mpdp = MaybeProjectDataPatch {
            game: Some(MaybeGameDataPatch {
                title: Some("x".repeat(GAME_TITLE_MAX_LENGTH + 1)),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPatch::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPatchError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_patch_overlong_publisher() {
        let mpdp = MaybeProjectDataPatch {
            game: Some(MaybeGameDataPatch {
                publisher: Some("x".repeat(GAME_PUBLISHER_MAX_LENGTH + 1)),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPatch::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPatchError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_patch_overlong_year() {
        let mpdp = MaybeProjectDataPatch {
            game: Some(MaybeGameDataPatch {
                year: Some("x".repeat(GAME_YEAR_MAX_LENGTH + 1)),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPatch::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPatchError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_patch_overlong_readme() {
        let mpdp = MaybeProjectDataPatch {
            readme: Some("x".repeat(README_MAX_LENGTH + 1)),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPatch::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPatchError(mpdp)
        );
    }

    #[test]
    fn try_from_project_data_patch_overlong_image() {
        let mpdp = MaybeProjectDataPatch {
            image: Some(Some("x".repeat(IMAGE_MAX_LENGTH + 1))),
            ..Default::default()
        };

        assert_eq!(
            ProjectDataPatch::try_from(mpdp.clone()).unwrap_err(),
            ProjectDataPatchError(mpdp)
        );
    }

    #[test]
    fn try_from_str_flag_tag() {
        assert_eq!(FlagTag::try_from("inappropriate").unwrap(), FlagTag::Inappropriate);
        assert_eq!(FlagTag::try_from("spam").unwrap(), FlagTag::Spam);
        assert_eq!(FlagTag::try_from("illegal").unwrap(), FlagTag::Illegal);
        assert_eq!(FlagTag::try_from("other").unwrap(), FlagTag::Other);
        assert_eq!(FlagTag::try_from("bogus").unwrap_err(), FlagTagError("bogus".into()));
    }

    #[test]
    fn try_from_flag_data_flag_inappropriate() {
        assert_eq!(
            Flag::try_from(FlagData {
                flag: FlagTag::Inappropriate,
                message: None
            }),
            Ok(Flag::Inappropriate)
        );
    }

    #[test]
    fn try_from_flag_data_flag_inappropriate_msg() {
        let fd = FlagData {
            flag: FlagTag::Inappropriate,
            message: Some("bad".into())
        };

        assert_eq!(
            Flag::try_from(fd.clone()),
            Err(FlagError(fd))
        );
    }

    #[test]
    fn try_from_flag_data_flag_spam() {
        assert_eq!(
            Flag::try_from(FlagData {
                flag: FlagTag::Spam,
                message: None
            }),
            Ok(Flag::Spam)
        );
    }

    #[test]
    fn try_from_flag_data_flag_spam_msg() {
        let fd = FlagData {
            flag: FlagTag::Spam,
            message: Some("bad".into())
        };

        assert_eq!(
            Flag::try_from(fd.clone()),
            Err(FlagError(fd))
        );
    }

    #[test]
    fn try_from_flag_data_flag_illegal() {
        let fd = FlagData {
            flag: FlagTag::Illegal,
            message: None
        };

        assert_eq!(
            Flag::try_from(fd.clone()),
            Err(FlagError(fd))
        );
    }

    #[test]
    fn try_from_flag_data_flag_illegal_msg() {
        assert_eq!(
            Flag::try_from(FlagData {
                flag: FlagTag::Illegal,
                message: Some("ok".into())
            }),
            Ok(Flag::Illegal("ok".into()))
        );
    }

    #[test]
    fn try_from_flag_data_flag_other() {
        let fd = FlagData {
            flag: FlagTag::Other,
            message: None
        };

        assert_eq!(
            Flag::try_from(fd.clone()),
            Err(FlagError(fd))
        );
    }

    #[test]
    fn try_from_flag_data_flag_other_msg() {
        assert_eq!(
            Flag::try_from(FlagData {
                flag: FlagTag::Other,
                message: Some("ok".into())
            }),
            Ok(Flag::Other("ok".into()))
        );
    }

    #[test]
    fn try_from_maybe_range_post_none_none() {
        assert_eq!(
            RangePost::try_from(MaybeRangePost { min: None, max: None }),
            Ok(RangePost { min: None, max: None })
        );
    }

    #[test]
    fn try_from_maybe_range_post_some_none() {
        assert_eq!(
            RangePost::try_from(MaybeRangePost { min: Some(1), max: None }),
            Ok(RangePost { min: Some(1), max: None })
        );
    }

    #[test]
    fn try_from_maybe_range_post_none_some() {
        assert_eq!(
            RangePost::try_from(MaybeRangePost { min: None, max: Some(1) }),
            Ok(RangePost { min: None, max: Some(1) })
        );
    }

    #[test]
    fn try_from_maybe_range_post_some_some_eq() {
        assert_eq!(
            RangePost::try_from(MaybeRangePost { min: Some(1), max: Some(1) }),
            Ok(RangePost { min: Some(1), max: Some(1) })
        );
    }

    #[test]
    fn try_from_maybe_range_post_some_some_less() {
        assert_eq!(
            RangePost::try_from(MaybeRangePost { min: Some(0), max: Some(1) }),
            Ok(RangePost { min: Some(0), max: Some(1) })
        );
    }

    #[test]
    fn try_from_maybe_range_post_some_some_more() {
        let mrp = MaybeRangePost { min: Some(1), max: Some(0) };
        assert_eq!(
            RangePost::try_from(mrp.clone()),
            Err(RangePostError(mrp))
        );
    }
}
