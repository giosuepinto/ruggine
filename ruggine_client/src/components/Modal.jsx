// ============================================================================
// Modal.jsx - Componente modale per creazione chat/DM
// Gestisce input e tab per DM o gruppo.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function Modal({ isOpen, onClose, modalType, setModalType, modalInput, setModalInput, handleCreateFromModal }) {
    if (!isOpen) return null;
    return (
        <div className="modal-overlay" onClick={onClose}>
            <div className="modal-content" onClick={e => e.stopPropagation()}>
                <div className="modal-header">
                    <h3>Nuova Chat</h3>
                    <button className="close-modal-btn" onClick={onClose}>×</button>
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
    );
}

export default Modal;
