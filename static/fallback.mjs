export class FallbackLanguageModel extends EventTarget {
    constructor(createOptions) {
        super();
        this.createOptions = createOptions;
        this.maxTemperature = 2.0;
        this.maxTopK = 8;
        this.defaultTemperature = 1.0;
        this.defaultTopK = 3;
    }

    static async create({ temperature = 1.0, topK = 3, systemPrompt = null, expectedInputs = [], initialPrompts = []} = {}) {
        const createOptions = {
            temperature,
            topK,
            systemPrompt,
            expectedInputs,
            initialPrompts,
        };

        normalizePrompts(createOptions.initialPrompts);
        return new FallbackLanguageModel(createOptions);
    }

    async prompt(input) {
        let inputs = [];
        if (typeof input === 'string') {
            inputs.push({role: 'user', type: 'text', content: input});
        } else if (input instanceof Array) {
            inputs.push(...input);
        } else {
            inputs.push(input);
        }

        normalizePrompts(inputs);

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
}

function normalizePrompts(prompts) {
    prompts.forEach(prompt => {
        prompt.role = prompt.role || 'user';
        prompt.type = prompt.type || 'text';
    })    
}