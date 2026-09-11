import type {
  ChannelInferrer,
  ChannelInferenceContext,
  ModelProtocol,
} from '../types'
import { isMultiProtocolPlatform } from '@/lib/sub2api-platform'

/**
 * Sub2API 推断器
 *
 * Sub2API 的 key 自带 platform 字段，可以直接推断协议类型。
 * 特殊情况：
 * - antigravity 平台同时支持 Claude 和 Gemini，需要根据模型名称判断
 * - deepseek 平台同时支持三种协议（用户可自选），默认使用 OpenAI
 */
export class Sub2ApiInferrer implements ChannelInferrer {
  inferFromChannel(context: ChannelInferenceContext): ModelProtocol | null {
    const { platform } = context

    if (platform === 'openai') return 'openai'
    if (platform === 'anthropic' || platform === 'grok') return 'anthropic'
    if (platform === 'gemini') return 'google-ai'

    // deepseek 支持三种协议，未显式选择时默认 OpenAI
    if (platform === 'deepseek') return 'openai'

    // antigravity 需要根据模型名称判断，返回 null 让模型推断处理
    if (platform === 'antigravity') return null

    // 未知 platform，返回 null 使用模型推断
    return null
  }

  inferFromModel(
    modelId: string,
    context: ChannelInferenceContext
  ): ModelProtocol | null {
    const { platform } = context

    // antigravity 特殊处理：支持 Claude 和 Gemini
    if (platform === 'antigravity') {
      if (modelId.startsWith('claude-')) return 'anthropic'
      if (modelId.startsWith('gemini-')) return 'google-ai'
      return 'anthropic' // 默认 Claude
    }

    // 其他情况返回 null，使用全局推断
    return null
  }

  getBaseUrl(
    protocol: ModelProtocol,
    baseUrl: string,
    platform?: string | null
  ): string {
    // antigravity 特殊路径处理
    if (platform === 'antigravity') {
      if (protocol === 'anthropic') {
        return normalizeBaseUrl(baseUrl, '/antigravity')
      }
      if (protocol === 'google-ai') {
        return normalizeBaseUrl(baseUrl, '/antigravity/v1beta')
      }
    }

    // 多协议平台（如 deepseek）的通用兼容协议走 OpenAI 兼容端点，
    // 需要 /v1 后缀；OpenAI 与 Anthropic 协议使用裸 Base URL
    if (isMultiProtocolPlatform(platform) && protocol === 'openai-compatible') {
      return normalizeBaseUrl(baseUrl, '/v1')
    }

    // 其他情况不需要额外的路径转换
    return baseUrl
  }
}

function normalizeBaseUrl(baseUrl: string, suffix: string): string {
  const trimmed = baseUrl.replace(/\/+$/, '')
  return trimmed.endsWith(suffix) ? trimmed : `${trimmed}${suffix}`
}
