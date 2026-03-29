"""Flipull / Plotting
Board state: 
- Attackable columns
- State of the grid (2d array and count)
- Currently held block (0 for wild)
- Moves - list of the form (R/C, #) for row/column and (1-based) number.

Main method:
- Load in the initial board state
- Call MakeMove
- MakeMove:
-- For each legal move, create an updated board state and call MakeMove
-- Note the number of blocks; if it's the lowest so far, record this moveset
-- Can abort early if you reach 0 blocks (or even <5 blocks)
- After the MakeMove call, print the best moveset to the console


TO DO:
- Am I getting equivalent scoring on the first 10 stages?
- Does the MISS on level 17 improve the score?
- How can I make the 6x6 stages fast enough? 
-- The RNG starts in a constant state from startup (NOT from resetting with F3!) Looks like RNG updates when a level layout is generated,
   so if you insert a coin *before* the demo starts playing its level, you'll get a fixed layout for stages 1, 2, and beyond - maybe through
   the full game. So maybe I can just pre-compute all 60 stages (and who cares if it takes 60 hours?)  Then for optimal scoring, you'd 
   want to repeat that WHOLE process for each RNG seed, to see whether one generates a series of levels with higher overall score potential.
-- Rust port? 
-- Non-exhaustive search? "Lightning storm" approach: At each point in the tree of moves, choose one at random, and stop the depth-first search
   once you reach a solution. Run this until you've seen a large number of valid solutions, keeping track of the highest-scoring one.
-- More work with the profiler?
-- Or is the number of possible layouts small enough to pre-compute them all (even if it takes a while)?

- Store the level/quota/columns/rows for all stages
- Make an initial 1cc attempt once things are in a runnable state

"""
from __future__ import annotations
from pathlib import Path
import numpy as np
from dataclasses import dataclass
from multiprocessing import Pool
import random
@dataclass
class BoardState:
    level_number: int
    qualify_blocks: int
    moves: list
    board: np.array
    attack_columns: tuple[int]
    attack_rows: tuple[int]
    block_count: int
    score: int
    held_block: int = 0

    @classmethod
    def load(cls) -> BoardState:
        letters = ".sotx"
        with Path("level.txt").open("r") as in_file:
            header_line = in_file.readline().strip()
            level_digits, qualify_digits, column_digits, row_digits = header_line.split()
            qualify_blocks = int(qualify_digits)
            attack_columns = []
            attack_rows = []
            for index, digit in enumerate(column_digits):
                if digit == "1":
                    attack_columns.append(index)
            for index, digit in enumerate(row_digits):
                if digit == "1":
                    attack_rows.append(index)
            board_grid = np.zeros((len(row_digits), len(column_digits)), dtype=int)
            row = 0
            block_count = 0
            for row in range(len(row_digits)):
                line = in_file.readline().strip()
                for column, char in enumerate(line):
                    board_grid[row, column] = letters.index(char)
                    if board_grid[row, column] > 0:
                        block_count += 1
                row += 1
        return BoardState(level_number=int(level_digits), qualify_blocks=qualify_blocks, moves=[], board=board_grid, 
                                attack_columns=tuple(attack_columns), 
                                attack_rows=tuple(attack_rows),
                                block_count=block_count, score=0)

    def print(self):
        print(f">>> Hold {self.held_block} moves {self.moves} score {self.score} board:\n{self.board}")

    def clone(self) -> BoardState:
        new_moves = self.moves[:]
        new_board = self.board.copy()
        new_board = BoardState(level_number=self.level_number, qualify_blocks=self.qualify_blocks, moves=new_moves, board=new_board, attack_columns=self.attack_columns, 
                               attack_rows=self.attack_rows, block_count=self.block_count, score=self.score, held_block=self.held_block)
        return new_board
    
    def is_legal_move(self, is_column: bool, move_position: int) -> tuple[bool, bool]:
        # Return tuple of the form (is_legal, is_rhs)
        row_count, column_count = self.board.shape
        if is_column:
            row = 0
            column = move_position
            dx = 0
            dy = 1
        else:
            row = move_position
            column = 0
            dx = 1
            dy = 0
        while True:
            current_block = self.board[row,column]
            if current_block != 0:
                is_top_right_side = False
                if (column == column_count - 1) and (row == 0 or self.board[row-1,column] == 0):
                    is_top_right_side = True
                if self.held_block == 0 or self.held_block == current_block:
                    return True, is_top_right_side
                return False, is_top_right_side
            # Now advance to next square:
            row += dy
            column += dx
            # If we hit the right edge, turn down:
            if column >= column_count:
                column -= 1
                row += 1
                dx = 0
                dy = 1
            if row >= row_count:
                return False, False

    def make_move(self, is_column: bool, move_position: int) -> bool:
        row_count, column_count = self.board.shape
        blocks_cleared = 0
        if is_column:
            move = f"C{move_position+1}"
        else:
            move = f"R{row_count-move_position}"
        self.moves.append(move)
        if is_column:
            row = 0
            column = move_position
            dx = 0
            dy = 1
        else:
            row = move_position
            column = 0
            dx = 1
            dy = 0
        while True:
            current_block = self.board[row,column]
            if current_block != 0:
                if self.held_block != 0 and self.held_block != current_block:
                    # If first block hit doesn't match, and it's not a wildcard, the move is illegal:
                    if blocks_cleared == 0:
                        return False
                    # We swap colors with this block, and stop:
                    swap = self.board[row, column]
                    self.board[row, column] = self.held_block
                    self.held_block = swap
                    self.score += 100 * (blocks_cleared * blocks_cleared)
                    self.raw_score += 100 * (blocks_cleared * blocks_cleared)
                    return True
                # Wildcard gets assigned a color now:
                if self.held_block == 0:
                    self.held_block = current_block
                # Clear a block:
                self.board[row,column] = 0
                blocks_cleared += 1
                self.block_count -= 1
                # And blocks above fall down:
                for above_row in range(row-1, -1, -1):
                    self.board[above_row+1, column] = self.board[above_row, column]
                self.board[0, column] = 0
            # Now advance to next square:
            row += dy
            column += dx
            # If we hit the right edge, turn down:
            if column >= column_count:
                column -= 1
                row += 1
                dx = 0
                dy = 1
            if row >= row_count:
                if blocks_cleared == 0:
                    return False
                else:
                    self.score += 100 * (blocks_cleared * blocks_cleared)
                    self.raw_score += 100 * (blocks_cleared * blocks_cleared)
                    return True

class MoveChecker:
    def __init__(self):
        self.best_score = 0
        self.best_board = None
        self.states_seen = {}
    def main(self, board: BoardState, is_column: bool, move_position: int) -> BoardState:
        result = board.make_move(is_column=is_column, move_position=move_position)
        if not result:
            return None
        self.try_move(board)
        return self.best_board
    def try_move(self, board: BoardState):
        # # Avoid re-checking a board state we've seen before:
        # key = (board.held_block, str(board.board))
        # if key in self.states_seen:
        #     other_board = self.states_seen[key]
        #     #score = self.states_seen[key]
        #     #print(f"Repeat: score {other_board.score} vs {board.score}")
        #     #print(other_board.moves)
        #     #print(board.moves)
        #     if other_board.score >= board.score:
        #         return
        # self.states_seen[key] = board #board.score
        try_moves = []
        seen_top_right = False
        for row in board.attack_rows:
            is_legal, is_top_right = board.is_legal_move(is_column=False, move_position=row)
            if not is_legal or (is_top_right and seen_top_right):
                continue
            if is_top_right:
                seen_top_right = True
            try_moves.append((False, row))
        for column in board.attack_columns:
            is_legal, is_top_right = board.is_legal_move(is_column=True, move_position=column)
            if not is_legal or (is_top_right and seen_top_right):
                continue
            if is_top_right:
                seen_top_right = True
            try_moves.append((True, column))
        for (is_column, move_position) in try_moves:
            new_board = board.clone()
            result = new_board.make_move(is_column=is_column, move_position=move_position)
            if not result:
                print("???? failed move!")
                continue
            # Try making more moves, unless we already hit our qualify target:
            if new_board.block_count > new_board.qualify_blocks:
                self.try_move(new_board)
            else:
                # Special logic on tutorial level: You get a flat 1000 point bonus
                if new_board.level_number == 0:
                    new_board.score += 1000
                else:
                    new_board.score += 1000 * (new_board.qualify_blocks - new_board.block_count + 1)
                if random.random() < 0.0001:
                    print(f"score {new_board.score} for {new_board.moves}")
                if new_board.score > self.best_score:
                    self.best_score = new_board.score
                    self.best_board = new_board
                    print(f"Best score {new_board.score} for {new_board.moves}")

def find_best_moves(work: tuple[BoardState, bool, int])-> BoardState:
    board = work[0]
    is_column = work[1]
    move_position = work[2]
    return MoveChecker().main(board, is_column, move_position)

class Flipull:
    def main(self):
        board = BoardState.load()
        job_list = []
        for row in board.attack_rows:
            new_board = board.clone()
            job_list.append((new_board, False, row))
        for column in board.attack_columns:
            new_board = board.clone()
            job_list.append((new_board, True, column))
        pool = Pool(8)
        results = pool.map(find_best_moves, job_list)
        #results = []
        #for job in job_list:
        #    results.append(find_best_moves(job))
        #results = [find_best_moves(job) for job in job_list]
        best_score = 0
        for best_end_board in results:
            if best_end_board is None:
                continue
            if best_end_board.score > best_score:
                best_score = best_end_board.score
                best_overall = best_end_board
        print(f"\n\n>>> Best score {best_score} best moves {best_overall.moves}")
        # # Re-trace our steps:
        # trace_board = board.clone()
        # trace_board.print()
        # for move in best_overall.moves:
        #     is_column = True if move[0] == "C" else False
        #     move_position = int(move[1])
        #     if is_column:
        #         move_position -= 1
        #     else:
        #         move_position = board.board.shape[0] - move_position
        #     print("%%", move, is_column, move_position)
        #     trace_board.make_move(is_column=is_column, move_position=move_position)
        #     trace_board.print()



if __name__ == "__main__":
    Flipull().main()
    #import cProfile
    #profiler = cProfile.Profile()
    #profiler.run("Flipull().main()")
    #profiler.dump_stats("Profiler.stats")
    

