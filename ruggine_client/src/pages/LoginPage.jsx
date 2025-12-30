
// ============================================================================
// LoginPage.jsx - Pagina di login e registrazione
// Gestisce autenticazione utente e cambio modalità.
// ============================================================================

// ================== Import base React ==================
import React, { useState } from 'react';

// ================== Import router ==================
import { useNavigate } from 'react-router-dom';

// ================== Import context ==================
import { useWebSocket } from '../context/WebSocketContext';

// ================== Import stile globale ==================
import '../App.css';

// ================== Import componenti ==================
import LoginForm from '../components/LoginForm';
import LoginMessage from '../components/LoginMessage';





// Componente pagina login
function LoginPage() {
    const { login } = useWebSocket();
    const [isLoginMode, setIsLoginMode] = useState(true);
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [message, setMessage] = useState('');
    const [isError, setIsError] = useState(false);
    const navigate = useNavigate();

    const handleApiCall = async (e) => {
        e.preventDefault();
        setMessage('Caricamento...');
        setIsError(false);
        const endpoint = isLoginMode ? 'login' : 'register';
        try {
            const response = await fetch(`http://127.0.0.1:3000/${endpoint}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            const data = await response.json();
            if (response.ok) {
                if (isLoginMode) {
                    sessionStorage.setItem('username', username);
                    sessionStorage.setItem('token', data.message);
                    setMessage('Login effettuato!');
                    window.location.href = '/chat';
                } else {
                    setIsLoginMode(true);
                    setPassword('');
                    setMessage("Registrazione avvenuta! Ora puoi fare il login.");
                }
            } else {
                setIsError(true);
                setMessage(data.error || data.message || "Errore Sconosciuto");
            }
        } catch (err) {
            setIsError(true);
            setMessage(`Errore di connessione: ${err.message}`);
        }
    };

    return (
        <div className="login-container">
            <div className="login-card">
                <h2 className="login-title">
                    {isLoginMode ? 'Bentornato su Ruggine' : 'Crea Account'}
                </h2>
                <LoginForm
                    isLoginMode={isLoginMode}
                    username={username}
                    setUsername={setUsername}
                    password={password}
                    setPassword={setPassword}
                    handleApiCall={handleApiCall}
                />
                <LoginMessage message={message} isError={isError} />
                <div className="secondary-action">
                    {isLoginMode ? "Non hai un account? " : "Hai già un account? "}
                    <button
                        className="link-button"
                        onClick={() => {
                            setIsLoginMode(!isLoginMode);
                            setMessage('');
                            setIsError(false);
                        }}
                    >
                        {isLoginMode ? 'Registrati ora' : 'Accedi'}
                    </button>
                </div>
            </div>
        </div>
    );
}

export default LoginPage;