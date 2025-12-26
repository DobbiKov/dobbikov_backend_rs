const apiBaseInput = document.getElementById('apiBase');
const form = document.getElementById('loginForm');
const statusEl = document.getElementById('status');
const goRegister = document.getElementById('goRegister');

const savedBase = localStorage.getItem('apiBase') || 'http://127.0.0.1:3000';
apiBaseInput.value = savedBase;

goRegister.addEventListener('click', () => {
  window.location.href = 'register.html';
});

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  statusEl.hidden = true;

  const apiBase = apiBaseInput.value.trim() || 'http://127.0.0.1:3000';
  localStorage.setItem('apiBase', apiBase);

  const payload = {
    username: form.username.value.trim(),
    password: form.password.value,
  };

  try {
    const res = await fetch(`${apiBase}/users/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (!res.ok) {
      const message = await res.text();
      throw new Error(message || 'Login failed');
    }

    const data = await res.json();
    localStorage.setItem('authToken', data.token);
    localStorage.setItem('authUser', JSON.stringify(data.user));
    localStorage.setItem('tokenExpiresAt', data.expires_at);

    const hash = new URLSearchParams({
      token: data.token,
      apiBase,
    }).toString();
    window.location.href = `admin.html#${hash}`;
  } catch (err) {
    statusEl.textContent = err.message || 'Login failed. Check your API server.';
    statusEl.hidden = false;
  }
});
