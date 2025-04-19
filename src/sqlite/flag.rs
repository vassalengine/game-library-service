use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    db::DatabaseError,
    model::{Flag, Project, User}
};

pub async fn add_flag<'e, E>(
    ex: E,
    reporter: User,
    proj: Project,
    flag: &Flag
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;



}
