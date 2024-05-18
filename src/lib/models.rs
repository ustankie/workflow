use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Debug,Clone)]
#[diesel(table_name = crate::schema::projects)]
pub struct Project {
    pub project_id: i32,
    pub project_name: String,
    pub username: String,
    pub planned_time: Option<String>
}

#[derive(Queryable, Selectable, Debug,Clone)]
#[diesel(table_name = crate::schema::tasks)]
pub struct Task {
    pub task_id: i32,
    pub project_id:i32,
    pub task_name: String,
    pub username: String,
    pub planned_time: Option<String>
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::apps)]
pub struct App {
    pub app_id: i32,
    pub app_name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::project_apps)]
pub struct ProjectApp {
    pub id: i32,
    pub project_id: i32,
    pub app_id: i32,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::log)]
pub struct Log {
    pub log_id: i32,
    pub task_id: i32,
    pub log_type: String,
    pub date: NaiveDateTime,
}

use crate::schema::apps;

#[derive(Insertable)]
#[diesel(table_name = apps)]
pub struct NewApp<'a> {
    pub app_name: &'a str,
}


#[derive(Insertable,PartialEq,Debug)]
#[diesel(table_name = crate::schema::projects)]
pub struct NewProject<'a>{
    pub project_name: &'a str,
    pub username: &'a str,
    pub planned_time:  Option<&'a str>
}


#[derive(Insertable,PartialEq,Debug)]
#[diesel(table_name = crate::schema::tasks)]
pub struct NewTask<'a>{
    pub project_id: i32,
    pub task_name: &'a str,
    pub username: &'a str,
    pub planned_time:  Option<&'a str>
}

#[derive(Insertable,PartialEq)]
#[diesel(table_name = crate::schema::project_apps)]
pub struct NewProjectApp{
    pub project_id: i32,
    pub app_id: i32,
}

#[derive(Insertable,PartialEq)]
#[diesel(table_name = crate::schema::log)]
pub struct NewLog{
    pub task_id: i32,
    pub log_type: String,
    pub date: NaiveDateTime
}