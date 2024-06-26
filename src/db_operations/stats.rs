use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::dsl::date;
use diesel::dsl::sql;
use diesel::prelude::QueryDsl;
use diesel::prelude::*;
use diesel::sql_types::Int4;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use diesel::sql_types::Timestamp;
use workflow::*;

use diesel::result::Error;
use workflow::models::*;

pub fn get_stats(
    _args: &[String],
) -> Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &'static str> {
    use workflow::schema::log::dsl::log;
    use workflow::schema::tasks::dsl::tasks;

    let connection: &mut PgConnection = &mut establish_connection();

    let result: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, Error> =
        tasks::table()
            .left_join(log.on(workflow::schema::log::task_id.eq(workflow::schema::tasks::task_id)))
            .order((
                workflow::schema::tasks::project_id.asc(),
                workflow::schema::tasks::task_id.asc(),
                workflow::schema::log::date.asc(),
            ))
            .select((
                Task::as_select(),
                sql::<Nullable<Int4>>("log.log_id"),
                sql::<Nullable<Text>>("log.log_type"),
                sql::<Nullable<Timestamp>>("log.date"),
            ))
            .load::<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>(connection);

    match result {
        Ok(x) => Ok(x),
        Err(_) => Err("An error occurred while fetching stats"),
    }
}

pub fn get_day_stats_tasks(
    date_to_seek: NaiveDate,
    seeked_project_id: Option<i32>,
) -> Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &'static str> {
    use workflow::schema::log::dsl::log;
    use workflow::schema::tasks::dsl::tasks;

    let connection: &mut PgConnection = &mut establish_connection();

    let mut result = tasks::table()
        .left_join(log.on(workflow::schema::log::task_id.eq(workflow::schema::tasks::task_id)))
        .order((
            workflow::schema::tasks::project_id.asc(),
            workflow::schema::tasks::task_id.asc(),
            workflow::schema::log::date.asc(),
        ))
        .into_boxed();
    if let Some(_) = seeked_project_id {
        result = result.filter(workflow::schema::tasks::project_id.eq(seeked_project_id.unwrap()));
    }
    result = result.filter(date(workflow::schema::log::date).eq(date_to_seek));

    let result = result
        .select((
            Task::as_select(),
            sql::<Nullable<Int4>>("log.log_id"),
            sql::<Nullable<Text>>("log.log_type"),
            sql::<Nullable<Timestamp>>("log.date"),
        ))
        .load::<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>(connection);
    match result {
        Ok(x) => Ok(x),
        Err(_) => Err("An error occurred while fetching day stats"),
    }
}
