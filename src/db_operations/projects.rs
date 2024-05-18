use diesel::prelude::*;
use workflow::*;
use diesel::prelude::QueryDsl;

use diesel::result::Error;
use workflow::models::*;
use crate::db_operations;

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
            println!("yes!");
            println!("{:?}", x);
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
    }) 
    {
        Ok(_) => {
            if display_communicates {
                println!("Transaction committed successfully:");
                println!("\nSaved project \"{}\"", project_name_);
            }
            Ok(project_id)
        }
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            return Err("Project of such name is already in the database, choose another name!");
        },
        Err(x) => {
            println!("{}",x);
            return Err("Database error occurred");
        }
    }
}

