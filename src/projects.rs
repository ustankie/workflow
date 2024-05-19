use std::process;
use chrono::NaiveDateTime;
use regex::Regex;
use workflow::models::Task;
use crate::stats;


use crate::db_operations;

pub fn add_project(args:Vec<String>){
    if args.len()<3{
        eprintln!("Too few args");
        process::exit(-1);
    }
    let time_regex=Regex::new(r"^\d+:\d+:\d+$").unwrap();
    let arg_regex=Regex::new(r"-.*").unwrap();

    if arg_regex.is_match(&args[2]){
        println!("First argument must be project name!");
        process::exit(-1);
    }
    let project_name=&args[2];
    println!("Project name: {}",project_name);

    let mut time_planned:Option<&str>=None;
    let mut project_apps:Option<&[String]>=None;

    let mut i=3;
    while i<args.len(){
        match &args[i][..]{
            "-t" => {
                    i+=1;
                    if time_regex.is_match(&args[i]){
                        time_planned=Some(&args[i]);
                    }else{
                        println!("Wrong time format!");
                    }
                    i+=1;
                },
            "-a"=>{
                    i+=1; 
                    if let None=project_apps{   
 
                        let j=i;
                        while i<args.len() && !(arg_regex.is_match(&args[i])){
                            i+=1;
                        }
                        project_apps=Option::from(&args[j..i]);
                    }
                }

            _=>{println!("Unknown argument '{}': try again",&args[i]); 
                }
        }
        
    }

    if let Err(x)=db_operations::projects::add_project(project_name, time_planned, project_apps, true){
        println!("{}",x);
    }


}
pub fn display_projects(){
    let _a=db_operations::projects::get_projects();
    let stats: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &str> = db_operations::stats::get_stats(&[]);
    stats::display_content(stats, stats::PrintMode::Project);
}
