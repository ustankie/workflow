use self::models::App;
use self::schema::apps::dsl::*;
use diesel::prelude::*;
use workflow::*;

use diesel::result::Error;
use std::process;
use workflow::models::*;

pub fn add_app(_app_name: &str, display_communicates: bool) -> Result<i32, &'static str> {
    let connection = &mut establish_connection();

    let res_app = create_app(connection, _app_name);

    match res_app {
        Ok(app) => {
            if display_communicates {
                println!("\nSaved app {} with id {}", _app_name, app.app_id);
            }
            Ok(app.app_id)
        }
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            Err("App of such name is already in the database, choose another name!")
        }
        Err(_) => Err("Database error occurred"),
    }
}

pub fn find_app(_app_name: &str) -> Result<Option<App>, &'static str> {
    use self::schema::apps::dsl::apps;

    let connection = &mut establish_connection();
    find_app_body(_app_name, connection)
}

fn find_app_body(
    app_name_: &str,
    connection: &mut PgConnection,
) -> Result<Option<App>, &'static str> {
    let app = apps
        .filter(app_name.eq(app_name_.to_lowercase()))
        .select(App::as_select())
        .first(connection);

    match app {
        Ok(x) => Ok(Some(x)),
        Err(Error::NotFound) => Ok(None),
        Err(x) => {
            println!("{}", x);
            Err("An error occured while fetching app {}")
        }
    }
}

pub fn get_tasks() -> Result<Vec<Task>, Error> {
    use self::schema::tasks::dsl::tasks;

    let connection = &mut establish_connection();
    let tasks_list = tasks.load::<Task>(connection)?;

    Ok(tasks_list)
}

pub fn get_recent_log(_task_id:i32)-> Result<Option<Log>, &'static str>{
    use workflow::schema::log::*;
    use self::schema::log::dsl::*;

    let connection:&mut PgConnection = &mut establish_connection();

    let result = log
        .filter(task_id.eq(_task_id))
        .select(Log::as_select())
        .order(date.desc())
        .first::<Log>(connection);

    println!("{:?}",result);

    match result {
        Ok(x) => Ok(Some(x)),
        Err(_) => Err("An error occured while fetching logs"),
    }
}

pub fn get_logs() -> Result<Vec<Log>, Error> {
    use self::schema::log::dsl::log;

    let connection: &mut PgConnection = &mut establish_connection();

    let result = log.load::<Log>(connection)?;

    Ok(result)
}

pub fn add_log(_task_id: i32, _log_type: String, display_communicates: bool) -> i32 {
    use self::schema::log::dsl::*;

    let connection = &mut establish_connection();

    let logs = create_log(connection, _task_id, _log_type.clone());

    if display_communicates {
        println!(
            "\nSaved log {} for task {} with id {}",
            _log_type, _task_id, logs.log_id
        );
    }
    logs.log_id
}


fn addApp(
    args: &[String],
    display_communicates: bool,
    connection: &mut PgConnection,
) -> Result<Vec<i32>, &'static str> {
    if args.len() < 1 {
        eprintln!("Too few args");
        process::exit(-1);
    }

    let mut ids: Vec<i32> = vec![];

    for x in args {
        println!("{}", x);
        match find_app_body(x, connection) {
            Ok(None) => match add_app(&(x.to_lowercase()), display_communicates) {
                Err(x) => return Err(x),
                Ok(x) => {
                    ids.push(x);
                }
            },
            Ok(Some(a)) => {
                if display_communicates {
                    println!("App {} already in db", x);
                }
                ids.push(a.app_id)
            }
            Err(x) => return Err(x),
        };
    }
    Ok(ids)
}


pub fn add_task(
    task_name: &str,
    planned_time: Option<&str>,
    task_apps: Option<&[String]>,
    display_communicates: bool,
) -> Result<i32, &'static str> {
    let connection = &mut establish_connection();
    let mut task_id = 0;

    match connection.transaction::<_, Error, _>(|connection| {
        let task = match create_task(connection, task_name, planned_time) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            }
        };

        task_id = task.task_id;

        if let Some(x) = task_apps {
            println!("yes!");
            println!("{:?}", x);
            let app_ids = match addApp(x, false, connection) {
                Ok(x) => x,
                Err(x) => {
                    println!("{}", x);
                    vec![]
                }
            };

            println!("{:?}", app_ids);
            for _app_id in app_ids {
                create_app_detail(connection, task_id, _app_id);
            }
        }
        Ok("")
    }) 
    {
        Ok(_) => {
            if display_communicates {
                println!("Transaction committed successfully:");
                println!("\nSaved task \"{}\"", task_name);
            }
            Ok(task_id)
        }
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            return Err("Task of such name is already in the database, choose another name!");
        },
        Err(_) => {
            return Err("Database error occurred");
        }
    }
}

