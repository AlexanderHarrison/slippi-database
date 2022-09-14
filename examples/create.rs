use slippi_situation_parser::{parse_game, Port};
use slippi_database::*;
use std::io::Read;

macro_rules! unwrap_or {
    ($opt:expr, $else:expr) => {
        match $opt {
            Some(data) => data,
            None => $else,
        }
    }
}

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    let db_path = std::path::Path::new("./slippi_tutor.sqlite");
    let mut db = init_or_open_db(&db_path);

    let file_count = path.read_dir().expect("path is not a directory")
        .count();

    for (i, f) in path.read_dir().unwrap().enumerate() {
        let f = f.unwrap();
        println!("({}/{}): {}", i, file_count, f.file_name().into_string().unwrap());
        let path = f.path();

        let mut slippi_file = std::fs::File::open(path).expect("error opening slippi file");
        let mut buf = Vec::new();
        slippi_file.read_to_end(&mut buf).unwrap();

        let low_parsed = unwrap_or!(parse_buf(&buf, Port::Low), continue);
        let high_parsed = unwrap_or!(parse_buf(&buf, Port::High), continue);

        let rows = generate_rows_from_game(&low_parsed, &high_parsed);
        add_rows_to_db(rows.into_iter().map(|r| r.into()), &mut db);
        let rows = generate_rows_from_game(&high_parsed, &low_parsed);
        add_rows_to_db(rows.into_iter().map(|r| r.into()), &mut db);
    }
}

