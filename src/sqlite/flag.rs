use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    db::DatabaseError,
    model::{Flag, Project, User}
};

impl From<Flag> for (u32, Option<String>) {
    fn from(f: Flag) -> (u32, Option<String>) {
        match f {
            Flag::Inappropriate => (0, None),
            Flag::Spam => (1, None),
            Flag::Illegal(msg) => (2, Some(msg)),
            Flag::Other(msg) => (3, Some(msg))
        }
    }
}

pub async fn add_flag<'e, E>(
    ex: E,
    reporter: User,
    proj: Project,
    flag: Flag
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let (flag, msg) = flag.into();

    sqlx::query!(
        "
INSERT INTO flags (
    user_id,
    project_id,
    flag,
    message
)
VALUES (?, ?, ?, ?)
        ",
        reporter.0,
        proj.0,
        flag,
        msg
    )
    .execute(ex)
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tuple_from_flag() {
        assert_eq!(
            <(u32, Option<String>)>::from(Flag::Inappropriate),
            (0, None)
        );

        assert_eq!(
            <(u32, Option<String>)>::from(Flag::Spam),
            (1, None)
        );

        assert_eq!(
            <(u32, Option<String>)>::from(Flag::Illegal("x".into())),
            (2, Some("x".into()))
        );

        assert_eq!(
            <(u32, Option<String>)>::from(Flag::Other("x".into())),
            (3, Some("x".into()))
        );
    }
}
