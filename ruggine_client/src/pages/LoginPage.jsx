// src/pages/LoginPage.jsx
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';

function LoginPage() {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [message, setMessage] = useState(''); // Per i messaggi di errore/successo
    const navigate = useNavigate(); // Hook per reindirizzare

    const handleApiCall = async (endpoint) => {
        setMessage('Caricamento...');
        try {
            const response = await fetch(`http://127.0.0.1:3000/${endpoint}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });

            const data = await response.json();

            if (response.ok) {
                setMessage(data.message);
                if (endpoint === 'login') {
                    // Login riuscito!
                    // Salviamo il nome utente (per ora) e andiamo alla chat
                    localStorage.setItem('username', username);
                    navigate('/chat'); // Reindirizza alla pagina della chat
                }
            } else {
                setMessage(`Errore: ${data.error || data.message}`);
            }
        } catch (err) {
            setMessage(`Errore di connessione: ${err.message}`);
        }
    };

    return (
        <div>
            <h1>Benvenuto in Ruggine</h1>
            <div>
                <input
                    type="text"
                    placeholder="Username"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                />
                <input
                    type="password"
                    placeholder="Password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                />
            </div>
            <button onClick={() => handleApiCall('login')}>Login</button>
            <button onClick={() => handleApiCall('register')}>Registrati</button>
            {message && <p>{message}</p>}
        </div>
    );
}

export default LoginPage;