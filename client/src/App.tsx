import React, { useCallback, useEffect, useState, useRef } from 'react';
import './App.css';
import { Challenge, Game, Action } from './types';
import GameBoard from './Game';
import Lobby from './Lobby';

enum Team {
  White = "White",
  Black = "Black",
}

const HOST = "ws://0.0.0.0:8008";

const App = () => {
  const [users, setUsers] = useState<any>([]);
  const [challenges, setChallenges] = useState<Challenge[]>([]);
  const [game, setGame] = useState<Game|null>(null);

  const ws = useRef<WebSocket>();

  // Accepts an existing challenge
  const acceptChallenge = (challenge: Challenge) => {
    ws.current?.send(JSON.stringify({
      type: 'acceptChallenge',
      data: { id: challenge.id },
    }));
  }

  // Sends a challenge to a user
  const sendChallenge = (user: any) => {
    ws.current?.send(JSON.stringify({
      type: 'sendChallenge',
      data: { target: user.id },
    }));
  }

  const sendHandshake = (name: string) => {
    ws.current?.send(JSON.stringify({
      type: 'handshake',
      data: name,
    }));
  }

  const sendAction = (action: Action) => {
    ws.current?.send(JSON.stringify({
      type: 'gameAction',
      data: action,
    }));
  }

  const updateChallenges = (newChallenge: Challenge) => {
    const i = challenges.findIndex(challenge => challenge.id == newChallenge.id); 
    if (i >= 0) {
      setChallenges([...challenges.slice(0, i), newChallenge, ...challenges.slice(i+1)]);
    } else {
      setChallenges([...challenges, newChallenge]);
    }
  }

  useEffect(() => {
    ws.current = new WebSocket(HOST);
    ws.current.onopen = (_: any) => sendHandshake('Ben');
  }, []);

  useEffect(() => {
    if (ws.current) {
      ws.current.onmessage = (event: any) => {
        const json = JSON.parse(event.data);
        if (json.type === 'lobbyUserBroadcast') {
          setUsers(json.data[1]);
        }
        if (json.type === 'challengeBroadcast') {
          updateChallenges(json.data);
        }
        if (json.type === 'gameBroadcast') {
          console.log('HIIIIIII');
          setGame(json.data);
        }
        console.log(event);
      };
      ws.current.onclose = (_event: any) => {
          console.log('WebSocket Disconnected');
      }
    }
  }, [ws, setGame, setUsers]);

  return (
    <>
      <div>Boombots</div>
      <Lobby 
        users={users} 
        challenges={challenges} 
        sendChallenge={sendChallenge} 
        acceptChallenge={acceptChallenge}
      />
      <GameBoard game={game} sendAction={sendAction} />
    </>
  );
}

export default App;
export { Team };
