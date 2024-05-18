use std::env;
use std::process;


pub mod db_operations;
pub mod projects;
pub mod tasks;
pub mod stats;
pub mod logs;



#[derive(Debug, PartialEq)]
pub enum Commands{
    AddProject,
    AllProjects,
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
    Day,
    NoSuchCommand
}


impl From<String> for Commands{
    fn from(input: String) -> Self { 
        match input.trim() {
            "project"=>Commands::AddProject,
            "projects"=>Commands::AllProjects,
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
            "day"=>Commands::Day,
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
    match command{
        Commands::AddApp =>add_app(&args[2..],true),
        Commands::AddTask=>tasks::add_task(args),
        Commands::AllTasks=>tasks::display_tasks(),
        Commands::Begin | Commands::End | Commands::Pause | Commands::Resume=>logs::add_log(args,command),
        Commands::Logs=>logs::display_logs(&args[2..]),
        Commands::Man=>display_man(),
        Commands::Stats=>stats::display_stats(&args[2..]),
        Commands::AddProject=>projects::add_project(args),
        Commands::AllProjects=>projects::display_projects(),
        Commands::Day=>stats::display_day_stats(&args[2..]),
        Commands::NoSuchCommand=> {println!("Wrong command!");},
        _ =>()
    };

}

fn display_man(){
    print!(
"-----------------------------------------------------
Welcome to workflow!
-----------------------------------------------------
COMMANDS:
- newtask NAME- creates new task
    OPTIONS:
        -t TIME - sets time user plans to spend on task. It should have format DAYS:HOURS:MINUTES
        -a APPLIST - sets apps used during completing the task;
- tasks - displays all tasks;

- app APPLIST - adds specified apps to db;

- begin ID/NAME - begins the task given by id or name; a task that has ended cannot be started again;

- end ID/NAME - ends the task given by id or name; cannot end a task that was not started;

- pause ID/NAME - pauses the task given by id or name, a task that has already been paused, has ended 
                or has not begun cannot be paused;

- resume ID/NAME - resumes the task given by id or name, a task that has not been paused, has ended 
                 or has not begun cannot be resumed;     

- logs - displays the history of all tasks
    OPTIONS
        -t, -tasks TASKIDLIST - displays only the history of the tasks specified in args by id;

- man - displays app's manual;

");
}


fn add_app(args:&[String], display_communicates:bool){
    if args.len()<1{
        eprintln!("Too few args");
        process::exit(-1);
    }

    for x in args{
        match db_operations::apps::find_app(x){
            Ok(None) =>{
                    match db_operations::apps::add_app(&(x.to_lowercase()),display_communicates){
                        Err(x)=> println!("{}",x),
                        _=>()
                    }
                },
            Ok(Some(_a))=>{if display_communicates{ println!("App {} already in db",x);}},
            Err(_)=>{println!("An error occured while fetching app {}", x);}
        };
            
    }


}



