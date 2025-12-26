const apiBaseInput = document.getElementById('apiBase');
const form = document.getElementById('registerForm');
const statusEl = document.getElementById('status');
const goLogin = document.getElementById('goLogin');

const savedBase = localStorage.getItem('apiBase') || 'http://127.0.0.1:3000';
apiBaseInput.value = savedBase;

goLogin.addEventListener('click', () => {
  window.location.href = 'login.html';
});

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  statusEl.hidden = true;

  const apiBase = apiBaseInput.value.trim() || 'http://127.0.0.1:3000';
  localStorage.setItem('apiBase', apiBase);

  const payload = {
    username: form.username.value.trim(),
    password: form.password.value,
    is_admin: form.isAdmin.value === 'true',
  };

  try {
    const res = await fetch(`${apiBase}/users/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (!res.ok) {
      const message = await res.text();
      throw new Error(message || 'Registration failed');
    }

    const data = await res.json();
    localStorage.setItem('authToken', data.token);
    localStorage.setItem('authUser', JSON.stringify(data.user));
    localStorage.setItem('tokenExpiresAt', data.expires_at);

    window.location.href = 'admin.html';
  } catch (err) {
    statusEl.textContent = err.message || 'Registration failed. Check your API server.';
    statusEl.hidden = false;
  }
});
