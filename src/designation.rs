#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Designation {
    Generic = 1,
    Episode = 2,
    Movie  = 3,
}

pub fn convert_i32_to_designation(input: i32) -> Designation {
    match input {
        1 => {
            return Designation::Generic;
        },
        2 => {
            return Designation::Episode;
        },
        3 => {
            return Designation::Movie;
        },
        _ => {
            return Designation::Generic;
        }
    }
}