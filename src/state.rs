use crate::{
    def,
    util,
};

use std::fmt;

const FEN_SQRS_INDEX: usize = 0;
const FEN_PLAYER_INDEX: usize = 1;
const FEN_CAS_RIGHTS_INDEX: usize = 2;
const FEN_ENP_SQR_INDEX: usize = 3;
const LAST_DUP_MOV_DISTANCE: usize = 5;
const MAX_NON_CAP_PLY_COUNT: u16 = 100;

pub struct State {
    pub squares: [u8; def::BOARD_SIZE],
    pub player: u8,
    pub cas_rights: u8,
    pub enp_square: usize,
    pub non_cap_mov_count: u16,

    pub wk_index: usize,
    pub bk_index: usize,

    pub taken_piece_stack: Vec<u8>,
    pub enp_sqr_stack: Vec<usize>,
    pub cas_rights_stack: Vec<u8>,
    pub history_mov_stack: Vec<(usize, usize, u8, u8)>,
    pub non_cap_mov_count_stack: Vec<u16>,
    pub wk_index_stack: Vec<usize>,
    pub bk_index_stack: Vec<usize>,
}

impl State {
    pub fn new(fen_string: &str) -> State {
        let fen_segment_list: Vec<&str> = fen_string.split(" ").collect();
        let (squares, wk_index, bk_index) = get_squares_from_fen(fen_segment_list[FEN_SQRS_INDEX]);
        let player = get_player_from_fen(fen_segment_list[FEN_PLAYER_INDEX]);
        let cas_rights = get_cas_rights_from_fen(fen_segment_list[FEN_CAS_RIGHTS_INDEX]);
        let enp_sqr = get_enp_sqr_from_fen(fen_segment_list[FEN_ENP_SQR_INDEX]);

        State {
            squares: squares,
            player: player,
            cas_rights: cas_rights,
            enp_square: enp_sqr,
            non_cap_mov_count: 0,
            wk_index: wk_index,
            bk_index: bk_index,
            
            taken_piece_stack: Vec::new(),
            enp_sqr_stack: Vec::new(),
            cas_rights_stack: Vec::new(),
            history_mov_stack: Vec::new(),
            non_cap_mov_count_stack: Vec::new(),
            wk_index_stack: Vec::new(),
            bk_index_stack: Vec::new(),
        }
    }

    pub fn is_draw(&self) -> bool {
        if self.non_cap_mov_count >= MAX_NON_CAP_PLY_COUNT {
            return true
        }

        if (self.non_cap_mov_count as usize) < LAST_DUP_MOV_DISTANCE + 1 {
            return false
        }

        let history_len = self.history_mov_stack.len();

        let (from, to, mov_piece, taken_piece) = self.history_mov_stack[history_len-1];
        let (last_from, last_to, last_mov_piece, last_taken_piece) = self.history_mov_stack[history_len-LAST_DUP_MOV_DISTANCE];
        if from == last_from && to == last_to && mov_piece == last_mov_piece && taken_piece == last_taken_piece {
            
            let (from, to, mov_piece, taken_piece) = self.history_mov_stack[history_len-2];
            let (last_from, last_to, last_mov_piece, last_taken_piece) = self.history_mov_stack[history_len-LAST_DUP_MOV_DISTANCE-1];
            if from == last_from && to == last_to && mov_piece == last_mov_piece && taken_piece == last_taken_piece {
                return true
            }
        }

        false
    }

    pub fn do_null_mov(&mut self) {
        self.enp_sqr_stack.push(self.enp_square);
        self.enp_square = 0;
        self.player = def::get_opposite_player(self.player);
    }

    pub fn undo_null_mov(&mut self) {
        self.enp_square = self.enp_sqr_stack.pop().unwrap();
        self.player = def::get_opposite_player(self.player);
    }

    pub fn do_mov(&mut self, from: usize, to: usize, mov_type: u8, promo: u8) {
        self.cas_rights_stack.push(self.cas_rights);
        self.enp_sqr_stack.push(self.enp_square);
        self.history_mov_stack.push((from, to, self.squares[from], self.squares[to]));
        self.non_cap_mov_count_stack.push(self.non_cap_mov_count);
        self.wk_index_stack.push(self.wk_index);
        self.bk_index_stack.push(self.bk_index);
        self.enp_square = 0;

        match mov_type {
            def::MOV_REG => self.do_reg_mov(from, to),
            def::MOV_PROMO => self.do_promo_mov(from, to, promo),
            def::MOV_CAS => self.do_cas_mov(to),
            def::MOV_ENP => self.do_enp_mov(from, to),
            def::MOV_CR_ENP => self.do_cr_enp_mov(from, to),
            _ => panic!("invalid mov type {}", mov_type),
        }

        self.player = def::get_opposite_player(self.player);
    }

    pub fn undo_mov(&mut self, from: usize, to: usize, mov_type: u8) {
        self.cas_rights = self.cas_rights_stack.pop().unwrap();
        self.enp_square = self.enp_sqr_stack.pop().unwrap();
        self.non_cap_mov_count = self.non_cap_mov_count_stack.pop().unwrap();
        self.wk_index = self.wk_index_stack.pop().unwrap();
        self.bk_index = self.bk_index_stack.pop().unwrap();
        self.history_mov_stack.pop();

        self.player = def::get_opposite_player(self.player);

        match mov_type {
            def::MOV_REG => self.undo_reg_mov(from, to),
            def::MOV_PROMO => self.undo_promo_mov(from, to),
            def::MOV_CAS => self.undo_cas_mov(to),
            def::MOV_ENP => self.undo_enp_mov(from, to),
            def::MOV_CR_ENP => self.undo_cr_enp_mov(from, to),
            _ => panic!("invalid mov type {}", mov_type),
        }
    }

    fn do_reg_mov(&mut self, from: usize, to: usize) {
        let moving_piece = self.squares[from];
        let taken_piece = self.squares[to];

        if taken_piece == 0 {
            self.non_cap_mov_count += 1;
        } else {
            self.non_cap_mov_count = 0;
        }

        self.taken_piece_stack.push(taken_piece);
        self.squares[to] = moving_piece;
        self.squares[from] = 0;

        if moving_piece == def::WR {
            if from == 0 {
                self.cas_rights &= 0b1011;
            } else if from == 7 {
                self.cas_rights &= 0b0111;
            }
        } else if moving_piece == def::BR {
            if from == 112 {
                self.cas_rights &= 0b1110;
            } else if from == 119 {
                self.cas_rights &= 0b1101;
            }
        } else if moving_piece == def::WK {
            if from == 4 {
                self.cas_rights &= 0b0011;
            }

            self.wk_index = to;
        } else if moving_piece == def::BK {
            if from == 116 {
                self.cas_rights &= 0b1100;
            }

            self.bk_index = to;
        }
    }

    fn undo_reg_mov(&mut self, from: usize, to: usize) {
        let moving_piece = self.squares[to];
        let taken_piece = self.taken_piece_stack.pop().unwrap();

        self.squares[to] = taken_piece;
        self.squares[from] = moving_piece;
    }

    fn do_promo_mov(&mut self, from: usize, to: usize, promo: u8) {
        let taken_piece = self.squares[to];

        if taken_piece == 0 {
            self.non_cap_mov_count += 1;
        } else {
            self.non_cap_mov_count = 0;
        }

        self.taken_piece_stack.push(taken_piece);
        self.squares[to] = promo;
        self.squares[from] = 0;

        self.non_cap_mov_count = 0;
    }

    fn undo_promo_mov(&mut self, from: usize, to: usize) {
        let moving_piece = if self.player == def::PLAYER_W {
            def::WP
        } else {
            def::BP
        };
        let taken_piece = self.taken_piece_stack.pop().unwrap();

        self.squares[to] = taken_piece;
        self.squares[from] = moving_piece;
    }

    fn do_cas_mov(&mut self, to: usize) {
        self.non_cap_mov_count = 0;

        if to == def::CAS_SQUARE_WK {
            self.cas_rights &= 0b0111;
            self.wk_index = to;

            let k_index = def::CAS_SQUARE_WK-2;
            let r_index = def::CAS_SQUARE_WK+1;
            let r_to_index = def::CAS_SQUARE_WK-1;

            self.squares[k_index] = 0;
            self.squares[r_index] = 0;
            self.squares[r_to_index] = def::WR;
            self.squares[def::CAS_SQUARE_WK] = def::WK;
        } else if to == def::CAS_SQUARE_BK {
            self.cas_rights &= 0b1101;
            self.bk_index = to;

            let k_index = def::CAS_SQUARE_BK-2;
            let r_index = def::CAS_SQUARE_BK+1;
            let r_to_index = def::CAS_SQUARE_BK-1;

            self.squares[k_index] = 0;
            self.squares[r_index] = 0;
            self.squares[r_to_index] = def::BR;
            self.squares[def::CAS_SQUARE_BK] = def::BK;
        } else if to == def::CAS_SQUARE_WQ {
            self.cas_rights &= 0b1011;
            self.wk_index = to;

            let k_index = def::CAS_SQUARE_WQ+2;
            let r_index = def::CAS_SQUARE_WQ-2;
            let r_to_index = def::CAS_SQUARE_WQ+1;

            self.squares[k_index] = 0;
            self.squares[r_index] = 0;
            self.squares[r_to_index] = def::WR;
            self.squares[def::CAS_SQUARE_WQ] = def::WK;
        } else if to == def::CAS_SQUARE_BQ {
            self.cas_rights &= 0b1110;
            self.bk_index = to;

            let k_index = def::CAS_SQUARE_BQ+2;
            let r_index = def::CAS_SQUARE_BQ-2;
            let r_to_index = def::CAS_SQUARE_BQ+1;

            self.squares[k_index] = 0;
            self.squares[r_index] = 0;
            self.squares[r_to_index] = def::BR;
            self.squares[def::CAS_SQUARE_BQ] = def::BK;
        }
    }

    fn undo_cas_mov(&mut self, to: usize) {
        if to == def::CAS_SQUARE_WK {
            self.squares[def::CAS_SQUARE_WK-2] = def::WK;
            self.squares[def::CAS_SQUARE_WK+1] = def::WR;
            self.squares[def::CAS_SQUARE_WK-1] = 0;
            self.squares[def::CAS_SQUARE_WK] = 0;
        } else if to == def::CAS_SQUARE_BK {
            self.squares[def::CAS_SQUARE_BK-2] = def::BK;
            self.squares[def::CAS_SQUARE_BK+1] = def::BR;
            self.squares[def::CAS_SQUARE_BK-1] = 0;
            self.squares[def::CAS_SQUARE_BK] = 0;
        } else if to == def::CAS_SQUARE_WQ {
            self.squares[def::CAS_SQUARE_WQ+2] = def::WK;
            self.squares[def::CAS_SQUARE_WQ-2] = def::WR;
            self.squares[def::CAS_SQUARE_WQ+1] = 0;
            self.squares[def::CAS_SQUARE_WQ] = 0;
        } else if to == def::CAS_SQUARE_BQ {
            self.squares[def::CAS_SQUARE_BQ+2] = def::BK;
            self.squares[def::CAS_SQUARE_BQ-2] = def::BR;
            self.squares[def::CAS_SQUARE_BQ+1] = 0;
            self.squares[def::CAS_SQUARE_BQ] = 0;
        }
    }

    fn do_enp_mov(&mut self, from: usize, to: usize) {
        let taken_index = if self.player == def::PLAYER_W {
            to - 16
        } else {
            to + 16
        };

        self.non_cap_mov_count = 0;

        let moving_piece = self.squares[from];
        let taken_piece = self.squares[taken_index];

        self.taken_piece_stack.push(taken_piece);
        self.squares[to] = moving_piece;
        self.squares[from] = 0;
        self.squares[taken_index] = 0;
    }

    fn undo_enp_mov(&mut self, from: usize, to: usize) {
        let taken_index = if self.player == def::PLAYER_W {
            to - 16
        } else {
            to + 16
        };

        let moving_piece = self.squares[to];
        let taken_piece = self.taken_piece_stack.pop().unwrap();

        self.squares[taken_index] = taken_piece;
        self.squares[from] = moving_piece;
        self.squares[to] = 0;
    }

    fn do_cr_enp_mov(&mut self, from: usize, to: usize) {
        self.enp_square = if self.player == def::PLAYER_W {
            to - 16
        } else {
            to + 16
        };

        self.non_cap_mov_count += 1;

        let moving_piece = self.squares[from];

        self.squares[to] = moving_piece;
        self.squares[from] = 0;
    }

    fn undo_cr_enp_mov(&mut self, from: usize, to: usize) {
        let moving_piece = self.squares[to];

        self.squares[from] = moving_piece;
        self.squares[to] = 0;
    }
}

impl  fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut display_string = String::new();

        let mut rank_left_index = 112;
        loop {
            for file_index in 0..def::DIM_SIZE {
                display_string.push(util::map_piece_code_to_char(self.squares[rank_left_index + file_index]));
            }

            display_string.push('\n');

            if rank_left_index == 0 {
                break
            }

            rank_left_index -= 16;
        }

        write!(formatter, "{}", display_string)
    }
}

fn get_squares_from_fen(fen_squares_string: &str) -> ([u8; def::BOARD_SIZE], usize, usize) {
    let mut squares = [0; def::BOARD_SIZE];
    let mut wk_index = 0;
    let mut bk_index = 0;

    let rank_string_list: Vec<&str> = fen_squares_string.split("/").collect();
    assert_eq!(def::DIM_SIZE, rank_string_list.len());

    let mut index = 112;
    for rank_index in 0..def::DIM_SIZE {
        let rank_string = rank_string_list[rank_index];

        for char_code in rank_string.chars() {
            if char_code.is_numeric() {
                index += char_code.to_digit(10).unwrap() as usize;
                continue
            }

            if char_code.is_alphabetic() {
                let piece = util::map_piece_char_to_code(char_code);
                squares[index] = piece;

                if piece == def::WK {
                    wk_index = index;
                }

                if piece == def::BK {
                    bk_index = index;
                }

                index += 1;
            }
        }

        if index == def::DIM_SIZE {
            break
        }

        index -= 24;
    }

    (squares, wk_index, bk_index)
}

fn get_player_from_fen(fen_player_string: &str) -> u8 {
    match fen_player_string {
        "w" => def::PLAYER_W,
        "b" => def::PLAYER_B,
        _ => panic!("invalid player {}", fen_player_string),
    }
}

fn get_cas_rights_from_fen(fen_cas_rights_player: &str) -> u8 {
    if fen_cas_rights_player == "-" {
        return 0
    }

    let mut cas_rights = 0;

    if fen_cas_rights_player.contains("K") {
        cas_rights |= 0b1000;
    }

    if fen_cas_rights_player.contains("Q") {
        cas_rights |= 0b0100;
    }

    if fen_cas_rights_player.contains("k") {
        cas_rights |= 0b0010;
    }

    if fen_cas_rights_player.contains("q") {
        cas_rights |= 0b0001;
    }

    cas_rights
}

fn get_enp_sqr_from_fen(fen_enp_sqr_string: &str) -> usize {
    if fen_enp_sqr_string == "-" {
        return 0
    }

    util::map_sqr_notation_to_index(fen_enp_sqr_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::def;

    #[test]
    fn test_new_startpos() {
        let state = State::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
    }

    #[test]
    fn test_do_move_1() {
        let mut state = State::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);

        state.do_mov(util::map_sqr_notation_to_index("e2"), util::map_sqr_notation_to_index("e4"), def::MOV_CR_ENP, 0);
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(util::map_sqr_notation_to_index("e3"), state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);

        state.undo_mov(util::map_sqr_notation_to_index("e2"), util::map_sqr_notation_to_index("e4"), def::MOV_CR_ENP);
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
    }

    #[test]
    fn test_do_move_2() {
        let mut state = State::new("r1bqk1nr/pPpp1ppp/2n5/2b1p3/2B1P3/2N2N2/P1PP1PPP/R1BQK2R w KQkq - 0 1");
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("a8")]);

        state.do_mov(util::map_sqr_notation_to_index("b7"), util::map_sqr_notation_to_index("a8"), def::MOV_PROMO, def::WQ);
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);
        assert_eq!(def::WQ, state.squares[util::map_sqr_notation_to_index("a8")]);

        state.undo_mov(util::map_sqr_notation_to_index("b7"), util::map_sqr_notation_to_index("a8"), def::MOV_PROMO);
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("a8")]);
    }

    #[test]
    fn test_do_move_3() {
        let mut state = State::new("r3k2r/pbppnppp/1bn2q2/4p3/2B5/2N1PN2/PPPP1PPP/R1BQK2R b Qkq - 0 1");
        assert_eq!(0b0111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);
        assert_eq!(def::BK, state.squares[util::map_sqr_notation_to_index("e8")]);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("a8")]);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("c8")]);

        state.do_mov(util::map_sqr_notation_to_index("e8"), util::map_sqr_notation_to_index("c8"), def::MOV_CAS, 0);
        assert_eq!(0b0110, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("e8")]);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("a8")]);
        assert_eq!(def::BK, state.squares[util::map_sqr_notation_to_index("c8")]);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("d8")]);

        state.undo_mov(util::map_sqr_notation_to_index("e8"), util::map_sqr_notation_to_index("c8"), def::MOV_CAS);
        assert_eq!(0b0111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);
        assert_eq!(def::BK, state.squares[util::map_sqr_notation_to_index("e8")]);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("a8")]);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("c8")]);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("d8")]);
    }

    #[test]
    fn test_do_move_4() {
        let mut state = State::new("4r1k1/pp1Q1ppp/3B4/q2p4/5P1P/P3PbPK/1P1r4/2R5 b - - 3 5");
        assert_eq!(0b0000, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("e2")]);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("d2")]);

        state.do_mov(util::map_sqr_notation_to_index("d2"), util::map_sqr_notation_to_index("e2"), def::MOV_REG, 0);
        assert_eq!(0b0000, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("d2")]);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("e2")]);

        state.do_mov(util::map_sqr_notation_to_index("d7"), util::map_sqr_notation_to_index("e8"), def::MOV_REG, 0);
        state.do_mov(util::map_sqr_notation_to_index("e2"), util::map_sqr_notation_to_index("h2"), def::MOV_REG, 0);

        state.undo_mov(util::map_sqr_notation_to_index("e2"), util::map_sqr_notation_to_index("h2"), def::MOV_REG);
        state.undo_mov(util::map_sqr_notation_to_index("d7"), util::map_sqr_notation_to_index("e8"), def::MOV_REG);

        state.undo_mov(util::map_sqr_notation_to_index("d2"), util::map_sqr_notation_to_index("e2"), def::MOV_REG);
        assert_eq!(0b0000, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("e2")]);
        assert_eq!(def::BR, state.squares[util::map_sqr_notation_to_index("d2")]);
    }

    #[test]
    fn test_do_move_5() {
        let mut state = State::new("r1bqkbnr/ppp1p1pp/2n5/3pPp2/3P4/8/PPP2PPP/RNBQKBNR w KQkq f6 0 1");
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(util::map_sqr_notation_to_index("f6"), state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("f6")]);
        assert_eq!(def::BP, state.squares[util::map_sqr_notation_to_index("f5")]);
        assert_eq!(def::WP, state.squares[util::map_sqr_notation_to_index("e5")]);

        state.do_mov(util::map_sqr_notation_to_index("e5"), util::map_sqr_notation_to_index("f6"), def::MOV_ENP, 0);
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(0, state.enp_square);
        assert_eq!(def::PLAYER_B, state.player);
        assert_eq!(def::WP, state.squares[util::map_sqr_notation_to_index("f6")]);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("f5")]);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("e5")]);

        state.undo_mov(util::map_sqr_notation_to_index("e5"), util::map_sqr_notation_to_index("f6"), def::MOV_ENP);
        assert_eq!(0b1111, state.cas_rights);
        assert_eq!(util::map_sqr_notation_to_index("f6"), state.enp_square);
        assert_eq!(def::PLAYER_W, state.player);
        assert_eq!(0, state.squares[util::map_sqr_notation_to_index("f6")]);
        assert_eq!(def::BP, state.squares[util::map_sqr_notation_to_index("f5")]);
        assert_eq!(def::WP, state.squares[util::map_sqr_notation_to_index("e5")]);
    }
}
