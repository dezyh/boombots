use crate::action::Action;
use crate::bitboard::Bitboard;
use crate::bitwise::Bitwise;
use crate::constants::*;
use crate::evaluate::{Evaluate, Outcome};
use crate::format::Format;
use crate::transpose::{Bound, TranspositionTable};
use std::cmp::{max, min};

pub struct NegamaxResult {
    pub depth: u8,
    pub action: Action,
    pub score: i16,
    pub nodes: u64,
    pub trans: u64,
}

pub struct NegamaxStats {
    pub score: i16,
    pub nodes: u64,
    pub trans: u64,
}

impl Search {
    /// Set alpha to LOSS and beta to WIN for white
    pub fn negamax_move(
        bitboard: &mut Bitboard,
        transpositions: &mut TranspositionTable,
        depth: u8,
        alpha: i16,
        beta: i16,
    ) -> NegamaxResult {
        let mut alpha = alpha;
        let original_alpha = alpha;
        let actions = Action::generate(&bitboard);
        let mut best_score = LOSS;
        let mut best_action = actions.first().expect("No actions");
        let mut aggregate = NegamaxStats { score: LOSS, trans: 0, nodes: 0 };

        for action in &actions {
            let delta = bitboard.delta(action.clone());
            let hash = bitboard.make(&delta);
            let stats = Search::negamax_score(bitboard, transpositions, depth - 1, -beta, -alpha);
            let score = -stats.score;
            aggregate.nodes += stats.nodes;
            aggregate.trans += stats.trans;
            bitboard.undo(&delta, hash);

            // Update the best known evaluation
            if best_score < score {
                best_score = score;
                best_action = action;
                aggregate.score = score;
            }

            // Update our alpha
            if alpha < score {
                alpha = score
            }

            // Check for alpha-beta cut-off
            if alpha >= beta {
                break;
            }
        }

        // Update the transposition table
        if best_score <= original_alpha {
            transpositions.store(
                bitboard.hash,
                best_score,
                best_action.clone(),
                depth,
                Bound::Upper,
            );
        } else if best_score >= beta {
            transpositions.store(
                bitboard.hash,
                best_score,
                best_action.clone(),
                depth,
                Bound::Lower,
            );
        } else {
            transpositions.store(
                bitboard.hash,
                best_score,
                best_action.clone(),
                depth,
                Bound::Exact,
            );
        }

        NegamaxResult {
            depth,
            action: best_action.clone(),
            score: best_score,
            nodes: aggregate.nodes,
            trans: aggregate.trans,
        }
    }

    fn negamax_score(
        bitboard: &mut Bitboard,
        transpositions: &mut TranspositionTable,
        depth: u8,
        alpha: i16,
        beta: i16,
    ) -> NegamaxStats {
        if let Some(outcome) = Evaluate::outcome(&bitboard) {
            match outcome {
                Outcome::Win => return NegamaxStats { score: WIN, nodes: 0, trans: 0 },
                Outcome::Loss => return NegamaxStats { score: LOSS, nodes: 0, trans: 0 },
                Outcome::Draw => return NegamaxStats { score: DRAW, nodes: 0, trans: 0 },
            }
        }

        let mut alpha = alpha;
        let mut beta = beta;
        let original_alpha = alpha;

        let previous = transpositions.lookup(bitboard.hash);
        if let Some(previous) = previous {
            if previous.depth >= depth {
                match previous.bound {
                    Bound::Exact => {
                        return NegamaxStats { score: previous.eval, nodes: 1, trans: 1 };
                    }
                    Bound::Lower => {
                        alpha = max(alpha, previous.eval);
                        if alpha >= beta {
                            return NegamaxStats { score: previous.eval, nodes: 1, trans: 1 };
                        }
                    }
                    Bound::Upper => {
                        beta = min(beta, previous.eval);
                        if alpha >= beta {
                            return NegamaxStats { score: previous.eval, nodes: 1, trans: 1 };
                        }
                    }
                }
            };
        }

        // Evaluate leaf nodes
        if depth == 0 {
            return NegamaxStats { score: Evaluate::evaluate(&bitboard), nodes: 1, trans: 0 };
        }

        // Otherwise keep searching deeper
        let actions = Action::generate(&bitboard);

        if actions.len() == 0 {
            return NegamaxStats { score: LOSS, nodes: 1, trans: 0 };
        }

        let mut best_score = LOSS;
        let mut best_action = actions[0];

        let mut aggregate = NegamaxStats { score: LOSS, nodes: 0, trans: 0 };

        for action in actions {
            let delta = bitboard.delta(action);
            let hash = bitboard.make(&delta);
            let stats = Search::negamax_score(bitboard, transpositions, depth - 1, -beta, -alpha);
            let score = -stats.score;
            aggregate.trans += stats.trans;
            aggregate.nodes += stats.nodes;

            bitboard.undo(&delta, hash);

            if best_score < score {
                best_score = score;
                best_action = action;
                aggregate.score = score;
            }

            if alpha < score {
                alpha = score;
            }
        }

        if best_score <= original_alpha {
            transpositions.store(bitboard.hash, best_score, best_action, depth, Bound::Upper);
        } else if best_score >= beta {
            transpositions.store(bitboard.hash, best_score, best_action, depth, Bound::Upper);
        } else {
            transpositions.store(bitboard.hash, best_score, best_action, depth, Bound::Upper);
        }

        aggregate
    }
}

pub struct Search {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_search_5() {
        let mut tt = TranspositionTable::new(28);
        let mut bb = Bitboard::new();
        let mut result;
        for i in 0..=4 {
            result = Search::negamax_move(&mut bb, &mut tt, i, LOSS, WIN);
            if i == 4 {
                assert_eq!(result.nodes, 85443758);
            }
        }
    }
}
