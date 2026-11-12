# Tetris
## Changelog

2026-11-04:
- Initial release.
- Add resources flag to writeup.

2026-11-05:
- Add part 2.
- Add note in part 2 about external crates.
- Clarify requirement for larger groups.
- Fix part 2 setup to actually work with an autograder.

2026-11-08:
- Fix tests to match README and fix runt command to use p5_tetris as binary.
- Fix mistake in README in hard drop section, if any cell is above row 20, top
  out, not all

2026-11-09
- Fix error in writeup, blocks should fall when greater than or equal to lock
  delay or gravity, not when greater than.
- Change required groups to strongly encouraged groups.
- Specify more precise requirements for bag randomizer so it actually matches
  the staff solution's randomizer.
- Fix T piece being a Z piece
- Add part 3 writeup

2026-11-10
- Fix incorrect test, topped-out.txt. The staff solution continued to draw
  new pieces to the board despite the user topping out.
- Clean up design doc section formatting.

## Introduction
For project 5 and project 6 you will be implementing a networked Tetris game.
If you've never played Tetris against other people try it out! It's a fun game
and regardless, it will be useful context for this project. Two popular
websites for this are [jstris](https://jstris.jezevec10.com/) and
[tetr.io](https://tetr.io/).

At a high level, multiplayer Tetris involves players, each with their own board
attempting to place tetrominoes in a 10 by 20 grid such that the pieces tile
the grid with no gaps. When a row is tiled, that row is removed and blocks
above are moved downwards to replace it.

It's possible for a single block to clear multiple lines. When this happens,
"garbage" is created and sent to the bottom of the other player's board. This
garbage is a row of filled grid cells with one omitted.

The player who last the longest before being forced a to place a block in the
grid higher than row 20 wins.

Project 5 will involve four parts:
1. implementing single player Tetris
2. implementing a bot to play Tetris. 
3. implementing multiplayer Tetris
4. writing a design document

For this project, we want you to implement something multithreaded for you bot
in (2) and have clients on separate threads communicating with a "server"
thread for part (3). We will expand in the corresponding sections.

The rest you may implement as you want, though we will load test your designs
and the most robust teams may receive a small amount of extra credit on this or
next assignment. So, try to write efficient code whenever possible.

### Submission and Logistics
We strongly encourage you to *work in groups of 3-4*. Please form groups on
gradescope and submit together for both the checkpoint and final submission. You
should have the same groups throughout this and next project as they will build
on each other.

You will submit your project using `cargo package` as previously. Your project
when built should create a binary. We will specify the behavior of that binary
below.

You must also submit a design document describing your design, implementation,
and any optimizations you implement. You should treat this as a proposal as
much as a writeup, and it may make sense to write this before starting your
code (though you will probably want to edit it when you finish). We will
specify what you need to put in this below as well.

(1) will be due Nov 11 as a check-in.
(2), (3), and (4) will be due Nov 18 and will possibly contain edits to your work
submitted for (1).

### AI Policy
As a reminder, you must not use LLMs to generate any part of your submission,
including code, comments, or writing in your design doc. Asking an LLM how to
implement something and using that as inspiration violates this policy.

We discourage, but begrudgingly allow the use of LLMs for seeking out
documentation or descriptions of high level concepts. Though, please seek 
official docs or reach out to a TA through OH or Ed first!

## Part 1: Single Player Tetris
You will implement a binary which simulates a game of Tetris given a list of
inputs describing what a player's action on every frame of gameplay.

You must support the `--simulate-inputs-single-player <inputs-file>` flag. The
flag takes a single argument, a file describing the input format. Upon
receiving a valid file, your binary should print to stdout a serialization of
the board state. You must also support the flag `resources` which specifies a
directory containing JSON files with information such as tetromino shapes,
config, or wall kicks (these will all be explained later). As an example
(assuming you are running your project with `cargo run`):
```
$ cargo run -- --simulate-inputs-single-player input-file.txt --resources resources
...o......
...ooo....
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
......o...
.....oo...
o....oo.o.
ooooooo.oo
oo...o..o.
ooooooooo.
ooooooooo.
ooooooooo.
ooooooooo.
```

We provide further examples with the release code.

### Input Format
The input format is an ascii encoded text file. It describes a single action
per frame of gameplay. The actions can be any of the following
- C: Rotate 90 degrees clockwise
- W: Rotate 90 degrees counter-clockwise
- L: Move piece left
- R: Move piece right
- D: Move piece down
- H: Hard drop piece
- E: No action

Inputs are concatinated into a sequence. For example, an input file describing
rotating clockwise twice, moving a piece left, waiting 3 frames, and then hard
dropping the piece would be:
```
CCLEEEH
```

You should ignore whitespace, so the above input is the same as writing
```
CCL
EEE H
```

### Output Format
The serialized output is a 10x22 grid containing "o" for every cell containing a
part of a tetronmino and "." for every empty cell. You should draw the current
falling piece in addition to any currently filled cells in on the board.

Assuming the piece being placed is a J piece, and the next pieces is an O piece.
the input sequence from above induces the output:
```
....oo....
....oo....
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..ooo.....
....o.....
```

When topped out (the game is over, more precisely defined later), don't show any
next approaching piece.

### Game Behavior
#### Game State
We loosely base our game behavior on the
[Tetris Guidelines](https://tetris.wiki/Tetris_Guideline). Our game will run in
discrete "frames" (which I might also refer to as ticks). Typically there are
60 frames per second, though that is irrelevant for this part of the assignment.

There are many constants used by Tetris which we will define in the provided
`config.json`.

We start by describing the game state at a given frame:
- A 10x20 grid of cells, each cell either filled or empty
- A random piece generator often called a 7-bag
- The current piece on the board
- The position of the current falling piece
- The rotation of the current falling piece
- The number of frames since the current falling piece has been forcefully moved
  down one row
- The gravity: the number of frames it takes for a piece to forcefully move
  down one row
- The number of rotations which have taken place since a piece has been able to
  move down one row

#### Randomizer
A [7-bag](https://tetris.wiki/Random_Generator) is the random generator Tetris
uses for supplying one of 7 pieces to the player to replace a placed piece. It
is analogous to putting each of the seven pieces into a bag and drawing them
randomly without replacement. When the bag is empty, those 7 pieces are placed
back into the bag and drawing continues.

When implementing your randomizer, you *must use the
[rand](https://docs.rs/rand/latest/rand/) version 0.10.3 and
[rand_xoshiro](https://docs.rs/rand_xoshiro/latest/rand_xoshiro/) version 0.8.1
crates with the random seed from `config.json` and the `Xoshiro128PlusPlus`
algorithm.* We provide an implementation matching the staff solution below.
When creating the bag randomizer, pass an iterator ordering the tetrominoes i,
j, l, o, s, t, z, the same they appear in the JSON:
```rs
type RngAlgorithm = Xoshiro128PlusPlus;

/// `BagRandomizer<T>` creates a bag randomizer of items of type T. This can
/// be thought of as a bag of items which are drawn without replacement. Once
/// the bag is empty, the bag is refilled with the items initally drawn. This
/// bounds the time between drawing a duplicate item to the total number of
/// items 2 * (#items - 1).
///
/// This struct provides a `cur()` method to get an item from the randomzier.
/// The same item will be returned until it is removed with a `next()` method.
///
/// ```rs
/// let mut randomizer = BagRandomizer::from_parts(1..=3, [4; 16]);
/// let first_item = *randomizer.cur();
/// let same_item = *randomizer.cur();
/// assert_eq!(first_item, same_item);
///
/// randomizer.next();
/// let different_item = *randomizer.cur();
/// assert_ne!(first_item, different_item);
///
/// randomizer.next();
/// let different_item = *randomizer.cur();
/// assert_ne!(first_item, different_item);
/// ```
pub struct BagRandomizer<T> {
    bag: Vec<T>,
    idx: usize,
    rng: RngAlgorithm,
}

impl<T> BagRandomizer<T> {
    /// Creates a `BagRandomizer` from an interator of items to put in the bag
    /// and a seed for the underlying RNG.
    pub fn from_parts(
        bag: impl Iterator<Item = T>,
        seed: <RngAlgorithm as SeedableRng>::Seed,
    ) -> Self {
        let mut rng = RngAlgorithm::from_seed(seed);

        let mut bag: Vec<T> = bag.collect();
        bag.shuffle(&mut rng);

        Self { idx: 0, bag, rng }
    }

    /// Removes the current exposed value in the bag, cycling to the next
    /// random value.
    pub fn next(&mut self) {
        self.idx += 1;
        if self.idx >= self.bag.len() {
            self.bag.shuffle(&mut self.rng);
            self.idx = 0;
        }
    }

    /// Returns the current exposed value in the bag.
    pub fn cur(&self) -> &T {
        &self.bag[self.idx]
    }
}
```

#### Per frame Sequence
Initially, a piece is drawn from the 7-bag. the position of the piece will be at
column 4, (more generally, (grid width - 1) / 2) and row 20 (more generally
height) both of these 0 indexed. It's rotation will be rotation 0, the spawn
rotation, and the number of frames since the piece has moved down will be 0.

On each frame, the game will:
1. Read the next input from the input file.
2. Perform an action based on the read input.
3. Increment the number of frames since a piece has forcefully moved down a
  row.
  - If this number if greater than or equal to the gravity and the piece can be
    moved down a row, move the piece down a row.
  - If this number is greater than or equal to the pieces *lock delay*, the
    number of frames before a piece locks into place, fill the cells currently
    taken up by the piece.
  - If any cell of this piece are above row 20, stop reading inputs and
    advancing the game state. The player "topped out" and lost.
4. If any rows are composed entirely of filled with cells, remove the row and
  move each above row down towards the bottom.
5. If a piece was locked, pick a new piece from the 7-bag as the current piece
  and spawn it in column 4 row 20 with rotation 0.

Notice that checks for losses occur before checks for line clears. The wiki
refers to this condition as a [partial lock out](https://tetris.wiki/Top_out).

(2) is underspecified. We explain it in depth in the following sections.

#### Rotations
This is for handling C and W commands.

We use the
[Super Rotation System (SRS)](https://tetris.wiki/Super_Rotation_System). it is
worth reading the linked wiki page (it has useful pictures!) though we will try
and summarize the important parts here.

SRS involves specifying the cells occupied by a tetromino for each rotation,
notated 0 (spawn or no rotations), R (right), 2 (two rotations), L (left).

Concretely, for each rotation we specify a length four list of the cells
occupied by the tetromino. We concatinate, in order, the lists representing
rotations 0, R, 2, and L. For example, we represent J with the following JSON:
```json
{
  "states": [
    [[ -1, 1], [-1,  0], [ 0,  0], [ 1,  0]],
    [[  0, 1], [ 1,  1], [ 0,  0], [ 0,  1]],
    [[ -1, 0], [ 0,  0], [ 1,  0], [ 1,  1]],
    [[  0, 1], [ 0,  0], [ 0,  1], [-1,  1]]
  ],
}
```
Each pair is a horizontal and then vertical offset where right and up are the
positive directions.

When the current piece is rotated, it's possible for it to overlap with filled
cells it didn't before. Because it's more fun, Tetris has a "wall kick" system
where depending on the piece and states being transitioned between, the game
will attempted an ordered list of translations on the piece to remove overlaps.
This can be represented as a compressed table. Each tetromino's rotation is
assigned a list of pairs. The offset is then the element-wise difference of
those lists. For example, the J piece has the following table (not we use "o"
to represent 0 and "t" to represent 2):
```
{
    "o": [[ 0,  0], [ 0,  0], [ 0,  0], [ 0,  0], [ 0,  0]],
    "r": [[ 0,  0], [ 1,  0], [ 1, -1], [ 0,  2], [ 1,  2]],
    "t": [[ 0,  0], [ 0,  0], [ 0,  0], [ 0,  0], [ 0,  0]],
    "l": [[ 0,  0], [-1,  0], [-1, -1], [ 0,  2], [-1,  2]]
}
```
If the piece is in the 0 state and transitions to the R state, the offsets
tried will be (in order) `(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)`.

J, L, S, Z, T tetrominoes all use the same wall kicks which we call "Normal". I
and O use special wall kick lists which we entitle "I" and "O" respectively.
Theses values can be found in `wall_kicks.json`.

For understanding wall kicks and a visualization I strongly recommend you read
[this](https://tetris.wiki/Super_Rotation_System#Wall_Kicks) and
[this](https://tetris.wiki/Super_Rotation_System#How_Guideline_SRS_Really_Works).
section of the wiki. Though, reading the entire thing might be best.

#### Translations
This is for handling L and R commands.

Translations left and right are straightforward, subtracting from the piece's
column on L and incrementing it on R. If the piece would be translated such
that one of its cells overlaps with another filled cell or is no longer on the
grid, the translation command should be ignored.

#### Drops
This is for handling D and H commands. D commands represent "soft drops" and H
commands represent "hard drops."

A soft drop temporarily decreases the gravity value by a value specified in the
config file. This makes the block fall faster.

A hard drop instantly moves a piece downwards until that piece cannot move down
a row without one of its cells overlapping a fill cell or being below row 0. It
then locks the piece in place. If any of the cells of the piece are above row
20, stop reading inputs and stop updating game state. The player topped out!

There is also a [wiki page on this](https://tetris.wiki/Drop) which is useful to
read, but note they define gravity as the reciprocal of what we do i.e. we use
the unit 1/G for gravity.

### Part 1 Implementation Advice
- We specify a lot of information in JSON files in `resources`. Reading JSON is
  annoying, so recommend you use the dependencies `serde` and `serde_json`. A
  small example is in the docs [here](https://serde.rs/derive.html).
- Writing a CLI from scratch isn't that hard, but there are also nice libraries
  like [clap](https://docs.rs/clap/latest/clap/) (used in lab 6) or
  [argh](https://docs.rs/argh/latest/argh/). Feel free to use either of these.
- It probably make sense to separate the logic for loading JSON files into its
  own struct. Similarly, it likely makes sense to make structs for tetrominoes,
  the game board, and game itself, which calls functions provided by the game
  board.
- Implement board serialization as soon as possible. This will allow you to
  test features as you implement them with provided integration tests.
- Implement input format parsing so you can do end to end testing as soon as
  possible with provided integration tests.
- Write unit or integration tests for your code as you implement new features.
- When locking a piece in place, remember to reset the counter tracking time
  since a piece has moved down a row due to gravity.

## Part 2: Single Player Tetris Bot
### Setup
For this part, you will implement a bot to play single player Tetris. First,
for grading reasons you should create the following enums, structs, and trait
as follows to module `bot` in a file `src/bot.rs`:
```rs
pub enum TetrominoType {
    I, J, L, O, S, Z, T
}

pub enum TetrisAction {
    C, W, L, R, D, H, E
}

#[derive(Clone, Copy)]
pub enum CellState {
    Filled, Empty
}

pub struct BoardState {
    pub tetromino_row: usize,
    pub tetromino_col: usize,
    pub tetromino_rotation: usize,
    pub tetromino: TetrominoType,
    pub board: [[CellState; 10]; 20],
}

pub trait SinglePlayerBot {
    /// Create a new bot.
    fn new() -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(&mut self, board_state: &BoardState) -> TetrisAction;
}
```

Now, add the following unit test to `tests.rs` in your `tests` directory where
`SinglePlayerBot` is your bot struct implementing `Bot`. It should compile.
```rs
use p5_tetris::bot::{BoardState, Bot, CellState, TetrominoType};

pub type MySinglePlayerBot = /* TODO: Replace this comment with your bot! */;

#[test]
fn create_bot_and_get_any_action_test() {
    let mut bot = MySinglePlayerBot::new();
    let _ = bot.get_move(&BoardState {
        tetromino_row: 20,
        tetromino_col: 4,
        tetromino_rotation: 0,
        tetromino: TetrominoType::J,
        board: [[CellState::Empty; 10]; 20],
    });
}
```

### Requirements Part 2
For part 2, your bot must be able to clear 40 lines in a normal, single player
Tetris board. We will test your bot using a simulator with no time limit per
frame (unlike part 3!!!), however there will be a time limit on the total amount
of time your bot can take to clear 40 lines. Bots based around tree search and
which parallelize their search should have no issue being fast enough.

The autograder will have a test for this, but we also recommend you write a unit
test for this yourself so you don't have to submit to get a ballpark idea of how
fast/good your bot is. You can do this with similar code to your `main.rs` in
part 1, but replacing reading from a file with calls to your bot. You may need
to add code to keep track of the number lines cleared and to check if you bot
topped out.

### Algorithm Advice
You have a lot of freedom in how you approach making a bot, though
most (good) bots you could imagine will likely be some variety of
exponential tree search ([something something Tetris is hard
](https://people.csail.mit.edu/dln/papers/tetris/tetris.pdf)). These
work by trying possible moves and assigning costs to those moves. Then
the move with minimum cost is chosen.  If you have taken CS 3700, this
may be a chance to dust off your knowledge of
[alpha-beta pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning)
or [Monte Carlo Tree
search](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search); if
not, this is a chance to learn something new!

That is, given a board state, you will use your Tetris simulator to try every
single move. This creates a bunch of new boards. You can then recurse, trying
every single move on each of these new boards, creating an n-ary tree of boards
where n is the number of moves from a board.

After doing this some number of times (that number left intentionally vague,
but you could imagine it just being some constant depth for example), each leaf
board can be scored by an objective function. Then the parent of a leaf board
can be scored as the min cost of all its children. This process can be iterated
eventually obtaining the cost for a the board resulting from a single move. The
bot can then choose the move with the minimum cost.

To be said is what the objective function should be. It should take some board
state and grade it based on features we thing are good. There are a lot of
things which could work and there isn't really a right answer here, but here are
some ideas you can try:
- Tetris boards want to have flat tops or only single wells. Add 1 cost for each
  cell which would need to be filled in to make the top flat ignoring the single
  deepest well.
- Tetris boards want to be solid. Add a bunch of cost for each enclosed empty
  cell.
- Tetris bots should want to clear lines. Add cost proportional to the row of
  the highest filled block.

### Implementation Advice
If you use a tree search, how fast it runs, and thus how far ahead you can
search, depends a lot on three things:
1. How well you parallelize work
2. How fast your simulator is
3. How fast your objective function is

We've mentioned ways to speed up (2) and (3) in P2 and lecture, so we will focus
on (1). Instead of implementing (1) as a naive recursive function, we could
split the computation up over multiple threads. There would be a work queue,
maybe represented by an mpsc channel.

An item in the queue is a board state tagged with a command sequence. When an
item is popped off the queue, a new thread be started to compute the board
states possible after k additional moves (for some tunable constant k). New
states, tagged with the move sequences resulting in those board states can be
added back to the work queue.

This process can continue until some stopping condition, like maybe all threads
have finished and there is no board in the queue generated by a sequence of
moves of length less than s (for another tunable constant s). At this point all
of these boards can be scored (possibly doing this in parallel as well with a
similar process to generating the board states) and the bot can choose the move
prefixing the move sequence generating the board with the min cost.

You may use external crates in your implementation, though their abstractions
may not conform super well to the description above. One popular library for
parallel programming in rust is [rayon](https://github.com/rayon-rs/rayon).
Feel free to use or ignore it (or other) creates.

## Part 3: Multiplayer Tetris
Tetris can be made multiplayer by giving each player their own board. Whenever
a player clears multiple lines, one less than that number of lines is sent to
the other player as garbage. In this part, you will first augment your simulator
to handle multiple players and then use your simulator to have multiple bots
compete with each other.

You will add a new flag to your simulator called
`--simulate-inputs-multi-player` which takes a list of input files and outputs
the board of each player. As an example with both `p1-inputs.txt` and
`p2-inputs.txt` identical, the command outputs the following:
```
$ cargo run -- --resources resources --simulate-inputs-multi-player p1-inputs.txt --simulate-inputs-multi-player p2-inputs.txt
...o......    ...o......
...ooo....    ...ooo....
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
..........    ..........
......o...    ......o...
.....oo...    .....oo...
o....oo.o.    o....oo.o.
ooooooo.oo    ooooooo.oo
oo...o..o.    oo...o..o.
ooooooooo.    ooooooooo.
ooooooooo.    ooooooooo.
ooooooooo.    ooooooooo.
ooooooooo.    ooooooooo.
```
### Input Format
The input format is identical to Part 1.

### Output Format
The output format is very similar to Part 1. Each player's board is drawn side
by side, deliminated by four spaces. Player 1, passed to the binary as the first
input file is drawn first, then player 2 drawn second. Board should be truncated
to 22 lines if garbage were to make them taller (see next section).

### Multiplayer Tetris Behavior and Garbage
The simulator now keeps track of two boards at the same time. Simulation per
board progresses as in single player Tetris until a player clears one or
multiple lines on a given tick. When this happens "garbage," a row with one cell
omitted, is sent to the other player. If multiple lines of garbage are sent on
the same tick, each row will have a cell in the same column removed.

Sent garbage can "cancel out" pending garbage sent from the other player.
Garbage not used to cancel other garbage is is added to a garbage queue and
tagged with the amount of time until they are added to the other player's board.

The sequence occurs after a game tick and is as follows:
1. Say player 1 has g1 lines of garbage in their queue, and player 2 has g2
  lines of garbage in their queue.
2. Additionally, say player 1 clears l1 lines and player 2 clears l2 lines.
3. Let s1 = max(l1 - 1, 0), s2 = max(l2 - 1, 0)
4. Player 2's queue gets max(s1 - g1, 0) new pieces of garbage tagged with timer
  "garbage_queue_time" from `config.json` and the oldest min(s1, g1) pieces of
  pending garbage, not including newly added garbage this tick, is removed.
5. Player 1's queue gets max(s2 - g1, 0) new pieces of garbage tagged with timer
  "garbage_queue_time" from `config.json` and the oldest min(s2, g2) pieces of
  pending garbage, not including the newly added garbage this tick, is removed.
6. All garbage in each player's queue tagged with timer equal to 0 has that
  garbage removed from the queue and applied to the bottom that player's board,
  moving all other rows upwards.
7. For a player, if any cell is above at or above the 20th row (row index >=
  20), that player tops out. No further garbage should be applied to topped out
  players.
8. All garbage timers are reduced by 1.

Note: when representing garbage, we can omit the actual missing column when
storing garbage, instead computing it on the fly when it is applied to the game
board.

In order for you solution to match with the staff solution and pasts tests you
must use a `Xoshiro128PlusPlus` per game board, using one call to `.next_u32()`
for an aligned block of garbage and only calling calling to the randomizer when
"on the fly" when garbage is actually added to the game board, once per block of
aligned garbage. The randomizers should be seeded with the random seed from
`config.json`.

### Multiplayer Tetris Bots
Multiplayer bots look similar to single player bots but take in the board state
of both players as well as the garbage state of both players. Thus, they require
a new interface. Add the following to your `bots` module (in `bot.rs`):
```rs
pub struct GarbageLine {
    pub num_rows_to_create: usize,
    pub ticks_till_applied: usize,
}

pub trait MultiPlayerBot {
    /// Create a new bot.
    fn new() -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(
        &mut self,
        p1_state: &BoardState,
        p1_garbage: &[GarbageLine],
        p2_state: &BoardState,
        p2_garbage: &[GarbageLine],
    ) -> TetrisAction;
}
```

And to facilitate grading, add the following to `tests.rs`
```rs
use p5_tetris::bot::MultiPlayerBot;

pub type MyMultiPlayerBot = /* TODO: Replace this comment with your bot! */;

#[test]
fn create_mut_bot_and_get_any_action_test() {
    let mut bot = MyMultiPlayerBot::new();
    let state = &BoardState {
        tetromino_row: 20,
        tetromino_col: 4,
        tetromino_rotation: 0,
        tetromino: TetrominoType::J,
        board: [[CellState::Empty; 10]; 20],
    };
    let _ = bot.get_move(&state, &[], &state, &[]);
}
```

### Connecting Bots to the Server
We now want you to create a harness to allow two bots to play each other. This
involves adding the following flags to your binary:
- `--list-bots`: print a newline separated list of implemented bots
- `--trace-bot-run <bot1> <bot1-tracefile> <bot2> <bot2-tracefile>`: print the
  board state after `<bot1>` or `<bot2>` tops out in the output format described
  above. Write `<bot1>`'s inputs to `<bot1-tracefile>` in the input format
  described above and write `<bot2>`'s inputs to `<bot2-tracefile>` in the input
  format described above.

As an example
```
$ cargo run -- --list-bots
good-bot
bad-bot
$ cargo run -- --trace-bot-run good-bot t1.txt bad-bot t2.txt
...o......    ...o......
...ooo....    .ooooooo..
..........    ...oooooo.
..........    ....oo....
..........    ....oo....
..........    ....oo....
..........    ...oo.....
..........    ....oo....
..........    ...oo.....
..........    ...o......
..........    ...ooo....
..........    ....oo....
..........    ...oo.....
......o...    ...oo.....
.....oo...    ...ooo....
o....oo.o.    ...ooo....
ooooooo.oo    ...oooo...
oo...o..o.    ....oo....
ooooooooo.    ....oo....
ooooooooo.    ....oo....
ooooooooo.    ...ooo....
ooooooooo.    ...ooo....
$ ls
t1.txt t2.txt
$ cat t2.txt
HHHHHHHHHHHHLLHRRRH
```

You should implement at least two bots, one named `good-bot` which should
hopefully play pretty well and one named `bad-bot` which can play however bad
you want. `good-bot` should be able to reliably beat `bad-bot`.

You can run a sanity check for implementing the harness correctly by simulating
an run using the created traces using `--simulate-inputs-multi-player` and
verifying the final board states are the same.

### Requirements for Part 3
We will test Part 3 as follows:
1. Verifying `--simulate-inputs-multi-player` agrees with our staff solution on
  various input traces
2. Running your harness pitting `good-bot` against `bad-bot` and verifying your
  program generates a valid trace and `good-bot` wins.
3. Running the bot you assign to `MyMultiPlayerBot` and verifying it returns a
  move in less than 1/60th of a second on the google cloud servers (e.g. it
  returns a move every frame).

The 1/60th of a second time limit creates a tradeoff. You don't have to make a
meaningful move every frame (e.g. you can return E), so bots which "think" in a
separate thread for multiple frames to find better moves may do better than bots
which choose poor moves quickly, though bots which move quickly can overwhelm
bots which are too slow.

## Integration Testing
We use [runt](https://github.com/rachitnigam/runt) for integration tests. You
can install it with `cargo install runt`. You can then run the integration
tests by building with `cargo build --release` and running `runt cli-tests`.

We encourage you to reading the documentation for `runt` (or simply following)
our example to add your own integration tests.

## Design Document
You may submit your design document as either a TXT (with at most 80 character
lines), Markdown, or PDF file. You should list the name and netids of all
students who contributed to your project on this document.

Your document should include the following sections
1. Architecture:
  - Describe the modules you wrote and any important structs or traits.
  - Describe how they interact and any invariants you maintain between them.
2. Implementation:
  - Describe any interesting algorithms you use or otherwise creative or
    non-obvious implementation decisions.
  - Explain where your code has performance bottlenecks and how you identified them.
    Profile your code to support your argument.
  - What was your bot's algorithm? If you read anything which inspired it,
    please cite it (just a link and brief description of what you used from the
    resource is fine).
3. Testing:
  - Describe how you tested your design.
  - What parts of your project did you devote more or less verification effort
    towards?
4. Work Credit: If you worked in a group, say who did what.
5. Known Issues: If there is something you know doesn't work, describe it here.
6. AI Usage: If you used any AI tools describe how you used them. We are curious
  how you use the tools, especially given the limiting AI policy.
7. Meta Comments: Write any opinions on the assignment. Some things we are
  curious about are:
  - How long did you spend on the assignment, which parts took the longest?
  - What did you like and dislike about the assignment, why?
  - Is there anything you'd like to change about the assignment?

Acknowledgement: The requirements for this document are loosely based off of the
requirements for the design document required by CS 4120's assignments.
