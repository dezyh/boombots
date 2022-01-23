import { Board, Game, Square, Bot, Row } from './types';
import { Team } from './App';
import './Game.css';

import { useState, useEffect } from 'react';

type Pos = {
  x: number,
  y: number,
}

type PartialAction = {
  a: Pos|null,
  b: Pos|null,
  n: number,
}

const idxToPos = (idx: number) => {
  let x = idx % 8;
  let y = Math.floor(idx / 8);
  return { x, y };
}

const posToIdx = (pos: Pos) => {
  return 8*pos.y + pos.x;
}



const GameFc = ({ game, sendAction }: any) => {
  const [input, setInput] = useState('');
  const [action, setAction] = useState<PartialAction>({a: null, b: null, n: 1});
  
  // Takes a bot click event (square index and board and computes the next partial action
  const updateAction = (idx: number, board: Board) => {

    let currentAction = {
      a: action.a || null,
      b: action.b || null,
      n: action.n,
    };
    let currentClick = idxToPos(idx);


    console.log('pre ayy');
    if (currentAction.a) {
      console.log('ayy', idx, posToIdx(currentAction.a));
    }

    if (!currentAction.a) {
      // Clicking on a new source
      if (board[idx]) {
        currentAction.a = idxToPos(idx);
        currentAction.n = 1;
      }
    } else if (idx === posToIdx(currentAction.a)) {
      console.log('hi');
      // Clicking on the source again, pick up another robot
      currentAction.n = (1 + currentAction.n) % (1 + board[idx]?.stack);
    } else if (currentClick != currentAction.a) {
      // Click on the move target
      let aIdx = posToIdx(currentAction.a);
      if (!board[idx]?.team || (board[aIdx]?.team === board[idx]?.team)) {
        currentAction.b = currentClick;
      }
    }
    setAction(currentAction);
  }

  const actionValid = () => {
    if (action.a === null)
      return false;

    if (action.a !== null && action.n === 0) 
      return true;

    if (action.a !== null && action.b !== null) 
      return true;
  
    return false;
  }

  const makeAction = () => {
    if (!action.b) {
      sendAction({
        a: action.a,
        b: { x: 0, y: 0 },
        n: action.n,
      });
    } else {
      sendAction(action);
    }
    clearAction();
  }

  const clearAction = () => {
    setAction({a: null, b: null, n: 1}); 
  }

  if (!game) {
    return (<p>Waiting</p>);
  }

  const ActionDisplay = ({ action }: { action: PartialAction }) => {
    if (action.n === 0) {
      return <p>{'Boom @ (' + action?.a?.x + ', ' + action?.a?.y + ')'}</p>
    } 
    return <p>{'Move ' + action.n + ' @ (' + action?.a?.x + ', ' + action?.a?.y + ') ==> (' + action?.b?.x + ', ' + action?.b?.y + ')'}</p>
  }

  return (
    <div>
      <p>White: {game?.white?.name}</p>
      <p>Black: {game?.white?.name}</p>
      <ActionDisplay action={action} />
      <button onClick={makeAction}>Make</button>
      <button onClick={clearAction}>Clear</button>
      <BoardFc board={game?.gamestate?.board} updateAction={updateAction} action={action} />
    </div>
  );
}

const BoardFc = ({ board, updateAction, action }: { board: Board, updateAction: any, action: PartialAction }) => {
  //let rows = [7, 6, 5, 4, 3, 2, 1, 0].map(row => board.slice(8*row, 8*row+8));
  let rows = [7, 6, 5, 4, 3, 2, 1, 0];

  return (
      <table className='board-container'>
        {rows.map(row => <BoardRowFc row={row} board={board} updateAction={updateAction} action={action} />)}
      </table>
  );
}

const BoardRowFc = ({ row, board, updateAction, action }: {row: number, board: Board, updateAction: any, action: PartialAction }) => {
  let idxs = [0, 1, 2, 3, 4, 5, 6, 7].map(col => 8*row + col);

  return (
    <tr className='board-row'>
      {idxs.map((idx: number) => <BoardSquare idx={idx} board={board} updateAction={updateAction} action={action} />)} 
    </tr>
  );
}

const BoardSquare = ({ idx, board, updateAction, action}: {idx: number, board: Board, updateAction: any, action: PartialAction }) => {
  console.log(board);
  if (action.n === 0 && action.a && posToIdx(action.a) === idx) {
    return <td onClick={e => updateAction(idx, board)} className='board-square action-boom'>
      <BoardBot idx={idx} board={board} updateAction={updateAction} action={action} />
    </td>
  }

  if (action.a !== null && posToIdx(action?.a) === idx) {
    return <td onClick={e => updateAction(idx, board)} className='board-square action-source'>
      <BoardBot idx={idx} board={board} updateAction={updateAction} action={action} />
    </td>
  }

  if (action.b !== null && posToIdx(action?.b) === idx && action.n !== 0) {
    return <td onClick={e => updateAction(idx, board)} className='board-square action-target'>
      <BoardBot idx={idx} board={board} updateAction={updateAction} action={action} />
    </td>
  }

  return (
    <td onClick={e => updateAction(idx, board)} className='board-square'>
      <BoardBot idx={idx} board={board} updateAction={updateAction} action={action} />
    </td>
  );
}

const BoardBot = ({ idx, board, updateAction, action }: {idx: number, board: Board, updateAction: any, action: PartialAction}) => {
  let bot = board[idx];

  if (bot?.team == Team.White) {
    return <div className='board-bot-white'>{bot.stack}</div>;
  }
  if (bot?.team == Team.Black) {
    return <div className='board-bot-black'>{bot.stack}</div>;
  }
  return null;
}

export default GameFc;
export { Team };
