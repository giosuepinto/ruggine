// ChatHeader.jsx
import { formatGroupName, formatMembersList } from "../pages/ChatPage";

function ChatHeader({ activeGroup, user, inviteInput, setInviteInput, handleInviteFromChat }) {
    return (
        <div className="chat-header">
            <div className="header-info">
                <h3>{formatGroupName(activeGroup, user)}</h3>
                <span>
                    {activeGroup.is_dm
                        ? "Chat privata"
                        : formatMembersList(activeGroup.members)
                    }
                </span>
            </div>
            {!activeGroup.is_dm && activeGroup.group_id !== 1 && (
                <div style={{display: 'flex', gap: '5px'}}>
                    <input placeholder="Aggiungi membro..." value={inviteInput} onChange={e => setInviteInput(e.target.value)} style={{padding: '6px 10px', borderRadius: '20px', border: '1px solid #ccc', outline: 'none', fontSize: '0.9rem'}} onKeyPress={e => e.key === 'Enter' && handleInviteFromChat()} />
                    <button onClick={handleInviteFromChat} className="mini-btn active" style={{ borderRadius: '20px', padding: '6px 15px' }}>+</button>
                </div>
            )}
        </div>
    );
}

export default ChatHeader;
