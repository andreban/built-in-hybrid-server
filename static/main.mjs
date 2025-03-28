import { FallbackLanguageModel } from './fallback.mjs';

const DEBUG_FALLBACK = true;

async function createModel(createOptions) {
    if (!DEBUG_FALLBACK && "ai" in self && "languageModel" in self.ai) {
        const capabilities = await self.ai.languageModel.capabilities();
        if (capabilities.available === 'readily') {
            console.info('Using the Built-in Prompt API');
            return self.ai.languageModel.create(createOptions);
        }
    }
    console.info('Using the Fallback Prompt API');    
    return FallbackLanguageModel.create(createOptions);
}
const button = document.getElementById('prompt');
const model = await createModel({ systemPrompt: "Create a joke about the topic provided by the user" });
button.addEventListener('click', async () => {    
    const result = await model.prompt('Cat');
    console.log(result);
});
