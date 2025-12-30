// ============================================================================
// MessagesContainer.jsx - Contenitore messaggi chat
// Mostra messaggi e gestisce scroll automatico.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function MessagesContainer({ activeMessages, user, messagesEndRef }) {
    return (
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
    );
}

export default MessagesContainer;
