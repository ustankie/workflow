use chrono::Duration;
use std::cmp::max;
use std::process;
use workflow::models::{Project, Task};

use crate::db_operations;
use crate::db_operations::projects::{get_date_projects, get_projects};
use crate::Commands;
use chrono::prelude::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use std::collections::{HashMap, HashSet};

pub enum PrintMode {
    All,
    Appearing,
    Project,
    ConcreteTasks,
}

#[derive(Debug, Clone)]
pub struct ProjectStats {
    project_id: i32,
    project_name: String,
    username: String,
    planned_time: Option<String>,
    total_time: Duration,
    total_worked: Duration,
    pause_num: i32,
    longest_pause: Duration,
    longest_work: Duration,
    total_tasks: i32,
    completed_tasks: i32,
}

#[derive(Debug, Clone)]
pub struct TaskStats {
    task_id: i32,
    project_id: i32,
    task_name: String,
    username: String,
    planned_time: Option<String>,
    total_time: Duration,
    total_worked: Duration,
    pause_num: i32,
    longest_pause: Duration,
    longest_work: Duration,
    since_last_log: Duration,
    percent: f32,
}
pub fn display_stats(args: &[String]) {
    let stats = db_operations::stats::get_stats(args);
    display_content(stats, PrintMode::All, None);
}
pub fn display_content(
    stats: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &str>,
    print_mode: PrintMode,
    concrete_tasks: Option<Vec<i32>>,
) {
    let all_projects = get_projects().ok();

    let (project_stats, task_stats) = get_stats_map(all_projects, stats);

    let mut table = Table::new();
    if let PrintMode::Project = print_mode {
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
                Cell::new("user")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("total tasks")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("completed tasks")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("planned_time")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("total_worked")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
            ]);
    } else if let PrintMode::ConcreteTasks = print_mode {
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("task_id")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("project_id")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("task_name")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("user")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("planned_time")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("total time")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("total worked")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("pause num")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("longest pause")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("longest work")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("since_last_log")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
            ]);
    } else {
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("task_id")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("project_id")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("task_name")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("user")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("planned_time")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("total time")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("total worked")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("pause num")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("longest pause")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("longest work")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
            ]);
    }
    let mut project_ids = HashSet::new();

    for task in task_stats {
        if let PrintMode::Project = print_mode {
        } else if let PrintMode::ConcreteTasks = print_mode {
            if let Some(ref x) = concrete_tasks {
                if x.contains(&task.task_id) {
                    table.add_row(vec![
                        Cell::new(task.task_id).set_alignment(CellAlignment::Center),
                        Cell::new(task.project_id).set_alignment(CellAlignment::Center),
                        Cell::new(task.task_name.clone()).set_alignment(CellAlignment::Center),
                        Cell::new(task.username.clone()).set_alignment(CellAlignment::Center),
                        Cell::new(task.planned_time.unwrap_or("null".to_string()))
                            .set_alignment(CellAlignment::Center),
                        Cell::new(format!(
                            "{:02}:{:02}:{:02}",
                            task.total_time.num_days(),
                            task.total_time.num_hours() - 24 * task.total_time.num_days(),
                            task.total_time.num_minutes() - task.total_time.num_hours() * 60
                        ))
                        .set_alignment(CellAlignment::Center),
                        Cell::new(format!(
                            "{:02}:{:02}:{:02}",
                            task.total_worked.num_days(),
                            task.total_worked.num_hours() - 24 * task.total_worked.num_days(),
                            task.total_worked.num_minutes() - task.total_worked.num_hours() * 60
                        ))
                        .set_alignment(CellAlignment::Center),
                        Cell::new(task.pause_num).set_alignment(CellAlignment::Center),
                        Cell::new(format!(
                            "{:02}:{:02}:{:02}",
                            task.longest_pause.num_days(),
                            task.longest_pause.num_hours() - 24 * task.longest_pause.num_days(),
                            task.longest_pause.num_minutes() - 60 * task.longest_pause.num_hours()
                        ))
                        .set_alignment(CellAlignment::Center),
                        Cell::new(format!(
                            "{:02}:{:02}:{:02}",
                            task.longest_work.num_days(),
                            task.longest_work.num_hours() - 24 * task.longest_work.num_days(),
                            task.longest_work.num_minutes() - 60 * task.longest_work.num_hours()
                        ))
                        .set_alignment(CellAlignment::Center),
                        Cell::new(format!(
                            "{:02}:{:02}:{:02}",
                            task.since_last_log.num_days(),
                            task.since_last_log.num_hours() - 24 * task.longest_work.num_days(),
                            task.since_last_log.num_minutes() - 60 * task.longest_work.num_hours()
                        ))
                        .set_alignment(CellAlignment::Center),
                    ]);
                }
            }
        } else {
            table.add_row(vec![
                Cell::new(task.task_id).set_alignment(CellAlignment::Center),
                Cell::new(task.project_id).set_alignment(CellAlignment::Center),
                Cell::new(task.task_name.clone()).set_alignment(CellAlignment::Center),
                Cell::new(task.username.clone()).set_alignment(CellAlignment::Center),
                Cell::new(task.planned_time.unwrap_or("null".to_string()))
                    .set_alignment(CellAlignment::Center),
                Cell::new(format!(
                    "{:02}:{:02}:{:02}",
                    task.total_time.num_days(),
                    task.total_time.num_hours() - 24 * task.total_time.num_days(),
                    task.total_time.num_minutes() - task.total_time.num_hours() * 60
                ))
                .set_alignment(CellAlignment::Center),
                Cell::new(format!(
                    "{:02}:{:02}:{:02}",
                    task.total_worked.num_days(),
                    task.total_worked.num_hours() - 24 * task.total_worked.num_days(),
                    task.total_worked.num_minutes() - task.total_worked.num_hours() * 60
                ))
                .set_alignment(CellAlignment::Center),
                Cell::new(task.pause_num).set_alignment(CellAlignment::Center),
                Cell::new(format!(
                    "{:02}:{:02}:{:02}",
                    task.longest_pause.num_days(),
                    task.longest_pause.num_hours() - 24 * task.longest_pause.num_days(),
                    task.longest_pause.num_minutes() - 60 * task.longest_pause.num_hours()
                ))
                .set_alignment(CellAlignment::Center),
                Cell::new(format!(
                    "{:02}:{:02}:{:02}",
                    task.longest_work.num_days(),
                    task.longest_work.num_hours() - 24 * task.longest_work.num_days(),
                    task.longest_work.num_minutes() - 60 * task.longest_work.num_hours()
                ))
                .set_alignment(CellAlignment::Center),
            ]);
        }

        project_ids.insert(task.project_id);
    }
    // project_ids.insert(0);
    match print_mode {
        PrintMode::All => {
            let mut vals: Vec<ProjectStats> = project_stats.values().cloned().collect();
            vals.sort_by(|a, b| {
                if a.project_id == 0 {
                    std::cmp::Ordering::Greater
                } else if b.project_id == 0 {
                    std::cmp::Ordering::Less
                } else {
                    a.project_id.cmp(&b.project_id)
                }
            });

            for project in vals {
                extend_table(&mut table, &project);
            }
        }

        PrintMode::Appearing => {
            let mut vals: Vec<i32> = project_ids.into_iter().collect();
            vals.sort_by(|a, b| {
                if a == &0 {
                    std::cmp::Ordering::Greater
                } else if b == &0 {
                    std::cmp::Ordering::Less
                } else {
                    a.cmp(&b)
                }
            });

            for x in vals {
                let project = project_stats.get(&x).unwrap();
                extend_table(&mut table, project);
            }
        }
        PrintMode::Project => {
            project_ids.insert(0);
            let mut vals: Vec<i32> = project_ids.into_iter().collect();
            vals.sort_by(|a, b| {
                if a == &0 {
                    std::cmp::Ordering::Greater
                } else if b == &0 {
                    std::cmp::Ordering::Less
                } else {
                    a.cmp(&b)
                }
            });

            for x in vals {
                let project = project_stats.get(&x).unwrap();
                extend_project_table(&mut table, project);
            }
        }

        PrintMode::ConcreteTasks => {}
    }

    println!("{table}");
}

fn extend_table(table: &mut Table, project: &ProjectStats) {
    table.add_row(vec![
        Cell::new("total").set_alignment(CellAlignment::Center),
        Cell::new(if project.project_id > 0 {
            project.project_id.to_string()
        } else {
            "total".to_string()
        })
        .set_alignment(CellAlignment::Center),
        Cell::new(project.project_name.clone()).set_alignment(CellAlignment::Center),
        Cell::new(project.username.clone()).set_alignment(CellAlignment::Center),
        Cell::new(project.clone().planned_time.unwrap_or("null".to_string()))
            .set_alignment(CellAlignment::Center),
        Cell::new(format!(
            "{:02}:{:02}:{:02}",
            project.total_time.num_days(),
            project.total_time.num_hours() - 24 * project.total_time.num_days(),
            project.total_time.num_minutes() - project.total_time.num_hours() * 60
        ))
        .set_alignment(CellAlignment::Center),
        Cell::new(format!(
            "{:02}:{:02}:{:02}",
            project.total_worked.num_days(),
            project.total_worked.num_hours() - 24 * project.total_worked.num_days(),
            project.total_worked.num_minutes() - project.total_worked.num_hours() * 60
        ))
        .set_alignment(CellAlignment::Center),
        Cell::new(project.pause_num).set_alignment(CellAlignment::Center),
        Cell::new(format!(
            "{:02}:{:02}:{:02}",
            project.longest_pause.num_days(),
            project.longest_pause.num_hours() - 24 * project.longest_pause.num_days(),
            project.longest_pause.num_minutes() - 60 * project.longest_pause.num_hours()
        ))
        .set_alignment(CellAlignment::Center),
        Cell::new(format!(
            "{:02}:{:02}:{:02}",
            project.longest_work.num_days(),
            project.longest_work.num_hours() - 24 * project.longest_work.num_days(),
            project.longest_work.num_minutes() - 60 * project.longest_work.num_hours()
        ))
        .set_alignment(CellAlignment::Center),
    ]);
}

fn extend_project_table(table: &mut Table, project: &ProjectStats) {
    table.add_row(vec![
        Cell::new(if project.project_id == 0 {
            "total".to_string()
        } else {
            project.project_id.to_string()
        })
        .set_alignment(CellAlignment::Center),
        Cell::new(project.project_name.clone()).set_alignment(CellAlignment::Center),
        Cell::new(project.username.clone()).set_alignment(CellAlignment::Center),
        Cell::new(project.total_tasks).set_alignment(CellAlignment::Center),
        Cell::new(project.completed_tasks).set_alignment(CellAlignment::Center),
        Cell::new(project.clone().planned_time.unwrap_or("null".to_string()))
            .set_alignment(CellAlignment::Center),
        Cell::new(format!(
            "{:02}:{:02}:{:02}",
            project.total_worked.num_days(),
            project.total_worked.num_hours() - 24 * project.total_worked.num_days(),
            project.total_worked.num_minutes() - project.total_worked.num_hours() * 60
        ))
        .set_alignment(CellAlignment::Center),
    ]);
}

fn get_stats_map(
    all_projects: Option<Vec<Project>>,
    stats: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &str>,
) -> (HashMap<i32, ProjectStats>, Vec<TaskStats>) {
    let mut project_stats = HashMap::new();
    let mut task_stats = vec![];
    if let Some(x) = all_projects {
        for project in x {
            let new_project_stats = ProjectStats {
                project_id: project.project_id,
                project_name: project.project_name,
                username: project.username,
                planned_time: project.planned_time,
                total_time: Duration::new(0, 0).unwrap_or_default(),
                total_worked: Duration::new(0, 0).unwrap_or_default(),
                pause_num: 0,
                longest_pause: Duration::new(0, 0).unwrap_or_default(),
                longest_work: Duration::new(0, 0).unwrap_or_default(),
                total_tasks: 0,
                completed_tasks: 0,
            };

            project_stats.insert(project.project_id, new_project_stats);
        }
    }

    let total_stats = ProjectStats {
        project_id: 0,
        project_name: "null".to_string(),
        username: "null".to_string(),
        planned_time: Some("null".to_string()),
        total_time: Duration::new(0, 0).unwrap_or_default(),
        total_worked: Duration::new(0, 0).unwrap_or_default(),
        pause_num: 0,
        longest_pause: Duration::new(0, 0).unwrap_or_default(),
        longest_work: Duration::new(0, 0).unwrap_or_default(),
        total_tasks: 0,
        completed_tasks: 0,
    };
    project_stats.insert(0, total_stats.clone());

    match stats {
        Err(x) => println!("{}", x),
        Ok(result) => {
            let mut i = 1;
            let mut pause_num = 0;
            let mut longest_pause = Duration::seconds(0);
            let mut longest_work = Duration::seconds(0);
            let mut begin;
            let mut total_time = Duration::seconds(0);
            let mut total_worked = Duration::seconds(0);
            while i < result.len() {
                pause_num = 0;
                longest_pause = Duration::seconds(0);
                longest_work = Duration::seconds(0);
                begin = result[i - 1].3.unwrap_or_default();
                total_time = Duration::seconds(0);
                total_worked = Duration::seconds(0);
                while i < result.len() && &result[i].0.task_id == &result[i - 1].0.task_id {
                    if <Option<String> as Clone>::clone(&result[i].2).unwrap_or("".to_string())
                        == Commands::Pause.to_string()
                        || (<Option<String> as Clone>::clone(&result[i].2).unwrap_or_default()
                            == Commands::End.to_string()
                            && <Option<String> as Clone>::clone(&result[i - 1].2)
                                .unwrap_or("".to_string())
                                != Commands::Pause.to_string())
                    {
                        let slot = result[i]
                            .3
                            .unwrap_or_default()
                            .signed_duration_since(result[i - 1].3.unwrap_or_default());
                        longest_work = max(longest_work, slot);
                        total_worked += slot;
                        if <Option<String> as Clone>::clone(&result[i].2).unwrap_or("".to_string())
                            == Commands::Pause.to_string()
                        {
                            pause_num += 1;
                        }
                    } else if <Option<String> as Clone>::clone(&result[i].2)
                        .unwrap_or("".to_string())
                        == Commands::Resume.to_string()
                        || (<Option<String> as Clone>::clone(&result[i].2).unwrap_or_default()
                            == Commands::End.to_string()
                            && <Option<String> as Clone>::clone(&result[i - 1].2)
                                .unwrap_or("".to_string())
                                == Commands::Pause.to_string())
                    {
                        let slot = result[i]
                            .3
                            .unwrap_or_default()
                            .signed_duration_since(result[i - 1].3.unwrap_or_default());
                        longest_pause = max(longest_pause, slot);
                    }

                    total_time = result[i].3.unwrap_or_default().signed_duration_since(begin);
                    i += 1;
                }

                if <Option<String> as Clone>::clone(&result[i - 1].2).unwrap_or_default()
                    == Commands::End.to_string()
                {
                    let new_project_stats =
                        project_stats.get_mut(&result[i - 1].0.project_id).unwrap();
                    new_project_stats.completed_tasks += 1;
                }

                let new_task_stats = TaskStats {
                    task_id: result[i - 1].0.task_id,
                    project_id: result[i - 1].0.project_id.clone(),
                    task_name: result[i - 1].clone().0.task_name,
                    username: result[i - 1].0.username.clone(),
                    planned_time: result[i - 1].clone().0.planned_time,
                    total_time,
                    total_worked,
                    pause_num,
                    longest_pause,
                    longest_work,
                    since_last_log: Local::now()
                        .naive_local()
                        .signed_duration_since(result[i - 1].3.unwrap_or_default()),
                    percent: 0.3
                };

                task_stats.push(new_task_stats);

                let new_project_stats = project_stats.get_mut(&result[i - 1].0.project_id).unwrap();
                new_project_stats.total_time += total_time;
                new_project_stats.total_worked += total_worked;
                new_project_stats.pause_num += pause_num;
                new_project_stats.longest_pause =
                    max(new_project_stats.longest_pause, longest_pause);
                new_project_stats.longest_work = max(longest_work, new_project_stats.longest_work);
                new_project_stats.total_tasks += 1;

                let total_stats = project_stats.get_mut(&0).unwrap();

                total_stats.total_time += total_time;
                total_stats.total_worked += total_worked;
                total_stats.pause_num += pause_num;
                total_stats.longest_pause = max(total_stats.longest_pause, longest_pause);
                total_stats.longest_work = max(longest_work, total_stats.longest_work);
                total_stats.total_tasks += 1;

                i += 1;
            }
            if i <= result.len() {
                let new_project_stats = project_stats.get_mut(&result[i - 1].0.project_id).unwrap();
                new_project_stats.total_time += total_time;
                new_project_stats.total_worked += total_worked;
                new_project_stats.pause_num += pause_num;
                new_project_stats.longest_pause =
                    max(new_project_stats.longest_pause, longest_pause);
                new_project_stats.longest_work = max(longest_work, new_project_stats.longest_work);
                new_project_stats.total_tasks += 1;

                let total_stats = project_stats.get_mut(&0).unwrap();
                total_stats.total_time += total_time;
                total_stats.total_worked += total_worked;
                total_stats.pause_num += pause_num;
                total_stats.longest_pause = max(total_stats.longest_pause, longest_pause);
                total_stats.longest_work = max(longest_work, total_stats.longest_work);
                total_stats.total_tasks += 1;

                let new_task_stats = TaskStats {
                    task_id: result[i - 1].0.task_id,
                    project_id: result[i - 1].0.project_id.clone(),
                    task_name: result[i - 1].clone().0.task_name,
                    username: result[i - 1].0.username.clone(),
                    planned_time: result[i - 1].clone().0.planned_time,
                    total_time,
                    total_worked,
                    pause_num,
                    longest_pause,
                    longest_work,
                    since_last_log: Local::now()
                        .naive_local()
                        .signed_duration_since(result[i - 1].3.unwrap_or_default()),
                    percent: 0.3,
                };
                if <Option<String> as Clone>::clone(&result[i - 1].2).unwrap_or_default()
                    == Commands::End.to_string()
                {
                    let new_project_stats =
                        project_stats.get_mut(&result[i - 1].0.project_id).unwrap();
                    new_project_stats.completed_tasks += 1;
                }

                task_stats.push(new_task_stats);
            }
        }
    }

    (project_stats, task_stats)
}

pub fn display_day_stats(args: &[String]) {
    let mut date_to_seek = Local::now().naive_local().date();
    let mut long_version = false;

    let mut i = 0;
    while i < args.len() {
        match &args[i][..] {
            "-d" | "-day" | "-ld" | "-dl" => {
                if "-ld" == &args[i][..] || "-dl" == &args[i][..] {
                    long_version = true;
                }
                i += 1;
                if args.len() < 2 {
                    println!("Too little arguments for this option!");
                    return;
                }

                let seeked_date = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d");
                if let Err(_) = seeked_date {
                    println!("Couldn't parse the date format, giving results for current day");
                }

                date_to_seek = seeked_date.unwrap_or(Local::now().naive_local().date());
                i += 1;
            }
            "-l" => {
                i += 1;
                long_version = true;
            }

            _ => {
                println!("Unknown argument '{}': try again", &args[i]);
                process::exit(-1);
            }
        }
    }

    let projects = get_date_projects(date_to_seek);

    if let Ok(x) = projects {
        let stats: Result<Vec<(Task, Option<i32>, Option<String>, Option<NaiveDateTime>)>, &str> =
            db_operations::stats::get_day_stats_tasks(date_to_seek, None);

        if x.len() > 0 {
            if date_to_seek == Local::now().naive_local().date() {
                print!("Today ({})", date_to_seek);
            } else {
                print!("On {} ", date_to_seek);
            }
            println!("you worked on the following {} projects:", x.len());
            display_content(stats, PrintMode::Project, None);
            if long_version {
                println!("In details:");
                for project in x {
                    println!("Project {}:", project.project_id);
                    let stats = db_operations::stats::get_day_stats_tasks(
                        date_to_seek,
                        Some(project.project_id),
                    );
                    display_content(stats, PrintMode::Appearing, None)
                }
            }
        } else {
            println!("Today you didn't work on any projects");
        }
    }
}
