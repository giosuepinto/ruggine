// src/pages/ChatPage.jsx
import { useState, useEffect, useRef } from 'react';
import { useWebSocket } from '../context/WebSocketContext';
import '../App.css';

const formatGroupName = (group, currentUsername) => {
    if (group.group_id === 1) return "🌍 GLOBAL CHAT";
    if (group.is_dm) {
        const otherUser = group.members.find(m => m.username !== currentUsername);
        return otherUser ? otherUser.username : 'Sconosciuto';
    }
    return group.group_name || 'Gruppo';
};

const getAvatarInitial = (name) => name ? name.charAt(0).toUpperCase() : '?';

function ChatPage() {
    const { user, groups, messages, sendMessage, sendInvite } = useWebSocket();

    const [activeGroupId, setActiveGroupId] = useState(null);
    const [input, setInput] = useState('');
    const [inviteInput, setInviteInput] = useState('');
    const messagesEndRef = useRef(null);

    // Auto-scroll
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [messages, activeGroupId]);

    // Pulisci input al cambio chat
    useEffect(() => {
        setInput('');
    }, [activeGroupId]);

    // Selezione automatica Global Chat
    useEffect(() => {
        if (!activeGroupId && groups.some(g => g.group_id === 1)) {
            setActiveGroupId(1);
        }
    }, [groups]);

    const activeGroup = groups.find(g => g.group_id === activeGroupId);
    const activeMessages = messages[activeGroupId] || [];

    const getLastMessagePreview = (groupId) => {
        const groupMsgs = messages[groupId];
        if (!groupMsgs || groupMsgs.length === 0) return "Nessun messaggio";
        const lastMsg = groupMsgs[groupMsgs.length - 1];
        let content = lastMsg.content;
        if (content.length > 25) content = content.substring(0, 25) + "...";
        return <span><span className="chat-preview-author">{lastMsg.from}:</span> {content}</span>;
    };

    const handleSend = () => {
        if (input.trim() && activeGroupId) {
            sendMessage(activeGroupId, input);
            setInput('');
        }
    };

    return (
        <div style={{ display: 'flex', height: '100%', width: '100%' }}>

            {/* SIDEBAR: SOLO LISTA CHAT ORA */}
            <div className="sidebar">
                <div className="sidebar-header" style={{ padding: '20px' }}>
                    <h3 style={{ margin: 0, color: '#333' }}>Le tue Chat</h3>
                </div>

                <div className="chat-list">
                    {groups.map(group => {
                        const name = formatGroupName(group, user);
                        // Non mostrare DM "pendenti" con 1 solo membro (già filtrati dal backend, ma per sicurezza)
                        if (group.is_dm && group.members.length < 2) return null;

                        return (
                            <div
                                key={group.group_id}
                                className={`chat-item ${activeGroupId === group.group_id ? 'active' : ''}`}
                                onClick={() => setActiveGroupId(group.group_id)}
                            >
                                <div className="avatar" style={group.group_id === 1 ? {backgroundColor: '#ffd700', color: 'black'} : {}}>{getAvatarInitial(name)}</div>
                                <div className="chat-info">
                                    <div className="chat-name">{name}</div>
                                    <div className="chat-preview">{getLastMessagePreview(group.group_id)}</div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* CHAT WINDOW */}
            <div className="chat-window">
                {activeGroupId && activeGroup ? (
                    <>
                        <div className="chat-header">
                            <div className="header-info">
                                <h3>{formatGroupName(activeGroup, user)}</h3>
                                <span>{activeGroup.is_dm ? "Chat privata" : `${activeGroup.members.length} membri`}</span>
                            </div>

                            {/* Invito membri (solo per Gruppi non globali) */}
                            {!activeGroup.is_dm && activeGroup.group_id !== 1 && (
                                <div style={{display: 'flex', gap: '5px'}}>
                                    <input
                                        placeholder="Aggiungi membro..."
                                        value={inviteInput} onChange={e => setInviteInput(e.target.value)}
                                        style={{padding: '6px 10px', borderRadius: '20px', border: '1px solid #ccc', outline: 'none', fontSize: '0.9rem'}}
                                    />
                                    <button
                                        onClick={() => { sendInvite(activeGroupId, inviteInput); setInviteInput(''); }}
                                        className="mini-btn active"
                                        style={{ borderRadius: '20px', padding: '6px 15px' }}
                                    >
                                        +
                                    </button>
                                </div>
                            )}
                        </div>

                        <div className="messages-container">
                            {activeMessages.map((msg, idx) => {
                                const isMe = msg.from?.toLowerCase() === user?.toLowerCase();
                                return (
                                    <div key={idx} className={`message ${isMe ? 'sent' : 'received'}`}>
                                        <span className="msg-author">{msg.from}</span>
                                        {msg.content}
                                    </div>
                                );
                            })}
                            <div ref={messagesEndRef} />
                        </div>

                        <div className="input-area">
                            <input
                                className="message-input" placeholder="Scrivi un messaggio..."
                                value={input} onChange={e => setInput(e.target.value)}
                                onKeyPress={e => e.key === 'Enter' && handleSend()}
                            />
                            <button className="send-btn" onClick={handleSend} disabled={!input.trim()}>➤</button>
                        </div>
                    </>
                ) : (
                    <div className="empty-state">
                        <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>💬</div>
                        Seleziona una chat o crea una nuova connessione nella pagina Inviti.
                    </div>
                )}
            </div>
        </div>
    );
}

export default ChatPage;