import { Team } from './Game';

type Action = {
  a: {
    x: number,
    y: number,
  },
  b: {
    x: number,
    y: number,
  },
  n: number,
}

type Challenge = {
  id: number,
  source: {
    id: number,
    name: string,
  },
  target: {
    id: number,
    name: string,
  },
  accepted: boolean,
}

type Bot = {
  team: Team,
  stack: number,
}

type User = {
  id: number,
  name: string,
}

type Square = Bot|null;

type Board = Square[64];
type Row = Square[8];

type GameState = {
  turn: Team,
  board: Board,
}

type Game = {
  id: number,
  white: User,
  black: User,
  gamestate: GameState,
}

export { Challenge, Action, Bot, User, Square, Board, GameState, Game, Row };
