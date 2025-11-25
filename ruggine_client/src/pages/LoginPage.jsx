// src/pages/LoginPage.jsx
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import '../App.css';
import { useWebSocket } from '../context/WebSocketContext';

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
                setMessage(data.message);
                if (isLoginMode) {
                    // === MODIFICATO PER SESSION STORAGE ===
                    // Se usiamo il context login:
                    // login(username, data.message);
                    // navigate('/chat');

                    // SE invece usiamo il refresh forzato (scelta consigliata per pulizia totale):
                    sessionStorage.setItem('username', username);
                    sessionStorage.setItem('token', data.message);
                    window.location.href = '/chat'; // Forza reload pulito
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

                {message && (
                    <div className={isError ? "error-message" : "success-message"}>
                        {message}
                    </div>
                )}

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