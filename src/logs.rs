use chrono::Local;
use crate::db_operations;
use crate::Commands;
use std::process;

pub fn display_logs(args:&[String]){
    let a=if args.len()>0 && (args[0]=="-t" || args[0]=="-tasks"){
        db_operations::logs::get_logs(&args[1..])
    }else if args.len()==0{
        db_operations::logs::get_logs(&[])
    }else{
        println!("No such option!");
        return;
    };
    
    if let Ok(x)=a{
        println!(
            "-------------------------------------------------------------------------------"
        );
        println!(
            "{: <10} | {: <10} | {: <10} | {: <10}",
            "log_id", "task_id", "log_type", "date"
        );
        println!(
            "-------------------------------------------------------------------------------"
        );
        for row in x{
            println!("{: <10} | {: <10} | {: <10} | {: <10}", row.log_id,row.task_id,row.log_type,row.date);
        }
    }

}





pub fn add_log(args:Vec<String>, log_type:Commands){
    if args.len()<3{
        eprintln!("Too few args");
        process::exit(-1);
    }

    if args.len()>3{
        eprintln!("Too many args");
        process::exit(-1);
    }
    let task_id=&args[2].parse::<i32>();


    match task_id{
        Ok(num)=>add_log_by_id(log_type, num),
        Err(_)=>{
            match db_operations::tasks::find_task(&args[2]){
                Ok(Some(task))=>add_log_by_id(log_type, &(task.task_id)),
                Ok(None)=>println!("No such task!"),
                Err(x)=>println!("{}",x),
            }
            }
    }
}

pub fn add_log_by_id(log_type:Commands,num:&i32){
    let recent_log=db_operations::logs::get_recent_log(*num,true);


    match recent_log{
        Err(x)=>{
                println!("{}",x);
                return;
            },
        Ok(None)=>{
                if log_type!=(Commands::Begin){
                    println!("First begin the task, then perform other operations!");
                    return;
                }
            }
        Ok(Some(x)) if x.log_type==Commands::End.to_string()=>{
            println!("Task has been ended!");
            return;},
        Ok(Some(x)) if log_type==Commands::End=>{
            let duration=Local::now().naive_local().signed_duration_since(x.date);
            if x.log_type==Commands::Pause.to_string(){
                print!("Ending pause that lasted: ");
            }else{
                print!("You've been working since last pause ");
            }
            println!( "{} days, {} hours, {} minutes", 
            duration.num_days(), duration.num_hours(), duration.num_minutes()); 
            let first_log=db_operations::logs::get_recent_log(*num,false);
            match first_log{
                Err(a) =>{
                        println!("{}",a);
                        
                    },
                Ok(None)=>println!("Error finding first log!"),
                Ok(Some(a))=>{
                    let duration=Local::now().naive_local().signed_duration_since(a.date);
                    println!("Total time spent on task:  {} days, {} hours, {} minutes",
                    duration.num_days(), duration.num_hours(), duration.num_minutes());
                }}
        }
        Ok(Some(x)) if x.log_type==Commands::Pause.to_string() && log_type==(Commands::Pause)=>{println!("Task has already been paused");return;},
        Ok(Some(x)) if x.log_type!=Commands::Pause.to_string() && log_type==(Commands::Resume) =>{println!("Pause task before you resume it");return;},
        Ok(Some(_)) if log_type==Commands::Begin => {println!("Task has already been started"); return;},
        Ok(Some(x)) if x.log_type==Commands::Pause.to_string()=>{
            let duration=Local::now().naive_local().signed_duration_since(x.date);
            println!("You've been working {} days, {} hours, {} minutes", duration.num_days(), duration.num_hours(), duration.num_minutes());}
        Ok(Some(x)) if x.log_type==Commands::Resume.to_string()=>{
            let duration=Local::now().naive_local().signed_duration_since(x.date);
            println!("Your pause was {} days, {} hours, {} minutes long",  duration.num_days(), duration.num_hours(), duration.num_minutes());}
        _=>()
    }
    
    db_operations::logs::add_log(*num, log_type.to_string(), true);
}
