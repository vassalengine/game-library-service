use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    db::DatabaseError,
    model::{FlagPost, Project, User}
};

impl<'a> From<&'a FlagPost> for (u32, Option<&'a str>) {
    fn from(f: &FlagPost) -> (u32, Option<&str>) {
        match f {
            FlagPost::Inappropriate => (0, None),
            FlagPost::Spam => (1, None),
            FlagPost::Illegal(msg) => (2, Some(msg)),
            FlagPost::Other(msg) => (3, Some(msg))
        }
    }
}

pub async fn add_flag<'e, E>(
    ex: E,
    reporter: User,
    proj: Project,
    flag: &FlagPost,
    now: i64
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
    flagged_at,
    flag,
    message
)
VALUES (?, ?, ?, ?, ?)
        ",
        reporter.0,
        proj.0,
        now,
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
    fn tuple_from_flag_post_inappropriate() {
        let (t, m): (u32, Option<&str>) = (&FlagPost::Inappropriate).into();
        assert_eq!((t, m), (0, None));
    }

    #[test]
    fn tuple_from_flag_post_spam() {
        let (t, m): (u32, Option<&str>) = (&FlagPost::Spam).into();
        assert_eq!((t, m), (1, None));
    }

    #[test]
    fn tuple_from_flag_post_illegal() {
        let f = FlagPost::Illegal("x".into());
        let (t, m): (u32, Option<&str>) = (&f).into();
        assert_eq!((t, m), (2, Some("x")));
    }

    #[test]
    fn tuple_from_flag_post_other() {
        let f = FlagPost::Other("x".into());
        let (t, m): (u32, Option<&str>) = (&f).into();
        assert_eq!((t, m), (3, Some("x")));
    }
}
