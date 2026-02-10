import * as React from 'react'
import { useState, useEffect, useCallback } from 'react'
import * as DialogPrimitive from '@radix-ui/react-dialog'
import { Rnd } from 'react-rnd'
import { XIcon } from 'lucide-react'

import { cn } from '@/lib/utils'

function ResizableDialog({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Root>) {
  return <DialogPrimitive.Root data-slot="resizable-dialog" {...props} />
}

function ResizableDialogTrigger({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Trigger>) {
  return (
    <DialogPrimitive.Trigger data-slot="resizable-dialog-trigger" {...props} />
  )
}

function ResizableDialogPortal({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Portal>) {
  return (
    <DialogPrimitive.Portal data-slot="resizable-dialog-portal" {...props} />
  )
}

function ResizableDialogClose({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Close>) {
  return <DialogPrimitive.Close data-slot="resizable-dialog-close" {...props} />
}

const ResizableDialogOverlay = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Overlay
    ref={ref}
    data-slot="resizable-dialog-overlay"
    className={cn(
      'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50 rounded-xl',
      className
    )}
    {...props}
  />
))
ResizableDialogOverlay.displayName = DialogPrimitive.Overlay.displayName

interface ResizableDialogContentProps extends React.ComponentPropsWithoutRef<
  typeof DialogPrimitive.Content
> {
  showCloseButton?: boolean
  defaultWidth?: number
  defaultHeight?: number
  minWidth?: number
  minHeight?: number
  maxWidth?: number
  maxHeight?: number
  onCloseAutoFocus?: (event: Event) => void
}

const ResizableDialogContent = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  ResizableDialogContentProps
>(
  (
    {
      className,
      children,
      showCloseButton = true,
      defaultWidth = 700,
      defaultHeight = 500,
      minWidth = 400,
      minHeight = 300,
      maxWidth,
      maxHeight,
      onCloseAutoFocus,
      ...props
    },
    ref
  ) => {
    const [size, setSize] = useState({
      width: defaultWidth,
      height: defaultHeight,
    })
    const [position, setPosition] = useState({ x: 0, y: 0 })
    const [isInitialized, setIsInitialized] = useState(false)

    // Center the dialog on mount
    useEffect(() => {
      const centerDialog = () => {
        const windowWidth = window.innerWidth
        const windowHeight = window.innerHeight
        setPosition({
          x: Math.max(0, (windowWidth - size.width) / 2),
          y: Math.max(0, (windowHeight - size.height) / 2),
        })
        setIsInitialized(true)
      }
      centerDialog()
    }, [size.width, size.height])

    const handleDragStop = useCallback(
      (_e: unknown, d: { x: number; y: number }) => {
        setPosition({ x: d.x, y: d.y })
      },
      []
    )

    const handleResizeStop = useCallback(
      (
        _e: unknown,
        _direction: unknown,
        refEl: HTMLElement,
        _delta: unknown,
        pos: { x: number; y: number }
      ) => {
        setSize({ width: refEl.offsetWidth, height: refEl.offsetHeight })
        setPosition({ x: pos.x, y: pos.y })
      },
      []
    )

    return (
      <ResizableDialogPortal data-slot="resizable-dialog-portal">
        <ResizableDialogOverlay />
        <DialogPrimitive.Content
          ref={ref}
          data-slot="resizable-dialog-content"
          className="fixed inset-0 z-50 pointer-events-none"
          onCloseAutoFocus={onCloseAutoFocus}
          {...props}
        >
          <Rnd
            size={size}
            position={position}
            onDragStop={handleDragStop}
            onResizeStop={handleResizeStop}
            minWidth={minWidth}
            minHeight={minHeight}
            maxWidth={maxWidth}
            maxHeight={maxHeight}
            bounds="window"
            dragHandleClassName="resizable-dialog-drag-handle"
            className={cn('pointer-events-auto', !isInitialized && 'opacity-0')}
            style={{
              display: 'flex',
              flexDirection: 'column',
            }}
          >
            <div
              className={cn(
                'bg-background flex flex-col h-full w-full rounded-lg border shadow-lg overflow-hidden',
                className
              )}
            >
              {children}
              {showCloseButton && (
                <DialogPrimitive.Close
                  data-slot="resizable-dialog-close"
                  className="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
                >
                  <XIcon />
                  <span className="sr-only">Close</span>
                </DialogPrimitive.Close>
              )}
            </div>
          </Rnd>
        </DialogPrimitive.Content>
      </ResizableDialogPortal>
    )
  }
)
ResizableDialogContent.displayName = 'ResizableDialogContent'

function ResizableDialogHeader({
  className,
  ...props
}: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot="resizable-dialog-header"
      className={cn(
        'resizable-dialog-drag-handle flex flex-col gap-2 p-6 pb-0 cursor-move select-none',
        className
      )}
      {...props}
    />
  )
}

function ResizableDialogBody({
  className,
  ...props
}: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot="resizable-dialog-body"
      className={cn('flex-1 overflow-auto p-6', className)}
      {...props}
    />
  )
}

function ResizableDialogFooter({
  className,
  ...props
}: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot="resizable-dialog-footer"
      className={cn(
        'flex flex-col-reverse gap-2 p-6 pt-0 sm:flex-row sm:justify-end',
        className
      )}
      {...props}
    />
  )
}

const ResizableDialogTitle = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Title>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Title
    ref={ref}
    data-slot="resizable-dialog-title"
    className={cn('text-lg leading-none font-semibold', className)}
    {...props}
  />
))
ResizableDialogTitle.displayName = DialogPrimitive.Title.displayName

const ResizableDialogDescription = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Description>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Description
    ref={ref}
    data-slot="resizable-dialog-description"
    className={cn('text-muted-foreground text-sm', className)}
    {...props}
  />
))
ResizableDialogDescription.displayName = DialogPrimitive.Description.displayName

export {
  ResizableDialog,
  ResizableDialogClose,
  ResizableDialogContent,
  ResizableDialogDescription,
  ResizableDialogFooter,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogOverlay,
  ResizableDialogPortal,
  ResizableDialogTitle,
  ResizableDialogTrigger,
}
