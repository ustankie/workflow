use std::process;
use crate::db_operations;

pub fn add_app(args:&[String], display_communicates:bool){
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
