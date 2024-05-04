use std::process;
use regex::Regex;



use crate::db_operations;

pub fn add_task(args:Vec<String>){
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




