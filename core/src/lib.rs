use serde::{Deserialize, Serialize};
pub mod net;

#[macro_use]
extern crate serde_big_array;
big_array! { BigArray; }

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Team {
    White,
    Black,
}

impl Team {
    pub fn next(&self) -> Team {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Bot {
    pub team: Team,
    pub stack: u8,
}

impl Bot {
    pub fn new(team: Team, stack: u8) -> Bot {
        Bot { team, stack }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct GameState {
    pub turn: Team,
    #[serde(with = "BigArray")]
    pub board: [Option<Bot>; 64],
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Pos {
    pub x: u8,
    pub y: u8,
}

impl Pos {
    pub fn new(x: u8, y: u8) -> Pos {
        Pos { x, y }
    }

    pub fn index(&self) -> usize {
        (self.x as usize) + 8 * (self.y as usize)
    }

    pub fn valid(&self) -> bool {
        self.x <= 7 && self.y <= 7
    }
}

impl GameState {
    fn get(&self, pos: Pos) -> Option<Bot> {
        self.board[pos.index()]
    }

    fn set(&mut self, pos: Pos, bot: Option<Bot>) {
        self.board[pos.index()] = bot;
    }

    // Booms a location on the board
    fn _boom(&mut self, pos: Pos) -> bool {
        match self.get(pos) {
            Some(_) => {
                self.set(pos, None);
                true
            }
            None => false,
        }
    }

    // Booms a position on the board and then, if there was a robot that boomed at that position,
    // store that boom in the chain for future chain booming.
    fn boom_chain(&mut self, pos: Pos, chain: &mut Vec<Pos>) {
        if let Some(_boom) = self.get(pos) {
            self.set(pos, None);
            chain.push(pos);
        }
    }

    pub fn new() -> GameState {
        let mut gs = GameState {
            turn: Team::White,
            board: [None; 64],
        };

        for x in 0..=7 {
            for y in 0..=1 {
                gs.set(Pos::new(x, y), Some(Bot::new(Team::White, 1)));
            }
            for y in 6..=7 {
                gs.set(Pos::new(x, y), Some(Bot::new(Team::Black, 1)));
            }
        }

        gs
    }

    pub fn valid(&self, action: &Action) -> bool {
        let source = self.get(action.a);
        let target = self.get(action.b);
        match (source, target, action.n) {
            // Boom
            (Some(source), _, 0) => GameState::valid_boom(self.turn, &source),
            // Stack
            (Some(source), Some(target), _) => {
                GameState::valid_stack(self.turn, &source, &target, &action)
            }
            // Move
            (Some(source), None, _) => GameState::valid_move(self.turn, &source, &action),
            // Invalid
            (None, _, _) => false,
        }
    }

    fn absdiff(a: u8, b: u8) -> u8 {
        ((a as i8) - (b as i8)).abs() as u8
    }

    // Checks if a stacking move is valid
    fn valid_stack(turn: Team, source: &Bot, target: &Bot, action: &Action) -> bool {
        // Calculate movement distances in x and y axis
        let xd = GameState::absdiff(action.a.x, action.b.x);
        let yd = GameState::absdiff(action.a.y, action.b.y);

        // Only allow moving robots that belong to the turn player
        let valid_source_team = source.team == turn;
        // Only allow stacking robots onto other turn player robots
        let valid_stack_team = source.team == target.team;
        // Only allow orthongonal movement
        let valid_move_direction = xd == 0 || yd == 0;
        // Only allow moving a distance less than or equal to the number of bots at the source but
        // always more than 0 as this indicates a boom
        let valid_move_distance = xd + yd > 0 && xd + yd <= source.stack;
        // Only allow moving up to as many bots as were at the source
        let valid_move_size = action.n <= source.stack;

        valid_source_team
            && valid_stack_team
            && valid_move_direction
            && valid_move_distance
            && valid_move_size
    }

    // Checks if a moving move is valid
    fn valid_move(turn: Team, source: &Bot, action: &Action) -> bool {
        // Calculate movement distances in x and y axis
        let xd = GameState::absdiff(action.a.x, action.b.x);
        let yd = GameState::absdiff(action.a.y, action.b.y);

        // Only allow moving robots that belong to the turn player
        let valid_source_team = source.team == turn;
        // Only allow orthongonal movement
        let valid_move_direction = xd == 0 || yd == 0;
        // Only allow moving a distance less than or equal to the number of bots at the source but
        // always more than 0 as this indicates a boom
        let valid_move_distance = xd + yd > 0 && xd + yd <= source.stack;
        // Only allow moving up to as many bots as were at the source
        let valid_move_size = action.n <= source.stack;

        valid_source_team && valid_move_direction && valid_move_distance && valid_move_size
    }

    fn valid_boom(turn: Team, source: &Bot) -> bool {
        // Only allow booming robots that belong to the turn player
        let valid_source_team = source.team == turn;

        valid_source_team
    }

    pub fn make(&mut self, action: &Action) {
        let source = self.get(action.a);
        let target = self.get(action.b);
        match (source, target, action.n) {
            // Boom
            (Some(_source), _, 0) => self.make_boom(&action),
            // Stack
            (Some(_source), Some(_target), _) => self.make_stack(&action),
            // Move
            (Some(_source), None, _) => self.make_move(&action),
            // Invalid
            (None, _, _) => {}
        }
        self.turn = self.turn.next();
    }

    fn make_boom(&mut self, action: &Action) {
        let mut booms: Vec<Pos> = vec![action.a];

        while !booms.is_empty() {
            let boom = booms.pop().expect("no booms could be popped");
            match (boom.x, boom.y) {
                // Bottom-Left Corner
                (0, 0) => {
                    self.boom_chain(Pos::new(0, 0), &mut booms);
                    self.boom_chain(Pos::new(0, 1), &mut booms);
                    self.boom_chain(Pos::new(1, 0), &mut booms);
                    self.boom_chain(Pos::new(1, 1), &mut booms);
                }
                // Top-Left Corner
                (0, 7) => {
                    self.boom_chain(Pos::new(0, 7), &mut booms);
                    self.boom_chain(Pos::new(0, 6), &mut booms);
                    self.boom_chain(Pos::new(1, 7), &mut booms);
                    self.boom_chain(Pos::new(1, 6), &mut booms);
                }
                // Bottom-Right Corner
                (7, 0) => {
                    self.boom_chain(Pos::new(7, 0), &mut booms);
                    self.boom_chain(Pos::new(7, 1), &mut booms);
                    self.boom_chain(Pos::new(6, 0), &mut booms);
                    self.boom_chain(Pos::new(6, 1), &mut booms);
                }
                // Top-Right Corner
                (7, 7) => {
                    self.boom_chain(Pos::new(7, 7), &mut booms);
                    self.boom_chain(Pos::new(7, 6), &mut booms);
                    self.boom_chain(Pos::new(6, 7), &mut booms);
                    self.boom_chain(Pos::new(6, 6), &mut booms);
                }
                // Left edge
                (0, y) => {
                    self.boom_chain(Pos::new(0, y - 1), &mut booms);
                    self.boom_chain(Pos::new(0, y), &mut booms);
                    self.boom_chain(Pos::new(0, y + 1), &mut booms);
                    self.boom_chain(Pos::new(1, y - 1), &mut booms);
                    self.boom_chain(Pos::new(1, y), &mut booms);
                    self.boom_chain(Pos::new(1, y + 1), &mut booms);
                }
                // Bottom edge
                (x, 0) => {
                    self.boom_chain(Pos::new(x - 1, 0), &mut booms);
                    self.boom_chain(Pos::new(x, 0), &mut booms);
                    self.boom_chain(Pos::new(x + 1, 0), &mut booms);
                    self.boom_chain(Pos::new(x - 1, 1), &mut booms);
                    self.boom_chain(Pos::new(x, 1), &mut booms);
                    self.boom_chain(Pos::new(x + 1, 1), &mut booms);
                }
                // Right edge
                (7, y) => {
                    self.boom_chain(Pos::new(7, y - 1), &mut booms);
                    self.boom_chain(Pos::new(7, y), &mut booms);
                    self.boom_chain(Pos::new(7, y + 1), &mut booms);
                    self.boom_chain(Pos::new(6, y - 1), &mut booms);
                    self.boom_chain(Pos::new(6, y), &mut booms);
                    self.boom_chain(Pos::new(6, y + 1), &mut booms);
                }
                // Top edge
                (x, 7) => {
                    self.boom_chain(Pos::new(x - 1, 7), &mut booms);
                    self.boom_chain(Pos::new(x, 7), &mut booms);
                    self.boom_chain(Pos::new(x + 1, 7), &mut booms);
                    self.boom_chain(Pos::new(x - 1, 6), &mut booms);
                    self.boom_chain(Pos::new(x, 6), &mut booms);
                    self.boom_chain(Pos::new(x + 1, 6), &mut booms);
                }
                (x, y) => {
                    self.boom_chain(Pos::new(x - 1, y - 1), &mut booms);
                    self.boom_chain(Pos::new(x - 1, y), &mut booms);
                    self.boom_chain(Pos::new(x - 1, y + 1), &mut booms);
                    self.boom_chain(Pos::new(x, y - 1), &mut booms);
                    self.boom_chain(Pos::new(x, y), &mut booms);
                    self.boom_chain(Pos::new(x, y + 1), &mut booms);
                    self.boom_chain(Pos::new(x + 1, y - 1), &mut booms);
                    self.boom_chain(Pos::new(x + 1, y), &mut booms);
                    self.boom_chain(Pos::new(x + 1, y + 1), &mut booms);
                }
            }
        }
    }

    fn make_stack(&mut self, action: &Action) {
        let source = self.get(action.a).unwrap();
        let target = self.get(action.b).unwrap();

        match source.stack - action.n {
            0 => {
                // Stack all
                let new_source = None;
                let new_target = Some(Bot::new(source.team, action.n + target.stack));
                self.set(action.a, new_source);
                self.set(action.b, new_target);
            }
            _ => {
                // Stack subset
                let new_source = Some(Bot::new(source.team, source.stack - action.n));
                let new_target = Some(Bot::new(source.team, action.n + target.stack));
                self.set(action.a, new_source);
                self.set(action.b, new_target);
            }
        }
    }

    fn make_move(&mut self, action: &Action) {
        let source = self.get(action.a).unwrap();

        match source.stack - action.n {
            0 => {
                // Move all
                let new_source = None;
                let new_target = Some(source);
                self.set(action.a, new_source);
                self.set(action.b, new_target);
            }
            _ => {
                // Move subset
                let new_source = Some(Bot::new(source.team, source.stack - action.n));
                let new_target = Some(Bot::new(source.team, action.n));
                self.set(action.a, new_source);
                self.set(action.b, new_target);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Action {
    // Source position
    pub a: Pos,
    // Target position
    pub b: Pos,
    // Number of robots with 0=boom
    pub n: u8,
}
