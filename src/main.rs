use std::fs::File;
use std::io::{BufRead, BufReader, Write, BufWriter};
use flipull::{BoardState, SolutionFinder};
use std::thread;
use std::collections::HashSet;

// Given a string of the form 00110, return a Vec of the indexes
// with 1 digits
fn parse_indexes(digits: &str) -> Vec<usize> {
    digits.chars()
    .enumerate()
    .filter_map(|(i,c)| if c == '1' { Some(i as usize) } else {None })
    .collect()
}

// Convert a block character to an integer:
// period for empty, s for square, o for circle, t for Taito triangle, x for X
fn parse_block(c: char) -> i32 {
    ".sotx".chars().position(|v| v==c).expect("Failed to parse block char") as i32
}


// Parse board state from a file:
fn load_board_state(desired_level_number: i32, miss_available: bool) -> BoardState {
    //let level_file = File::open("all_levels.txt").expect("Failed to open file");
    let level_file = File::open("plottinga_levels.txt").expect("Failed to open file");
    let mut reader = BufReader::new(level_file);
    loop {
        // Read and parse header line, which looks like this:
        // 10 8 6 6
        // Round#, quota, row count, column count
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read header");
        //println!("Read line: {}", line);
        let header_fields: Vec<&str> = line.split_whitespace().collect();
        let level_number: i32 = header_fields[0].parse().unwrap();
        let qualify_blocks: usize = header_fields[1].parse().unwrap();
        let row_count: usize = header_fields[2].parse().unwrap();
        let column_count: usize = header_fields[3].parse().unwrap();
        // Read and parse the second header line, which has a list of moves from the bottom row
        // to the top row, like this:
        // R1 R2 R3 R4 R5 R6 C6 C1 C5 C5 C3 C2
        line = String::new();
        reader.read_line(&mut line).expect("Failed to read header line 2");
        let legal_move_strings: Vec<&str> = line.split_whitespace().collect();
        let mut legal_move_is_column: Vec<bool> = Vec::new();
        let mut legal_move_position: Vec<i32> = Vec::new();
        for move_string in legal_move_strings {
            //println!("Move string {}", move_string);
            if move_string == "X" {
                legal_move_is_column.push(true);
                legal_move_position.push(-1);
            }
            else if move_string.chars().next() == Some('R') {
                legal_move_is_column.push(false);
                let row_number: i32 = move_string[1..].parse().expect("Not a valid number");
                let move_position = row_count as i32 - row_number;
                legal_move_position.push(move_position);
                if move_position < 0 || move_position as usize >= row_count {
                    panic!("ERROR loading board: Move row# {}", row_number)
                }
            }
            else if move_string.chars().next() == Some('C') {
                legal_move_is_column.push(true);
                let column_number: i32 = move_string[1..].parse().expect("Not a valid number");
                let move_position = column_number - 1;
                legal_move_position.push(move_position);
                if move_position < 0 || move_position as usize >= column_count {
                    panic!("ERROR loading board: Move row# {}", move_position)
                }

            }
            else {
                panic!("ERROR parsing level!");
            }
        }


        // Parse the body lines, like "sootx", to get the starting grid of blocks:
        let mut grid = Vec::new();
        for _row in 0..row_count {
            line = String::new();
            reader.read_line(&mut line).expect("Failed to read body");
            for block_char in line.chars().take(column_count) {
                grid.push(parse_block(block_char));
            }
        }
        if level_number == desired_level_number 
        {
            // Construct and return the BoardState struct:
            return BoardState {level_number: level_number, 
                qualify_blocks: qualify_blocks, 
                row_count: row_count, 
                column_count: column_count, 
                legal_move_is_column: legal_move_is_column,
                legal_move_position: legal_move_position, 
                moves: String::new(),
                move_list: Vec::new(),  
                board: grid,
                block_count: row_count * column_count,
                raw_score: 0,
                score: 0,
                held_block: 0,
                previous_move_index: 0,
                miss_available: miss_available
            };
        }
        // Skip empty line:
        line = String::new();
        reader.read_line(&mut line).expect("Failed to read body");
    }
}

fn main_single_threaded() {
    println!("Hello world! Single threaded.");
    let board = load_board_state(0, false);
    //println!("{:#?}", board);
    println!("Board is parsed");
    let mut finder = SolutionFinder {
        best_score: 0, 
        best_board: None
    };
    let best_board = finder.find_best_solution(board).unwrap();
    println!("Overall best score is {} raw score {} for moves {}", best_board.score, best_board.raw_score, best_board.moves);
    best_board.print_board();
}

fn main() {
    println!("Hello world!");
    let output_file = File::create("PlottingOutput.txt").expect("Open");
    let mut writer = BufWriter::new(output_file);
    for level_number in 0..60 {
        let board = load_board_state(level_number, false);
        let best_a = get_best_solution_multi_threaded(board);
        let board = load_board_state(level_number, true);
        let best_b = get_best_solution_multi_threaded(board);
        writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t", level_number, best_a.score, best_a.raw_score, best_a.moves, best_b.score, best_b.raw_score, best_b.moves).expect("Write");
        println!("Level {} handled", level_number);
    }
}

fn get_best_solution_multi_threaded(board: BoardState) -> BoardState {
    
    let mut thread_handles = Vec::new();
    // One thread per possible move, disregarding any duplicates:
    let mut seen = HashSet::new();
    for move_index in 0..board.legal_move_is_column.len() {
        let is_column = board.legal_move_is_column[move_index];
        let move_position = board.legal_move_position[move_index];
        if move_position < 0 {
            continue;
        }
        let pair = (is_column, move_position);
        if !seen.insert(pair) {
            continue;
        }
        let mut new_board = board.clone();
        // println!("Make move {} {} {}", move_index, is_column, move_position);
        new_board.make_move(move_index);
        let mut finder = SolutionFinder {
            best_score: 0,
            best_board: None
        };

        let handle = thread::spawn(move || {
            return finder.find_best_solution(new_board);
        });
        thread_handles.push(handle);
    }

    let mut best_score = 0;
    let mut best_board: Option<BoardState> = None;

    for handle in thread_handles {
        let result = handle.join().expect("Thread panic");
        if result.is_some() {
            let board = result.unwrap();
            if board.score > best_score {
                best_score = board.score;
                best_board = Some(board);
            }
        }
    }
    let best = best_board.unwrap();
    println!("Overall best score is {} raw score {} for moves {}", best_score, best.raw_score, best.moves);
    return best;

    // // Replay:
    // let mut replay_board = board.clone();
    // for move_index in &best.move_list {
    //     replay_board.make_move(*move_index);
    //     replay_board.print_board();
    // }
    // return best;
    
}
