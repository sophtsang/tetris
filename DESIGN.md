# Tetris Design Document
People:
## Part 1: Single-Player Tetris
i am supposed to implement single player tetris, where i can imagine 

### in ```main.rs```:
the parent thread or main server that launches 1 or 2 child threads for single player or multiplayer, respectively
spawn threads and then like check on_tick() whether a thread has finished some rows -> garbage given to the other thread

```on_tick():``` checks for corresponding player with ```uid``` has recently inputted a not processed (in terms of updating the game board) command like: rotate down, ... or no recent or unprocessed action. 

### in ```/board``` module and board representation:
row 0 is the floor -> clearing a row would be 0 -> 19 + 2 buffer rows for checking topped out, 
each row segment contains 10 column cells: occupied or empty

we only need to keep track of the shape of falling pieces: test for collisions of piece w/ floor and settled rows
when player hard drops or piece collides with non-wall is the orientation of the piece locked -> written to the board
### piece states: ```TetrisState``` Free -> Falling -> Locked
1) ```TetrisState::Free```: able to freely rotate (soft drop)
    - in this state, the tetris piece falls relatively slowly, maybe 1 cell every 3 frames or smth
2) ```TetrisState::Falling```: not able to rotate, piece falls (hard drop)
    - in this state, the tetris piece will fall in greater gravity, maybe 1 cell every frame
3) ```TetrisState::Locked```: piece is set, no change in position anymore, converted to and drawn as ```o```s on the board
    - when drawing ```o```s, check if rows filled on the board
    - _stretch_: have some fun clearing row animation

also need a function to detect if board topped out, as in overflow -> lose: ```fn topped_out()```: basically check if rows 20-21 are occupied, if yes, lose

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

random generator outputs a piece index, which corresponds with a shape from tetrominoes.json, this shape is initalized as some ```Tetris``` piece with ```TetrisState::Free```

### rotation logic:
for each given [tetrominoes.json](resources/tetrominoes.json) piece, this should be written out in [wall_kicks.json](resources/wall_kicks.json)
