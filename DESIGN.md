# Tetris Design Document
1. Architecture: Describe the modules you wrote and any important structs or
  traits. Describe how they interact and any invariants you maintain between
  them.

TODO: Describe modules we wrote and what the do at a high level. put the file names they are stored in
- parsing
- game loop

We use the `Tetromino` enum to represent possible tetromino types, and the
`TetrominoConfig` struct to represent information of how each tetromino behaves
regarding shape, rotation, and wall-kick rules. These are passed in at runtime
via `resources/tetrominoes.json` and `resources/wall_kicks.json`, which we use
to generate a value of type `Tetrominoes` that acts as a lookup table mapping
tetromino names to configurations (instead of an arbitrarily-indexed 
fixed-length array).



2. Implementation: Describe any interesting algorithms you use or otherwise
  creative or non-obvious implementation decisions. Explain where you expect
  there may be performance bottlenecks in your code and how you identified
  these spots. In particular, what was your bot's algorithm? If you read
  anything which inspired it, please cite it (just a link and brief description
  of how you used the resource is fine).
3. Testing: Describe how you tested your design and what parts of your project
  did you devote more or less verification effort towards.
  verification effort towards.
4. Work Credit: If you worked in a group, say who did what.
5. Known Issues: If there is something you know doesn't work, describe it here.
6. Meta Comments: Write any opinions on the assignment. Some things we are
  curious about are:
  - how long did you spend on the assignment, which parts took the longest?
  - what did you like and dislike about the assignment, why?
  - is there anything you'd like to change about the assignment?

## Part 1: Single-Player Tetris
i am supposed to implement single player tetris, where i can imagine 

### in ```main.rs```:
the parent thread or main server that launches 1 or 2 child threads for single player or multiplayer, respectively
spawn threads and then like check on_tick() whether a thread has finished some rows -> garbage given to the other thread

```on_tick():``` checks for corresponding player with ```uid``` has recently inputted a not processed (in terms of updating the game board) command like: rotate down, ... or no recent or unprocessed action. 

### in ```/board``` module and board representation:
row 0 is the floor (0,0 is the bottom left) -> clearing a row would be 0 -> 19 + 2 buffer rows for checking topped out, each row segment contains 10 column cells: occupied or empty

we only need to keep track of the shape of falling pieces: test for collisions of piece w/ floor and settled rows
when player hard drops or piece collides with non-wall is the orientation of the piece locked -> written to the board
### piece states: ```TetrisState``` Free -> Falling -> Locked
1) ```TetrisState::Free```: able to freely rotate (soft drop)
    - in this state, the tetris piece falls relatively slowly, maybe 1 cell every 3 frames or smth
2) ```TetrisState::Falling```: still able to rotate, piece falls faster (hard drop)
    - in this state, the tetris piece will fall in greater gravity, maybe 1 cell every frame
3) ```TetrisState::Locked```: piece is set, no change in position anymore, converted to and drawn as ```o```s on the board
    - when drawing ```o```s, check if rows filled on the board
    - _stretch_: have some fun clearing row animation

also need a function to detect if board topped out, as in overflow -> lose: ```fn topped_out()```: basically check if row 20-21 are occupied after a piece is locked, if yes, lose

clear consecutive full row function: ```fn clear_rows()```:
board module that stores filled or not filled (union find-esque storage)

don't rly care about the actual shape of the tetris piece after it has been dropped: so for occupied spaces in the board, we can keep track of board segements -> list of horizontal lists:
keep track of: column indices of a row that are free v. occupied
this is to detect collisions
process from bottom to top?
if the result of a drop causes ```i```th row to be fully occupied, then:
we have a function called clear row:
- ```board[0..i]``` + ```board[i+1..]``` + new empty top row
-> however many rows``` are occupied at ```on_tick()``` produces however many garbage rows to the opponent (for multiplayer)
then we also have a function called add row:
add a row with 1 missing cell (randomized column)

### piece generation:
after a piece is LOCKED and the board updated -> generate a new piece:
randomized generator thingy: ```7-bag``` and ```rand```/```rand_xoshiro```/```Xoshiro128PlusPlus``` algorithm with ```random_seed``` from [config.json](resources/config.json)
idk what this is specifically, but the readme says to use this particular random num generator

*represent the tetromino by its defined center*

random generator outputs a piece index, which corresponds with a shape from tetrominoes.json, this shape is initalized as some ```Tetris``` piece with ```TetrisState::Free```

### rotation logic:
for each given [tetrominoes.json](resources/tetrominoes.json) piece, this should be written out in [wall_kicks.json](resources/wall_kicks.json)

rotate by *Super Rotation System (SRS)*

### for incrementing frames
the number of frames since the current falling piece has been forcefully moved down one row
- frame increases as the piece has not been moved down one cell yet
- frame = 0 the instance the cell has moved down to a new cell
- gravity == max frames before a piece  has to forcibly be moved down to the next row (naturally)

### bounds checking
check whether any of the 4 cells of a tetromino has exceeded the bounds

## Steps:
### json and arg parsing:
tetromino.json, config.json and user's input parsing

### game representation:
where are all the pieces -> visualizer 

### piece generator:
7-bag randomizer
part of the game state, is a seperate module

### handle per-frame sequence step
once you get the piece, how to compute frames, when rotate and translate

### wall-kicking:


___

## Interface
### Parsing

`clap`
--simulate-inputs-single-player {filename}
--resources {folder}

Get a struct with type `Config` with all the relevant files and resources.

Parse config.json, tetrominoes.json, wall-kicks.json
```rust
struct Config {
    width: u32,
    height: u32,
    gravity: u32,
    gravity_decrease: u32,
    lock_delay: u32,
    random_seed: Vec<u32>
}
```

```rust
struct Wallkick {
    o: u32[5][2],
    r: u32[5][2],
    t: u32[5][2],
    l: u32[5][2],
}
```

```rust
// initialized at the beginning of the game
struct TetrinomoConfig {
    states: u32[4][2],
    wall_kick: Wallkick,
}

impl TetrominoConfig {
    fn new(i: Tetromino, config: JsonType) -> Self {
        Self {
            states: config[i]["states"],
            wall_kick: config[i]["wall_kick"]
        }
    }
}

// enum for representing all possible Tetromino types
pub enum Tetromino {
    I,
    J,
    L,
    O,
    S,
    T,
    Z    
}

struct Tetrominoes {
    i: TetrominoConfig,
    ...
}

// in board, init Tetrominoes::new(), which initalizes all TetrominoConfigs
// and this is one instance that can be accessed for any new tetromino generated
impl Tetrominoes {
    fn config(&self, tetromino: Tetromino) -> &TetrominoConfig {
        match tetromino {
            Tetromino::I => self.i,
            ...
        }
    }
}

___

type RngAlgorithm = Xoshiro128PlusPlus;

struct PieceGenerator {
    bag: Tetromino[7], 
    idx: usize,
    rng: RngAlgorithm
}

impl PieceGenerator {
    pub fn new(
        bag: impl Iterator<Item = Tetromino>, 
        seed: <RngAlgorithm as SeedableRng>::Seed
    ) -> Self {
        let mut rng = RngAlgorithm::from_seed(seed);
        let mut bag: Tetromino[7] = bag.collect();
        bag.shuffle(&mut rng);
        Self { idx: 0, bag, rng }
    }

    // for getting a new piece, update randomized iter
    pub fn next(&mut self) {
        self.idx += 1;
        if self.idx >= self.bag.len() {
            self.bag.shuffle(&mut self.rng); // 
            // start over, s.t. we get index 0 again
            self.idx = 0;
        }
    }

    // then call cur to get the piece
    pub fn cur(&self) -> &Tetromino {
        // remember that bag contains the Tetromino pieces
        &self.bag[self.idx]
    }
}

struct Board {
    grid: u32[][]
}

impl Board {
    fn serialize(&self) -> String
    fn update(...)
}

struct Game {
    board: Board,
    piece_generator: PieceGenerator, // 7-bag
    current_piece: Tetromino,
    current_piece_position: u32[2],
    current_piece_rotation: Rotation,
    frames_since_last_gravity_event: u32,
    rotations_since_moved_down: u32,
}

impl {
    fn frame(&mut self)
}

enum Action {
    C, W, L, R, D, H, E
}

main() {
    // parse stuff
    //
    // let piece_generator = PieceGenerator::new(bag, seed)
    // make board: let board = Board::new();
    // let game = Game{
    //  board: board, piece_generator: 
    // }

    // thread stuff eventually
    while not_lose {
        // game stuff on parsed stuff
        // board module,
        // piecegenerator
    }

    // death
}