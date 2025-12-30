// ============================================================================
// LoginForm.jsx - Form di login/registrazione
// Gestisce input utente e invio credenziali.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function LoginForm({ isLoginMode, username, setUsername, password, setPassword, handleApiCall }) {
    return (
        <form onSubmit={handleApiCall}>
            <input
                className="login-input"
                type="text"
                placeholder="Username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
            />
            <input
                className="login-input"
                type="password"
                placeholder="Password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
            />
            <button type="submit" className="login-button">
                {isLoginMode ? 'Accedi' : 'Registrati'}
            </button>
        </form>
    );
}

export default LoginForm;
