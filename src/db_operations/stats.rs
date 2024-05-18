use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::sql_types::Timestamp;
use workflow::*;
use diesel::prelude::QueryDsl;
use diesel::dsl::sql;
use diesel::sql_types::Text;
use diesel::sql_types::Int4;
use diesel::sql_types::Nullable;
use chrono::prelude::*;
use diesel::dsl::date;

use diesel::result::Error;
use workflow::models::*;

pub fn get_stats(_args: &[String]) -> Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &'static str>{
    use workflow::schema::tasks::dsl::tasks;
    use workflow::schema::log::dsl::log;

    let connection: &mut PgConnection = &mut establish_connection();

    let result: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, Error> =tasks::table()
        .left_join(log.on(workflow::schema::log::task_id.eq(workflow::schema::tasks::task_id)))
        .order((workflow::schema::tasks::task_id.asc(), workflow::schema::log::date.asc())) 
        .select((Task::as_select(),sql::<Nullable<Int4>>("log.log_id"),sql::<Nullable<Text>>("log.log_type"),sql::<Nullable<Timestamp>>("log.date") ))
        .load::<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>(connection);
        
    match result{
        Ok(x)=>Ok(x),
        Err(_)=>Err("An error occurred while fetching stats")
    }

}

pub fn get_day_stats_tasks(date_to_seek:NaiveDate,seeked_project_id: i32) -> Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &'static str>{
    use workflow::schema::tasks::dsl::tasks;
    use workflow::schema::log::dsl::log;

    let connection: &mut PgConnection = &mut establish_connection();
    

    let result: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, Error> =tasks::table()
        .left_join(log.on(workflow::schema::log::task_id.eq(workflow::schema::tasks::task_id)))
        .order((workflow::schema::tasks::task_id.asc(), workflow::schema::log::date.asc())) 
        .filter(date(workflow::schema::log::date).eq(date_to_seek).and(workflow::schema::tasks::project_id.eq(seeked_project_id)))
        .select((Task::as_select(),sql::<Nullable<Int4>>("log.log_id"),sql::<Nullable<Text>>("log.log_type"),sql::<Nullable<Timestamp>>("log.date") ))
        .load::<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>(connection);
        
    match result{
        Ok(x)=>Ok(x),
        Err(_)=>Err("An error occurred while fetching day stats")
    }

}
