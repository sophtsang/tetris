# Tetris Design Document
**Members**: Cam Mazzacane (clm357), Utku Melemetci (um44), Sidharth Rao (smr353), Xueqing Tsang (xt73)

## 1. Architecture: 
### Invariants
1) Once a player's game state is ```ToppedOut```, their board is frozen and their future actions do not update the game. In multiplayer Tetris, other players continue to play as normal.
2) Board updates only occur if a player's piece is locked. Updates to ```board``` include checking whether the player has topped out, clearing out rows, and generating garbage. 

____

### [```resources```](src/resources.rs)
The ```resources``` module contains game related resources and config types. 
Here, the `Tetromino` enum is defined to represent possible tetromino types, and the
`TetrominoConfig` struct to represent information of how each tetromino behaves
regarding shape, rotation, and wall-kick rules. 

```rust
pub enum Tetromino {
    I,
    J,
    L,
    O,
    S,
    T,
    Z    
}
```

The `Config` struct contains game configurations such as ```gravity```, ```lock_delay```, ```random_seed```, etc.

These are passed in at runtime
via `resources/tetrominoes.json` and `resources/wall_kicks.json`, which we use
to generate a value of type `Tetrominoes` that acts as a lookup table mapping
tetromino names to configurations (instead of an arbitrarily-indexed 
fixed-length array).

### [```main```](src/main.rs)
The ```main``` module initializes and runs the game loop. We define the ```Args``` struct for representing command-line arguments such as game flags and config paths. The `clap` library was used for parsing these command-line arguments.

This module also contains separate game loops for single- and multiplayer Tetris, and simulates the per frame action updates of a game. 

### [```game```](src/game.rs)
The ```game``` module encapsulates the game logic and states for an individual player. No internal game logic should be exposed outside of ```Game```. This is with the expection of information that must be shared between ```Game``` instances during mulitplayer. Shared information such as number of cleared rows per round and garbage queues is thus stored in ```GameLog``` in [```multiplayer.rs```](src/multiplayer.rs).

We define the ```GameState``` enum to represent three possible game states:
1) ```Locked{ rows_cleared }```: occurs when a piece is locked into place this turn and the player does not top out. ```row_cleared``` is the number of rows cleared by this piece.
2) ```Playing```: the game continues as normal with the existing tetromino piece.
3) ```ToppedOut```: occurs when the locking of a piece this turn causes the player to top out. This means game over for the player.

Most of the game logic is contained within ```update()```, where per frame:
1) We parse the player's action and update the piece's position and rotation accordingly.
2) We update the frame count, where ```gravity``` represents the number of frames before the piece is moved down a cell. 
3) If a piece is locked and if frames elapsed is greater than ```lock_delay```, then we check whether that piece causes the player to top out. If not, the piece is then drawn on the board and can possibly clear rows and generate garbage to the opposing player if in multiplayer Tetris. A new tetromino piece is then spawned.
4) A ```GameState``` is returned to inform ```main``` and ```multiplayer``` about any changes in the player's game state.

### [```board```](src/board.rs)
The ```board``` module contains piece and game board logic, along with how the game is visualized to the user. Specifically, this module takes care of how the piece interacts with the board for each action frame. This includes updating the pose of a piece, drawing pieces and generated garbage onto a board, clearing rows, etc. The purpose of having separate ```game``` and ```board``` modules is to decouple logic between game and board states.

We define the ```Board``` struct to contain a flattened representation of the game board and keep track of the state of the currently falling tetromino piece. 

### [```random```](src/randoms.rs)
The ```random``` module contains logic for generating a new piece. We define a ```PieceGenerator``` struct that represents a bag randomizer of the 7 ```Tetromino``` variants. The ```Xoshiro128PlusPlus``` algorithm is used as the random number generator along with a random seed provided by ```config.json```. ```PieceGenerator``` models the 7-bag random generator that Tetris uses, and guarantees that all 7 possible tetromino pieces are able to be eventually drawn.
    
### [```bot```](src/bot.rs)
The ```bot``` module encapsulates good and bad Tetris bot logic and functionality.

 We define the required ```SinglePlayerBot``` and ```MultiPlayerBot``` traits and the ```TetrisAction``` enum, which converts to and from our ```Action``` inputs. The ```PlacementBot``` and ```MultiplayerPlacementBot``` structs represent the good single- and multiplayer Tetris bots. Each bot keeps its own copy of the game in sync through ```update_with()``` and returns the next step of its current ```plan``` from ```get_move()```. 

The ```BadBot``` struct represents the bad Tetris bot, which will always hard drop a piece. The harness in ```main``` looks up bots by name (```good-bot```, ```bad-bot```) and only talks to them through ```MultiPlayerBot```.

### [```multiplayer```](src/multiplayer.rs)
The ```multiplayer``` module encapsulates logic and stores information that must be shared between ```Game``` instances during mulitplayer Tetris. This module also contains implementation of the garbage mechanic.

The ```GameLog``` struct keeps track of each player's game states, number of cleared rows per round and garbage queues. If a player's game state is ```Locked{ rows_cleared }```, their corresponding ```cleared_rows``` is incremented. The nature of ```GameLog``` keeping track of all players' game states enable implementation of the garbage mechanic in multiplayer Tetris.

The ```Garbage``` struct represents a block of aligned garbage that a player has generated after one game update round. The amount of garbage that a player has accumulated across frames is stored in their corresponding queue of ```Garbage```s in ```GameLog```. This module takes care of timer incrementation, accumulation and removal, and other garbage-related updates.

### [```action_inputs```](src/action_inputs.rs)
The ```action_inputs``` module contains the definition of an ```Action``` enum that represents possible player actions. This module also contains logic for parsing action inputs to an ```Action```.

### [```load_resources```](src/load_resources.rs)
The ```load_resources``` module contains logic for parsing simulated game files and configs. 


## 2. Implementation: 
### Actions and Wall-Kicks
`action_inputs.rs` contains an enum `Action` with a descriptively-named variant for each type of action, as well as a function `parse_action_input` that takes an input string and attempts to parse it into a vector of `Action`s. 

On every frame, the game reads the player's next action. The action then both moves the current piece (or not, depending on the action) and updates the number of frames in the game state to track gravity.

By designing the `board.rs/move_current_piece` function to succeed even if the piece on the board does not actually move upon acting (e.g. when it is against a wall), we can simply call this function every time a movement action occurs.

#### Wall-Kicks

Since it is not guaranteed for the kick table configuration file to exist, we load the kick table first as a `RawTetrominoConfig`, then convert it into a `TetrominoConfig` containing the wall kick table if the file can be loaded successfully. 

To actually perform a wall kick, the function `rotate_current_piece` iterates through the stored kick table in order. For each wall kick, it calculates the resulting location; if there are no overlaps between the existing board and the piece's new location, it chooses that wall kick.


### Tetris Bot
The bad bot is represented by the ```BadBot``` struct and will hard drop every piece. The expected behavior is that the bad bot should top out very fast.

The good bot is represented by the ```PlacementBot``` and ```MultiplayerPlacementBot``` structs for single- and multiplayer Tetris, respectively. We define ```DEPTH``` as the number of generated pieces to consider after the current piece (we use ```DEPTH = 1```). For each frame, the bot determines its next action input as follows:
1) The bot's ```plan``` represents the sequence of actions that reaches the single best placement found before a piece is hard dropped. When ```plan``` is populated, the bot will execute one action per frame.
2) If ```plan``` is empty, the bot will try every possible placement of the current piece. BFS is performed over ```L, R, C``` and ```W``` action inputs, keeping only sequences of moves that leave the piece in a feasible placement and in the same row. Wall-kicks that change the height of the piece are skipped, and each (column, rotation) is visited once, so the path kept is the shortest.
3) For each reachable spot, the bot will copy ```Game```, hard drop the piece from this placement, and record the path and resulting game state. The copy includes the piece generator, so the resulting game spawns the actual next piece and the bot knows exactly which pieces come next. These placements are scored in parallel with Rayon as follows:
    - If a piece is locked, ```score = (rows_cleared * 200) + future_score(resulting game, DEPTH)```. ```future_score``` runs the same placement and drop simulation on the next piece and keeps its best score (scored with ```DEPTH - 1```). Once ```DEPTH``` reaches 0, it returns ```-board_cost``` of the final board, so line clears are rewarded at every step but the board itself is only judged at the end.
    - If the player tops out, or a future piece has nowhere to go, ```score = -1,000,000```.
    - ```board_cost = HOLE_COST * holes + FLATNESS_COST * flatness + HEIGHT_COST * height```, using the constants below:

      | constant | value | applied to |
      |---|---|---|
      | ```LINE_CLEAR_REWARD``` | 200 | each row cleared, at every step |
      | ```HOLE_COST``` | 20 | each empty cell with a filled cell above it in the same column |
      | ```FLATNESS_COST``` | 1 | cells needed to fill every column up to the tallest one, minus the deepest column (one well is free) |
      | ```HEIGHT_COST``` | 1 | row of the highest filled cell |
      | ```TOP_OUT_COST``` | 1,000,000 | any placement that tops out |

4) The placement with the highest score (ties go to the shorter path) gets the hard drop action added to the end of its action sequence, which then becomes the bot's new ```plan```.
5) After the bot executes ```plan```'s next action, it will continue following ```plan``` if the piece did not fall a row, lock, or cause the bot to top out. Otherwise, the ```plan``` is thrown away and ```get_move``` replans from scratch.

```MultiplayerPlacementBot``` also keeps copies of both players' games and the garbage queues, replaying both players' actions every frame so its copy matches the real match (it treats itself as player 1, so its own action comes first). It searches only its own board and ignores the opponent and pending garbage. Garbage only lands when a piece locks, between plans, so the next search already sees it.


### Harness

The harnesses for multiplayer play and bot play follow similar procedures:

- Wait for a move from all (both) players
- Write a trace file for the corresponding player with the move
- Apply the move to the player's respective `Game` and update its `GameLog`
- Tell each player the other players' moves

For each player there exists a copy of the game state from the point of view of that player.


### Garbage 
One ```Garbage``` instance represents a block of aligned garbage that takes up ```num_rows_to_create``` rows in the player's board. We use a queue to track of a player's accumulated blocks of garbage in order of its age. The implementation details of garbage removal is as follows:
1) We want to remove the oldest pieces of garbage first. One cleared row can only remove one piece or row of garbage (if drawn on the board). As a result, we decrement a ```Garbage```'s ```num_rows_to_create``` by ```min(num_cleared_rows, num_rows_to_create)```.
2) ```Garbage``` is only popped if its ```num_rows_to_create == 0```. 

If the opponent has remaining cleared rows, even after clearing out all of their accumulated garbage, then a new ```Garbage``` with ```num_rows_to_create``` equal to the remaining rows is enqueued.

Only if a player's current piece is locked and they have ```Garbage``` with ```frames_till_applied == 0``` is that block of garbage drawn onto their board. Given that ```Garbage``` is not removed when its timer hits 0, multiple ```Garbage``` blocks can be generated onto the board in the same frame. We generate a different random empty column via ```Xoshiro128PlusPlus``` generator per block.

The timer for each ```Garbage``` in a player's queue is decremented after garbage blocks are possibly generated onto their board. This timer is separate from ```frames_since_next_gravity_event```, which is incremented and determines when a piece is moved down.
___
### Performance
We have a profiler that plays the bot on a fixed game until it clears 40 lines, once as the single player bot and once as the multiplayer bot against an idle opponent. It times every move the bot makes, counts how many frames the bot spends on each piece, and records the slowest move per piece, which is the frame where the bot plans its placement. It also times the individual operations the bot's search repeats for every placement it tries.

All measurements below come from running the profiler on a 22-core WSL2 machine.

| single player | ```DEPTH = 1``` |
|---|---|
| frames / pieces to clear 40 lines | 489 / 122 |
| frames per piece | on average 4.0 |
| search time per piece (median) | 1.75-1.84 ms |
| search time per piece (max) | 2.9-4.2 ms |
| ```get_move``` on non-search frames (median) | < 40 ns |
| ```get_move``` averaged over all frames | 0.42-0.44 ms |
| ```update_with``` per frame | avg ~0.15 µs, max 1-30 µs |
| whole 40-line run (wall-clock / CPU) | ~0.21 s / ~1.1 s |

| operation (```parts```) | average time |
|---|---|
| ```Game::clone``` | 333 ns |
| ```Board::clone``` | 319 ns |
| ```Game::clone``` + hard drop | 634 ns |
| board cost (holes + flatness + height) | 268 ns |

### Bottlenecks
1. **All of the work happens on one frame per piece.** This means that the good-bot searches only when its plan is empty, i.e. on the first frame of a new piece. So the worst frame, not the average frame, decides whether the bot meets the 1/60 s multiplayer budget. When ```DEPTH = 2```, almost every piece's took more than 1/60s to output an action.
2. **The search is exponential in ```DEPTH```.** Each piece has up to a few dozen (column, rotation) placements, and the bot tries every placement of the current piece and the next ```DEPTH``` pieces, so the number of simulated drops is roughly (placements)^(```DEPTH``` + 1). Going from ```DEPTH = 2``` to ```DEPTH = 1``` removes one factor of possible placements and allows for search time to be confined within the 1/60 s cap.
3. **Every simulated placement clones a full ```Game```.** Each clone + drop takes ~634 ns no matter the depth, so the search time grows with the number of placements tried: ~50 ms per search with ```DEPTH = 2```, and ~1.8 ms with ```DEPTH = 1```, where the bot tries far fewer placements. Cloning is expensive because each ```Board``` owns its own copy of ```Tetrominoes``` (7 shapes with 4 wall-kick ```Vec```s each), which never changes during a game. ```drop_options``` also clones the board twice per placement.
4. **Symmetric placements are searched more than once.** Rotations of O (and the two equivalent rotations of I, S and Z) can produce the same board, so those subtrees are repeated.

To address these bottlenecks:
1. **Rayon**: the bot scores each placement of the current piece in parallel, which made a DEPTH = 2 run use about 16x more CPU time than wall time.
2. **Plan once per piece**: the bot saves the moves for its chosen placement and only searches again when a new piece spawns, so the other frames of a piece cost almost nothing.

## 3. Testing: 
The design of simulated single- and multiplayer Tetris games was verified mainly on the provided single- and multiplayer [cli-tests](/cli-tests). Specifically, these CLI tests covered gravity and drop timing, piece movement, hard drops, wall kicks, line clears, and topping out behaviors. For multiplayer Tetris, garbage generation behavior was also verified. Beyond the provided CLI tests, expected behavior in piece spawning, locking, and overlaps with the board, along with row-clearing in the ```board``` module was verified on unit tests. Correct configuration from the provided JSON files were also verified on unit tests in ```resources```.

Checks that the single- and multiplayer Tetris bots implementations matched the required interface were done via [tests](/tests/tests.rs). The good bot's ability to 40 lines was verified over [bot_40_lines](/tests/bot_40_lines.rs) tests.

## 4. Reflection
### Work Credits
**Cam**: Implemented Actions in `board.rs` and `game.rs` (movement, dropping, piece locking, wall kicks/rotation). Implemented the `--trace-bot-run` harness and corresponding CLI flags.

**Utku**: Implemented single and multiplayer Tetris bots in ```bot.rs```.

**Sidharth**: Worked on implementation of board-related functionality in ```board``` module. Implemented single and multiplayer Tetris bots in ```bot.rs```.

**Xueqing**: Worked on outlining the single- and multiplayer game loops in ```main.rs```. Implemented the multiplayer Tetris garbage mechanic in ```multiplayer.rs```. 
___ 
### Meta Comments
AI was used miniminally amongst members and only to verify overall Tetris architecture and mechanics.

There were no known issues in the assignment beyond some outdated CLI tests. 

Each member spent around 10-15 hrs on the entire assignment, with implementation of the good bot, multiplayer Tetris garbage mechanic, and harness taking the longest.
___ 