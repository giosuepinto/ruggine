// ============================================================================
// InputArea.jsx - Componente area di input messaggi
// Gestisce input testo e invio messaggi.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function InputArea({ input, setInput, handleSend }) {
    return (
        <div className="input-area">
            <input
                className="message-input" placeholder="Scrivi un messaggio..."
                value={input} onChange={e => setInput(e.target.value)}
                onKeyPress={e => e.key === 'Enter' && handleSend()}
            />
            <button className="send-btn" onClick={handleSend} disabled={!input.trim()}>➤</button>
        </div>
    );
}

export default InputArea;
