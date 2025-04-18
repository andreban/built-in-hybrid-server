export class FallbackLanguageModel extends EventTarget {
    constructor(createOptions, capabilities) {
        super();
        this.createOptions = createOptions;
        this.maxTemperature = capabilities.maxTemperature;
        this.maxTopK = capabilities.maxTopK;
        this.defaultTemperature = capabilities.defaultTemperature;
        this.defaultTopK = capabilities.defaultTopK;
    }

    static async create(options) {
        const response = await fetch('/language-model/capabilities', { method: 'POST' });
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${result.status} / ${result.statusText}`);
        }
        if (!response.body) {
            throw new Error('Response body is null');
        }        
        
        const capabilities = await response.json();
        console.info('capabilities:', capabilities);
        
        const createOptions = {
            temperature: options.temperature || capabilities.defaultTemperature,
            topK: options.topK || capabilities.defaultTopK,
            systemPrompt: options.systemPrompt || null,
            expectedInputs: options.expectedInputs || [],
            initialPrompts: options.initialPrompts || [],
        };

        normalizePrompts(createOptions.initialPrompts);
        return new FallbackLanguageModel(createOptions, capabilities);
    }

    async prompt(input) {
        const inputs = normalizeInputs(input);
        const result = await fetch('/language-model/prompt', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                createOptions: this.createOptions,
                inputs: inputs,
            })
        });

        if (!result.ok) {
            throw new Error(`HTTP error! status: ${result.status} / ${result.statusText}`);
        }
        if (!result.body) {
            throw new Error('Response body is null');
        }       
        
        return result.text();
    }

    async promptStreaming(input) {
        const inputs = normalizeInputs(input);
        const result = await fetch('/language-model/prompt-streaming', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                createOptions: this.createOptions,
                inputs: inputs,
            })
        });

        if (!result.ok) {
            throw new Error(`HTTP error! status: ${result.status}`);
        }
        if (!result.body) {
            throw new Error('Response body is null');
        }

        return result.body.pipeThrough(new TextDecoderStream())
    }

    async countTokens(input) { // Changed parameter name from 'inputs' to 'input'
        const normalizedInputs = normalizeInputs(input); // Use a different variable name
        const result = await fetch('/language-model/count-tokens', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                createOptions: this.createOptions,
                inputs: normalizedInputs, // Use the normalized inputs
            })
        });

        if (!result.ok) {
            throw new Error(`HTTP error! status: ${result.status}`);
        }
        if (!result.body) {
            throw new Error('Response body is null');
        }

        return Number.parseInt(await result.text()); // Return the parsed number
    }
}

function normalizeInputs(input) {
    const inputs = [];
    if (typeof input === 'string') {
        inputs.push({role: 'user', type: 'text', content: input});
    } else if (input instanceof Array) {
        inputs.push(...input);
    } else {
        inputs.push(input);
    }

    normalizePrompts(inputs);
    return inputs;
}

function normalizePrompts(prompts) {
    prompts.forEach(prompt => {
        prompt.role = prompt.role || 'user';
        prompt.type = prompt.type || 'text';
    })    
}

// Function to create the language model instance
export async function createModel(createOptions = {}, forceFallback = false) {
    if (!forceFallback && "LanguageModel" in self) {
        try {
            const availability = await self.LanguageModel.availability();
            console.info('Built-in Prompt API availability:', availability);
            if (availability === 'available') {
                console.info('Attempting to use the Built-in Prompt API');
                const model = await self.LanguageModel.create(createOptions);
                console.info('Using the Built-in Prompt API');
                return model;
            } else {
                 console.warn('Built-in Prompt API not readily available:', availability);
            }
        } catch (error) {
            console.error('Error checking or creating Built-in model:', error);
        }
    }
    console.info('Using the Fallback Prompt API');
    const model = await FallbackLanguageModel.create(createOptions);
    return model;
}