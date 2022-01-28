# Boombots
A simple 2-player game player over a standard 8x8 board.

## Projects
### Game Server (v0.0.1)
A parallel and asynchronous game server written in Rust. 
### Web Client (v0.0.1)
A React client to interface with the game server over websockets.
### Cli Client - (in-progress)
A Rust terminal client to interface with the game server.

## Game
### Gameplay
1. White starts first and players alternate turns after each action
2. On a players turn, they can either:
    - Boom one of their stacks of robots
    - Move some (or all) of one of their stacks of robots to another square
3. If booming a stack of robots, this causes all robots in a 3x3 radius to also boom, creating a chain reaction until there are no more robots in a blast radius.
4. If moving some of a stack of robots, the player may move 1-N robots from the stack of height N up to N units in each cardinal direction (N, E, S, W)
    - The number of robots moved and the distance the robots move are separate. For example, given a robot with a stack height of 5, one could move 2 robots 5 units to the left.
    - A robot can not move onto a square controlled by an opponents robot. A robot can move onto a square controlled by the same players robot, in which case they combine to form a larger stack.
5. When a player has no more robots, they lose the game. If both players robots all boom on the same turn, the game ends in a draw.
