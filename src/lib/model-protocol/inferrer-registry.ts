import type { ChannelType } from '@/lib/bindings'
import type { ChannelInferrer } from './types'
import { Sub2ApiInferrer } from './channel-inferrers/sub2api-inferrer'
import { NewApiInferrer } from './channel-inferrers/newapi-inferrer'
import { CliProxyInferrer } from './channel-inferrers/cliproxy-inferrer'
import { GeneralInferrer } from './channel-inferrers/general-inferrer'

const inferrerMap = new Map<ChannelType, ChannelInferrer>([
  ['sub-2-api', new Sub2ApiInferrer()],
  ['new-api', new NewApiInferrer()],
  ['cli-proxy-api', new CliProxyInferrer()],
  ['general', new GeneralInferrer()],
])

/**
 * 获取指定 Channel 类型的推断器
 */
export function getInferrer(channelType: ChannelType): ChannelInferrer {
  const inferrer = inferrerMap.get(channelType)
  if (!inferrer) {
    throw new Error(`No inferrer registered for channel type: ${channelType}`)
  }
  return inferrer
}
