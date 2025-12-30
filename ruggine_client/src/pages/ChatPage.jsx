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

// --- NUOVA FUNZIONE HELPER ---
const formatMembersList = (members) => {
    if (!members || members.length === 0) return "Nessun membro";

    // Prendi i primi 5 nomi
    const names = members.slice(0, 5).map(m => m.username).join(", ");

    // Se ce ne sono più di 5, aggiungi "..."
    const suffix = members.length > 5 ? "..." : "";

    return `${members.length} membri: ${names}${suffix}`;
};

function ChatPage() {
    const { user, groups, messages, sendMessage, sendInvite, createChat, wsError, setWsError } = useWebSocket();

    const [activeGroupId, setActiveGroupId] = useState(null);
    const [input, setInput] = useState('');

    // --- STATI PER LA UI NUOVA ---
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [modalType, setModalType] = useState('dm');
    const [modalInput, setModalInput] = useState('');
    const [inviteInput, setInviteInput] = useState('');
    const [toast, setToast] = useState(null);

    const messagesEndRef = useRef(null);

    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [messages, activeGroupId]);

    useEffect(() => {
        setInput('');
    }, [activeGroupId]);

    useEffect(() => {
        if (!activeGroupId && groups.some(g => g.group_id === 1)) {
            setActiveGroupId(1);
        }
    }, [groups]);

    useEffect(() => {
        if (toast) {
            const timer = setTimeout(() => setToast(null), 3000);
            return () => clearTimeout(timer);
        }
    }, [toast]);

    // Mostra toast di errore WebSocket
    useEffect(() => {
        if (wsError) {
            setToast({ msg: wsError, type: 'error' });
            setWsError(null);
        }
    }, [wsError, setWsError]);

    const activeGroup = groups.find(g => g.group_id === activeGroupId);
    const activeMessages = messages[activeGroupId] || [];

    // --- HANDLERS ---

    const showToast = (msg, type = 'success') => {
        setToast({ msg, type });
    };

    const handleCreateFromModal = () => {
        if (!modalInput.trim()) return;
        createChat(modalType, modalInput);
        setIsModalOpen(false);
        setModalInput('');
        showToast(
            modalType === 'dm'
                ? `Richiesta inviata a ${modalInput}`
                : `Gruppo '${modalInput}' creato!`
        );
    };

    const handleInviteFromChat = () => {
        if (!inviteInput.trim()) return;
        sendInvite(activeGroupId, inviteInput);
        setInviteInput('');
        showToast(`Invito inviato a ${inviteInput}`);
    };

    const handleSend = () => {
        if (input.trim() && activeGroupId) {
            sendMessage(activeGroupId, input);
            setInput('');
        }
    };

    const getLastMessagePreview = (groupId) => {
        const groupMsgs = messages[groupId];
        if (!groupMsgs || groupMsgs.length === 0) return "Nessun messaggio";
        const lastMsg = groupMsgs[groupMsgs.length - 1];
        let content = lastMsg.content;
        if (content.length > 25) content = content.substring(0, 25) + "...";
        return <span><span className="chat-preview-author">{lastMsg.from}:</span> {content}</span>;
    };

    return (
        <div style={{ display: 'flex', height: '100%', width: '100%' }}>

            {toast && (
                <div className="toast-container">
                    <div className={`toast ${toast.type}`}>
                        {toast.type === 'success' ? '✅' : '⚠️'} {toast.msg}
                    </div>
                </div>
            )}

            {isModalOpen && (
                <div className="modal-overlay" onClick={() => setIsModalOpen(false)}>
                    <div className="modal-content" onClick={e => e.stopPropagation()}>
                        <div className="modal-header">
                            <h3>Nuova Chat</h3>
                            <button className="close-modal-btn" onClick={() => setIsModalOpen(false)}>×</button>
                        </div>

                        <div className="modal-tabs">
                            <button className={`modal-tab ${modalType === 'dm' ? 'active' : ''}`} onClick={() => setModalType('dm')}>DM Utente</button>
                            <button className={`modal-tab ${modalType === 'group' ? 'active' : ''}`} onClick={() => setModalType('group')}>Gruppo</button>
                        </div>

                        <input
                            autoFocus
                            className="login-input"
                            placeholder={modalType === 'dm' ? "Username utente..." : "Nome del gruppo..."}
                            value={modalInput}
                            onChange={e => setModalInput(e.target.value)}
                            onKeyPress={e => e.key === 'Enter' && handleCreateFromModal()}
                        />

                        <button className="login-button" onClick={handleCreateFromModal}>
                            {modalType === 'dm' ? 'Invia Richiesta' : 'Crea Gruppo'}
                        </button>
                    </div>
                </div>
            )}

            <div className="sidebar">
                <div className="sidebar-header">
                    <h3 style={{ margin: 0, color: '#333' }}>Le tue Chat</h3>
                    <button className="add-chat-btn" onClick={() => setIsModalOpen(true)} title="Nuova Chat">+</button>
                </div>

                <div className="chat-list">
                    {groups.map(group => {
                        const name = formatGroupName(group, user);
                        if (group.is_dm && group.members.length < 2) return null;

                        return (
                            <div
                                key={group.group_id}
                                className={`chat-item ${activeGroupId === group.group_id ? 'active' : ''}`}
                                onClick={() => setActiveGroupId(group.group_id)}
                            >
                                <div className="avatar" style={group.group_id === 1 ? {background: 'none', color: 'inherit', padding: 0, position: 'relative', overflow: 'visible'} : {}}>
                                    {group.group_id === 1 ? (
                                        <img src="/ruggine_logo.png" alt="Ruggine Logo" style={{ width: 60, height: 60, objectFit: 'contain', borderRadius: '50%', display: 'block', margin: '0 auto' }} />
                                    ) : (
                                        getAvatarInitial(name)
                                    )}
                                </div>
                                <div className="chat-info">
                                    <div className="chat-name">{name}</div>
                                    <div className="chat-preview">{getLastMessagePreview(group.group_id)}</div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>

            <div className="chat-window">
                {activeGroupId && activeGroup ? (
                    <>
                        <div className="chat-header">
                            <div className="header-info">
                                <h3>{formatGroupName(activeGroup, user)}</h3>
                                {/* --- QUI USIAMO LA NUOVA FUNZIONE --- */}
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
                        Clicca sul <strong style={{color: '#667eea', margin: '0 5px'}}>+</strong> in alto a sinistra per iniziare.
                    </div>
                )}
            </div>
        </div>
    );
}

export default ChatPage;