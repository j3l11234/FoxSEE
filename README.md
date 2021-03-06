# FoxSEE Chess Engine 🦊
A tiny yet strong chess engine written in Rust!  

**Current Version**  
0.1.8

![alt text](https://travis-ci.org/redsalmon91/FoxSEE.svg?branch=master)

## Main Features

- 0x88 Board Representation
- Minimax Search with Alpha-Beta Pruning
- Aspiration Window
- Null-move Pruning
- History Heuristic
- Refutation Table
- Quiescence Search
- SEE
- MVV-LVA
- Piece-square Evaluation Table
- UCI-compatible (only the part needed for CCRL testing/competition)

## Other Features

- Always search captures before non-capture moves.
- Generate & search caslting moves at the very end.
- Extend search depth when in-check: I have tried to add a "backpropagation score" to each in-check ply, but it performed worse than doing depth extension. I have also limited the extension to `< ply / 2` to avoid search explosion. In test, this helps to quickly identify a mate sequence.
- Extend search depth when SEE score is larger than a certain threshold: This feature is not stable yet; I still need to figure out the best parameters (the "threshold") to use.
- I use `-non_captured_move_count` when one side has `score > advantage score` to encourage exchange when one side is in advantage & to avoid repeated moves.

The following are inspired by the 0x88 bit-mask:

- Bit-based piece type checking.
- Bit-based piece color checking.
- Encode all move info into an `u32` integer.

## Notes
- The engine is covered with good amount of tests (a good habit makes coding fun). You can run them with `cargo test --release`. 
- Transposition Table is not used in this version because all PRNG implementations I have tried lead to some key collisions and fail a high percentage of tests that I have prepared (> 1%). I need to find a better way to do trail-error while generating zobrist keys. Hopefully it will be added in the `0.2.x` versions.
- LMR & Futility Pruning are not used. I am not a big fan of these aggressive forward-pruning techniques; I believe the same performance gain can be achieved if the move ordering is good.

## How to build
Run `cargo build --release`

## References
- CPW (https://www.chessprogramming.org)

## Tools used in Testing
- Lichess FEN Editor (https://lichess.org/editor)
