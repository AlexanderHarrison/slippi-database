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

use slippi_situation_parser::{Action, states::HighLevelAction};
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
pub struct RowRef<'a> {
    pub opponent_initiation: &'a Action,
    pub player_response: &'a Action,
    pub response_delay: usize,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub opponent_initiation: Action,
    pub player_response: Action,
    pub response_delay: usize,
}

#[derive(Clone, Debug)]
pub struct DBRow {
    pub init_pos_x  : u16,
    pub init_pos_y  : u16,
    pub init_vel_x  : u16,
    pub init_vel_y  : u16,
    pub resp_pos_x  : u16,
    pub resp_pos_y  : u16,
    pub resp_vel_x  : u16,
    pub resp_vel_y  : u16,
    pub init_state  : u8,
    pub resp_state  : u8,
    pub init_action : u16,
    pub resp_action : u16,
    pub delay       : u16,
}

pub fn add_rows_to_db<'a, 'b, R>(rows: R, db: &mut Connection) where
    R: Iterator<Item=DBRow> + 'a,
{
    let (_, row_count) = rows.size_hint();
    
    let mut st = db.prepare( "INSERT INTO Fox_Fox VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)").unwrap();
    for (i, row) in rows.enumerate() {
        if let Some(r) = row_count {
            print!("\r({}/{})", i+1, r);
        } else {
            print!("\r({})", i+1);
        }

        use rusqlite::params;
        st.execute(
            params![
                row.init_pos_x ,
                row.init_pos_y ,
                row.init_vel_x ,
                row.init_vel_y ,
                row.resp_pos_x ,
                row.resp_pos_y ,
                row.resp_vel_x ,
                row.resp_vel_y ,
                row.init_state ,
                row.resp_state ,
                row.init_action,
                row.resp_action,
                row.delay      ,
            ]
        ).expect("error inserting into db");
    }
    println!();
}

pub fn init_or_open_db(path: &std::path::Path) -> Connection {
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

pub fn generate_rows_from_game<'a>(mut player_actions: &'a [Action], mut opponent_actions: &'a [Action]) -> Vec<RowRef<'a>> {
    let mut rows = Vec::new();

    let (mut initiation, new_opp_actions) = unwrap_or!(opponent_actions.split_first(), return rows);
    opponent_actions = new_opp_actions;

    let (mut response, new_pla_actions) = unwrap_or!(player_actions.split_first(), return rows);
    player_actions = new_pla_actions;

    loop {
        while response.frame_start <= initiation.frame_start {
            (response, player_actions) = unwrap_or!(player_actions.split_first(), return rows);
        }

        rows.push(RowRef { 
            player_response: response,
            opponent_initiation: initiation,
            response_delay: response.frame_start - initiation.frame_start,
        });

        (initiation, opponent_actions) = unwrap_or!(opponent_actions.split_first(), break);
    }
        
    rows
}

impl<'a> Into<DBRow> for RowRef<'a> {
    fn into(self) -> DBRow {
        let init = self.opponent_initiation;
        let resp = self.player_response;
        let delay = self.response_delay;

        assert_eq!(std::mem::size_of::<HighLevelAction>(), 2);

        DBRow {
            init_pos_x : init.initial_position.x.round() as u16,
            init_pos_y : init.initial_position.y.round() as u16,
            init_vel_x : (init.initial_velocity.x * 10.0).round() as u16,
            init_vel_y : (init.initial_velocity.y * 10.0).round() as u16,
            resp_pos_x : resp.initial_position.x.round() as u16,
            resp_pos_y : resp.initial_position.y.round() as u16,
            resp_vel_x : (resp.initial_velocity.x * 10.0).round() as u16,
            resp_vel_y : (resp.initial_velocity.y * 10.0).round() as u16,
            init_state : init.actionable_state as u8,
            resp_state : resp.actionable_state as u8,
            init_action: unsafe { std::mem::transmute::<_, u16>(init.action_taken) },
            resp_action: unsafe { std::mem::transmute::<_, u16>(resp.action_taken) },
            delay: delay as u16,
        }
    }
}

impl Into<DBRow> for Row {
    fn into(self) -> DBRow {
        let init = self.opponent_initiation;
        let resp = self.player_response;
        let delay = self.response_delay;

        assert_eq!(std::mem::size_of::<HighLevelAction>(), 2);

        DBRow {
            init_pos_x : init.initial_position.x.round() as u16,
            init_pos_y : init.initial_position.y.round() as u16,
            init_vel_x : (init.initial_velocity.x * 10.0).round() as u16,
            init_vel_y : (init.initial_velocity.y * 10.0).round() as u16,
            resp_pos_x : resp.initial_position.x.round() as u16,
            resp_pos_y : resp.initial_position.y.round() as u16,
            resp_vel_x : (resp.initial_velocity.x * 10.0).round() as u16,
            resp_vel_y : (resp.initial_velocity.y * 10.0).round() as u16,
            init_state : init.actionable_state as u8,
            resp_state : resp.actionable_state as u8,
            init_action: unsafe { std::mem::transmute::<_, u16>(init.action_taken) },
            resp_action: unsafe { std::mem::transmute::<_, u16>(resp.action_taken) },
            delay: delay as u16,
        }
    }
}

impl<'a> Into<Row> for RowRef<'a> {
    fn into(self) -> Row {
        Row {
            opponent_initiation: self.opponent_initiation.clone(),
            player_response: self.player_response.clone(),
            response_delay: self.response_delay,
        }
    }
}
