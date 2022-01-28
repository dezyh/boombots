use std::cmp::Ordering;

use crate::bitboard::Bitboard;
use crate::bitwise::Bitwise;
use crate::constants::*;
use crate::format::Format;
use tinyvec::{Array, ArrayVec};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Action {
    pub source: u8,
    pub target: u8,
    pub robots: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq)]
pub struct ScoredAction {
    pub action: Action,
    pub score: u16,
}

impl PartialOrd for ScoredAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredAction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialEq for ScoredAction {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Forward,
    Backward,
    Left,
    Right,
}

// Precomputations
//let allies_attack_range = Bitwise::adj_any(bitboard.board[bitboard.turn]);
// let enemies_attack_range = Bitwise::adj_any(bitboard.board[opponent]);
// let enemies_unattacked =
//    bitboard.board[opponent] & !Bitwise::adj_any(bitboard.board[bitboard.turn]);
//let allies_attacked = bitboard.board[bitboard.turn] & enemies_attack_range;
//let allies_stacked = bitboard.board[bitboard.turn] & !bitboard.board[1];
//let enemies_stacked = bitboard.board[opponent] & !bitboard.board[1];

impl Action {
    fn checking(bitboard: &Bitboard, action: &Action) -> bool {
        // Get all opponent robots
        let opps = bitboard.board[bitboard.opponent];
        // Get all squares that are a check on an opponents robot
        let checks = Bitwise::adj_any(opps);
        // Calculate the position of the action target index
        let target = Bitwise::pos(action.target);
        // Check if our target matches a target position that is a check
        target & checks != 0
    }

    fn booming(action: &Action) -> bool {
        action.robots == 0
    }

    fn stacking(bitboard: &Bitboard, action: &Action) -> u16 {
        let bots = bitboard.board[bitboard.turn];
        let target = Bitwise::pos(action.target);
        // Ensure this is not a boom and that we are stacking
        if action.robots != 0 && bots & target != 0 {
            return (bitboard.height(target) + action.robots) as u16;
        }
        0
    }

    fn direction(bitboard: &Bitboard, action: &Action) -> Direction {
        match action.source < action.target {
            true => match action.target - action.source <= 7 {
                true => Direction::Left,
                false => match bitboard.turn {
                    WHITE => Direction::Forward,
                    BLACK => Direction::Backward,
                    _ => panic!("Never reaches here"),
                },
            },
            false => match action.source - action.target <= 7 {
                true => Direction::Right,
                false => match bitboard.turn {
                    WHITE => Direction::Backward,
                    BLACK => Direction::Forward,
                    _ => panic!("Never reaches here"),
                },
            },
        }
    }

    fn distance_forward(action: &Action) -> u16 {
        match action.source < action.target {
            true => (action.target - action.source) as u16 / 8,
            false => (action.source - action.target) as u16 / 8,
        }
    }

    fn distance_sideways(action: &Action) -> u16 {
        match action.source < action.target {
            true => (action.target - action.source) as u16,
            false => (action.source - action.target) as u16,
        }
    }

    pub fn generate(bitboard: &Bitboard, pv_move: Option<Action>) -> ArrayVec<[ScoredAction; 256]> {
        let mut actions = ArrayVec::<[ScoredAction; 256]>::new();

        Action::generate_booms(&bitboard, &mut actions);
        for height in (1..=12).rev() {
            Action::generate_for_height(&bitboard, height, &mut actions);
        }

        Action::sort(&bitboard, &mut actions, pv_move);

        actions
    }

    /// Sort our actions in space based on a heuristic
    // ------------------------------------------------------------------------------------------------
    // General
    // 1. If we are ever up in pieces, it is in our interested to spread pieces and take 1 for 1
    //    trades.
    // 2. If material is even, we should prioritize stacking, because it could lead to a better
    //    gamesate for us.
    // 3. If material is worse, we wouldn't have gone down this path unless there was an attacking
    //    option that yeilded an even or winning gamestate.
    // ------------------------------------------------------------------------------------------------
    // Booms
    // 1. We don't need to determine how good a boom is because it is only 1 move deep until we do
    //    that anyway and then the evaluation function will tell us.
    // 2. Thus, we should always explore booms and never prune them from the search tree.
    // ------------------------------------------------------------------------------------------------
    // Stacking
    // 1. Use this stacking order, powers of 2, then even, then odd. 8, 4, 2, 6, 7, 5, 3.
    // 2. It's probably better to stack powers of 2 and then even numbers because otherwise you're wasting turns to build such stacks.
    // 3. We will ignore stacking past 8. This is because, stacks of 8 can cross the entire
    //    board. We could even try stopping at 7 but stacks of 8 are probably more common due
    //    to being a power of 2.
    // 4. Stacking 5, 6, 7, 8 in the very back row is awarded more than stacking 5, 6, 7, 8 in the front row because the front row is wasting reach.
    // ------------------------------------------------------------------------------------------------
    // Catapults
    // 1. Landing next to 0 enemies is not very efficient, it is better to catapult towards 1 or
    //    even better, 2 enemies as 1 enemy is like check while 2 enemies is checkmate, a boom
    //    cannot be avoided, it also gains a tempo, because the opponent has to evade.
    // 2. These moves are usually forward facing. We should explore send 1, 2, 3 units towards the
    //    enemy. Sending more than 3 units is probably never a good idea.
    //
    // ------------------------------------------------------------------------------------------------
    // Repositioning
    // 1. Moving a large stack to the other side of the board could be very useful so that we can
    //    target weak spots when our opponents are not very mobile.
    // 2. Moving the whole stack 6, 5, 4, 3, 2, 1 positions adjacently
    // 3. Moving backwards is never really a good idea
    // =  Moving X units Y distance adjacently = X * Y score, X and Y are capped at 6
    // ------------------------------------------------------------------------------------------------
    // Shuffle / Encroach
    fn sort(
        bitboard: &Bitboard,
        actions: &mut ArrayVec<[ScoredAction; 256]>,
        pv_move: Option<Action>,
    ) {
        actions.iter_mut().for_each(|scored_action| {
            // Score depending on the direction and distance moved
            match Action::direction(&bitboard, &scored_action.action) {
                Direction::Forward => {
                    scored_action.score += 15 * Action::distance_sideways(&scored_action.action)
                }
                Direction::Left | Direction::Right => {
                    scored_action.score += 5 * Action::distance_sideways(&scored_action.action)
                }
                Direction::Backward => {}
            }

            // Score depending on stacking
            scored_action.score += 10 * Action::stacking(&bitboard, &scored_action.action);

            // Score depending on booming
            if Action::booming(&scored_action.action) {
                scored_action.score += 500;
            }

            // Score depending on checking
            if Action::checking(&bitboard, &scored_action.action) {
                scored_action.score += 300;
            }

            if let Some(action) = pv_move {
                if scored_action.action == action {
                    scored_action.score += 1000;
                }
            }
        });

        actions.sort_unstable_by(|a, b| b.cmp(a));
    }

    /// Generates all good booms.
    // NOTE: Only considers booms which could have a positive net value to the turn player.
    // This is decided by ensuring that the boom takes at least 1 enemy robot with it and
    // calculates this by checking for adjacent enemy robots to the source.
    // NOTE: Multiple booms which could result in the same gamestate, could be eliminated by calculating islands within
    // possible boom sources, but hopefully the TT will eliminate us having to
    // search these branches (unless the TT gets overwritten).
    // NOTE: It might be smart to value booms higher in the TT as their might often be multiple
    // booms which lead the same game state, and thus reducing lots of branching.
    fn generate_booms(bitboard: &Bitboard, generated: &mut ArrayVec<[ScoredAction; 256]>) {
        let bots = bitboard.board[bitboard.turn];
        let opps = bitboard.board[bitboard.opponent];

        // Find all possible boom locations that could provide net value. This only occurs when the
        // boom source is touching an enemy robot.
        let mut sources = bots & Bitwise::adj_any(opps);

        // We could compute the boom islands to elimite booms that would lead to the same game
        // state but hopefully the transition table will solve this issue for us.

        while sources != 0 {
            // Pop a source from our sources
            let source_pos = Bitwise::lsb(sources);
            sources ^= source_pos;

            // Calculate the source position index
            let source_idx = Bitwise::idx(source_pos) as u8;

            // Store the boom
            generated.push(ScoredAction {
                score: 0,
                action: Action { source: source_idx, target: 0, robots: 0 },
            });
        }
    }

    fn generate_for_height(
        bitboard: &Bitboard,
        height: usize,
        generated: &mut ArrayVec<[ScoredAction; 256]>,
    ) {
        // Get all the turn players robots at the given height
        let mut bots = bitboard.board[height] & bitboard.board[bitboard.turn];

        // If there is none, we can return early
        if bots == 0 {
            return;
        }

        // Otherwise
        while bots != 0 {
            // Pick a bot to generate moves for and remove it from the queue
            let source = Bitwise::lsb(bots);
            bots ^= source;

            // Generate all possible target squares using a lookup table
            let source_pos = Bitwise::idx(source);
            let mut actions = MOVES_LOOKUP[height][source_pos];
            // Remove all invalid positions to move to which are those occupied by opponents robots
            actions &= !bitboard.board[bitboard.opponent];

            while actions != 0 {
                // Pick a target from the available target space and remove it from the space for
                // future iterations
                let target = Bitwise::lsb(actions);
                actions ^= target;

                // Convert the target square position into an index between 0-63
                let target_pos = Bitwise::idx(target);

                // For each robot stack size
                for moved in 1..=height {
                    generated.push(ScoredAction {
                        score: 0,
                        action: Action {
                            source: source_pos as u8,
                            target: target_pos as u8,
                            robots: moved as u8,
                        },
                    });
                }
            }
        }
    }

    fn is_attack(target_adj: u64, enemies_unattacked: u64) -> bool {
        (target_adj & enemies_unattacked) != 0
    }

    fn is_defend(source_adj: u64, target_adj: u64, enemies_attack_range: u64) -> bool {
        (source_adj & enemies_attack_range) & (!target_adj & enemies_attack_range) != 0
    }

    fn is_stack(bitboard: &Bitboard, target: u64) -> bool {
        bitboard.board[bitboard.turn] & target != 0
    }
}
