use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, Color,
    ContentArrangement, Table,
};
use std::process;

use crate::db_operations::{self, apps::get_app_stats};

pub fn add_app(args: &[String], display_communicates: bool) {
    if args.len() < 1 {
        eprintln!("Too few args");
        process::exit(-1);
    }

    for x in args {
        match db_operations::apps::find_app(x) {
            Ok(None) => {
                match db_operations::apps::add_app(&(x.to_lowercase()), display_communicates) {
                    Err(x) => println!("{}", x),
                    _ => (),
                }
            }
            Ok(Some(_a)) => {
                if display_communicates {
                    println!("App {} already in db", x);
                }
            }
            Err(_) => {
                println!("An error occured while fetching app {}", x);
            }
        };
    }
}

pub fn display_apps(args: &[String]) {
    let apps = get_app_stats(args);

    if let Ok(x) = apps {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("app_id")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("app_name")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
                Cell::new("used in projects")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Cyan),
            ]);
        for row in x {
            table.add_row(vec![
                Cell::new(row.0.app_id).set_alignment(CellAlignment::Center),
                Cell::new(row.0.app_name).set_alignment(CellAlignment::Center),
                Cell::new(row.1.unwrap_or(0)).set_alignment(CellAlignment::Center),
            ]);
        }
        println!("{table}");
    }
}
