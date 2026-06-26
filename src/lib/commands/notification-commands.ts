import type { AppCommand } from './types'
import { notifications } from '@/lib/notifications'
import { buildReleaseUrl, showUpdateNotification } from '@/services/updater'

export const notificationCommands: AppCommand[] = [
  {
    id: 'notification.test-toast',
    labelKey: 'commands.testToast.label',
    descriptionKey: 'commands.testToast.description',
    group: 'debug',
    keywords: ['test', 'toast', 'notification', 'debug'],
    async execute() {
      await notifications.success('Test Toast', 'This is a test notification')
    },
  },
  {
    id: 'notification.test-update-toast',
    labelKey: 'commands.testUpdateToast.label',
    descriptionKey: 'commands.testUpdateToast.description',
    group: 'debug',
    keywords: [
      'test',
      'update',
      'toast',
      'notification',
      'debug',
      'later',
      '升级',
      '更新',
    ],
    isAvailable: () => import.meta.env.DEV,
    execute() {
      showUpdateNotification(
        {
          version: '99.0.0-test',
          body: 'Manual test — click 稍后 / Later to dismiss and snooze.',
          channel: 'managed',
          releaseUrl: buildReleaseUrl('99.0.0-test'),
        },
        { force: true }
      )
    },
  },
]
