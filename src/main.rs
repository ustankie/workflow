use std::env;
use std::io;

use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use termion::event::Key;

use termion::input::TermRead;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub mod apps;
pub mod db_operations;
pub mod logs;
pub mod projects;
pub mod stats;
pub mod tasks;

#[derive(Debug, PartialEq)]
pub enum Commands {
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
    NoSuchCommand,
}

impl From<String> for Commands {
    fn from(input: String) -> Self {
        match input.trim() {
            "newproject" => Commands::AddProject,
            "projects" => Commands::AllProjects,
            "app" => Commands::AddApp,
            "newtask" => Commands::AddTask,
            "tasks" => Commands::AllTasks,
            "logs" => Commands::Logs,
            "app-task" => Commands::AddAppToTask,
            "stats" => Commands::Stats,
            "begin" => Commands::Begin,
            "end" => Commands::End,
            "pause" => Commands::Pause,
            "resume" => Commands::Resume,
            "man" => Commands::Man,
            "day" => Commands::Day,
            _ => Commands::NoSuchCommand,
        }
    }
}

impl ToString for Commands {
    fn to_string(&self) -> String {
        match self {
            Self::Begin => String::from("B"),
            Self::End => String::from("E"),
            Self::Pause => String::from("P"),
            Self::Resume => String::from("R"),
            _ => String::from(""),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Issue a command!");
        return;
    }

    let command = Commands::from(args[1].clone());
    match command {
        Commands::AddApp => apps::add_app(&args[2..], true),
        Commands::AddTask => tasks::add_task(args),
        Commands::AllTasks => tasks::display_tasks(),
        Commands::Begin | Commands::End | Commands::Pause | Commands::Resume => {
            logs::add_log(args, command)
        }
        Commands::Logs => logs::display_logs(&args[2..]),
        Commands::Man => display_man(),
        Commands::Stats => stats::display_stats(&args[2..]),
        Commands::AddProject => projects::add_project(args),
        Commands::AllProjects => projects::display_projects(),
        Commands::Day => stats::display_day_stats(&args[2..]),
        Commands::NoSuchCommand => {
            println!("Wrong command!");
        }
        _ => (),
    };
}

struct App {
    scroll_position: u16,
    msg_end: u16,
}

impl App {
    fn handle_input(&mut self, key: Key) {
        match key {
            Key::Up => {
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            Key::Down => {
                // You can adjust the scroll limit according to your content
                if self.scroll_position < self.msg_end {
                    self.scroll_position += 1;
                }
            }
            _ => {}
        }
    }
}

fn display_man() {
    let message=
"-----------------------------------------------------
Welcome to workflow!
-----------------------------------------------------
COMMANDS:
- newproject NAME- creates new project
    OPTIONS:
        -t TIME - sets time user plans to spend on project. It should have format DAYS:HOURS:MINUTES
        -a APPLIST - sets apps used in the project;

- newtask NAME PROJECT_ID - creates new task in a project of the given id
    OPTIONS:
        -t TIME - sets time user plans to spend on task. It should have format DAYS:HOURS:MINUTES;

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

- stats - displays stats for every task and project, i.e. number of pauses made during the task,
            time spent working etc.;

- day - displays day stats: a table of projects cdeveloped on the current day
    OPTIONS
        -l - displays additional information for every project developed on the current day and 
            tasks within it
        -d, -date DATE - sets the date to display instead of the current date, DATE should be 
            in format \"YYYY-MM-DD\"

        (possible merging -ld, -dl options)

- man - displays app's manual;";

    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to initialize terminal");
    let mut app = App {
        scroll_position: 0,
        msg_end: (message.lines().count() - 3) as u16,
    };

    terminal
        .draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(2)
                .split(f.size());

            let scrollable_text = message
                .lines()
                .skip(app.scroll_position as usize)
                .collect::<Vec<_>>()
                .join("\n");

            let paragraph = Paragraph::new(scrollable_text)
                .block(Block::default().borders(Borders::ALL).title("Man"));

            f.render_widget(paragraph, chunks[0]);
        })
        .expect("Error displaying man");

    for key in io::stdin().keys() {
        if let Ok(key) = key {
            match key {
                Key::Char('q') => break,
                _ => {
                    app.handle_input(key);
                    terminal
                        .draw(|f| {
                            let chunks = Layout::default()
                                .constraints([Constraint::Percentage(100)].as_ref())
                                .margin(2)
                                .split(f.size());

                            let scrollable_text = message
                                .lines()
                                .skip(app.scroll_position as usize)
                                .collect::<Vec<_>>()
                                .join("\n");

                            let paragraph = Paragraph::new(scrollable_text)
                                .block(Block::default().title("Man").borders(Borders::ALL));
                            f.render_widget(paragraph, chunks[0]);
                        })
                        .unwrap();
                }
            }
        }
    }

    disable_raw_mode().expect("Error ending raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("");
}
