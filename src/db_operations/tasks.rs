use diesel::prelude::*;
use workflow::*;
use diesel::prelude::QueryDsl;

use diesel::result::Error;
use workflow::models::*;

pub fn find_task(task_name_: &str) -> Result<Option<Task>, &'static str> {
    
    use self::schema::tasks::dsl::*;
    let connection = &mut establish_connection();
    let app = tasks
        .filter(task_name.eq(task_name_.to_lowercase()))
        .select(Task::as_select())
        .first(connection);

    match app {
        Ok(x) => Ok(Some(x)),
        Err(Error::NotFound) => Ok(None),
        Err(x) => {
            println!("{}", x);
            Err("An error occured while fetching task")
        }
    }
}

pub fn get_tasks() -> Result<Vec<Task>, Error> {
    use self::schema::tasks::dsl::tasks;

    let connection = &mut establish_connection();
    let tasks_list = tasks.load::<Task>(connection)?;

    Ok(tasks_list)
}

pub fn add_task(
    project_id:i32,
    task_name_: &str,
    planned_time: Option<&str>,
    display_communicates: bool,
) -> Result<i32, &'static str> {
    let connection = &mut establish_connection();
    let mut task_id = 0;

    match connection.transaction::<_, Error, _>(|connection| {
        let task = match create_task(connection,project_id, task_name_, planned_time) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            }
        };

        task_id = task.task_id;

        Ok("")
    }) 
    {
        Ok(_) => {
            if display_communicates {
                println!("Transaction committed successfully:");
                println!("\nSaved task \"{}\"", task_name_);
            }
            Ok(task_id)
        }
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _x)) => {

            return Err("Task of such name is already in this project, choose another name!");
        },
        Err(_x) => {
            return Err("Database error occurred");
        }
    }
}

