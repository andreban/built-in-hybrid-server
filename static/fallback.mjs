export class FallbackLanguageModel extends EventTarget {
    constructor(createOptions, capabilities) {
        super();
        this.createOptions = createOptions;
        this.maxTemperature = capabilities.maxTemperature;
        this.maxTopK = capabilities.maxTopK;
        this.defaultTemperature = capabilities.defaultTemperature;
        this.defaultTopK = capabilities.defaultTopK;
    }

    static async create({ temperature = 1.0, topK = 3, systemPrompt = null, expectedInputs = [], initialPrompts = []} = {}) {
        const createOptions = {
            temperature,
            topK,
            systemPrompt,
            expectedInputs,
            initialPrompts,
        };

        const response = await fetch('/language-model/capabilities');
        const capabilities = await response.json();
        console.info('capabilities:', capabilities);

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
