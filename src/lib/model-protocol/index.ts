import type { ChannelType, Provider } from '@/lib/bindings'
import type {
  ModelProtocol,
  ModelProtocolInfo,
  ChannelInferenceContext,
} from './types'
import { getInferrer } from './inferrer-registry'
import { inferProtocolFromModelId } from './global-inference'

/**
 * 推断模型协议
 *
 * 推断优先级：
 * 1. Channel 自身推断 (inferFromChannel)
 * 2. Channel 的模型推断 (inferFromModel)
 * 3. 全局模型名称推断 (inferProtocolFromModelId)
 *
 * @param channelType - Channel 类型
 * @param platform - Platform 标识
 * @param baseUrl - Base URL
 * @param modelId - 模型 ID (可选)
 * @returns 模型协议类型
 */
export function inferModelProtocol(
  channelType: ChannelType,
  platform: string | null,
  baseUrl: string,
  modelId?: string
): ModelProtocol {
  const context: ChannelInferenceContext = { channelType, platform, baseUrl }
  const inferrer = getInferrer(channelType)

  // 1. 尝试从 Channel 推断
  const channelResult = inferrer.inferFromChannel(context)
  if (channelResult) return channelResult

  // 2. 如果有模型 ID，尝试 Channel 的模型推断
  if (modelId) {
    const modelResult = inferrer.inferFromModel(modelId, context)
    if (modelResult) return modelResult
  }

  // 3. 使用全局模型名称推断
  if (modelId) {
    return inferProtocolFromModelId(modelId)
  }

  // 4. 默认返回 OpenAI 兼容
  return 'openai-compatible'
}

/**
 * 推断模型协议信息（包含 Base URL）
 */
export function inferModelProtocolInfo(
  channelType: ChannelType,
  platform: string | null,
  baseUrl: string,
  modelId?: string
): ModelProtocolInfo {
  const protocol = inferModelProtocol(channelType, platform, baseUrl, modelId)
  const inferrer = getInferrer(channelType)
  const transformedBaseUrl = inferrer.getBaseUrl(protocol, baseUrl, platform)

  return {
    protocol,
    baseUrl: transformedBaseUrl,
  }
}

// 导出类型和工具函数
export * from './types'
export { inferProtocolFromModelId } from './global-inference'
export { getInferrer } from './inferrer-registry'
export {
  protocolToOpenCodeNpm,
  normalizeBaseUrlForOpenCode,
} from './opencode-npm'

/**
 * 用户显式选择的 Provider 对应的模型协议
 *
 * 仅用于多协议平台（例如 sub2api 的 deepseek 分组）：
 * - openai → OpenAI Responses / Chat Completions
 * - anthropic → Anthropic Messages
 * - generic-chat-completion-api → 通用兼容
 */
export function providerToModelProtocol(provider: Provider): ModelProtocol {
  switch (provider) {
    case 'anthropic':
      return 'anthropic'
    case 'generic-chat-completion-api':
      return 'openai-compatible'
    case 'openai':
    default:
      return 'openai'
  }
}

/** 客户端 API 类型（openclaw / dsh 使用的命名） */
export type ClientApiType =
  | 'anthropic-messages'
  | 'openai-responses'
  | 'openai-completions'

/**
 * 用户显式选择的 Provider 对应的客户端 API 类型
 *
 * 仅多协议平台会传入显式 Provider；未选择时调用方仍使用自身的推断逻辑。
 */
export function providerToClientApiType(provider: Provider): ClientApiType {
  switch (provider) {
    case 'anthropic':
      return 'anthropic-messages'
    case 'generic-chat-completion-api':
      return 'openai-completions'
    case 'openai':
    default:
      return 'openai-responses'
  }
}
