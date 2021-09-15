//!Enum for identifying whether a generic file is owned
//!by something more complicated like an Episode or Movie
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Designation {
    Generic = 1,
    Episode = 2,
    Movie = 3,
}

pub fn convert_i32_to_designation(input: i32) -> Designation {
    match input {
        1 => Designation::Generic,
        2 => Designation::Episode,
        3 => Designation::Movie,
        _ => Designation::Generic,
    }
}
