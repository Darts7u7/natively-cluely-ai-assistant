
const { GoogleGenAI } = require('@google/genai');
require('dotenv').config();

async function run() {
    const apiKey = process.env.GOOGLE_API_KEY || process.env.GEMINI_API_KEY;
    if (!apiKey) {
        console.error('No API KEY found');
        return;
    }



    console.log('API Key preset:', apiKey.substring(0, 5) + '...');
    const client = new GoogleGenAI({ apiKey });

    console.log('\nVerifying API Key with gemini-2.0-flash-exp...');
    try {
        const genResult = await client.models.generateContent({
            model: 'gemini-2.0-flash-exp',
            contents: [{ parts: [{ text: "Hi" }] }]
        });
        console.log('Generate Content Success');
    } catch (error) {
        console.error('Generate Content Failed:', error.message || error);
    }

    console.log('\nTesting embedding with text-embedding-004 (default)...');
    try {
        const result = await client.models.embedContent({
            model: 'text-embedding-004',
            contents: [{ parts: [{ text: "Test embedding" }] }]
        });
        console.log(`Success with text-embedding-004`);
    } catch (error) {
        console.error(`Failed with text-embedding-004:`, error.message || error);
    }
}

run();




