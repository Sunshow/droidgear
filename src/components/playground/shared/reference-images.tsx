import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Upload, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { parseImageUrls } from '@/lib/playground-utils'

const MAX_IMAGE_BYTES = 20 * 1024 * 1024

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      if (typeof reader.result === 'string') resolve(reader.result)
      else reject(new Error('read failed'))
    }
    reader.onerror = () => reject(reader.error ?? new Error('read failed'))
    reader.readAsDataURL(file)
  })
}

/**
 * Reference-image grid shared by the Playground pages. Images are added from
 * one or more URLs or local files; a local file is read as a base64 data URL so
 * the upstream sees the same string either way.
 */
export function ReferenceImages({
  value,
  onChange,
}: {
  value: string
  onChange: (value: string) => void
}) {
  const { t } = useTranslation()
  const imageUrls = parseImageUrls(value)
  const [open, setOpen] = useState(false)
  const [tab, setTab] = useState<'url' | 'file'>('url')
  const [urls, setUrls] = useState('')
  const [fileDataUrls, setFileDataUrls] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)
  const fileInput = useRef<HTMLInputElement>(null)
  const fileReadToken = useRef(0)

  const reset = () => {
    fileReadToken.current += 1
    setTab('url')
    setUrls('')
    setFileDataUrls([])
    setError(null)
  }

  const handleOpenChange = (next: boolean) => {
    setOpen(next)
    if (!next) reset()
  }

  const addMany = (images: string[]) => {
    if (images.length === 0) return
    onChange([...imageUrls, ...images].join('\n'))
    setOpen(false)
    reset()
  }

  const addUrls = () => {
    const candidates: string[] = []
    for (const [index, line] of urls.split(/\r?\n/).entries()) {
      const candidate = line.trim()
      if (!candidate) continue
      try {
        const parsed = new URL(candidate)
        if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
          throw new Error('unsupported protocol')
        }
      } catch {
        setError(
          t('playground.referenceImages.invalidLine', { line: index + 1 })
        )
        return
      }
      candidates.push(candidate)
    }
    addMany(candidates)
  }

  const addFiles = async (files: File[]) => {
    const token = ++fileReadToken.current
    setError(null)
    if (files.length === 0) return
    if (files.some(file => !file.type.startsWith('image/'))) {
      setError(t('playground.referenceImages.fileInvalid'))
      return
    }
    if (files.some(file => file.size > MAX_IMAGE_BYTES)) {
      setError(t('playground.referenceImages.fileTooLarge'))
      return
    }
    try {
      const dataUrls = await Promise.all(files.map(readFileAsDataUrl))
      if (fileReadToken.current === token) setFileDataUrls(dataUrls)
    } catch {
      if (fileReadToken.current === token) {
        setError(t('playground.referenceImages.fileReadFailed'))
      }
    }
  }

  const removeAt = (index: number) => {
    onChange(imageUrls.filter((_, current) => current !== index).join('\n'))
  }

  return (
    <>
      <div className="grid grid-cols-3 gap-2">
        {imageUrls.map((imageUrl, index) => (
          <div
            key={`${index}-${imageUrl.slice(0, 40)}`}
            className="group relative aspect-square overflow-hidden rounded-md border bg-muted"
          >
            <img
              src={imageUrl}
              alt={t('playground.referenceImages.alt', { index: index + 1 })}
              className="size-full object-cover"
              draggable={false}
            />
            <Button
              type="button"
              variant="secondary"
              size="icon-sm"
              className="absolute end-1 top-1"
              aria-label={t('playground.referenceImages.remove')}
              onClick={() => removeAt(index)}
            >
              <X />
            </Button>
          </div>
        ))}
        <button
          type="button"
          onClick={() => setOpen(true)}
          aria-label={t('playground.referenceImages.add')}
          className="flex aspect-square flex-col items-center justify-center gap-1 rounded-md border border-dashed bg-muted/30 text-muted-foreground transition-colors hover:border-primary hover:text-primary"
        >
          <Plus className="size-5" />
          <span className="text-[11px]">
            {t('playground.referenceImages.add')}
          </span>
        </button>
      </div>
      <p className="text-xs text-muted-foreground">
        {t('playground.referenceImages.hint')}
      </p>

      <Dialog open={open} onOpenChange={handleOpenChange}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>
              {t('playground.referenceImages.addTitle')}
            </DialogTitle>
            <DialogDescription>
              {t('playground.referenceImages.addDescription')}
            </DialogDescription>
          </DialogHeader>
          <Tabs
            value={tab}
            onValueChange={next => {
              setTab(next as 'url' | 'file')
              setError(null)
            }}
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="url">
                {t('playground.referenceImages.urlTab')}
              </TabsTrigger>
              <TabsTrigger value="file">
                {t('playground.referenceImages.fileTab')}
              </TabsTrigger>
            </TabsList>
            <TabsContent value="url" className="space-y-3 pt-3">
              <Textarea
                value={urls}
                onChange={event => {
                  setUrls(event.target.value)
                  setError(null)
                }}
                aria-label={t('playground.referenceImages.urlTab')}
                placeholder={t('playground.referenceImages.urlPlaceholder')}
                className="min-h-24 font-mono text-xs"
              />
              {error && <p className="text-xs text-destructive">{error}</p>}
              <Button
                type="button"
                className="w-full"
                onClick={addUrls}
                disabled={urls.trim() === ''}
              >
                {t('common.add')}
              </Button>
            </TabsContent>
            <TabsContent value="file" className="space-y-3 pt-3">
              <input
                ref={fileInput}
                type="file"
                accept="image/*"
                multiple
                className="hidden"
                onChange={event => {
                  void addFiles(Array.from(event.target.files ?? []))
                  event.target.value = ''
                }}
              />
              <Button
                type="button"
                variant="outline"
                className="w-full"
                onClick={() => fileInput.current?.click()}
              >
                <Upload />
                {t('playground.referenceImages.choose')}
              </Button>
              {fileDataUrls.length > 0 && (
                <div className="grid max-h-60 grid-cols-3 gap-2 overflow-auto">
                  {fileDataUrls.map((dataUrl, index) => (
                    <div
                      key={index}
                      className="relative aspect-square overflow-hidden rounded-md border bg-muted"
                    >
                      <img
                        src={dataUrl}
                        alt=""
                        className="size-full object-cover"
                      />
                      <Button
                        type="button"
                        variant="secondary"
                        size="icon-sm"
                        className="absolute end-1 top-1"
                        aria-label={t('playground.referenceImages.remove')}
                        onClick={() =>
                          setFileDataUrls(previous =>
                            previous.filter((_, current) => current !== index)
                          )
                        }
                      >
                        <X />
                      </Button>
                    </div>
                  ))}
                </div>
              )}
              {error && <p className="text-xs text-destructive">{error}</p>}
              <Button
                type="button"
                className="w-full"
                onClick={() => addMany(fileDataUrls)}
                disabled={fileDataUrls.length === 0}
              >
                {t('common.add')}
              </Button>
            </TabsContent>
          </Tabs>
        </DialogContent>
      </Dialog>
    </>
  )
}
