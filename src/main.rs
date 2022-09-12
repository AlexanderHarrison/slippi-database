// tablebase
// each row contains:
//
// - stage
//
// player:
// - position x
// - position y
// - velocity x
// - velocity y
// - action state
// - action taken
//
// opponent:
// - position x
// - position y
// - velocity x
// - velocity y
// - action state
// - action state duration

use slippi_situation_parser::{parse_game, Port, Action, states::HighLevelAction};
use rusqlite::Connection;

macro_rules! unwrap_or {
    ($opt:expr, $else:expr) => {
        match $opt {
            Some(data) => data,
            None => $else,
        }
    }
}

#[derive(Clone, Debug)]
struct Row<'a> {
    pub opponent_initiation: &'a Action,
    pub player_responce: &'a Action,
    pub responce_delay: usize,
}

fn main() {
    let path = std::env::args_os().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);

    let db_path = std::path::Path::new("./fox_ditto_db.sqlite");
    let mut db = init_or_open_db(&db_path);

    let file_count = path.read_dir().expect("path is not a directory")
        .count();

    for (i, f) in path.read_dir().unwrap().enumerate() {
        let f = f.unwrap();
        println!("({}/{}): {}", i, file_count, f.file_name().into_string().unwrap());
        let path = f.path();

        let low_parsed = unwrap_or!(parse_game(&path, Port::Low), continue);
        let high_parsed = unwrap_or!(parse_game(&path, Port::High), continue);

        let rows = generate_rows_from_game(&low_parsed, &high_parsed);
        add_rows_to_db(&rows, &mut db);
        let rows = generate_rows_from_game(&high_parsed, &low_parsed);
        add_rows_to_db(&rows, &mut db);
    }
}

fn add_rows_to_db<'a, 'b>(rows: &'a [Row<'b>], db: &mut Connection) {
    let row_count = rows.len();
    let mut st = db.prepare( "INSERT INTO Fox_Fox VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)").unwrap();
    for (i, row) in rows.iter().enumerate() {
        print!("\r({}/{})", i+1, row_count);
        //pos * 1,
        //vel * 10
        use rusqlite::params;
        let init = row.opponent_initiation;
        let resp = row.player_responce;
        let delay = row.responce_delay;

        assert_eq!(std::mem::size_of::<HighLevelAction>(), 2);

        st.execute(
            params![
                init.initial_position.x.round() as u16,
                init.initial_position.y.round() as u16,
                (init.initial_velocity.x * 10.0).round() as u16,
                (init.initial_velocity.y * 10.0).round() as u16,
                resp.initial_position.x.round() as u16,
                resp.initial_position.y.round() as u16,
                (resp.initial_velocity.x * 10.0).round() as u16,
                (resp.initial_velocity.y * 10.0).round() as u16,
                init.actionable_state as u8,
                resp.actionable_state as u8,
                unsafe { std::mem::transmute::<_, u16>(init.action_taken) },
                unsafe { std::mem::transmute::<_, u16>(resp.action_taken) },
                delay
            ]
        ).expect("error inserting into db");
    }
    println!();
}

fn init_or_open_db(path: &std::path::Path) -> Connection {
    if path.exists() {
        Connection::open(path).expect("error opening database")
    } else {
        let connection = Connection::open(path).expect("error opening database");
        connection.execute(
            "CREATE TABLE Fox_Fox (
                InitPosx   MEDIUMINT,
                InitPosy   MEDIUMINT,
                InitVelx   MEDIUMINT,
                InitVely   MEDIUMINT,
                RespPosx   MEDIUMINT,
                RespPosy   MEDIUMINT,
                RespVelx   MEDIUMINT,
                RespVely   MEDIUMINT,
                InitState  SMALLINT,
                RespState  SMALLINT,
                InitAction MEDIUMINT,
                RespAction MEDIUMINT,
                Delay      MEDIUMINT
            )", 
            [],
        ).unwrap();

        connection
    }
}

fn generate_rows_from_game<'a>(mut player_actions: &'a [Action], mut opponent_actions: &'a [Action]) -> Vec<Row<'a>> {
    let mut rows = Vec::new();

    let (mut initiation, new_opp_actions) = unwrap_or!(opponent_actions.split_first(), return rows);
    opponent_actions = new_opp_actions;

    let (mut responce, new_pla_actions) = unwrap_or!(player_actions.split_first(), return rows);
    player_actions = new_pla_actions;

    loop {
        while responce.frame_start <= initiation.frame_start {
            (responce, player_actions) = unwrap_or!(player_actions.split_first(), return rows);
        }

        rows.push(Row { 
            player_responce: responce,
            opponent_initiation: initiation,
            responce_delay: responce.frame_start - initiation.frame_start,
        });

        (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), break);
    }
        
    rows
}

