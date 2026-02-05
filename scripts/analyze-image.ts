#!/usr/bin/env npx ts-node

/**
 * Analyze Image CLI - Self-contained image analysis using AWS Bedrock
 *
 * Usage:
 *   npx ts-node scripts/analyze-image.ts -i <image-path> [-p "prompt"] [-m model-id]
 *
 * Examples:
 *   npx ts-node scripts/analyze-image.ts -i screenshot.png
 *   npx ts-node scripts/analyze-image.ts -i screenshot.png -p "Describe the terrain and biomes visible"
 *   npx ts-node scripts/analyze-image.ts -i screenshot.png -m us.anthropic.claude-sonnet-4-5-20250929-v1:0
 */

import { readFileSync, existsSync, writeFileSync } from 'fs';
import path from 'path';
import { BedrockRuntimeClient, InvokeModelCommand } from '@aws-sdk/client-bedrock-runtime';
import { fromNodeProviderChain } from '@aws-sdk/credential-providers';

// ============================================================================
// Configuration
// ============================================================================

const CONFIG = {
  awsProfile: '230639770018_cr-AdminAccess',
  region: 'us-east-1',
  accountId: '230639770018',
  defaultModel: 'us.amazon.nova-pro-v1:0',
  anthropicRegions: ['us-east-1', 'us-west-2'],
  maxRetries: 10,
  maxTokens: 4096,
  temperature: 0.2
};

// ============================================================================
// Types
// ============================================================================

interface LLMMessage {
  role: 'user' | 'assistant' | 'system';
  content: Array<{
    text?: string;
    image?: {
      format: 'jpeg' | 'png' | 'gif' | 'webp';
      source: { bytes: string };
    };
  }>;
}

interface LLMResponse {
  data: string;
  metrics: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
    model: string;
    region?: string;
  };
}

interface AnalyzeOptions {
  image: string;
  prompt: string;
  model: string;
  temperature: number;
  maxTokens: number;
  system?: string;
}

// ============================================================================
// Bedrock Service (Simplified)
// ============================================================================

class BedrockService {
  private client: BedrockRuntimeClient;
  private region: string;

  constructor() {
    this.region = CONFIG.region;
    this.client = new BedrockRuntimeClient({
      region: this.region,
      credentials: fromNodeProviderChain({ profile: CONFIG.awsProfile })
    });
  }

  async generateWithImages(
    messages: LLMMessage[],
    systemPrompt?: string,
    config?: { temperature?: number; maxTokens?: number },
    modelId?: string
  ): Promise<LLMResponse> {
    const selectedModel = modelId || CONFIG.defaultModel;
    const isAnthropicModel = /anthropic/i.test(selectedModel);

    // Transform messages for the model format
    const transformedMessages = messages.map(message => ({
      role: message.role,
      content: message.content.map(block => {
        if (block.image) {
          const format = (block.image.format || 'jpeg').toLowerCase();
          const mediaType = format === 'jpg' ? 'image/jpeg' : `image/${format}`;

          if (isAnthropicModel) {
            return {
              type: 'input_image',
              source: {
                type: 'base64',
                media_type: mediaType,
                data: block.image.source.bytes
              }
            };
          }

          return {
            image: {
              format: (format === 'jpg' ? 'jpeg' : format) as 'jpeg' | 'png' | 'gif' | 'webp',
              source: {
                bytes: block.image.source.bytes
              }
            }
          };
        }

        if (isAnthropicModel) {
          return {
            type: 'text',
            text: block.text ?? ''
          };
        }

        return {
          text: block.text ?? ''
        };
      })
    }));

    const filteredMessages = isAnthropicModel
      ? transformedMessages.filter(message => message.role !== 'system')
      : transformedMessages;

    let requestBody: any;

    if (isAnthropicModel) {
      requestBody = {
        anthropic_version: 'bedrock-2023-05-31',
        max_tokens: config?.maxTokens || CONFIG.maxTokens,
        messages: filteredMessages.map(message => ({
          role: message.role === 'system' ? 'user' : message.role,
          content: message.content
        }))
      };

      if (systemPrompt?.trim()) {
        requestBody.system = systemPrompt;
      }

      if (typeof config?.temperature === 'number') {
        requestBody.temperature = config.temperature;
      }
    } else {
      // Nova format
      const inferenceConfig: Record<string, number> = {
        maxTokens: config?.maxTokens || CONFIG.maxTokens,
        temperature: config?.temperature ?? CONFIG.temperature
      };

      requestBody = {
        messages: filteredMessages,
        inferenceConfig
      };

      if (systemPrompt?.trim()) {
        requestBody.system = [{ text: systemPrompt.trim() }];
      }
    }

    const commandInput: any = {
      contentType: 'application/json',
      body: JSON.stringify(requestBody),
      modelId: selectedModel
    };

    // Handle inference profile ARNs
    if (selectedModel.startsWith('arn:aws:bedrock:') && selectedModel.includes(':inference-profile/')) {
      commandInput.inferenceProfileArn = selectedModel;
    } else if (selectedModel.startsWith('global.')) {
      const constructedArn = `arn:aws:bedrock:${this.region}:${CONFIG.accountId}:inference-profile/${selectedModel}`;
      commandInput.inferenceProfileArn = constructedArn;
      commandInput.modelId = constructedArn;
    }

    const command = new InvokeModelCommand(commandInput);

    let response;
    let lastError: unknown;

    for (let attempt = 1; attempt <= CONFIG.maxRetries; attempt++) {
      try {
        response = await this.client.send(command);
        break;
      } catch (err) {
        lastError = err;

        // Check for rate limiting
        const isRateLimit = this.isRateLimitError(err);
        if (isRateLimit) {
          const delay = Math.min(1000 * Math.pow(2, attempt - 1), 30000);
          console.warn(`Rate limited, waiting ${delay}ms before retry ${attempt}/${CONFIG.maxRetries}`);
          await this.sleep(delay);
          continue;
        }

        if (attempt < CONFIG.maxRetries) {
          const delay = Math.min(250 * Math.pow(2, attempt - 1), 30000);
          console.warn(`Request failed, retrying in ${delay}ms (attempt ${attempt}/${CONFIG.maxRetries})`);
          await this.sleep(delay);
          continue;
        }

        throw err;
      }
    }

    if (!response) {
      throw lastError || new Error('Bedrock request failed without a response.');
    }

    const decodedBody = new TextDecoder().decode(response.body);
    const result = JSON.parse(decodedBody);

    const { text, inputTokens, outputTokens } = this.extractContentFromResponse(result);

    return {
      data: text,
      metrics: {
        inputTokens,
        outputTokens,
        totalTokens: inputTokens + outputTokens,
        model: selectedModel,
        region: this.region
      }
    };
  }

  private extractContentFromResponse(result: any): { text: string; inputTokens: number; outputTokens: number } {
    const usage = result?.usage ?? {};
    const inputTokens = usage.inputTokens ?? usage.input_tokens ?? 0;
    const outputTokens = usage.outputTokens ?? usage.output_tokens ?? 0;

    // Nova format
    if (result?.output?.message?.content) {
      const contentArray = result.output.message.content;
      const textBlock = contentArray.find((block: any) => typeof block?.text === 'string');
      if (textBlock?.text) {
        return { text: textBlock.text, inputTokens, outputTokens };
      }
    }

    // Anthropic format
    if (Array.isArray(result?.content)) {
      const textSegments = result.content
        .filter((block: any) => block?.type === 'text' && typeof block.text === 'string')
        .map((block: any) => block.text);

      if (textSegments.length > 0) {
        return { text: textSegments.join('\n'), inputTokens, outputTokens };
      }
    }

    // Titan format
    if (typeof result?.outputText === 'string') {
      return { text: result.outputText, inputTokens, outputTokens };
    }

    if (Array.isArray(result?.results) && result.results[0]?.outputText) {
      return { text: result.results[0].outputText, inputTokens, outputTokens };
    }

    throw new Error('Unsupported Bedrock response format: ' + JSON.stringify(result).slice(0, 500));
  }

  private isRateLimitError(error: unknown): boolean {
    if (!error || typeof error !== 'object') return false;

    const statusCode = (error as any)?.$metadata?.httpStatusCode;
    if (statusCode === 429) return true;

    const code = (error as any)?.name ?? (error as any)?.code;
    if (typeof code === 'string' && /throttl|rate.?exceed/i.test(code)) return true;

    const message = (error as any)?.message;
    if (typeof message === 'string' && /throttl|rate limit/i.test(message)) return true;

    return false;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// ============================================================================
// CLI
// ============================================================================

const DEFAULT_PROMPT = 'Describe what you see in this image in detail. Focus on the visual elements, composition, colors, and any notable features.';

async function run(options: AnalyzeOptions): Promise<void> {
  const llmService = new BedrockService();

  const resolvedPath = path.resolve(options.image);
  if (!existsSync(resolvedPath)) {
    throw new Error(`Image not found at ${resolvedPath}`);
  }

  const buffer = readFileSync(resolvedPath);
  const extension = path.extname(resolvedPath).toLowerCase();
  const format = extension === '.png' ? 'png' : 'jpeg';

  const messages: LLMMessage[] = [
    {
      role: 'user',
      content: [
        {
          image: {
            format,
            source: {
              bytes: buffer.toString('base64')
            }
          }
        },
        {
          text: options.prompt || DEFAULT_PROMPT
        }
      ]
    }
  ];

  console.log(`\n🔍 Analyzing image: ${resolvedPath}`);
  console.log(`📝 Prompt: ${options.prompt || DEFAULT_PROMPT}`);
  console.log(`🤖 Model: ${options.model}`);
  console.log(`🌡️  Temperature: ${options.temperature}`);
  console.log('');

  const response = await llmService.generateWithImages(
    messages,
    options.system,
    {
      temperature: options.temperature,
      maxTokens: options.maxTokens
    },
    options.model
  );

  console.log('━'.repeat(80));
  console.log('📋 Analysis Result:');
  console.log('━'.repeat(80));
  console.log(response.data);
  console.log('━'.repeat(80));

  const { totalTokens, inputTokens, outputTokens, model, region } = response.metrics;
  console.log(`\n📊 Tokens: ${totalTokens} (input: ${inputTokens}, output: ${outputTokens})`);
  console.log(`🏷️  Model: ${model} | Region: ${region}`);
}

function parseArgs(): AnalyzeOptions {
  const args = process.argv.slice(2);
  const options: AnalyzeOptions = {
    image: '',
    prompt: DEFAULT_PROMPT,
    model: CONFIG.defaultModel,
    temperature: CONFIG.temperature,
    maxTokens: CONFIG.maxTokens
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    const next = args[i + 1];

    switch (arg) {
      case '-i':
      case '--image':
        options.image = next || '';
        i++;
        break;
      case '-p':
      case '--prompt':
        options.prompt = next || DEFAULT_PROMPT;
        i++;
        break;
      case '-m':
      case '--model':
        options.model = next || CONFIG.defaultModel;
        i++;
        break;
      case '-t':
      case '--temperature':
        options.temperature = parseFloat(next || '0.2');
        i++;
        break;
      case '--max-tokens':
        options.maxTokens = parseInt(next || '4096', 10);
        i++;
        break;
      case '-s':
      case '--system':
        options.system = next || '';
        i++;
        break;
      case '-h':
      case '--help':
        console.log(`
Usage: npx ts-node scripts/analyze-image.ts [options]

Options:
  -i, --image <path>        Path to the image to analyze (required)
  -p, --prompt <text>       Prompt to send with the image
  -m, --model <id>          Model identifier (default: ${CONFIG.defaultModel})
  -t, --temperature <value> Sampling temperature (default: ${CONFIG.temperature})
  --max-tokens <value>      Maximum tokens to generate (default: ${CONFIG.maxTokens})
  -s, --system <text>       Optional system prompt
  -h, --help                Show this help message

Examples:
  npx ts-node scripts/analyze-image.ts -i screenshot.png
  npx ts-node scripts/analyze-image.ts -i game.png -p "Describe the terrain and biomes"
  npx ts-node scripts/analyze-image.ts -i game.png -m us.anthropic.claude-sonnet-4-5-20250929-v1:0
`);
        process.exit(0);
    }
  }

  if (!options.image) {
    console.error('Error: Image path is required. Use -i or --image to specify the image path.');
    console.error('Use -h or --help for usage information.');
    process.exit(1);
  }

  return options;
}

async function main(): Promise<void> {
  const options = parseArgs();

  try {
    await run(options);
  } catch (error) {
    console.error('\n❌ Analysis failed:');
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}

main().catch((error) => {
  console.error('\n❌ Unexpected failure:');
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
