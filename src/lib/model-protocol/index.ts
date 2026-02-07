import type { ChannelType } from '@/lib/bindings'
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
