const messages = document.getElementById('messages');
const fetchPromptButton = document.getElementById('fetchPrompt');
const replyForm = document.getElementById('replyForm');
const responseInput = document.getElementById('response');

const chatHistory = [
    { role: 'server', text: 'Ready to fetch the latest prompt from getprompt.' },
];

function createMessage(role, text) {
    const message = document.createElement('div');
    message.className = `message ${role}`;
    message.setAttribute('data-role', role === 'ai' ? 'AI' : 'SERVER');
    const copy = document.createElement('p');
    copy.textContent = text;
    message.appendChild(copy);
    return message;
}

function renderMessages() {
    messages.innerHTML = '';
    chatHistory.forEach(({ role, text }) => {
        messages.appendChild(createMessage(role, text));
    });
    messages.scrollTop = messages.scrollHeight;
}

async function fetchPrompt() {
    fetchPromptButton.disabled = true;
    fetchPromptButton.textContent = 'Fetching...';

    try {
        const response = await fetch('getprompt');
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const text = await response.text();
        chatHistory.push({ role: 'server', text: text || 'Received empty prompt response.' });
    } catch (error) {
        chatHistory.push({ role: 'server', text: `Unable to fetch prompt: ${error.message}` });
    } finally {
        fetchPromptButton.disabled = false;
        fetchPromptButton.textContent = 'Fetch Prompt';
        renderMessages();
    }
}

replyForm.addEventListener('submit', (event) => {
    event.preventDefault();
    const text = responseInput.value.trim();
    if (!text) return;

    chatHistory.push({ role: 'ai', text });
    responseInput.value = '';
    renderMessages();
});

fetchPromptButton.addEventListener('click', fetchPrompt);

renderMessages();