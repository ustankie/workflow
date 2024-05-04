use std::env;
use std::process;
use chrono::Local;
use db_operations::get_stats;

pub mod db_operations;
pub mod tasks;


#[derive(Debug, PartialEq)]
enum Commands{
    AddTask,
    AddApp,
    AddAppToTask,
    AllTasks,
    Begin,
    Pause,
    Resume,
    End,
    Logs,
    Man,
    Stats,
    NoSuchCommand
}


impl From<String> for Commands{
    fn from(input: String) -> Self { 
        match input.trim() {
            "app"=>Commands::AddApp,
            "newtask"=>Commands::AddTask,
            "tasks"=>Commands::AllTasks,
            "logs"=>Commands::Logs,
            "app-task"=>Commands::AddAppToTask,
            "stats"=>Commands::Stats,
            "begin"=> Commands::Begin,
            "end"=> Commands::End,
            "pause"=>Commands::Pause,
            "resume"=>Commands::Resume,
            "man"=>Commands::Man,
            _=>Commands::NoSuchCommand
        }
    }
}

impl ToString for Commands{
    fn to_string(&self) -> String {
        match self {
            Self::Begin=> String::from("B"),
            Self::End => String::from("E"),
            Self::Pause=>String::from("P"),
            Self::Resume=>String::from("R"),
            _=>String::from("")
        }
    }
}


fn main(){
    let args:Vec<String>=env::args().collect();

    if args.len() < 2 {
        println!("Issue a command!");
        return;
    }

    let command=Commands::from(args[1].clone());
    let _res=match command{
        Commands::AddApp =>add_app(&args[2..],true),
        Commands::AddTask=>tasks::add_task(args),
        Commands::AllTasks=>display_tasks(),
        Commands::Begin | Commands::End | Commands::Pause | Commands::Resume=>add_log(args,command),
        Commands::Logs=>display_logs(&args[2..]),
        Commands::Man=>display_man(),
        Commands::Stats=>display_stats(&args[2..]),
        _ =>()
    };

}

fn display_man(){
    print!(
"-----------------------------------------------------
Welcome to workflow!
-----------------------------------------------------
COMMANDS:
newtask NAME- creates new task
    OPTIONS:
        -t TIME - sets time user plans to spend on task. It should have format DAYS:HOURS:MINUTES
        -a APPLIST - sets apps used during completing the task;
tasks - displays all tasks;

app APPLIST - adds specified apps to db;

begin ID/NAME - begins the task given by id or name; a task that has ended cannot be started again;

end ID/NAME - ends the task given by id or name; cannot end a task that was not started;

pause ID/NAME - pauses the task given by id or name, a task that has already been paused, has ended 
                or has not begun cannot be paused;

resume ID/NAME - resumes the task given by id or name, a task that has not been paused, has ended 
                 or has not begun cannot be resumed;     

logs - displays the history of all tasks
    OPTIONS
        -t, -tasks TASKIDLIST - displays only the history of the tasks specified in args by id;

man - displays app's manual;

");
}

fn display_stats(args:&[String]){
    let a=if args.len()>0 && (args[0]=="-t" || args[0]=="-tasks"){
        db_operations::get_logs(&args[1..])
    }else if args.len()==0{
        db_operations::get_logs(&[])
    }else{
        println!("No such option!");
        return;
    };

    get_stats(args);
}

fn add_app(args:&[String], display_communicates:bool){
    if args.len()<1{
        eprintln!("Too few args");
        process::exit(-1);
    }


    for x in args{
        match db_operations::find_app(x){
            Ok(None) =>{
                    match db_operations::add_app(&(x.to_lowercase()),display_communicates){
                        Err(x)=> println!("{}",x),
                        _=>()
                    }
                },
            Ok(Some(_a))=>{if display_communicates{ println!("App {} already in db",x);}},
            Err(_)=>{println!("An error occured while fetching app {}", x);}
        };
            
    }


}


fn display_tasks(){
    let a=db_operations::get_tasks();
    if let Ok(x)=a{
        println!(
            "-----------------------------------------------------"
        );
        println!(
            "{: <10} | {: <10} | {: <10} | {: <10}",
            "task_id", "task_name", "user", "planned_time"
        );
        println!(
            "-----------------------------------------------------"
        );
        for row in x{
            println!("{: <10} | {: <10} | {: <10} | {: <10}", row.task_id,row.task_name,row.username,row.planned_time.unwrap_or("null".to_string()));
        }
    }

}

fn display_logs(args:&[String]){
    let a=if args.len()>0 && (args[0]=="-t" || args[0]=="-tasks"){
        db_operations::get_logs(&args[1..])
    }else if args.len()==0{
        db_operations::get_logs(&[])
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





fn add_log(args:Vec<String>, log_type:Commands){
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
            match db_operations::find_task(&args[2]){
                Ok(Some(task))=>add_log_by_id(log_type, &(task.task_id)),
                Ok(None)=>println!("No such task!"),
                Err(x)=>println!("{}",x),
            }
            }
    }
}

fn add_log_by_id(log_type:Commands,num:&i32){
    let recent_log=db_operations::get_recent_log(*num,true);


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
            let first_log=db_operations::get_recent_log(*num,false);
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
    
    db_operations::add_log(*num, log_type.to_string(), true);
}
