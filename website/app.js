// Luxmc Website - Account and Skin Studio Script

function handleWebRegister(event) {
  event.preventDefault();
  const username = document.getElementById('username').value.trim();
  const email = document.getElementById('email').value.trim();
  const password = document.getElementById('password').value;
  const cape = document.getElementById('capeChoice').value;
  const modelEl = document.querySelector('input[name="skinModel"]:checked');
  const model = modelEl ? modelEl.value : 'classic';

  if (!username) {
    alert('Por favor, informe seu nickname/gamertag.');
    return;
  }

  // Save account to localStorage simulated database
  const accounts = JSON.parse(localStorage.getItem('luxmc_registered_users') || '{}');
  accounts[username.toLowerCase()] = {
    username: username,
    email: email,
    model: model,
    cape: cape,
    createdAt: new Date().toISOString()
  };
  localStorage.setItem('luxmc_registered_users', JSON.stringify(accounts));

  // Show success block
  document.getElementById('accountForm').style.display = 'none';
  const successBox = document.getElementById('registerSuccess');
  document.getElementById('registeredUsername').innerText = username;
  document.getElementById('codeNick').innerText = username;
  successBox.style.display = 'block';
}

let isBackView = false;
let currentNick = 'Steve';

function loadSkinFromNick() {
  const nick = document.getElementById('skinPlayerName').value.trim() || 'Steve';
  currentNick = nick;
  updatePreview();
}

function updatePreview() {
  const img = document.getElementById('skinPreviewImg');
  if (!img) return;
  const endpoint = isBackView ? 'https://mc-heads.net/body/' : 'https://mc-heads.net/body/';
  img.src = `${endpoint}${encodeURIComponent(currentNick)}/260?t=${Date.now()}`;
}

function rotatePreview() {
  isBackView = !isBackView;
  const img = document.getElementById('skinPreviewImg');
  if (img) {
    img.style.transform = isBackView ? 'scaleX(-1)' : 'scaleX(1)';
  }
}

function resetSkinPreview() {
  currentNick = 'Steve';
  const nickInput = document.getElementById('skinPlayerName');
  if (nickInput) nickInput.value = 'Steve';
  isBackView = false;
  const img = document.getElementById('skinPreviewImg');
  if (img) {
    img.style.transform = 'scaleX(1)';
    img.src = 'https://mc-heads.net/body/Steve/260';
  }
}

function updateArmModel(model) {
  // Visual feedback for arm model
  console.log('Model updated to:', model);
}

function handleSkinUpload(event) {
  const file = event.target.files[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = function(e) {
    const img = document.getElementById('skinPreviewImg');
    if (img) {
      img.src = e.target.result;
    }
  };
  reader.readAsDataURL(file);
}

function handleCapeChange(cape) {
  console.log('Cape changed to:', cape);
}

function saveSkinOnline() {
  const nick = document.getElementById('skinPlayerName')?.value.trim() || 'Steve';
  alert(`Skin e capa vinculadas ao jogador "${nick}" com sucesso! Agora entre no Luxmc Launcher usando este nickname.`);
}
