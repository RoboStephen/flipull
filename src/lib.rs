use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct BoardState {
    pub level_number: i32,
    pub qualify_blocks: usize,
    pub row_count: usize,
    pub column_count: usize,
    // Vector of moves you can make, listed from bottom to top.
    // Entries have the form "R2" for bottom row, "C1" for leftmost column, 
    // or "X" of that row is an illegal move.
    pub legal_move_is_column: Vec<bool>,
    pub legal_move_position: Vec<i32>,
    // Vector of columns and rows we can move in:
    //pub attack_columns: Vec<usize>,
    //pub attack_rows: Vec<usize>,
    // String of the form "C3, R1, " listing the moves we made:
    pub moves: String,
    pub move_list: Vec<usize>, 
    // Current board state:
    pub board: Vec<i32>,
    // Current number of blocks:
    pub block_count: usize,
    // Current score without time-penalty and score with time-penalty:
    pub raw_score: i32,
    pub score: i32,
    // Currently-held block (0 for wildcard):
    pub held_block: i32,
    pub previous_move_index: usize,
    pub miss_available: bool
}
impl BoardState {
    pub fn print_board(&self) {
        let b = ".sotx";
        println!(">>> Board raw_score {} hold {} after moves {}", self.raw_score, b.chars().nth(self.held_block as usize).expect("invalid block"), self.moves);
        let mut grid = String::new();
        for row in 0..self.row_count {
            for column in 0..self.column_count {
                let block = self.board[row*self.column_count+column] as usize;
                grid.push(b.chars().nth(block).expect("Invalid block"));
            }
            grid.push_str("\n");
        }
        println!("{}", grid);
    }
    fn is_legal_move(&self, is_column: bool, move_position: usize) -> (bool, bool) {
        let mut is_top_right_side = false;
        let mut row: usize = 0;
        let mut column: usize = 0;
        let mut dx: usize = 0;
        let mut dy: usize = 0;
        if is_column {
            column = move_position;
            dy = 1;
        } else {
            row = move_position;
            dx = 1;
        }
        loop {
            let current_block = self.board[row * self.column_count + column];
            if current_block != 0 {
                if (column == self.column_count - 1) && (row == 0 || self.board[(row-1)*self.column_count + column] == 0) {
                    is_top_right_side = true;
                }
                if self.held_block == 0 || self.held_block == current_block {
                    //println!("{}:{} is legal - hold {} touch {} at ({},{})", is_column, move_position, self.held_block, current_block, row, column);
                    return (true, is_top_right_side);
                }
                //println!("{}:{} NOT legal - hold {} touch {} at ({},{})", is_column, move_position, self.held_block, current_block, row, column);
                return (false, is_top_right_side);
            }
            row += dy;
            column += dx;
            if column >= self.column_count {
                column -= 1;
                dx = 0;
                dy = 1;
                row += 1;
            }
            if row >= self.row_count {
                return (false, is_top_right_side);
            }
        }
    }

    pub fn make_move(&mut self, move_index: usize) {
        if (move_index == 999) {
            self.held_block = 0;
            self.miss_available = false;
            return;
        }
        let is_column = self.legal_move_is_column[move_index];
        let move_position = self.legal_move_position[move_index];
        // Penalize the score for the time spent moving:
        let move_score_penalty = if self.level_number < 32 {3} else {6};
        let move_score_penalty = 0;
        self.score -= move_score_penalty * (move_index as i32 - self.previous_move_index as i32).abs();
        // Penalize the score for the time spent watching a shot (2 or 4 seconds)
        self.score -= if self.level_number < 32 {20} else {40};
        self.previous_move_index = move_index;
        if is_column {
            write!(self.moves, "C{}, ", move_position+1).unwrap();
        }
        else {
            write!(self.moves, "R{}, ", self.row_count as i32 - move_position).unwrap();
        }
        self.move_list.push(move_index);
        let mut blocks_cleared = 0;
        let mut row: usize = 0;
        let mut column: usize = 0;
        let mut dx: usize = 0;
        let mut dy: usize = 0;
        if is_column {
            column = move_position as usize;
            dy = 1;
        } else {
            row = move_position as usize;
            dx = 1;
        }
        loop {
            let current_block = self.board[row*self.column_count + column];
            if current_block != 0 {
                if self.held_block != 0 && self.held_block != current_block {
                    if blocks_cleared == 0 {
                        println!("INVALID MOVE {}:{} held block is {} current block is {}", is_column, move_position, self.held_block, current_block);
                        return;
                    }
                    // Swap colors with this block, and stop:
                    self.board[row*self.column_count + column] = self.held_block;
                    self.held_block = current_block;
                    self.score += 100 * (blocks_cleared * blocks_cleared);
                    self.raw_score += 100 * (blocks_cleared * blocks_cleared);
                    return;
                }
                // Wildcard gets assigned a color:
                if self.held_block == 0 {
                    self.held_block = current_block;
                }
                // Clear a block:
                self.board[row*self.column_count + column] = 0;
                blocks_cleared += 1;
                self.block_count -= 1;
                // Blocks above fall down:
                for above_row in (0..row).rev() {
                    self.board[(above_row+1)*self.column_count + column] = self.board[above_row*self.column_count + column];
                }
                self.board[0 + column] = 0;
            }
            // Advance to the next square:
            row += dy;
            column += dx;
            // If we hit the right edge, turn down:
            if column >= self.column_count {
                column -= 1;
                row += 1;
                dx = 0;
                dy = 1;
            }
            if row >= self.row_count {
                if blocks_cleared == 0{
                    panic!("ERROR, no blocks!");
                }
                self.score += 100 * (blocks_cleared * blocks_cleared);
                self.raw_score += 100 * (blocks_cleared * blocks_cleared);
                return;
            }
        }
    }
}

pub struct SolutionFinder {
    pub best_score: i32,
    pub best_board: Option<BoardState>
}

impl SolutionFinder {
    pub fn find_best_solution(&mut self, board: BoardState) -> Option<BoardState> {
        //println!("\nFind best solution - board has {} entries, moves are {}", board.block_count, board.moves);
        //board.print_board();

        // Make a list of moves that are legal to make right now. To save time, loop over them in an order that starts
        // by looking at the move closet to the previous move. If there are 2 (or move) moves that target the same
        // row or column, do the closest. If you see 2 (or more) moves that all hit the topmost block in the rightmost 
        // column first, do the closest.
        let mut is_legal: bool;
        let mut is_top_right: bool;
        let mut saw_top_right = false;
        let mut try_legal_moves: Vec<usize> = Vec::new();

        let mut seen_array = [false; 25];
        
        for move_delta in 0..12 {
            for delta_sign in &[1, -1] {
                let move_index: i32 = board.previous_move_index as i32 + delta_sign * move_delta;
                if move_index < 0 || move_index as usize >= board.legal_move_is_column.len() {
                    continue;
                }
                let is_column = board.legal_move_is_column[move_index as usize];
                let move_position = board.legal_move_position[move_index as usize];
                if move_position < 0 {
                    continue;
                }
                let seen_index = move_position as usize + (if is_column {10} else {0});
                if seen_array[seen_index] {
                    continue;
                }
                seen_array[seen_index] = true;

                (is_legal, is_top_right) = board.is_legal_move(is_column, move_position as usize);
                if is_top_right {
                    if saw_top_right {
                        continue;
                    }
                    saw_top_right = true;
                }
                if is_legal == false {
                    continue;
                }
                try_legal_moves.push(move_index as usize)
            } // 
        }

        // Special logic: If no moves are available, but a miss is available, then we lose an extra life, get a wildcard-block, 
        // and continue the search:
        if try_legal_moves.len() == 0 && board.miss_available {
            let mut new_board = board.clone();
            new_board.held_block = 0;
            new_board.miss_available = false;
            new_board.move_list.push(999);
            write!(new_board.moves, "MISS, ").unwrap();
            self.find_best_solution(new_board);
        }

        // println!("Move positions: {:?}", try_legal_moves);
        for move_index in try_legal_moves {
            let mut new_board = board.clone();
            new_board.make_move(move_index);

            if new_board.block_count > new_board.qualify_blocks {
                self.find_best_solution(new_board);
            } else {
                // Bonus for getting block count below the quota - except on the tutorial level where 
                // you get a flat 1000 bonus:
                let bonus;
                if new_board.level_number == 0 {
                    bonus = 1000;
                } else {
                    bonus = 1000 * (new_board.qualify_blocks - new_board.block_count + 1) as i32;
                }
                new_board.score += bonus;
                new_board.raw_score += bonus;
                if new_board.score > self.best_score {
                    self.best_score = new_board.score;
                    println!("Best score {} raw score {} moves {}", self.best_score, new_board.raw_score, new_board.moves);
                    self.best_board = Some(new_board);
                }
            }
        }

        return self.best_board.clone();
    }
}