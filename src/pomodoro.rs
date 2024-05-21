use chrono::NaiveTime;
use crossterm::{cursor, terminal, ExecutableCommand};
use regex::Regex;
use std::io::{self, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{process, thread};
use terminal_fonts::{map_block, to_block, to_string};
use termion::terminal_size;

use crate::{db_operations, logs, stats, Commands};

fn green(v: &str) -> String {
    format!("{}{}{}", "\u{001b}[32m", v, "\u{001b}[0m")
}

fn red(v: &str) -> String {
    format!("{}{}{}", "\u{001b}[31m", v, "\u{001b}[0m")
}
enum PomodoroCommands {
    Yes,
    No,
    Work,
    Pause,
    End,
    Exit,
    ChangeTask,
    CurrentTask,
    Clear,
    NoSuchCommand,
}

impl From<String> for PomodoroCommands {
    fn from(input: String) -> Self {
        match input.trim() {
            "yes" => PomodoroCommands::Yes,
            "no" => PomodoroCommands::No,
            "work" => PomodoroCommands::Work,
            "pause" => PomodoroCommands::Pause,
            "end" => PomodoroCommands::End,
            "exit" => PomodoroCommands::Exit,
            "changetask" => PomodoroCommands::ChangeTask,
            "currenttask" => PomodoroCommands::CurrentTask,
            "clear" => PomodoroCommands::Clear,
            _ => PomodoroCommands::NoSuchCommand,
        }
    }
}
pub fn pomodoro(args: &[String]) {
    if args.len() < 1 {
        eprintln!("Too few args");
        process::exit(-1);
    }

    if args.len() > 2 {
        eprintln!("Too many args");
        process::exit(-1);
    }
    let mut commands_num = 2;

    let task_id = &args[0].parse::<i32>();
    let clearing = args.len() > 1 && args[1] == "clearing";

    let task_id = match task_id {
        Err(_) => {
            println!("Wrong argument!");
            return;
        }
        Ok(x) => match db_operations::tasks::find_task_by_id(x) {
            Ok(Some(_)) => x,
            Ok(None) => {
                println!("No such task!");
                return;
            }
            Err(x) => {
                println!("{}", x);
                return;
            }
        },
    };

    if !pomodoro_possible(Commands::Pause, task_id) {
        process::exit(-1);
    }

    let mut stdout = stdout();

    ctrlc::set_handler(move || {
        let mut stdout = io::stdout();

        stdout.execute(cursor::Show).unwrap();
        stdout.execute(cursor::MoveToColumn(0)).unwrap();
        stdout
            .execute(cursor::MoveDown(commands_num as u16))
            .unwrap();
        stdout
            .execute(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();

        println!("Exiting...");
        io::stdout().flush().unwrap();
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let term = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGQUIT, Arc::clone(&term)).unwrap();
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
    stdout.execute(cursor::Hide).unwrap();
    let zero_secs = NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();

    let mut count_time = NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();

    let time_str: String = count_time.format("%H:%M:%S").to_string();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();
    let time_str: String = to_string(&map_block(&to_block(&time_str), red));

    println!("{}", red(&time_str));
    println!("\n");
    loop {
        stdout.execute(cursor::Show).unwrap();

        let last_command = action_commands(
            &mut commands_num,
            &mut count_time,
            time_str.lines().count() as i32,
            *task_id,
            &clearing,
        );
        stdout.execute(cursor::Hide).unwrap();

        while count_time != zero_secs {
            show_clock(&mut count_time, &term);
            if count_time != zero_secs {
                clock_stopped(
                    &mut commands_num,
                    &mut count_time,
                    &term,
                    time_str.lines().count() as i32,
                    &clearing,
                );
                (term).store(false, Ordering::Relaxed);
            }
        }
        print!("\x07");

        let time_str: String = count_time.format("%H:%M:%S").to_string();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        let time_str: String = to_string(&map_block(&to_block(&time_str), red));

        println!("{}", red(&time_str));

        stdout.execute(cursor::Show).unwrap();
        stdout.execute(cursor::MoveToColumn(0)).unwrap();
        stdout
            .execute(cursor::MoveDown(commands_num as u16))
            .unwrap();
        save_log(
            &mut commands_num,
            &mut count_time,
            last_command,
            &task_id,
            time_str.lines().count() as i32,
            &clearing,
        );

        io::stdout().flush().unwrap();

        stdout.execute(cursor::Hide).unwrap();
    }
}

fn show_clock(count_time: &mut NaiveTime, term: &Arc<AtomicBool>) {
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();

    let zero_secs = NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();
    let one_sec = Duration::from_secs(1);
    while *count_time > zero_secs && !(*term).load(Ordering::Relaxed) {
        let time_str: String = count_time.format("%H:%M:%S").to_string();

        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        let time_str: String = to_string(&map_block(&to_block(&time_str), green));

        println!("{}", green(&time_str));

        stdout.flush().unwrap();

        *count_time = *count_time - chrono::Duration::seconds(1);

        thread::sleep(one_sec);
    }
}
fn clock_stopped(
    commands_num: &mut i32,
    count_time: &mut NaiveTime,
    term: &Arc<AtomicBool>,
    line_count: i32,
    clearing: &bool,
) {
    let mut stdout = stdout();
    let time_str: String = count_time.format("%H:%M:%S").to_string();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();
    let time_str: String = to_string(&map_block(&to_block(&time_str), red));

    println!("{}", red(&time_str));

    stdout.execute(cursor::Show).unwrap();
    stdout.execute(cursor::MoveToColumn(0)).unwrap();
    stdout
        .execute(cursor::MoveDown(*commands_num as u16))
        .unwrap();
    loop {
        print!("Clock was stopped, continue counting? [yes/no]: ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        *commands_num += 1;

        match PomodoroCommands::from(command) {
            PomodoroCommands::Yes => {
                println!("Clock running: to stop it press ctrl + \\");
                *commands_num += 1;
                show_clock(count_time, term);
                return;
            }
            PomodoroCommands::No => {
                *count_time = NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();
                return;
            }
            _ => {
                println!("Wrong command, write yes or no!");
                io::stdout().flush().unwrap();
                *commands_num += 1;
            }
        }
        let size = terminal_size().unwrap().1 as i32;
        if *clearing && commands_num > &mut (size - line_count) {
            clear_terminal(commands_num, line_count);
        }

        let time_str: String = count_time.format("%H:%M:%S").to_string();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        let time_str: String = to_string(&map_block(&to_block(&time_str), red));

        println!("{}", red(&time_str));
        stdout
            .execute(cursor::MoveDown(*commands_num as u16))
            .unwrap();
    }
}

fn save_log(
    commands_num: &mut i32,
    count_time: &mut NaiveTime,
    last_command: PomodoroCommands,
    task_id: &i32,
    line_count: i32,
    clearing: &bool,
) {
    let mut repeat_question = true;
    while repeat_question {
        match last_command {
            PomodoroCommands::Work => {
                print!("Save log \"pause\"? [yes/no]: ");
            }
            PomodoroCommands::Pause => {
                print!("Save log \"resume\"? [yes/no]: ");
            }
            _ => {
                return;
            }
        }

        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        repeat_question = false;
        *commands_num += 1;

        match PomodoroCommands::from(command) {
            PomodoroCommands::Yes => match last_command {
                PomodoroCommands::Work => {
                    let lines = logs::add_log_by_id(Commands::Pause, task_id);
                    *commands_num += lines as i32 + 1;
                }
                PomodoroCommands::Pause => {
                    let lines = logs::add_log_by_id(Commands::Resume, task_id);
                    *commands_num += lines as i32 + 1;
                }
                _ => (),
            },
            PomodoroCommands::No => (),
            _ => {
                println!("Wrong command, write yes or no!");
                io::stdout().flush().unwrap();
                *commands_num += 1;
                repeat_question = true;
            }
        }
        let size = terminal_size().unwrap().1 as i32;
        if *clearing && commands_num > &mut (size - line_count) {
            clear_terminal(commands_num, line_count);
        }
        let time_str: String = count_time.format("%H:%M:%S").to_string();
        let mut stdout: io::Stdout = stdout();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        let time_str: String = to_string(&map_block(&to_block(&time_str), red));

        println!("{}", red(&time_str));
        stdout
            .execute(cursor::MoveDown(*commands_num as u16))
            .unwrap();
    }
}

fn action_commands(
    commands_num: &mut i32,
    count_time: &mut NaiveTime,
    line_count: i32,
    mut task_id: i32,
    clearing: &bool,
) -> PomodoroCommands {
    let mut repeat_question = true;
    let time_regex = Regex::new(r"^\d+:\d+:\d+$").unwrap();
    let mut last_command = PomodoroCommands::NoSuchCommand;
    while repeat_question {
        print!(">> ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        let args: Vec<&str> = command.split_whitespace().collect();

        repeat_question = false;
        // *commands_num += 1;
        last_command = PomodoroCommands::from(args.clone()[0].to_string());

        match PomodoroCommands::from(args.clone()[0].to_string()) {
            PomodoroCommands::Work => {
                if action_possible(Commands::Resume, &task_id, commands_num) {
                    let lines = logs::add_log_by_id(Commands::Resume, &task_id);
                    *commands_num += lines as i32 + 1;
                } else {
                    println!("Continuing a previously started work");
                    io::stdout().flush().unwrap();
                    let stats = db_operations::stats::get_stats(&[]);
                    let lines = stats::display_content(
                        stats,
                        stats::PrintMode::ConcreteTasks,
                        Some(vec![task_id]),
                    );
                    io::stdout().flush().unwrap();

                    *commands_num += lines as i32 + 2;
                }

                if args.len() >= 2 {
                    if !time_regex.is_match(args[1]) {
                        println!("Time format should be HH:MM:SS");
                        repeat_question = true;
                        *commands_num += 1;
                    } else {
                        *count_time = NaiveTime::parse_from_str(args[1], "%H:%M:%S").unwrap();
                    }
                } else {
                    *count_time = NaiveTime::parse_from_str("00:00:05", "%H:%M:%S").unwrap();
                }
            }
            PomodoroCommands::Pause => {
                if action_possible(Commands::Pause, &task_id, commands_num) {
                    let lines = logs::add_log_by_id(Commands::Pause, &task_id);
                    *commands_num += lines as i32 + 1;
                } else {
                    println!("Continuing a previously started pause");
                    io::stdout().flush().unwrap();
                    let stats = db_operations::stats::get_stats(&[]);
                    let lines = stats::display_content(
                        stats,
                        stats::PrintMode::ConcreteTasks,
                        Some(vec![task_id]),
                    );
                    io::stdout().flush().unwrap();

                    *commands_num += lines as i32 + 2;
                }

                if args.len() >= 2 {
                    if !time_regex.is_match(args[1]) {
                        println!("Time format should be HH:MM:SS");
                        *commands_num += 1;
                        repeat_question = true;
                    } else {
                        *count_time = NaiveTime::parse_from_str(args[1], "%H:%M:%S").unwrap();
                    }
                } else {
                    *count_time = NaiveTime::parse_from_str("00:05:00", "%H:%M:%S").unwrap();
                }
            }
            PomodoroCommands::Exit => process::exit(0),
            PomodoroCommands::CurrentTask => {
                let stats = db_operations::stats::get_stats(&[]);
                let lines = stats::display_content(
                    stats,
                    stats::PrintMode::ConcreteTasks,
                    Some(vec![task_id]),
                );
                *commands_num += lines as i32;
            }
            PomodoroCommands::ChangeTask => {
                repeat_question = true;

                if args.len() < 2 {
                    println!("No task id!");
                    *commands_num += 1;
                } else {
                    let new_task_id = args[1].parse::<i32>();
                    match new_task_id {
                        Ok(x) => {
                            match db_operations::tasks::find_task_by_id(&x) {
                                Ok(Some(_)) => {
                                    if !pomodoro_possible(Commands::Pause, &x) {
                                        println!("Impossible to change task");
                                        io::stdout().flush().unwrap();
                                    } else {
                                        println!("Changing task_id to {}", x);
                                        io::stdout().flush().unwrap();

                                        task_id = x;
                                    }
                                }
                                Ok(None) => {
                                    println!("No such task!");
                                    io::stdout().flush().unwrap();

                                    repeat_question = true;
                                }
                                Err(x) => {
                                    println!("{}", x);
                                    io::stdout().flush().unwrap();

                                    repeat_question = true;
                                }
                            }
                            *commands_num += 1;
                        }
                        Err(_) => {
                            io::stdout().flush().unwrap();
                            println!("Couldn't parse task id")
                        }
                    }
                }
            }
            PomodoroCommands::Clear => {
                clear_terminal(commands_num, line_count);
                repeat_question = true;
            }
            _ => {
                println!("Wrong command!");
                io::stdout().flush().unwrap();
                *commands_num += 2;
                repeat_question = true;
            }
        }
        let size = terminal_size().unwrap().1 as i32;
        if *clearing && commands_num > &mut (size - line_count) {
            clear_terminal(commands_num, line_count);
        }
        let time_str: String = count_time.format("%H:%M:%S").to_string();
        let mut stdout: io::Stdout = stdout();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        let time_str: String = to_string(&map_block(&to_block(&time_str), red));

        println!("{}", red(&time_str));
        stdout
            .execute(cursor::MoveDown(*commands_num as u16))
            .unwrap();
    }
    last_command
}

fn clear_terminal(commands_num: &mut i32, line_count: i32) {
    let mut stdout = stdout();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();
    stdout.execute(cursor::MoveToColumn(0)).unwrap();
    stdout
        .execute(cursor::MoveDown((line_count - 1) as u16))
        .unwrap();
    stdout
        .execute(terminal::Clear(terminal::ClearType::FromCursorDown))
        .unwrap();
    *commands_num = 2;
}

pub fn action_possible(log_type: Commands, num: &i32, commands_num: &mut i32) -> bool {
    let recent_log = db_operations::logs::get_recent_log(*num, true);

    match recent_log {
        Err(x) => {
            println!("{}", x);
            io::stdout().flush().unwrap();
        }
        Ok(None) => {
            if log_type != (Commands::Begin) {
                println!("First begin the task, then perform other operations!");
                io::stdout().flush().unwrap();
            }
        }
        Ok(Some(x)) if x.log_type == Commands::End.to_string() => {
            println!("Task has been ended!");
            io::stdout().flush().unwrap();
        }

        Ok(Some(x))
            if x.log_type == Commands::Pause.to_string() && log_type == (Commands::Pause) =>
        {
            println!("Task has already been paused");
            io::stdout().flush().unwrap();
        }

        Ok(Some(x))
            if x.log_type != Commands::Pause.to_string() && log_type == (Commands::Resume) =>
        {
            println!("Pause task before you resume it");
            io::stdout().flush().unwrap();
        }

        _ => return true,
    }

    *commands_num += 1;
    false
}

fn pomodoro_possible(log_type: Commands, num: &i32) -> bool {
    let recent_log = db_operations::logs::get_recent_log(*num, true);

    match recent_log {
        Err(x) => {
            println!("{}", x);
        }
        Ok(None) => {
            if log_type != (Commands::Begin) {
                println!("First begin the task, then perform other operations!");
            }
        }
        Ok(Some(x)) if x.log_type == Commands::End.to_string() => {
            println!("Task has been ended!");
        }

        _ => return true,
    }
    io::stdout().flush().unwrap();
    false
}
