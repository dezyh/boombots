import { Challenge } from './types';
import './Lobby.css';

const Users = ({ users, sendChallenge }: any) => {
  return (
    <div className='user-container'>
      <div className='user-header'>Users</div>
      <div className='user-body'>
        {users.map((user: any) => (
          <div className='user-item' key={user.id} onClick={_ => sendChallenge(user)}>
            {user.id + ' - ' + user.name}
          </div>
        ))}
      </div>
    </div>
  );
}

const Challenges = ({ challenges, acceptChallenge }: { challenges: Challenge[], acceptChallenge: any }) => {
  return (
    <div className='challenge-container'>
      <div className='challenge-header'>Challenges</div>
      <div className='challenge-body'>
        {challenges.map(challenge => (
          <div key={challenge.id} className='challenge-item' onClick={_ => acceptChallenge(challenge)}>
              {(challenge.accepted ? '[Accepted]' : '[Pending]') + ' ' + challenge.source.name + '(' + challenge.source.id + ') -> ' + challenge.target.name + '(' + challenge.target.id + ')'}
          </div>
        ))}
      </div>
    </div>
  );
}

const Lobby = ({ users, challenges, acceptChallenge, sendChallenge }: any) => {
  return (
    <div className='lobby-container'>
      <Users users={users} sendChallenge={sendChallenge} />
      <Challenges challenges={challenges} acceptChallenge={acceptChallenge} />
    </div>
  );
}

export default Lobby;
