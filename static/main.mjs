import { FallbackLanguageModel } from './fallback.mjs';

// --- DOM Elements ---
const forceFallbackCheckbox = document.getElementById('force-fallback');
const temperatureInput = document.getElementById('temperature');
const temperatureValueSpan = document.getElementById('temperature-value');
const topKInput = document.getElementById('top-k');
const topKValueSpan = document.getElementById('top-k-value');
const systemPromptWrapper = document.getElementById('system-prompt-wrapper'); // Added
const systemPromptToggle = document.getElementById('system-prompt-toggle');
const systemPromptInput = document.getElementById('system-prompt');
const initialPromptsWrapper = document.getElementById('initial-prompts-wrapper'); // Added
const initialPromptsToggle = document.getElementById('initial-prompts-toggle'); // Added
const initialPromptsListDiv = document.getElementById('initial-prompts-list');
const addInitialPromptButton = document.getElementById('add-initial-prompt');
const resetSettingsButton = document.getElementById('reset-settings-button'); // Added
const userPromptInput = document.getElementById('user-prompt');
const sendButton = document.getElementById('send-prompt');
const clearHistoryButton = document.getElementById('clear-history-button'); // Added
const apiUsedDiv = document.getElementById('api-used');
const historyContainer = document.getElementById('history-container');

// --- State ---
let conversationHistory = []; // Array to store { role: 'user'/'assistant', content: '...' }

// --- Helper Functions ---

// Function to create the language model instance
async function createModel(createOptions, forceFallback) {
    let apiType = 'Fallback'; // Default to fallback
    if (!forceFallback && "ai" in self && "languageModel" in self.ai) {
        try {
            const availability = await self.ai.languageModel.availability();
            console.info('Prompt API capabilities:', availability);
            if (availability === 'available') {
                console.info('Attempting to use the Built-in Prompt API');
                const model = await self.ai.languageModel.create(createOptions);
                apiType = 'Built-in';
                console.info('Using the Built-in Prompt API');
                return { model, apiType };
            } else {
                 console.warn('Built-in Prompt API not readily available:', availability);
            }
        } catch (error) {
            console.error('Error checking or creating Built-in model:', error);
        }
    }
    console.info('Using the Fallback Prompt API');
    const model = await FallbackLanguageModel.create(createOptions);
    return { model, apiType };
}

// Function to render conversation history
function renderHistory() {
    historyContainer.innerHTML = ''; // Clear previous history
    conversationHistory.forEach(entry => {
        const div = document.createElement('div');
        div.innerHTML = `<strong>${entry.role}:</strong> ${entry.content}`;
        historyContainer.appendChild(div);
    });
    // Scroll to bottom
    historyContainer.scrollTop = historyContainer.scrollHeight;
}

// Function to add a new initial prompt entry UI
function addInitialPromptEntry(role = 'user', content = '') {
    const entryDiv = document.createElement('div');
    entryDiv.className = 'initial-prompt-entry';

    const roleSelect = document.createElement('select');
    roleSelect.innerHTML = `
        <option value="user" ${role === 'user' ? 'selected' : ''}>User</option>
        <option value="assistant" ${role === 'assistant' ? 'selected' : ''}>Assistant</option>
    `;

    const contentTextarea = document.createElement('textarea');
    contentTextarea.placeholder = 'Prompt content...';
    contentTextarea.value = content;

    const removeButton = document.createElement('button');
    removeButton.textContent = 'X';
    removeButton.className = 'remove-btn';
    removeButton.type = 'button';
    removeButton.onclick = () => {
        entryDiv.remove();
    };

    entryDiv.appendChild(roleSelect);
    entryDiv.appendChild(contentTextarea);
    entryDiv.appendChild(removeButton);
    initialPromptsListDiv.appendChild(entryDiv);
}

// Function to collect initial prompts from the UI
function getInitialPrompts() {
    const prompts = [];
    const entries = initialPromptsListDiv.querySelectorAll('.initial-prompt-entry');
    entries.forEach(entry => {
        const role = entry.querySelector('select').value;
        const content = entry.querySelector('textarea').value.trim();
        if (content) {
            prompts.push({ role, content });
        }
    });
    return prompts;
}

// Update temperature display
temperatureInput.addEventListener('input', () => {
    temperatureValueSpan.textContent = temperatureInput.value;
});

// Update Top K display
topKInput.addEventListener('input', () => {
    topKValueSpan.textContent = topKInput.value;
});

// Toggle System Prompt visibility // Added
systemPromptToggle.addEventListener('click', () => {
    systemPromptWrapper.classList.toggle('visible');
});

// Toggle Initial Prompts visibility // Added
initialPromptsToggle.addEventListener('click', () => {
    initialPromptsWrapper.classList.toggle('visible');
});

// Add initial prompt button
addInitialPromptButton.addEventListener('click', () => {
    addInitialPromptEntry();
});

// Clear History button // Added
clearHistoryButton.addEventListener('click', () => {
    conversationHistory = []; // Clear the array
    renderHistory(); // Update the display
});

// Reset Settings button // Added
resetSettingsButton.addEventListener('click', () => {
    // Reset UI elements to defaults
    forceFallbackCheckbox.checked = false;
    temperatureInput.value = 1.0; // Default from fallback.mjs (adjust if different)
    temperatureValueSpan.textContent = temperatureInput.value;
    topKInput.value = 3; // Default from fallback.mjs
    topKValueSpan.textContent = topKInput.value;
    systemPromptInput.value = '';
    initialPromptsListDiv.innerHTML = ''; // Clear initial prompts UI

    // Optionally collapse sections
    systemPromptWrapper.classList.remove('visible');
    initialPromptsWrapper.classList.remove('visible');
});


// Handle main prompt submission
sendButton.addEventListener('click', async () => {
    const userPrompt = userPromptInput.value.trim();
    if (!userPrompt) {
        alert('Please enter a user prompt.');
        return;
    }

    // Disable button and show loading state
    sendButton.disabled = true;
    historyContainer.lastChild?.scrollIntoView(); // Scroll to the user prompt added
    apiUsedDiv.textContent = 'Working...';

    // Get config
    const forceFallback = forceFallbackCheckbox.checked;
    const temperature = parseFloat(temperatureInput.value);
    const topK = parseInt(topKInput.value, 10);
    const systemPrompt = systemPromptInput.value.trim();
    const initialPrompts = getInitialPrompts();

    console.log('systemPrompt:', systemPrompt);
    // Prepare createOptions
    const createOptions = {
        temperature: temperature,
        topK: topK,
        // Only include systemPromptText if it's not empty
        ...(systemPrompt && { systemPrompt: systemPrompt }),
        initialPrompts: initialPrompts
    };

    // Add current user prompt to history (before sending)
    conversationHistory.push({ role: 'user', content: userPrompt });
    renderHistory(); // Show user prompt immediately

    try {
        // Create model (decides built-in vs fallback)
        const { model, apiType } = await createModel(createOptions, forceFallback);
        apiUsedDiv.textContent = apiType;

        console.info('Conversation history:', conversationHistory);
        // Pass the entire conversation history to the prompt method.
        const result = await model.prompt(conversationHistory); // Using non-streaming prompt

        // Add assistant response to history
        conversationHistory.push({ role: 'assistant', content: result });
        renderHistory(); // Update history display with assistant response

        // Clear the user prompt input for the next turn
        userPromptInput.value = '';

    } catch (error) {
        console.error('Error during prompt generation:', error);
        // Display error in history instead
        conversationHistory.push({ role: 'assistant', content: `Error: ${error.message}` });
        renderHistory();
        apiUsedDiv.textContent = 'Error';
    } finally {
        // Re-enable button
        sendButton.disabled = false;
    }
});

temperatureValueSpan.textContent = temperatureInput.value;
topKValueSpan.textContent = topKInput.value;
renderHistory();
