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
                'Content-Type': 'text/event-stream'
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

        const reader = result.body.getReader();
        const decoder = new TextDecoder();

        return new ReadableStream({
            async start(controller) {
                try {
                    while (true) {
                        const { done, value } = await reader.read();
                        if (done) {
                            break;
                        }
                        const chunk = decoder.decode(value, { stream: true });
                        controller.enqueue(chunk);
                    }
                } catch (error) {
                    console.error('Error reading stream:', error);
                    controller.error(error);
                } finally {
                    // Ensure the stream is closed even if loop breaks unexpectedly
                    // or finishes normally. TextDecoder doesn't need explicit closing.
                    try {
                       controller.close();
                    } catch (e) {
                       // Ignore errors if controller is already closed or errored
                       if (e.name !== 'TypeError') { // TypeError: Cannot close a readable stream that is locked or has been disturbed
                           console.error('Error closing stream controller:', e);
                       }
                    }
                    // Release the lock on the original reader
                    reader.releaseLock(); 
                }
            },
            cancel(reason) {
                console.log('Stream cancelled:', reason);
                reader.cancel(reason);
            }
        });
    }
}

function normalizeInputs(input) {
    let inputs = [];
    if (typeof input === 'string') {
        inputs.push({role: 'user', type: 'text', content: input});
    } else if (input instanceof Array) {
        inputs.push(...input);
    } else {
        inputs.push(input);
    }

    return normalizePrompts(inputs);
}

function normalizePrompts(prompts) {
    prompts.forEach(prompt => {
        prompt.role = prompt.role || 'user';
        prompt.type = prompt.type || 'text';
    })    
}
