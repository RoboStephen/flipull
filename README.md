# Overview
Plotting, released as Flipull in Japan, is a puzzle game developed by Taito and released to arcades in 1989. This repository stores code and notes I used to set a new world record score in this game. 

# Game overview
The game takes place on a grid. At the bottom right is a stack of some puzzle blocks of various colors: red circle, blue triangle (Taito logo), green square, black X. 

When playing the game, you control a blob character (named Amsha) who can move up or down the left side of the screen (by moving the joystick up and down), and throw a block (by pressing the button). Once a block is thrown, it flies straight to the right until it encounters a puzzle block or a wall: 
- If a thrown block hits a puzzle block it matches, it clears that block, and any blocks above the cleared block fall down one square. If the thrown block runs into two or more matching blocks, it clears all of them.
- After clearing one or more matching puzzle blocks, if the thrown block hits a non-matching puzzle block, it swaps colors with that block, then returns to the player's control. 
- If the first puzzle block hit by a thrown block is not a match, it bounces away, clearing no blocks.
- If a thrown block traveling to the right hits a wall (including the right wall of the game grid), it turns downward and continues. If a thrown block traveling down hits the floor, it stops.
- On the first round you can "attack" any row from the left, and any column from above (after your thrown block bounces off the ceiling and turns downward). In later rounds you will not always be able to directly attack each row and column.

Your goal in each round is to eliminate puzzle blocks until the number of blocks remaining reaches a quota. For instance, if the text "QUALIFY 5 OR LESS" appears at the top right, then you'll successfully complete the level once there are 5, 4, or fewer blocks remaining in the stack. You begin each round with a wild block , with a lightning bolt on it. The wild block can match any color. When the thrown wild block hits another puzzle block, it instantly changes to that color, then the throw proceeds normally. If at any point there are no legal moves - for instance, because you're holding a green square, and there are no green squares that can be reached by a throw - the text "MISS" appears. You begin with two wild blocks in reserve (extra lives), so that after a MISS you receive a wild block and continue the level. On a third MISS, the game ends.

The game features 60 rounds in all - a special tutorial round 0, then regular levels 1 through 31 followed by a "HALF CLEAR!" celebration, then regular levels 32 through 59 followed by an "ALL CLEAR!" celebration.

Each round is timed, and if you run out of time before completing the round, the game ends. The in-game timer ticks down by roughly 10 seconds every 7 actual seconds. In rounds 32 and beyond, the in-game timer speeds up, ticking down by 10 seconds roughly every 3.5 actual seconds.
# Scoring
- Clearing a single block scores 100 points. Clearing a row of 2, 3, 4, or 5 blocks scores 400, 900, 1600, or 2500 points respectively (clearing n blocks scores n x n x 100 points). 
- At the end of a round, you get a 1000-point bonus, plus another 1000 points for each extra block removed beyond the quota. For example, if the quota is 9, then you receive 1000, 2000, or 3000 points by ending the round with 9, 8, or 7 blocks remaining. Note that the round ends as soon as you reach the quota, even if further legal moves remain. The training level is an exception to this scoring rule - you receive 1000 points at the end of the training level regardless of whether you meet the quota, exceed the quota, or have no legal moves available.
- At the end of the round, you get a 10-point bonus for each second remaining on the timer. 

The round-end bonus and time-bonus points are added after the end of round jingle completes, as the intro for the next round starts.

If you lose the game by running out of time or three MISS events, you have the option to continue play, but your score is reset to zero. If you successfully complete the game through level 59, the game ends.

# Score optimization and world record
Judging by the high scores reported to the MAME Action Replay Page (MARP), an ordinary 1-credit clear (1cc) of the game obtains a score of roughly 390,000 points. The top Japan score I was able to track down is 460,780. However, scores around 550,000 are possible [http://replay.marpirc.net/r/(flipull,plottinga,plottingb,plottingu,plotting)], and a YouTube video of a MAME playback scored exactly 560,000 points https://www.youtube.com/watch?v=cXYslkn45To. Clearing groups of blocks at a time is the key to scoring. In particular, there's a large score reward for clearing as many blocks as possible in the last move of a puzzle, gaining 1000 points for each extra block cleared beyond the quota. 

I implemented a depth-first search to identify the highest-scoring solution for each level. While solutions involve up to 20 or more moves, enumerating all solutions remains tractable because the tree of possible moves is not a very bushy  tree - after the first move (using a wild block), there are typically only 2 or 3 moves available at any point in time. Below is an overview of the main move optimizer function:
* Consider each possible move (throwing a block from each row)
* Eliminate illegal moves which do not eliminate a puzzle block
* If there are two (or more) moves which attack the same block, choose just one - and take the one where the row is as close as possible to the previous move (or for the first move, closest to the bottom of the screen, where you begin each round)
* In particular: If there are two (or more) moves that start by attacking the top block on the rightmost column, consider just one. (Late in a round, once top rows of the stack have been cleared, there are typically several different rows that all end up making  this same move)
* Loop over all legal moves (if any). Make the move, keeping track of the points accumulated for clearing blocks, and penalize the score by 3 or 6 points per row traveled (in the first or last half of the game) to reward faster solutions. Then recurse (call the optimizer function again with this new board-state).
* Whenever the quota of blocks is reached, note the final score (including the end-of-round bonus), and remember this solution if it's the best solution seen so far

After this search completes, we obtain an optimal scoring route for the puzzle. The search was implemented in rust, using multiple threads, and using some optimization for speed, leading to search times that ranged from seconds to minutes on a desktop PC. A variant on this search allows for one "MISS" in the route, which can gain additional points by obtaining a wild block parway through the level.

The old 560,000-point world record was already very strong. For most rounds, this exhaustive search identified a route with the same block-clearing score as the WR, in many cases exactly the same route. However, we identified several levels where scoring could be improved. For instance, round 8 scores 9,900 points in the WR run (setting aside time bonus), and an optimal solution scores 11,800 points, for a gain of 1,900 points. In the updated route rounds 8, 9, 11, 28, 39, 40, and 51 all score additional points from clearing blocks. Rounds 20, 31, and 37 deliberately score 100 *fewer* points from clearing blocks in order to have a faster route that gains a higher time bonus.

An additional note is that the "extra lives" can be treated, not as a backup plan, but as a resource to deliberately spend for additional points. The 560K world record deliberately uses a "MISS" on levels 17 and 51 in order to score additional points. My route takes deaths on levels 21 and 30 instead, for a slightly higher score gain.

Once this route was assembled, I used it in a live run and scored a total of 575,560 points. I uploaded the resulting .inp file (a record of all inputs, usable to replay the game) to MARP, and a video recording of the gameplay to YouTube: https://www.youtube.com/watch?v=hP7FoXSuePo

# Further scoring
The new high score of 575,560 is fairly well optimized, but it could be raised further:
* The execution of the route could be made more efficient by eliminating any careful hesitations before moving or throwing. My WR run moves quickly but not as fast as possible, since throwing a block from the wrong row can derail the roughly 54-minute run. (Several earlier attempts failed in this way). A tool-assisted speedrun (TAS) could use frame-perfect movement to shave seconds off the clock in each stage, scoring an additional 10 points for each in-game second saved. 
* I've described a route that uses the "default" set of puzzles seen if you begin the game immediately after it powers on. There are many other sets of levels accessible by inserting a credit when the RNG seed is in a different state. Some of these may have a higher overall scoring potential. In principle the score optimization exercise outlined above could be repeated for every RNG seed to identify the highest possible score.
* Routing for speed could be more sophisticated. The route minimizes the time spent moving the character, but didn't consider the time each block-throw takes. It also uses a greedy approach to choose which row to use for each move, which may not be full optimal -  for instance, it may choose to move up 1 row then down 3, rather than down 2 then down 1 (where the latter approach is faster overall).

I estimate that these methods could potentially increase the world record by several thousand points - which is to say, less than 1% higher than where it stands now. While unlikely, it's always possible that some additional scoring tech could be discovered - either a bonus deliberately hidden by the developers, or an unintended glitch that enables obtaining higher scores.

# Other romsets

There are five versions (romsets) of the  game: plotting and flipull are the main English and Japanese language releases, respectively. plottinga, plottingb, and plottingu are earlier revisions of the game. 

Romset plottinga (world set 2) enables even higher scores. It had a world record score of 580,330 posted on Twin Galaxies: https://www.twingalaxies.com/games/leaderboard-details/Plotting-world-set-1/mame  The .inp file was tricky to find (https://www.twingalaxies.com/wiki_index?title=Historical:The-Old-Twin-Galaxies-INP-Archive) but eventually located at: https://drive.google.com/file/d/1f0y6ntA9GSipw3WtRWCwXIT-NqQT7pQh/view?usp=drive_link Adding further confusion is that using the old version 0.106 of MAME, "plotting" was actually world set 2 (now referred to as "plottinga" since MAME 0.130u4)

The plottinga world record was already highly optimized, scoring the maximum number of points from matching blocks without losing lives. However, I was able to boost the score by almost 10,000 points to a new WR of 590,260 through faster movement and by deliberately taking a death on levels 20 and 25 in order to score additional points: https://www.youtube.com/watch?v=3ziS4AMAgYY I managed to beat this 18-year old high score for this 37-year old game!
