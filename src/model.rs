use serde::{Deserialize, Serialize};

use crate::pagination::Pagination;

// TODO: rationalize struct naming---names should reflect whether the
// structs are input or ouptut

// TODO: User should become a newtype so that you can't construct one
// without having verified that the user exists
//

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct User(pub String);

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Users {
    pub users: Vec<User>
}

#[derive(Debug, PartialEq)]
pub struct UserID(pub i64);

#[derive(Debug, PartialEq)]
pub struct PackageID(pub i64);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Project(pub String);

#[derive(Debug, PartialEq)]
pub struct ProjectID(pub i64);

#[derive(Debug, PartialEq)]
pub struct Owner(pub String);

#[derive(Debug, PartialEq)]
pub struct Owned(pub Owner, pub ProjectID);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GameData {
    pub title: String,
    pub title_sort_key: String,
    pub publisher: String,
    pub year: String
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ReleaseData {
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
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PackageData {
    pub name: String,
    pub description: String,
    pub releases: Vec<ReleaseData>
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct PackageDataPut {
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GameDataPatch {
    pub title: Option<String>,
    pub title_sort_key: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ProjectDataPatch {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub game: GameDataPatch,
    pub readme: Option<String>,
    pub image: Option<Option<String>>
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectDataPost {
    pub description: String,
    pub tags: Vec<String>,
    pub game: GameData,
    pub readme: String,
    pub image: Option<String>
}

// TODO: maybe use a date type for ctime, mtime?

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectSummary {
    pub name: String,
    pub description: String,
    pub revision: i64,
    pub created_at: String,
    pub modified_at: String,
    pub tags: Vec<String>,
    pub game: GameData
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Projects {
    pub projects: Vec<ProjectSummary>,
    pub meta: Pagination
}
