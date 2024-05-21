use chrono::NaiveTime;
use crossterm::{cursor, terminal, ExecutableCommand};
use regex::Regex;
use std::io::{self, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{process, thread};
use terminal_fonts::{map_block, to_block, to_string};

fn blue(v: &str) -> String {
    format!("{}{}{}", "\u{001b}[34m", v, "\u{001b}[0m")
}
enum PomodoroCommands {
    Yes,
    No,
    Work,
    Pause,
    End,
    Exit,
    ChangeTask,
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
            "clear" => PomodoroCommands::Clear,
            _ => PomodoroCommands::NoSuchCommand,
        }
    }
}
pub fn pomodoro() {
    let mut stdout = stdout();

    let term = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGQUIT, Arc::clone(&term)).unwrap();
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
    stdout.execute(cursor::Hide).unwrap();
    let zero_secs = NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();

    let mut count_time = NaiveTime::parse_from_str("00:00:00", "%H:%M:%S").unwrap();

    let mut commands_num = 2;

    let time_str: String = count_time.format("%H:%M:%S").to_string();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();
    let time_str: String = to_string(&map_block(&to_block(&time_str), blue));

    println!("{}", blue(&time_str));
    println!("\n");
    loop {
        stdout.execute(cursor::Show).unwrap();

        action_commands(
            &mut commands_num,
            &mut count_time,
            time_str.lines().count() as i32,
        );
        stdout.execute(cursor::Hide).unwrap();

        while count_time != zero_secs {
            show_clock(&mut count_time, &term);
            if count_time != zero_secs {
                clock_stopped(&mut commands_num, &mut count_time, &term);
                (term).store(false, Ordering::Relaxed);
            }
        }

        let time_str: String = count_time.format("%H:%M:%S").to_string();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        let time_str: String = to_string(&map_block(&to_block(&time_str), blue));

        println!("{}", blue(&time_str));

        stdout.execute(cursor::Show).unwrap();
        stdout.execute(cursor::MoveToColumn(0)).unwrap();
        stdout
            .execute(cursor::MoveDown(commands_num as u16))
            .unwrap();
        save_log(&mut commands_num);

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
        let time_str: String = to_string(&map_block(&to_block(&time_str), blue));

        println!("{}", blue(&time_str));

        stdout.flush().unwrap();

        *count_time = *count_time - chrono::Duration::seconds(1);

        thread::sleep(one_sec);
    }
}
fn clock_stopped(commands_num: &mut i32, count_time: &mut NaiveTime, term: &Arc<AtomicBool>) {
    let mut stdout = stdout();
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
    }
}

fn save_log(commands_num: &mut i32) {
    let mut repeat_question = true;
    while repeat_question {
        print!("Save log? [yes/no]: ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        repeat_question = false;
        *commands_num += 1;

        match PomodoroCommands::from(command) {
            PomodoroCommands::Yes => (),
            PomodoroCommands::No => (),
            _ => {
                println!("Wrong command, write yes or no!");
                io::stdout().flush().unwrap();
                *commands_num += 1;
                repeat_question = true;
            }
        }
    }
}

fn action_commands(commands_num: &mut i32, count_time: &mut NaiveTime, line_count: i32) {
    let mut repeat_question = true;
    let mut stdout = stdout();
    let time_regex = Regex::new(r"^\d+:\d+:\d+$").unwrap();

    while repeat_question {
        print!(">> ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        let args: Vec<&str> = command.split_whitespace().collect();

        repeat_question = false;
        *commands_num += 1;

        match PomodoroCommands::from(args.clone()[0].to_string()) {
            PomodoroCommands::Work => {
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
            PomodoroCommands::End => todo!(),
            PomodoroCommands::Exit => process::exit(0),
            PomodoroCommands::ChangeTask => {
                repeat_question = true;

                if args.len() < 2 {
                    println!("No task id!");
                    *commands_num += 1;
                } else {
                    let task_id = args[1].parse::<i32>();
                    match task_id {
                        Ok(_x) => (),
                        Err(_) => println!("Couldn't parse task id"),
                    }
                }
            }
            PomodoroCommands::Clear => {
                stdout.execute(cursor::MoveTo(0, 0)).unwrap();
                stdout.execute(cursor::MoveToColumn(0)).unwrap();
                stdout
                    .execute(cursor::MoveDown((line_count + 1) as u16))
                    .unwrap();
                stdout
                    .execute(terminal::Clear(terminal::ClearType::FromCursorDown))
                    .unwrap();
                *commands_num = 1;
                repeat_question = true;
            }
            _ => {
                println!("Wrong command!");
                io::stdout().flush().unwrap();
                *commands_num += 1;
                repeat_question = true;
            }
        }
    }
}
