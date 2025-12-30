// ============================================================================
// LoginMessage.jsx - Messaggio di stato login
// Mostra messaggi di errore o successo.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function LoginMessage({ message, isError }) {
    if (!message) return null;
    return <div className={isError ? "error-message" : "success-message"}>{message}</div>;
}

export default LoginMessage;
