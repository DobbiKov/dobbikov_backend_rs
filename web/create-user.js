const apiBaseInput = document.getElementById('apiBase');
const form = document.getElementById('createUserForm');
const statusEl = document.getElementById('status');
const goAdmin = document.getElementById('goAdmin');

const savedBase = localStorage.getItem('apiBase') || 'http://127.0.0.1:3000';
apiBaseInput.value = savedBase;

if (!localStorage.getItem('authToken')) {
  window.location.href = '/login';
}

goAdmin.addEventListener('click', () => {
  window.location.href = '/admin';
});

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  statusEl.hidden = true;

  const apiBase = apiBaseInput.value.trim() || 'http://127.0.0.1:3000';
  localStorage.setItem('apiBase', apiBase);

  const token = localStorage.getItem('authToken');
  if (!token) {
    statusEl.textContent = 'Missing token. Please log in again.';
    statusEl.hidden = false;
    return;
  }

  const payload = {
    username: form.username.value.trim(),
    password: form.password.value,
    is_admin: form.isAdmin.value === 'true',
  };

  try {
    const res = await fetch(`${apiBase}/users/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(payload),
    });

    if (!res.ok) {
      const message = await res.text();
      throw new Error(message || 'User creation failed');
    }

    form.reset();
    statusEl.textContent = 'User created.';
    statusEl.hidden = false;
  } catch (err) {
    statusEl.textContent = err.message || 'User creation failed.';
    statusEl.hidden = false;
  }
});
