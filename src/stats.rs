use std::cmp::max;
use chrono::Duration;
use crate::db_operations;
use crate::Commands;

pub fn display_stats(args:&[String]){
    let _aa=if args.len()>0 && (args[0]=="-t" || args[0]=="-tasks"){
        db_operations::logs::get_logs(&args[1..])
    }else if args.len()==0{
        db_operations::logs::get_logs(&[])
    }else{
        println!("No such option!");
        return;
    };

    let stats=db_operations::stats::get_stats(args);

    match stats{
        Err(x)=>println!("{}",x),
        Ok(result)=>{
            println!(
                "--------------------------------------------------------------------------------------------------------------------------"
            );
            println!(
                "{: <10} | {: <10} | {: <10} | {: <13} | {: <10} | {: <10} | {: <10} | {: <10}| {: <10}",
                "task_id", "task_name", "user", "planned time","total time","total worked", "pause num", "longest pause", "longest work"
            );
            println!(
                "--------------------------------------------------------------------------------------------------------------------------"
            );
            let mut i=1;
            let _beginned=<Option<String> as Clone>::clone(&result[i].2).unwrap_or_default()==Commands::Begin.to_string();
            let mut pause_num=0;
            let mut longest_pause=Duration::seconds(0);
            let mut longest_work=Duration::seconds(0);
            let mut begin=result[i].3.unwrap_or_default();
            let mut total_time=Duration::seconds(0);
            let mut total_worked=Duration::seconds(0);
            while i<result.len(){
                pause_num=0;
                longest_pause=Duration::seconds(0);
                longest_work=Duration::seconds(0);
                begin=result[i-1].3.unwrap_or_default();
                total_time=Duration::seconds(0);
                total_worked=Duration::seconds(0);
                while i<result.len() && &result[i].0.task_id==&result[i-1].0.task_id{
                    if <Option<String> as Clone>::clone(&result[i].2).unwrap_or("".to_string())==Commands::Pause.to_string() 
                    || (<Option<String> as Clone>::clone(&result[i].2).unwrap_or_default()==Commands::End.to_string()
                    && <Option<String> as Clone>::clone(&result[i-1].2).unwrap_or("".to_string())!=Commands::Pause.to_string()){
                        let slot=result[i].3.unwrap_or_default().signed_duration_since(result[i-1].3.unwrap_or_default());
                        longest_work=max(longest_work, slot);
                        total_worked+=slot;
                        if <Option<String> as Clone>::clone(&result[i].2).unwrap_or("".to_string())==Commands::Pause.to_string(){
                            pause_num+=1;
                        }

                    }else if <Option<String> as Clone>::clone(&result[i].2).unwrap_or("".to_string())==Commands::Resume.to_string() 
                    || (<Option<String> as Clone>::clone(&result[i].2).unwrap_or_default()==Commands::End.to_string()
                        && <Option<String> as Clone>::clone(&result[i-1].2).unwrap_or("".to_string())==Commands::Pause.to_string()){

                            
                            let slot=result[i].3.unwrap_or_default().signed_duration_since(result[i-1].3.unwrap_or_default());
                            longest_pause=max(longest_pause, slot); 
                            

                    }

                    // if <Option<String> as Clone>::clone(&result[i].2).unwrap_or("".to_string())==Commands::End.to_string(){
                    total_time=result[i].3.unwrap_or_default().signed_duration_since(begin);
                    // }
                    i+=1;
                }
                

                println!("{: <10} | {: <10} | {: <10} | {: <14}| {: <11}| {: <13}| {: <11}| {: <13}| {: <10}", 
                result[i-1].0.task_id,result[i-1].0.task_name,result[i-1].0.username,result[i-1].clone().0.planned_time.unwrap_or("null".to_string()),
                format!("{}:{}:{}", total_time.num_days(), total_time.num_hours()-24*total_time.num_days(), total_time.num_minutes()-total_time.num_hours()*60),
                format!("{}:{}:{}", total_worked.num_days(), total_worked.num_hours()-24*total_worked.num_days(), total_worked.num_minutes()-total_worked.num_hours()*60),
                pause_num,
                format!("{}:{}:{}", longest_pause.num_days(), longest_pause.num_hours()-24*longest_pause.num_days(), longest_pause.num_minutes()-60*longest_pause.num_hours()),
                format!("{}:{}:{}", longest_work.num_days(), longest_work.num_hours()-24*longest_work.num_days(), longest_work.num_minutes()-60*longest_work.num_hours()));
                i+=1;
            }
            println!("{: <10} | {: <10} | {: <10} | {: <14}| {: <11}| {: <13}| {: <11}| {: <13}| {: <10}", 
            result[i-1].0.task_id,result[i-1].0.task_name,result[i-1].0.username,result[i-1].clone().0.planned_time.unwrap_or("null".to_string()),
            format!("{}:{}:{}", total_time.num_days(), total_time.num_hours()-24*total_time.num_days(), total_time.num_minutes()-total_time.num_hours()*60),
            format!("{}:{}:{}", total_worked.num_days(), total_worked.num_hours()-24*total_worked.num_days(), total_worked.num_minutes()-total_worked.num_hours()*60),
            pause_num,
            format!("{}:{}:{}", longest_pause.num_days(), longest_pause.num_hours()-24*longest_pause.num_days(), longest_pause.num_minutes()-60*longest_pause.num_hours()),
            format!("{}:{}:{}", longest_work.num_days(), longest_work.num_hours()-24*longest_work.num_days(), longest_work.num_minutes()-60*longest_work.num_hours()));
        }


    }
}
