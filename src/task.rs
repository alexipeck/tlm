#[derive(Clone, Debug)]
pub enum Task {
    Encode = 0,
    Copy = 1,
    Move = 2,
    Rename = 3,
    Reserve = 4,
    Delete = 5,
    Reencode = 6,
    Duplicate = 7,
}

pub fn convert_task_id_to_task(task_id: usize) -> Task {
    match task_id {
        0 => {
            return Task::Encode;
        }
        1 => {
            return Task::Copy;
        }
        2 => {
            return Task::Move;
        }
        3 => {
            return Task::Rename;
        }
        4 => {
            return Task::Reserve;
        }
        5 => {
            return Task::Delete;
        }
        6 => {
            return Task::Reencode;
        }
        7 => {
            return Task::Duplicate;
        }
        _ => {
            panic!("Not valid task ID");
        }
    }
}
