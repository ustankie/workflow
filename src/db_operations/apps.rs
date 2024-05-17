use self::models::App;
use self::schema::apps::dsl::*;
use diesel::prelude::*;
use workflow::*;
use diesel::prelude::QueryDsl;

use diesel::result::Error;
use std::process;

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

pub fn add_multiple_apps(
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


pub fn find_app(_app_name: &str) -> Result<Option<App>, &'static str> {
    

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



