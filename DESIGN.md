# Tetris Design Document
**Members**: Cam Mazzacane (clm357), Utku Melemetci (um44), Sidharth Rao (smr353), Xueqing Tsang (xt73)

## 1. Architecture: 
_TODO: Describe the modules you wrote and any important structs or traits. Describe how they interact and any invariants you maintain between them._

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

_TODO: maybe some more stuff idk_

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
The ```board``` module contains piece and game board logic, along with how the game is visualized to the user. The purpose of having separate ```game``` and ```board``` modules is to decouple logic between game and board states.

_TODO: elaborate on board architecture and impl blah blah blah yap_.

### [```random```](src/randoms.rs)
The ```random``` module contains logic for generating a new piece. We define a ```PieceGenerator``` struct that represents a bag randomizer of the 7 ```Tetromino``` variants. The ```Xoshiro128PlusPlus``` algorithm is used as the random number generator along with a random seed provided by ```config.json```. ```PieceGenerator``` models the 7-bag random generator that Tetris uses, and guarantees that all 7 possible tetromino pieces are able to be eventually drawn.
    
### [```bot```](src/bot.rs)
The ```bot``` module _TODO: i have no clue_.

### [```multiplayer```](src/multiplayer.rs)
The ```multiplayer``` module encapsulates logic and stores information that must be shared between ```Game``` instances during mulitplayer Tetris. This module also contains implementation of the garbage mechanic.

The ```GameLog``` struct keeps track of each player's game states, number of cleared rows per round and garbage queues. If a player's game state is ```Locked{ rows_cleared }```, their corresponding ```cleared_rows``` is incremented. The nature of ```GameLog``` keeping track of all players' game states enable implementation of the garbage mechanic in multiplayer Tetris.

The ```Garbage``` struct represents a block of aligned garbage that a player has generated after one game update round. The amount of garbage that a player has accumulated across frames is stored in their corresponding queue of ```Garbage```s in ```GameLog```. This module takes care of timer incrementation, accumulation and removal, and other garbage-related updates.

### [```action_inputs```](src/action_inputs.rs)
The ```action_inputs``` module contains the definition of an ```Action``` enum that represents possible player actions. This module also contains logic for parsing action inputs to an ```Action```.

### [```load_resources```](src/load_resources.rs)
The ```load_resources``` module contains logic for parsing simulated game files and configs. 

_TODO: whatever RawTetrominoConfig is: String wall_kick instead of WallKick_


## 2. Implementation: 
_TODO: Describe any interesting algorithms you use or otherwise creative or non-obvious implementation decisions. In particular, what was your bot's algorithm? If you read anything which inspired it, please cite it (just a link and brief description of how you used the resource is fine)._

### Actions and Wall-Kicks

### Tetris Bot

### Harness

### Garbage 
One ```Garbage``` instance represents a block of aligned garbage that takes up ```num_rows_to_create``` rows in the player's board. We use a queue to track of a player's accumulated blocks of garbage in order of its age. The implementation details of garbage removal is as follows:
1) We want to remove the oldest pieces of garbage first. One cleared row can only remove one piece or row of garbage (if drawn on the board). As a result, we decrement a ```Garbage```'s ```num_rows_to_create``` by ```min(num_cleared_rows, num_rows_to_create)```.
2) ```Garbage``` is only popped if its ```num_rows_to_create == 0```. 

If the opponent has remaining cleared rows, even after clearing out all of their accumulated garbage, then a new ```Garbage``` with ```num_rows_to_create``` equal to the remaining rows is enqueued.

Only if a player's current piece is locked and they have ```Garbage``` with ```frames_till_applied == 0``` is that block of garbage drawn onto their board. Given that ```Garbage``` is not removed when its timer hits 0, multiple ```Garbage``` blocks can be generated onto the board in the same frame. We generate a different random empty column via ```Xoshiro128PlusPlus``` generator per block.

The timer for each ```Garbage``` in a player's queue is decremented after garbage blocks are possibly generated onto their board. This timer is separate from ```frames_since_next_gravity_event```, which is incremented and determines when a piece is moved down.
___
### Performance
_TODO: Explain where you expect there may be performance bottlenecks in your code and how you identified these spots._

## 3. Testing: 
_TODO: Describe how you tested your design and what parts of your project
did you devote more or less verification effort towards.
verification effort towards._

_Because I am a terrible programmer, I only tested the design of ```main.rs, game.rs, random.rs``` and ```multiplayer.rs``` (garbage functionality) on the provided single- and multiplayer [cli-tests](/cli-tests)._

## 4. Reflection
### Work Credits
**Cam**: Implemented Actions in `board.rs` and `game.rs` (movement, dropping, piece locking, wall kicks/rotation). Implemented the `--trace-bot-run` harness and corresponding CLI flags.

**Utku**:

**Sidharth**:

**Xueqing**: Worked on outlining the single- and multiplayer game loops in ```main.rs```. Implemented multiplayer support and the garbage mechanic in ```multiplayer.rs```. 
___ 
### Known Issues
If there is something you know doesn't work, describe it here. 

_Everything works, we're goated._
___
### Meta Comments
_TODO: Write any opinions on the assignment. Some things we are
  curious about are:_
  - how long did you spend on the assignment, which parts took the longest?
  - what did you like and dislike about the assignment, why?
  - is there anything you'd like to change about the assignment?

___ 