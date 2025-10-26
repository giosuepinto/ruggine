import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';

import LoginPage from './pages/LoginPage';
import ChatPage from './pages/ChatPage';

// Definiamo le nostre rotte
const router = createBrowserRouter([
    {
        path: '/',
        element: <LoginPage />, // La pagina di default è il Login
    },
    {
        path: '/chat',
        element: <ChatPage />, // La nostra chat
    },
]);

ReactDOM.createRoot(document.getElementById('root')).render(
    <React.StrictMode>
        <RouterProvider router={router} />
    </React.StrictMode>
);