extern crate tlm;
extern crate diesel;

use self::tlm::*;
use self::model::*;
use self::diesel::prelude::*;

fn main() {
    use tlm::schema::content::dsl::*;

    let connection = establish_connection();
    
    let x: String = String::from(r"D:\Desktop\tlmfiles\Better Call Saul\Season 2\Better Call Saul - S02E01 - Switch Bluray-1080p.mkv");
    let y: i32 = 0;
    let mut new_content: Vec<ContentModel> = Vec::new();
    for i in 0..8000 {
        new_content.push(create_content(&connection, x.clone(), y));
    }
    
    let results = content.filter(content_uid.gt(-1))
        .load::<ContentModel>(&connection)
        .expect("Error loading posts");
    println!("Displaying {} posts", results.len());
    for output in results {
        println!("{}", output.full_path);
        println!("----------\n");
    }
}