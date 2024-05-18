
diesel::table! {
    apps (app_id) {
        app_id -> Int4,
        #[max_length = 20]
        app_name -> Varchar,
    }
}

diesel::table! {
    log (log_id) {
        log_id -> Int4,
        task_id -> Int4,
        #[max_length = 1]
        log_type -> Varchar,
        date -> Timestamp,
    }
}

diesel::table! {
    project_apps (id) {
        id -> Int4,
        project_id -> Int4,
        app_id -> Int4,
    }
}

diesel::table! {
    projects (project_id) {
        project_id -> Int4,
        #[max_length = 20]
        project_name -> Varchar,
        #[max_length = 20]
        username -> Varchar,
        #[max_length = 20]
        planned_time -> Nullable<Varchar>,
    }
}

diesel::table! {
    tasks (task_id) {
        task_id -> Int4,
        project_id -> Int4,
        #[max_length = 20]
        task_name -> Varchar,
        #[max_length = 20]
        username -> Varchar,
        #[max_length = 20]
        planned_time -> Nullable<Varchar>,
    }
}

diesel::joinable!(log -> tasks (task_id));
diesel::joinable!(project_apps -> apps (app_id));
diesel::joinable!(project_apps -> projects (project_id));
diesel::joinable!(tasks -> projects (project_id));

diesel::allow_tables_to_appear_in_same_query!(
    apps,
    log,
    project_apps,
    projects,
    tasks,
);
