

use diesel::prelude::*;
use workflow::*;
use diesel::prelude::QueryDsl;

use diesel::result::Error;
use workflow::models::*;

pub fn get_recent_log(_task_id:i32, order: bool)-> Result<Option<Log>, &'static str>{
    use workflow::schema::log::*;
    use self::schema::log::dsl::*;

    let connection:&mut PgConnection = &mut establish_connection();

    let result:Result<Vec<Log>,Error> =if order{ log
        .filter(task_id.eq(_task_id))
        .order(date.desc())
        .limit(1)
        .load::<Log>(connection)}
        else{
            log
            .filter(task_id.eq(_task_id))
            .order(date.asc())
            .limit(1)
            .load::<Log>(connection)
        };

    

    match result {
        Ok(x) if x.len()>0 => {let a=x[0].clone();
            Ok(Some(a))},
        Ok(_)=>Ok(None),
        Err(_) => Err("An error occured while fetching logs"),
    }
}

pub fn get_logs(args: &[String]) -> Result<Vec<Log>, Error> {
    use workflow::schema::log::dsl::*;
    

    let connection: &mut PgConnection = &mut establish_connection();

    if args.len()==0{

        let result = log.load::<Log>(connection)?;

        Ok(result)
    }else{
        let mut logs=vec![];
        for x in args{
            let task_id_=x.parse::<i32>().unwrap_or(-1);
            let result = log.filter(task_id.eq(task_id_)).load::<Log>(connection)?;
            logs.extend(result);
            
        }
        Ok(logs)
    }

    

}

pub fn add_log(_task_id: i32, _log_type: String, display_communicates: bool)  {
    let connection = &mut establish_connection();

    let logs = create_log(connection, _task_id, _log_type.clone());

    match logs{
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _))=>println!("No such task, create it!"),
        Err(_)=>println!("Database error while creating the task"),
        Ok(log)=>{
            if display_communicates {
                println!(
                    "\nSaved log {} for task {} with id {}",
                    _log_type, _task_id, log.log_id
                );
            }

        }
    }
}

