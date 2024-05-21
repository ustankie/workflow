use crate::stats;
use crate::Commands;
use chrono::NaiveDateTime;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use regex::Regex;
use std::collections::HashMap;
use std::process;
use workflow::models::Task;

use crate::db_operations;

pub fn add_project(args: Vec<String>) {
    if args.len() < 3 {
        eprintln!("Too few args");
        process::exit(-1);
    }
    let time_regex = Regex::new(r"^\d+:\d+:\d+$").unwrap();
    let arg_regex = Regex::new(r"-.*").unwrap();

    if arg_regex.is_match(&args[2]) {
        println!("First argument must be project name!");
        process::exit(-1);
    }
    let project_name = &args[2];
    println!("Project name: {}", project_name);

    let mut time_planned: Option<&str> = None;
    let mut project_apps: Option<&[String]> = None;

    let mut i = 3;
    while i < args.len() {
        match &args[i][..] {
            "-t" => {
                i += 1;
                if time_regex.is_match(&args[i]) {
                    time_planned = Some(&args[i]);
                } else {
                    println!("Wrong time format!");
                }
                i += 1;
            }
            "-a" => {
                i += 1;
                if let None = project_apps {
                    let j = i;
                    while i < args.len() && !(arg_regex.is_match(&args[i])) {
                        i += 1;
                    }
                    project_apps = Option::from(&args[j..i]);
                }
            }

            _ => {
                println!("Unknown argument '{}': try again", &args[i]);
            }
        }
    }

    if let Err(x) =
        db_operations::projects::add_project(project_name, time_planned, project_apps, true)
    {
        println!("{}", x);
    }
}
pub fn display_projects() {
    let _a = db_operations::projects::get_projects();
    let stats: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &str> =
        db_operations::stats::get_stats(&[]);
    stats::display_content(stats, stats::PrintMode::Project, None);
}

pub fn display_project_apps() {
    let project_apps = db_operations::projects::get_apps_in_projects();

    let project_apps = project_apps.ok().unwrap_or(vec![]);
    if project_apps.len() == 0 {
        println!("No apps in the projects!");

        return;
    }

    let mut prev_project_id = 0;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("project_id")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
            Cell::new("project_name")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
            Cell::new("app_id")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
            Cell::new("app_name")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
        ]);
    let mut app_count = 0;
    let mut prev_color = Color::DarkGreen;
    let mut cur_color = Color::DarkCyan;
    for (project, app_name, app_id) in project_apps {
        if project.project_id != prev_project_id {
            if prev_project_id != 0 {
                table.add_row(vec![
                    Cell::new("total")
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                    Cell::new("total")
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                    Cell::new("total")
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                    Cell::new(app_count)
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                ]);
            }
            (cur_color, prev_color) = (prev_color, cur_color);

            table.add_row(vec![
                Cell::new(project.project_id)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(project.project_name)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(
                    app_id
                        .map(|id| id.to_string())
                        .unwrap_or("null".to_string()),
                )
                .set_alignment(CellAlignment::Center)
                .fg(cur_color),
                Cell::new(app_name.clone().unwrap_or("null".to_string()))
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
            ]);
            app_count = 0;
        } else {
            table.add_row(vec![
                Cell::new(project.project_id)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(project.project_name)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(
                    app_id
                        .map(|id| id.to_string())
                        .unwrap_or("null".to_string()),
                )
                .fg(cur_color)
                .set_alignment(CellAlignment::Center),
                Cell::new(app_name.clone().unwrap_or("null".to_string()))
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
            ]);
        }

        prev_project_id = project.project_id;
        if app_name.is_some() {
            app_count += 1;
        }
    }

    table.add_row(vec![
        Cell::new("total")
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
        Cell::new("total")
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
        Cell::new("total")
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
        Cell::new(app_count)
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
    ]);
    println!("{}", table);
}

pub fn display_project_tasks(args: &[String]) {
    let project_regex = Regex::new(r"\d+").unwrap();
    let arg_regex: Regex = Regex::new(r"-.*").unwrap();
    let mut i = 0;
    let mut seeked_project_id: Option<Vec<i32>> = None;
    let mut found_project = false;
    let mut command: HashMap<String, bool> = HashMap::new();
    while i < args.len() {
        match &args[i][..] {
            "-pr" => {
                i += 1;
                let mut project_ids = vec![];
                if found_project {
                    eprintln!("Error: Multiple usage of the same options");
                    return;
                }
                while i < args.len() && !arg_regex.is_match(&args[i]) {
                    if !project_regex.is_match(&args[i]) {
                        println!("Project id should be integer!");
                        return;
                    }
                    project_ids.push(args[i].parse::<i32>().expect("Couldn't parse project_id"));

                    i += 1;
                }

                seeked_project_id = Some(project_ids);

                found_project = true;
            }
            "-e" => {
                if command.contains_key(&Commands::End.to_string())
                    && !command.get(&Commands::End.to_string()).unwrap()
                {
                    println!("Cannot apply contradictory filters!");
                    process::exit(-1);
                }
                command.insert(Commands::End.to_string(), true);
                i += 1;
            }
            "-ne" => {
                if command.contains_key(&Commands::End.to_string())
                    && !!command.get(&Commands::End.to_string()).unwrap()
                {
                    println!("Cannot apply contradictory filters!");
                    process::exit(-1);
                }
                command.insert(Commands::End.to_string(), false);
                i += 1;
            }
            "-b" => {
                if command.contains_key(&Commands::Begin.to_string())
                    && !command.get(&Commands::Begin.to_string()).unwrap()
                {
                    println!("Cannot apply contradictory filters!");
                    process::exit(-1);
                }
                command.insert(Commands::Begin.to_string(), true);
                i += 1;
            }
            "-nb" => {
                if command.contains_key(&Commands::Begin.to_string())
                    && !!command.get(&Commands::Begin.to_string()).unwrap()
                {
                    println!("Cannot apply contradictory filters!");
                    process::exit(-1);
                }
                command.insert(Commands::Begin.to_string(), false);
                i += 1;
            }
            _ => {
                println!("Unknown argument '{}': try again", &args[i]);
                process::exit(-1);
            }
        }
    }

    let project_tasks =
        db_operations::projects::get_tasks_in_projects(seeked_project_id.clone(), command);

    let project_tasks = project_tasks.ok().unwrap_or(vec![]);
    if project_tasks.len() == 0 {
        if seeked_project_id.is_none() || seeked_project_id.clone().unwrap().len() > 0 {
            println!("No tasks in these projects!");
        } else {
            println!("No tasks in this project!");
        }
        return;
    }

    let mut prev_project_id = 0;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("project_id")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
            Cell::new("project_name")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
            Cell::new("task_id")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
            Cell::new("task_name")
                .set_alignment(CellAlignment::Center)
                .fg(Color::Cyan),
        ]);
    let mut task_count = 0;
    let mut prev_color = Color::DarkGreen;
    let mut cur_color = Color::DarkCyan;
    for (project, task_name, _planned_time, task_id) in project_tasks {
        if project.project_id != prev_project_id {
            if prev_project_id != 0 {
                table.add_row(vec![
                    Cell::new("total")
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                    Cell::new("total")
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                    Cell::new("total")
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                    Cell::new(task_count)
                        .set_alignment(CellAlignment::Center)
                        .fg(cur_color),
                ]);
            }
            (cur_color, prev_color) = (prev_color, cur_color);

            table.add_row(vec![
                Cell::new(project.project_id)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(project.project_name)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(
                    task_id
                        .map(|id| id.to_string())
                        .unwrap_or("null".to_string()),
                )
                .fg(cur_color)
                .set_alignment(CellAlignment::Center),
                Cell::new(task_name.clone().unwrap_or("null".to_string()))
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
            ]);
            task_count = 0;
        } else {
            table.add_row(vec![
                Cell::new(project.project_id)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(project.project_name)
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
                Cell::new(
                    task_id
                        .map(|id| id.to_string())
                        .unwrap_or("null".to_string()),
                )
                .fg(cur_color)
                .set_alignment(CellAlignment::Center),
                Cell::new(task_name.clone().unwrap_or("null".to_string()))
                    .set_alignment(CellAlignment::Center)
                    .fg(cur_color),
            ]);
        }

        prev_project_id = project.project_id;
        if task_name.is_some() {
            task_count += 1;
        }
    }

    table.add_row(vec![
        Cell::new("total")
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
        Cell::new("total")
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
        Cell::new("total")
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
        Cell::new(task_count)
            .set_alignment(CellAlignment::Center)
            .fg(cur_color),
    ]);
    println!("{}", table);
}
