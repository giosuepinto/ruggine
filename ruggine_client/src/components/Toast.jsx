// ============================================================================
// Toast.jsx - Componente per notifiche toast
// Mostra messaggi di successo o errore temporanei.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function Toast({ toast }) {
    if (!toast) return null;
    return (
        <div className="toast-container">
            <div className={`toast ${toast.type}`}>
                {toast.type === 'success' ? '✅' : '⚠️'} {toast.msg}
            </div>
        </div>
    );
}

export default Toast;
