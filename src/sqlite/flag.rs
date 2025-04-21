use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    db::DatabaseError,
    model::{Flag, Project, User}
};

impl<'a> From<&'a Flag> for (u32, Option<&'a str>) {
    fn from(f: &Flag) -> (u32, Option<&str>) {
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
    flag: &Flag,
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
    fn tuple_from_flag_inappropriate() {
        let (t, m): (u32, Option<&str>) = (&Flag::Inappropriate).into();
        assert_eq!((t, m), (0, None));
    }

    #[test]
    fn tuple_from_flag_spam() {
        let (t, m): (u32, Option<&str>) = (&Flag::Spam).into();
        assert_eq!((t, m), (1, None));
    }

    #[test]
    fn tuple_from_flag_illegal() {
        let f = Flag::Illegal("x".into());
        let (t, m): (u32, Option<&str>) = (&f).into();
        assert_eq!((t, m), (2, Some("x")));
    }

    #[test]
    fn tuple_from_flag_other() {
        let f = Flag::Other("x".into());
        let (t, m): (u32, Option<&str>) = (&f).into();
        assert_eq!((t, m), (3, Some("x")));
    }
}
