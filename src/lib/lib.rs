use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use chrono::Local;
use crate::models::*;

pub mod models;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

use self::models::{NewApp, App};

    pub fn create_app(conn: &mut PgConnection, app_name: &str) ->Result<App,diesel::result::Error>{
        use crate::schema::apps;

        let new_app = NewApp { app_name };


        diesel::insert_into(apps::table)
        .values(&new_app)
        .returning(App::as_returning())
        .get_result(conn)

    // match app {
    //     Ok(app) => Ok(app),
    //     Err(x) => Err("Error creating app"),
    // }

    }

    use self::models::{NewTask,Task};

    pub fn create_task(conn: &mut PgConnection,project_id: i32 , task_name: &str, _planned_time: Option<&str>)-> Result<Task,diesel::result::Error>{
        use crate::schema::tasks;

        let new_task=NewTask{project_id, task_name,username: &whoami::username(),planned_time:_planned_time};

        diesel::insert_into(tasks::table)
            .values(&new_task)
            .returning(Task::as_returning())
            .get_result(conn)
            
    }
    pub fn create_project(conn: &mut PgConnection,project_name: &str, _planned_time: Option<&str>)-> Result<Project,diesel::result::Error>{
        use crate::schema::projects;


        let new_project=NewProject{project_name,username: &whoami::username(),planned_time:_planned_time};

        diesel::insert_into(projects::table)
            .values(&new_project)
            .returning(Project::as_returning())
            .get_result(conn)
            
    }
    use self::models::{NewProjectApp,ProjectApp};

    pub fn create_app_detail(conn: &mut PgConnection, task_id: i32, app_id:i32)-> ProjectApp{
        

        let new_app_detail=NewProjectApp{project_id: task_id,app_id};

        diesel::insert_into(schema::project_apps::table)
            .values(&new_app_detail)
            .returning(ProjectApp::as_returning())
            .get_result(conn)
            .expect("Error creating new task")
    }


    use self::models::{NewLog,Log};

    pub fn create_log(conn: &mut PgConnection, task_id: i32, log_type:String)-> Result<Log,diesel::result::Error>{
        use crate::schema::log;

        let current_local_time = Local::now().naive_local();

        let new_log=NewLog{task_id,log_type,date: current_local_time};

        diesel::insert_into(log::table)
            .values(&new_log)
            .returning(Log::as_returning())
            .get_result(conn)

    }




