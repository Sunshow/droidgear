import { GeneralInferrer } from './general-inferrer'

/**
 * NewAPI 推断器
 *
 * NewAPI 没有 platform 字段，使用与 General 相同的推断逻辑
 */
export class NewApiInferrer extends GeneralInferrer {}
