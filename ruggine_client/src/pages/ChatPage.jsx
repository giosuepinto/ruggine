// src/pages/ChatPage.jsx
import { useState, useEffect, useRef, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import '../App.css';

// Helper per formattare il nome del gruppo
const formatGroupName = (group, currentUsername) => {
    if (group.is_dm) {
        const otherUser = group.members.find(m => m.username !== currentUsername);
        return otherUser ? otherUser.username : 'Sconosciuto';
    } else {
        return group.group_name || 'Gruppo';
    }
};

// Helper per ottenere l'iniziale per l'avatar
const getAvatarInitial = (name) => name ? name.charAt(0).toUpperCase() : '?';

function ChatPage() {
    const socket = useRef(null);
    const navigate = useNavigate();
    const username = localStorage.getItem('username');

    // Stati
    const [isConnected, setIsConnected] = useState(false);
    const [groups, setGroups] = useState([]);
    const [activeGroupId, setActiveGroupId] = useState(null);
    const [messages, setMessages] = useState({}); // { group_id: [msgs] }
    const [input, setInput] = useState('');

    // Stati UI per creazione chat
    const [newChatInput, setNewChatInput] = useState('');
    const [newChatType, setNewChatType] = useState('dm'); // 'dm' | 'group'
    const [inviteInput, setInviteInput] = useState('');

    // --- EFFECT: Connessione WebSocket ---
    useEffect(() => {
        if (!username) { navigate('/'); return; }
        const token = localStorage.getItem('token');

        socket.current = new WebSocket('ws://127.0.0.1:3000/ws');

        socket.current.onopen = () => {
            console.log('WS Connesso');
            setIsConnected(true);
            socket.current.send(JSON.stringify({ type: "auth", token: token }));
        };

        socket.current.onclose = () => {
            console.log('WS Disconnesso');
            setIsConnected(false);
        };

        socket.current.onmessage = (event) => {
            const msg = JSON.parse(event.data);

            if (msg.type === 'update_group_list') {
                setGroups(msg.groups);
                // Popola i messaggi iniziali se non presenti
                const initialMsgs = {};
                msg.groups.forEach(g => {
                    // Trasforma ServerMessage in formato UI interno
                    initialMsgs[g.group_id] = g.messages.map(m => ({
                        from: m.from_username,
                        content: m.content
                    }));
                });
                // Merge intelligente: se ho già messaggi locali (più recenti), non sovrascriverli brutalmente
                // Per semplicità qui sostituiamo o facciamo merge se vuoto.
                setMessages(prev => {
                    const newState = { ...initialMsgs }; // Parte con lo storico
                    // Se avevamo già stato attivo, potremmo volerlo preservare,
                    // ma qui ci fidiamo che update_group_list arrivi all'inizio.
                    // Per una gestione perfetta servirebbe un merge più complesso.
                    return newState;
                });
            }
            else if (msg.type === 'group_chat') {
                setMessages(prev => ({
                    ...prev,
                    [msg.group_id]: [
                        ...(prev[msg.group_id] || []),
                        { from: msg.from_username, content: msg.content }
                    ]
                }));
            }
            else if (msg.type === 'error') {
                if (msg.content.includes("Token") || msg.content.includes("autenticarti")) {
                    localStorage.clear();
                    navigate('/');
                } else {
                    alert(msg.content);
                }
            }
        };

        return () => { if (socket.current) socket.current.close(); };
    }, [username, navigate]);


    // --- HANDLERS ---
    const handleSendMessage = () => {
        if (!input.trim() || !activeGroupId) return;
        socket.current.send(JSON.stringify({
            type: "send_group_message",
            group_id: activeGroupId,
            content: input
        }));
        setInput('');
    };

    const handleCreateChat = () => {
        if (!newChatInput.trim()) return;
        const type = newChatType === 'dm' ? "create_dm" : "create_group";
        const payload = newChatType === 'dm' ? { username: newChatInput } : { name: newChatInput };

        socket.current.send(JSON.stringify({ type, ...payload }));
        setNewChatInput('');
    };

    const handleInvite = () => {
        if (!inviteInput.trim() || !activeGroupId) return;
        socket.current.send(JSON.stringify({
            type: "invite_to_group",
            group_id: activeGroupId,
            username: inviteInput
        }));
        setInviteInput('');
    };

    const handleLogout = () => {
        if (socket.current) socket.current.close();
        localStorage.clear();
        navigate('/');
    };

    // --- RENDER HELPERS ---
    const activeGroup = groups.find(g => g.group_id === activeGroupId);
    const activeGroupMessages = messages[activeGroupId] || [];

    // Calcolo anteprima ultimo messaggio per ogni gruppo
    const getLastMessagePreview = (groupId) => {
        const groupMsgs = messages[groupId];
        if (!groupMsgs || groupMsgs.length === 0) return "Nessun messaggio";

        const lastMsg = groupMsgs[groupMsgs.length - 1];
        const isMe = lastMsg.from === username;
        const author = isMe ? "Tu" : lastMsg.from;

        let content = lastMsg.content;
        if (content.length > 20) content = content.substring(0, 20) + "...";

        return <span title={lastMsg.content}><span className="chat-preview-author">{author}:</span> {content}</span>;
    };

    return (
        <div className="app-container">

            {/* 1. NAVBAR SUPERIORE */}
            <div className="navbar">
                <div className="nav-brand">
                    <span className={`status-dot ${isConnected ? 'status-online' : 'status-offline'}`}></span>
                    Ruggine Chat
                </div>
                <div className="nav-user-info">
                    <span>Ciao, <strong>{username}</strong></span>
                    <button onClick={handleLogout} className="logout-btn">Logout</button>
                </div>
            </div>

            {/* 2. AREA CONTENUTO */}
            <div className="main-content">

                {/* SIDEBAR (Lista Chat) */}
                <div className="sidebar">
                    {/* Header Sidebar: Creazione */}
                    <div className="sidebar-header">
                        <div className="new-chat-controls">
                            <input
                                type="text" className="new-chat-input" placeholder="Cerca o crea..."
                                value={newChatInput} onChange={e => setNewChatInput(e.target.value)}
                            />
                            <button className="create-btn" onClick={handleCreateChat}>+</button>
                        </div>
                        <div style={{marginTop: '5px', display: 'flex', gap: '5px', justifyContent: 'center'}}>
                            <button className={`mini-btn ${newChatType === 'dm' ? 'active' : ''}`} onClick={() => setNewChatType('dm')}>DM</button>
                            <button className={`mini-btn ${newChatType === 'group' ? 'active' : ''}`} onClick={() => setNewChatType('group')}>Gruppo</button>
                        </div>
                    </div>

                    {/* Lista Chat */}
                    <div className="chat-list">
                        {groups.map(group => {
                            const name = formatGroupName(group, username);
                            return (
                                <div
                                    key={group.group_id}
                                    className={`chat-item ${activeGroupId === group.group_id ? 'active' : ''}`}
                                    onClick={() => setActiveGroupId(group.group_id)}
                                >
                                    <div className="avatar">{getAvatarInitial(name)}</div>
                                    <div className="chat-info">
                                        <div className="chat-name">{name}</div>
                                        <div className="chat-preview">{getLastMessagePreview(group.group_id)}</div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>

                {/* CHAT WINDOW (Area Messaggi) */}
                <div className="chat-window">
                    {activeGroupId ? (
                        <>
                            {/* Header Chat */}
                            <div className="chat-header">
                                <div className="header-info">
                                    <h3>{formatGroupName(activeGroup, username)}</h3>
                                    <span>
                                        {activeGroup.is_dm
                                            ? "Chat privata"
                                            : `${activeGroup.members.length} membri: ` + activeGroup.members.map(m => m.username).join(", ")}
                                    </span>
                                </div>
                                {/* Sezione Inviti (Solo per Gruppi) */}
                                {!activeGroup.is_dm && (
                                    <div style={{display: 'flex', gap: '5px'}}>
                                        <input
                                            placeholder="Invita utente..."
                                            value={inviteInput} onChange={e => setInviteInput(e.target.value)}
                                            style={{padding: '5px', borderRadius: '5px', border: '1px solid #ccc'}}
                                        />
                                        <button onClick={handleInvite} className="mini-btn active">Invita</button>
                                    </div>
                                )}
                            </div>

                            {/* Messaggi */}
                            <div className="messages-container">
                                {activeGroupMessages.map((msg, idx) => {
                                    const isMe = msg.from === username;
                                    return (
                                        <div key={idx} className={`message ${isMe ? 'sent' : 'received'}`}>
                                            <span className="msg-author">{msg.from}</span>
                                            {msg.content}
                                        </div>
                                    );
                                })}
                            </div>

                            {/* Input */}
                            <div className="input-area">
                                <input
                                    className="message-input"
                                    placeholder="Scrivi un messaggio..."
                                    value={input}
                                    onChange={e => setInput(e.target.value)}
                                    onKeyPress={e => e.key === 'Enter' && handleSendMessage()}
                                />
                                <button className="send-btn" onClick={handleSendMessage} disabled={!input.trim()}>
                                    ➤
                                </button>
                            </div>
                        </>
                    ) : (
                        <div className="empty-state">
                            <div style={{fontSize: '4rem', marginBottom: '20px'}}>💬</div>
                            <h2>Benvenuto in Ruggine Chat</h2>
                            <p>Seleziona una chat dalla barra laterale o creane una nuova per iniziare.</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

export default ChatPage;