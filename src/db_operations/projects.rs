use crate::db_operations;
use crate::Commands;
use chrono::NaiveDate;
use diesel::associations::HasTable;
use diesel::dsl::{date, sql};
use diesel::prelude::QueryDsl;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use workflow::models::*;
use workflow::*;

pub fn find_project(project_name_: &str) -> Result<Option<Project>, &'static str> {
    use self::schema::projects::dsl::*;
    let connection = &mut establish_connection();
    let app = projects
        .filter(project_name.eq(project_name_.to_lowercase()))
        .select(Project::as_select())
        .first(connection);

    match app {
        Ok(x) => Ok(Some(x)),
        Err(Error::NotFound) => Ok(None),
        Err(x) => {
            println!("{}", x);
            Err("An error occured while fetching project")
        }
    }
}

pub fn get_projects() -> Result<Vec<Project>, Error> {
    use self::schema::projects::dsl::projects;

    let connection = &mut establish_connection();
    let projects_list = projects.load::<Project>(connection)?;

    Ok(projects_list)
}

pub fn get_date_projects(date_to_seek: NaiveDate) -> Result<Vec<Project>, Error> {
    use self::schema::log::dsl::log;
    use self::schema::projects::dsl::projects;
    use self::schema::tasks::dsl::tasks;

    let connection = &mut establish_connection();
    let projects_list = tasks
        .inner_join(log.on(workflow::schema::log::task_id.eq(workflow::schema::tasks::task_id)))
        .inner_join(
            projects
                .on(workflow::schema::projects::project_id.eq(workflow::schema::tasks::project_id)),
        )
        .filter(date(workflow::schema::log::date).eq(date_to_seek))
        .order(workflow::schema::projects::project_id.asc())
        .select(Project::as_select())
        .distinct()
        .load::<Project>(connection)?;

    Ok(projects_list)
}

pub fn add_project(
    project_name_: &str,
    planned_time: Option<&str>,
    project_apps: Option<&[String]>,
    display_communicates: bool,
) -> Result<i32, &'static str> {
    let connection = &mut establish_connection();
    let mut project_id = 0;

    match connection.transaction::<_, Error, _>(|connection| {
        let project = match create_project(connection, project_name_, planned_time) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            }
        };

        project_id = project.project_id;

        if let Some(x) = project_apps {
            let app_ids = match db_operations::apps::add_multiple_apps(x, false, connection) {
                Ok(x) => x,
                Err(x) => {
                    println!("{}", x);
                    vec![]
                }
            };

            println!("{:?}", app_ids);
            for _app_id in app_ids {
                create_app_detail(connection, project_id, _app_id);
            }
        }
        Ok("")
    }) {
        Ok(_) => {
            if display_communicates {
                println!("Transaction committed successfully");
                println!("Saved project \"{}\"", project_name_);
            }
            Ok(project_id)
        }
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            return Err("Project of such name is already in the database, choose another name!");
        }
        Err(x) => {
            println!("{}", x);
            return Err("Database error occurred");
        }
    }
}

pub fn get_apps_in_projects() -> Result<Vec<(Project, Option<String>)>, &'static str> {
    use workflow::schema::*;

    let connection: &mut PgConnection = &mut establish_connection();

    let result = projects::dsl::projects
        .left_join(
            project_apps::dsl::project_apps::table()
                .on(project_apps::dsl::project_id.eq(projects::dsl::project_id)),
        )
        .left_join(apps::dsl::apps::table().on(apps::dsl::app_id.eq(project_apps::dsl::app_id)))
        .order(projects::dsl::project_id.asc())
        .select((Project::as_select(), sql::<Nullable<Text>>("apps.app_name")))
        .distinct()
        .load::<(Project, Option<String>)>(connection);

    match result {
        Ok(x) => Ok(x),
        Err(_) => Err("An error occurred while fetching apps"),
    }
}

pub fn get_tasks_in_projects(
    project_ids: Option<Vec<i32>>,
    ended: Option<bool>,
) -> Result<Vec<(Project, Option<String>, Option<String>)>, &'static str> {
    use workflow::schema::*;

    let connection: &mut PgConnection = &mut establish_connection();

    let mut result = projects::dsl::projects
        .left_join(
            tasks::dsl::tasks::table().on(tasks::dsl::project_id.eq(projects::dsl::project_id)),
        )
        .left_join(log::dsl::log::table().on(tasks::dsl::task_id.eq(log::dsl::task_id)))
        .into_boxed();
    let mut sub_query = projects::dsl::projects
        .left_join(
            tasks::dsl::tasks::table().on(tasks::dsl::project_id.eq(projects::dsl::project_id)),
        )
        .left_join(
            workflow::schema::log::table.on(workflow::schema::tasks::task_id
                .eq(workflow::schema::log::task_id)
                .and(workflow::schema::log::log_type.eq(Commands::End.to_string()))),
        )
        .into_boxed();
    if project_ids.is_some() {
        let seeked_ids = project_ids.unwrap();
        result = result.filter(workflow::schema::tasks::project_id.eq_any(seeked_ids.clone()));
        sub_query=sub_query.filter(workflow::schema::tasks::project_id.eq_any(seeked_ids));
    }

    let result = if ended.is_some() && ended.unwrap() {
        result.filter(workflow::schema::log::log_type.eq(Commands::End.to_string()))
            .order(projects::dsl::project_id.asc())
            .select((
                Project::as_select(),
                sql::<Nullable<Text>>("tasks.task_name"),
                sql::<Nullable<Text>>("tasks.planned_time"),
            ))
            .distinct()
            .load::<(Project, Option<String>, Option<String>)>(connection)
    } else if ended.is_some() && !ended.unwrap() {
        sub_query.filter(workflow::schema::log::task_id.is_null())
            .order(projects::dsl::project_id.asc())
            .select((
                Project::as_select(),
                sql::<Nullable<Text>>("tasks.task_name"),
                sql::<Nullable<Text>>("tasks.planned_time"),
            ))
            .distinct()
            .load::<(Project, Option<String>, Option<String>)>(connection)
    } else {
        result.order(projects::dsl::project_id.asc())
            .select((
                Project::as_select(),
                sql::<Nullable<Text>>("tasks.task_name"),
                sql::<Nullable<Text>>("tasks.planned_time"),
            ))
            .distinct()
            .load::<(Project, Option<String>, Option<String>)>(connection)
    };

        
    match result {
        Ok(x) => Ok(x),
        Err(_) => Err("An error occurred while fetching apps"),
    }
}
