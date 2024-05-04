
diesel::table! {
    tasks (task_id) {
        task_id -> Int4,
        task_name -> Text,
        username -> Text,
        planned_time->Nullable<Varchar>
    }
}

diesel::table! {
    apps (app_id) {
        app_id -> Int4,
        app_name -> Text,
    }
}

diesel::table! {
    task_apps (id) {
        id->Int4,
        task_id->Int4,
        app_id -> Int4,
    }
}

diesel::table! {
    log (log_id) {
        log_id->Int4,
        task_id->Int4,
        log_type -> Varchar,
        date->Timestamp,

    }
}