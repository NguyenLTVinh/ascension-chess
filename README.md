# Ascension Chess

A chess variant implemented in Rust, featuring special "Ascended" pieces and an in-game upgrade system.

## How to Play

### Local Play
Run the game locally in hotseat mode (two players on one screen):
```bash
cargo run
```

### Online Play
1. **Start the server** (or use an existing one):
   ```bash
   cargo run --bin server
   ```
2. **Client 1 (Create a room)**:
   ```bash
   cargo run -- --create --server 127.0.0.1:8080
   ```
3. **Client 2 (Join the room)**:
   Use the room code generated for Client 1:
   ```bash
   cargo run -- --password <ROOM_CODE> --server 127.0.0.1:8080
   ```
   *Alternatively, create/join a specific room name:*
   ```bash
   cargo run -- --password myroom --server 127.0.0.1:8080
   ```

## Controls

- **Move**: Click to select a piece, click a valid square to move.
- **Upgrade**: Select a piece and press `U` to ascend it (costing points).
- **Promotion**: When a pawn or Hawk Warrior reaches the end:
  - Standard: `Q` (Queen), `R` (Rook), `B` (Bishop), `N` (Knight)
  - Special: `H` (Hawk Warrior), `E` (War Elephant), `A` (Archbishop), `C` (Cannon), `M` (Monarch)

## Game Rules

- **Turn**: +1 Point
- **Capture**: +Value of captured piece
- **Check / Promotion**: +2 Points
- **Castle**: +3 Points

### Ascended Pieces

You can upgrade standard pieces or promote to these special units.

| Piece | Symbol | Cost | Value | Movement |
|-------|--------|-------|-------|----------|
| **Hawk Warrior** | H | 5 pts | 3 pts |**Move**: Forward 1<br>**Capture**: Forward/Diagonal/Side 1 |
| **War Elephant** | E | 7 pts | 5 pts |**Move**: Knight + Diagonal 2<br>**Capture**: Knight + Diagonal 2 |
| **Archbishop** | A | 7 pts | 5 pts |**Move**: Diagonal + Forward/Side 3<br>**Capture**: Diagonal + Forward/Side 3 |
| **Cannon** | C | 8 pts | 7 pts |**Move**: Forward/Side (Rook-like)<br>**Capture**: Forward/Side + Jump One (Xiangqi style) |
| **Monarch** | M | 12 pts | 12 pts | **Move**: Forward/Diagonal/Side (Queen) + Knight<br>**Capture**: Forward/Diagonal/Side (Queen) + Knight |

### Upgrade Costs & Transformations
- **Pawn** → Hawk Warrior (5 pts)
- **Knight** → War Elephant (7 pts)
- **Bishop** → Archbishop (7 pts)
- **Rook** → Cannon (8 pts)
- **Queen** → Monarch (12 pts)

## Build

Requirements: Rust (cargo).

```bash
cargo build --release
```
