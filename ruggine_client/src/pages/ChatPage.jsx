
// Pagina principale della chat: sidebar, messaggi, input e toast.

// ================== Import base React ==================
import { useState, useEffect, useRef } from 'react';

// ================== Import context ==================
import { useWebSocket } from '../context/WebSocketContext';

// ================== Import stile globale ==================
import '../App.css';

// ================== Import componenti ==================
import Toast from '../components/Toast';
import Modal from '../components/Modal';
import Sidebar from '../components/Sidebar';
import ChatHeader from '../components/ChatHeader';
import MessagesContainer from '../components/MessagesContainer';
import InputArea from '../components/InputArea';


export const formatGroupName = (group, currentUsername) => {
    if (group.group_id === 1) return "🌍 GLOBAL CHAT";
    if (group.is_dm) {
        const otherUser = group.members.find(m => m.username !== currentUsername);
        return otherUser ? otherUser.username : 'Sconosciuto';
    }
    return group.group_name || 'Gruppo';
};
export const getAvatarInitial = (name) => name ? name.charAt(0).toUpperCase() : '?';
export const formatMembersList = (members) => {
    if (!members || members.length === 0) return "Nessun membro";
    const names = members.slice(0, 5).map(m => m.username).join(", ");
    const suffix = members.length > 5 ? "..." : "";
    return `${members.length} membri: ${names}${suffix}`;
};

// Componente principale della pagina chat
function ChatPage() {
    const { user, groups, messages, sendMessage, sendInvite, createChat, wsError, setWsError } = useWebSocket();
    const [activeGroupId, setActiveGroupId] = useState(null);
    const [input, setInput] = useState('');

    // ================== Stati locali ==================
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [modalType, setModalType] = useState('dm');
    const [modalInput, setModalInput] = useState('');
    const [inviteInput, setInviteInput] = useState('');
    const [toast, setToast] = useState(null);
    const messagesEndRef = useRef(null);


    // ================== Effetti ==================
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [messages, activeGroupId]);
    useEffect(() => { setInput(''); }, [activeGroupId]);
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
    useEffect(() => {
        if (wsError) {
            setToast({ msg: wsError, type: 'error' });
            setWsError(null);
        }
    }, [wsError, setWsError]);

    const activeGroup = groups.find(g => g.group_id === activeGroupId);
    const activeMessages = messages[activeGroupId] || [];


    // ================== Handler azioni UI ==================
    const showToast = (msg, type = 'success') => setToast({ msg, type });
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
            <Toast toast={toast} />
            <Modal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                modalType={modalType}
                setModalType={setModalType}
                modalInput={modalInput}
                setModalInput={setModalInput}
                handleCreateFromModal={handleCreateFromModal}
            />
            <Sidebar
                groups={groups}
                user={user}
                activeGroupId={activeGroupId}
                setActiveGroupId={setActiveGroupId}
                getLastMessagePreview={getLastMessagePreview}
                setIsModalOpen={setIsModalOpen}
            />
            <div className="chat-window">
                {activeGroupId && activeGroup ? (
                    <>
                        <ChatHeader
                            activeGroup={activeGroup}
                            user={user}
                            inviteInput={inviteInput}
                            setInviteInput={setInviteInput}
                            handleInviteFromChat={handleInviteFromChat}
                        />
                        <MessagesContainer
                            activeMessages={activeMessages}
                            user={user}
                            messagesEndRef={messagesEndRef}
                        />
                        <InputArea
                            input={input}
                            setInput={setInput}
                            handleSend={handleSend}
                        />
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