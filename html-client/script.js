const messages = document.getElementById('messages');
// const fetchPromptButton = document.getElementById('fetchPrompt');
const replyForm = document.getElementById('replyForm');
const responseInput = document.getElementById('response');

const chatHistory = [
];

function createMessage(role, text) {
    const message = document.createElement('div');
    message.className = `message ${role}`;
    message.setAttribute('data-role', role === 'ai' ? 'AI' : 'Human');
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
    // fetchPromptButton.disabled = true;
    // fetchPromptButton.textContent = 'Fetching...';

    try {
        const response = await fetch('getprompt');
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const text = await response.text();
        const textParsed = JSON.parse(text);
        chatHistory.push({ role: 'server', text: textParsed.value || 'Received empty prompt response.' });
    } catch (error) {
        chatHistory.push({ role: 'server', text: `Unable to fetch prompt: ${error.message}` });
    } finally {
        // fetchPromptButton.disabled = false;
        // fetchPromptButton.textContent = 'Fetch Prompt';
        renderMessages();
    }
}

replyForm.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        replyForm.dispatchEvent(new Event('submit'));
    }
});
replyForm.addEventListener('submit', async (event) => {
    event.preventDefault();
    const text = responseInput.value.trim();
    if (!text) return;

    chatHistory.push({ role: 'ai', text });
    renderMessages();
    responseInput.value = '';

    try {
        const historyPayload = chatHistory
            .filter((entry) => entry.role !== 'server')
            .map(({ role, text }) => ({
                role: role === 'ai' ? 'user' : 'model',
                parts: [{ text }],
            }));

        const response = await fetch('/sendresponse', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ history: historyPayload }),
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        chatHistory.push({
            role: 'server',
            text: `Rating: ${data.rating}/10\nCommentary: ${data.commentary}\nNext prompt: ${data.next_prompt}`,
        });
    } catch (error) {
        chatHistory.push({ role: 'server', text: `Unable to send response: ${error.message}` });
    } finally {
        renderMessages();
    }
});

// fetchPromptButton.addEventListener('click', fetchPrompt);


document.addEventListener('DOMContentLoaded', () => {
    fetchPrompt();
    renderMessages();
});