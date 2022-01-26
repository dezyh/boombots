use crate::bitboard::Bitboard;
use crate::bitwise::Bitwise;
use crate::constants::*;
use crate::format::Format;

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub source: u8,
    pub target: u8,
    pub robots: u8,
}

// TODO: Optimize heap allocations away
pub struct Actions {
    pub attacks: Vec<Action>,
    pub defends: Vec<Action>,
    pub others: Vec<Action>,
    pub special: Vec<Action>,
}

impl Actions {
    pub fn new() -> Self {
        Self { attacks: Vec::new(), defends: Vec::new(), others: Vec::new(), special: Vec::new() }
    }
}

impl Action {
    pub fn generate(bitboard: &Bitboard) -> Vec<Action> {
        let mut actions = Actions::new();
        for height in (1..=12).rev() {
            Action::generate_for_height(bitboard, height, &mut actions);
        }
        actions.special.append(&mut actions.attacks);
        actions.special.append(&mut actions.defends);
        actions.special.append(&mut actions.others);
        actions.special
    }

    fn generate_for_height(bitboard: &Bitboard, height: usize, generated: &mut Actions) {
        let mut bots = bitboard.board[height] & bitboard.board[bitboard.turn];
        let opponent = if bitboard.turn == WHITE { BLACK } else { WHITE };

        // Precomputations
        //let allies_attack_range = Bitwise::adj_any(bitboard.board[bitboard.turn]);
        let enemies_attack_range = Bitwise::adj_any(bitboard.board[opponent]);
        let enemies_unattacked =
            bitboard.board[opponent] & !Bitwise::adj_any(bitboard.board[bitboard.turn]);
        //let allies_attacked = bitboard.board[bitboard.turn] & enemies_attack_range;
        //let allies_stacked = bitboard.board[bitboard.turn] & !bitboard.board[1];
        //let enemies_stacked = bitboard.board[opponent] & !bitboard.board[1];
        while bots != 0 {
            // Pick a bot to generate moves for and remove it from the queue
            let source = Bitwise::lsb(bots);
            bots ^= source;

            // ???
            let source_adj = Bitwise::adj(source);

            // Generate moves using a lookup table
            let source_pos = Bitwise::idx(source);
            let mut actions = MOVES_LOOKUP[height][source_pos];
            actions &= !bitboard.board[opponent];

            while actions != 0 {
                // Pick a target from the available target space and remove it from the space for
                // future iterations
                let target = Bitwise::lsb(actions);
                actions ^= target;

                let target_adj = Bitwise::adj(target);
                let target_pos = Bitwise::idx(target);

                if Action::is_attack(target_adj, enemies_unattacked) {
                    let mut size = 1;
                    while size <= height && size < 3 {
                        generated.attacks.push(Action {
                            source: source_pos as u8,
                            target: target_pos as u8,
                            robots: size as u8,
                        });
                        size += 1;
                    }
                    while size <= height {
                        generated.others.push(Action {
                            source: source_pos as u8,
                            target: target_pos as u8,
                            robots: size as u8,
                        });
                        size += 1;
                    }
                } else if Action::is_defend(source_adj, target_adj, enemies_attack_range) {
                    let mut size = height;
                    while size > 0 && size >= height - 1 {
                        generated.defends.push(Action {
                            source: source_pos as u8,
                            target: target_pos as u8,
                            robots: size as u8,
                        });
                        size -= 1;
                    }
                    while size > 0 {
                        generated.others.push(Action {
                            source: source_pos as u8,
                            target: target_pos as u8,
                            robots: size as u8,
                        });
                        size -= 1;
                    }
                } else {
                    let mut size = height;
                    while size > 0 {
                        generated.others.push(Action {
                            source: source_pos as u8,
                            target: target_pos as u8,
                            robots: size as u8,
                        });
                        size -= 1;
                    }
                }
            }

            // Add all explosions
            if (source & enemies_attack_range) != 0 {
                generated.attacks.push(Action { source: source_pos as u8, target: 0, robots: 0 });
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
