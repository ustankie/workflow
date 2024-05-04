use std::env;
use regex::Regex;
use std::process;

use crate::db_operations::get_recent_log;
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
    NoSuchCommand
}


impl From<String> for Commands{
    fn from(input: String) -> Self { 
        match input.trim() {
            "newtask"=>Commands::AddTask,
            "tasks"=>Commands::AllTasks,
            "app"=>Commands::AddApp,
            "app-task"=>Commands::AddAppToTask,
            "begin"=> Commands::Begin,
            "end"=> Commands::End,
            "logs"=>Commands::Logs,
            "pause"=>Commands::Pause,
            "resume"=>Commands::Resume,
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

    for (i, x) in args.iter().enumerate() {
        if i>0{
            println!("{:?}",(Commands::from(String::from(x))));
        }
    }

    println!("{}",&args[1]);
    let command=Commands::from(args[1].clone());
    let res=match command{
        Commands::AddApp =>addApp(&args[2..],true),
        Commands::AddTask=>tasks::addTask(args),
        Commands::AllTasks=>displayTasks(),
        Commands::Begin | Commands::End | Commands::Pause | Commands::Resume=>add_log(args,command),
        Commands::Logs=>display_logs(),
        _ =>()
    };

    println!("{:?}",db_operations::get_recent_log(1).unwrap_or_default());


}

fn addApp(args:&[String], display_communicates:bool){
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
            Ok(Some(a))=>{if display_communicates{ println!("App {} already in db",x);}},
            Err(_)=>{println!("An error occured while fetching app {}", x);}
        };
            
    }


}


fn displayTasks(){
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

fn display_logs(){
    let a=db_operations::get_logs();
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
        Ok(num)=>{
            let recent_log=db_operations::get_recent_log(*num);


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
                Ok(Some(x)) if x.log_type==Commands::End.to_string()=>println!("Task has been ended!"),
                Ok(Some(x)) if x.log_type==Commands::Pause.to_string() && log_type==(Commands::Pause)=>println!("Task already paused"),
                Ok(Some(x)) if x.log_type==Commands::Resume.to_string() && log_type==(Commands::Resume)=>println!("Task already resumed"),
                Ok(Some(x)) if log_type==(Commands::Begin)=> println!("Task has been already started"),
                _=>()

            }
            
            db_operations::add_log(*num, log_type.to_string(), true);},
        Err(_)=>{println!("Error adding log");}
    }
}

pub fn addTask(args:Vec<String>){
    if args.len()<3{
        eprintln!("Too few args");
        process::exit(-1);
    }
    let time_regex=Regex::new(r"^\d+:\d+:\d+$").unwrap();
    let arg_regex=Regex::new(r"-.*").unwrap();

    if arg_regex.is_match(&args[2]){
        println!("First argument must be task name!");
        process::exit(-1);
    }
    let task_name=&args[2];
    println!("Task name: {}",task_name);

    let mut time_planned:Option<&str>=None;
    let mut task_apps:Option<&[String]>=None;

    let mut i=3;
    while i<args.len(){
        println!("{}, {}",arg_regex.is_match(&args[i]),&args[i]);
        match &args[i][..]{
            "-t" => {
                    i+=1;
                    if time_regex.is_match(&args[i]){
                        time_planned=Some(&args[i]);
                        println!("{:?}",time_planned);
                    }else{
                        println!("Wrong time format!");
                    }
                    i+=1;
                },
            "-a"=>{
                    i+=1; 
                    if let None=task_apps{   
                        println!("a");
                        
                        
                        let j=i;
                        while i<args.len() && !(arg_regex.is_match(&args[i])){
                            i+=1;
                        }
                        task_apps=Option::from(&args[j..i]);
                    }
                }

            _=>{println!("Unknown argument '{}': try again",&args[i]); 
                }
        }
        
    }

    if let Err(x)=db_operations::add_task(task_name, time_planned, task_apps, true){
        println!("{}",x);
    }


}
